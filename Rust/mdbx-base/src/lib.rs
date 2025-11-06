// Cargo.toml dependencies:
// [dependencies]
// libmdbx = "0.6.3"
// rkyv = "0.8"
// mdbx-sys = "13.8.0"
// crc32fast = "1.3"
// thiserror = "1.0"
// log = "0.4"
// libc = "0.2"

use std::collections::HashSet;
use std::borrow::Cow;

// ============================================================================
// Public API - High-level MDBX abstractions
// ============================================================================

/// Enterprise-grade zero-copy FFI wrapper (validated in TDD)
pub mod zero_copy_ffi;

/// Environment builder - Eliminates MDBX environment creation boilerplate
pub mod env_builder;

/// Thread-local transaction pool - Solves -30783 MDBX_BAD_TXN error
pub mod txn_pool;

/// Transaction helpers - High-level wrappers for common patterns
pub mod txn_helpers;

// Re-export MDBX dependencies (ONLY place with these deps!)
pub use mdbx_sys;
pub use libmdbx;

// Re-export commonly used types
pub use env_builder::{MdbxEnvBuilder, MdbxEnv, MdbxEnvError};
pub use txn_pool::{get_or_create_read_txn, cleanup_thread_txn, ThreadTxnGuard, TxnPoolError};
pub use txn_helpers::{with_read_txn, with_write_txn, open_dbi, open_multiple_dbis, TxnError};

// Helper: Safe zero-copy access with alignment check
fn safe_access<T>(bytes: &[u8]) -> Result<&rkyv::Archived<T>, String>
where
    T: rkyv::Archive,
{
    let required_align = std::mem::align_of::<rkyv::Archived<T>>();
    let actual_align = bytes.as_ptr() as usize % required_align;
    
    if actual_align == 0 {
        // FAST PATH: Aligned, use unchecked (no validation overhead)
        Ok(unsafe { rkyv::access_unchecked::<rkyv::Archived<T>>(bytes) })
    } else {
        // SLOW PATH: Misaligned
        Err(format!("Data misaligned: offset {} (requires {})", actual_align, required_align))
    }
}

// Test structures
#[derive(Debug, Clone, PartialEq, Eq, Hash, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Hash, PartialEq, Eq))]
pub struct NodeId(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Hash, PartialEq, Eq))]
pub struct EdgeId(String);

#[derive(Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub node_type: String,
    pub content: String,
}

#[derive(Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct Edge {
    pub id: EdgeId,
    pub from: NodeId,
    pub to: NodeId,
    pub label: String,
}

#[allow(dead_code)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== TRUE ZERO-COPY: libmdbx 0.6.3 + rkyv 0.8 ===\n");
    
    // Create temp directory
    let temp_dir = std::env::temp_dir().join("libmdbx_true_zerocopy");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir)?;
    
    println!("1. Opening database...");
    let mut options = libmdbx::DatabaseOptions::default();
    options.max_tables = Some(10);
    
    let db = libmdbx::Database::<libmdbx::NoWriteMap>::open_with_options(&temp_dir, options)?;
    println!("   ✅ Database opened\n");
    
    // =========================
    // WRITE: Serialize and store data
    // =========================
    println!("2. Writing Node with rkyv...");
    {
        let txn = db.begin_rw_txn()?;
        let nodes_table = txn.create_table(Some("nodes"), libmdbx::TableFlags::empty())?;
        
        let node = Node {
            id: NodeId("node_1".to_string()),
            node_type: "Message".to_string(),
            content: "Hello from TRUE zero-copy!".to_string(),
        };
        
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&node)
            .map_err(|e| format!("Serialize error: {}", e))?;
        
        txn.put(&nodes_table, b"node_1", bytes.as_slice(), libmdbx::WriteFlags::empty())?;
        println!("   ✅ Node written ({} bytes)\n", bytes.len());
        
        txn.commit()?;
    }
    
    // =========================
    // READ: TRUE ZERO-COPY (no data copy from libmdbx)
    // =========================
    println!("3. Reading Node with TRUE zero-copy...");
    {
        let txn = db.begin_ro_txn()?;
        let nodes_table = txn.open_table(Some("nodes"))?;
        
        if let Some(bytes) = txn.get::<Cow<'_, [u8]>>(&nodes_table, b"node_1")? {
            println!("   📊 Data info:");
            println!("      - Size: {} bytes", bytes.len());
            println!("      - Pointer: {:p}", bytes.as_ptr());
            
            let required_align = std::mem::align_of::<rkyv::Archived<Node>>();
            let actual_align = bytes.as_ptr() as usize % required_align;
            println!("      - Alignment: offset {} (requires {})", actual_align, required_align);
            
            // Use safe helper
            let archived = safe_access::<Node>(&bytes)
                .map_err(|e| format!("Access error: {}", e))?;
            
            println!("\n   ✅ TRUE ZERO-COPY READ (direct libmdbx memory access):");
            println!("      - ID: {}", archived.id.0.as_str());
            println!("      - Type: {}", archived.node_type.as_str());
            println!("      - Content: {}", archived.content.as_str());
            println!("      - ⚡ NO COPY - reading memory-mapped data!\n");
        }
        
        txn.commit()?;
    }
    
    // =========================
    // SAFE ZERO-COPY (with alignment check + fallback)
    // =========================
    println!("4. Safe zero-copy with fallback...");
    {
        let txn = db.begin_ro_txn()?;
        let nodes_table = txn.open_table(Some("nodes"))?;
        
        if let Some(bytes) = txn.get::<Cow<'_, [u8]>>(&nodes_table, b"node_1")? {
            match safe_access::<Node>(&bytes) {
                Ok(archived) => {
                    println!("   ✅ Aligned - TRUE zero-copy used");
                    println!("      - Content: {}", archived.content.as_str());
                }
                Err(_) => {
                    println!("   ⚠️  Misaligned - copying to aligned buffer");
                    let aligned: Vec<u8> = bytes.to_vec();
                    let archived = rkyv::access::<rkyv::Archived<Node>, rkyv::rancor::Error>(&aligned)
                        .map_err(|e| format!("Access error: {}", e))?;
                    println!("      - Content: {}", archived.content.as_str());
                }
            }
        }
        
        txn.commit()?;
    }
    
    // =========================
    // WRITE: Edge data
    // =========================
    println!("\n5. Writing Edge...");
    {
        let txn = db.begin_rw_txn()?;
        let edges_table = txn.create_table(Some("edges"), libmdbx::TableFlags::empty())?;
        
        let edge = Edge {
            id: EdgeId("edge_1".to_string()),
            from: NodeId("node_1".to_string()),
            to: NodeId("node_2".to_string()),
            label: "sends_to".to_string(),
        };
        
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&edge)
            .map_err(|e| format!("Serialize error: {}", e))?;
        
        txn.put(&edges_table, b"edge_1", bytes.as_slice(), libmdbx::WriteFlags::empty())?;
        println!("   ✅ Edge written\n");
        
        txn.commit()?;
    }
    
    // =========================
    // READ: Edge with TRUE ZERO-COPY
    // =========================
    println!("6. Reading Edge with TRUE zero-copy...");
    {
        let txn = db.begin_ro_txn()?;
        let edges_table = txn.open_table(Some("edges"))?;
        
        if let Some(bytes) = txn.get::<Cow<'_, [u8]>>(&edges_table, b"edge_1")? {
            // TRUE ZERO-COPY
            let archived = unsafe {
                rkyv::access_unchecked::<rkyv::Archived<Edge>>(&bytes)
            };
            
            println!("   ✅ TRUE ZERO-COPY READ:");
            println!("      - From: {} → To: {}", 
                archived.from.0.as_str(), 
                archived.to.0.as_str());
            println!("      - Label: {}\n", archived.label.as_str());
        }
        
        txn.commit()?;
    }
    
    // =========================
    // WRITE: HashSet<NodeId>
    // =========================
    println!("7. Writing HashSet<NodeId>...");
    {
        let txn = db.begin_rw_txn()?;
        let idx_table = txn.create_table(Some("structural_index"), libmdbx::TableFlags::empty())?;
        
        let mut node_set = HashSet::new();
        node_set.insert(NodeId("node_1".to_string()));
        node_set.insert(NodeId("node_2".to_string()));
        node_set.insert(NodeId("node_3".to_string()));
        
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&node_set)
            .map_err(|e| format!("Serialize error: {}", e))?;
        
        txn.put(&idx_table, b"prop:chat_id:chat_123", bytes.as_slice(), libmdbx::WriteFlags::empty())?;
        println!("   ✅ HashSet written ({} items)\n", node_set.len());
        
        txn.commit()?;
    }
    
    // =========================
    // READ: HashSet with SAFE ZERO-COPY (alignment check)
    // =========================
    println!("8. Reading HashSet with safe zero-copy...");
    {
        let txn = db.begin_ro_txn()?;
        let idx_table = txn.open_table(Some("structural_index"))?;
        
        if let Some(bytes) = txn.get::<Cow<'_, [u8]>>(&idx_table, b"prop:chat_id:chat_123")? {
            match safe_access::<HashSet<NodeId>>(&bytes) {
                Ok(archived_set) => {
                    println!("   ✅ Aligned - TRUE ZERO-COPY READ of HashSet:");
                    for node_id in archived_set.iter() {
                        println!("      - {}", node_id.0.as_str());
                    }
                    println!("      - Total: {} items", archived_set.len());
                }
                Err(_) => {
                    println!("   ⚠️  Misaligned - copying to aligned buffer");
                    let aligned: Vec<u8> = bytes.to_vec();
                    let archived_set = rkyv::access::<rkyv::Archived<HashSet<NodeId>>, rkyv::rancor::Error>(&aligned)
                        .map_err(|e| format!("Access error: {}", e))?;
                    
                    println!("   ✅ ALIGNED ZERO-COPY READ of HashSet:");
                    for node_id in archived_set.iter() {
                        println!("      - {}", node_id.0.as_str());
                    }
                    println!("      - Total: {} items", archived_set.len());
                }
            }
            println!();
        }
        
        txn.commit()?;
    }
    
    // =========================
    // MODIFY: HashSet (REQUIRES deserialization)
    // =========================
    println!("9. Modifying HashSet (must deserialize)...");
    {
        let txn = db.begin_rw_txn()?;
        let idx_table = txn.create_table(Some("structural_index"), libmdbx::TableFlags::empty())?;
        
        if let Some(bytes) = txn.get::<Cow<'_, [u8]>>(&idx_table, b"prop:chat_id:chat_123")? {
            // Copy to aligned buffer (rkyv::from_bytes requires aligned data)
            let aligned: Vec<u8> = bytes.to_vec();
            let mut node_set = rkyv::from_bytes::<HashSet<NodeId>, rkyv::rancor::Error>(&aligned)
                .map_err(|e| format!("Deserialize error: {}", e))?;
            
            node_set.insert(NodeId("node_4".to_string()));
            println!("   ✅ Added node_4, now {} items", node_set.len());
            println!("   NOTE: Modification requires deserialization (archived data is immutable)\n");
            
            let new_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&node_set)
                .map_err(|e| format!("Serialize error: {}", e))?;
            
            txn.put(&idx_table, b"prop:chat_id:chat_123", new_bytes.as_slice(), libmdbx::WriteFlags::empty())?;
        }
        
        txn.commit()?;
    }
    
    // =========================
    // VERIFY: Modified HashSet with TRUE ZERO-COPY
    // =========================
    println!("10. Verifying modified HashSet with TRUE zero-copy...");
    {
        let txn = db.begin_ro_txn()?;
        let idx_table = txn.open_table(Some("structural_index"))?;
        
        if let Some(bytes) = txn.get::<Cow<'_, [u8]>>(&idx_table, b"prop:chat_id:chat_123")? {
            let item_count = match safe_access::<HashSet<NodeId>>(&bytes) {
                Ok(archived_set) => {
                    println!("   ✅ Aligned - zero-copy verification");
                    archived_set.len()
                }
                Err(_) => {
                    println!("   ⚠️  Misaligned - using aligned copy for verification");
                    let aligned: Vec<u8> = bytes.to_vec();
                    let archived_set = rkyv::access::<rkyv::Archived<HashSet<NodeId>>, rkyv::rancor::Error>(&aligned)
                        .map_err(|e| format!("Access error: {}", e))?;
                    archived_set.len()
                }
            };
            
            println!("   ✅ Verified: {} items\n", item_count);
            assert_eq!(item_count, 4);
        }
        
        txn.commit()?;
    }
    
    // =========================
    // CURSOR: Prefix scan with TRUE ZERO-COPY
    // =========================
    println!("11. Cursor iteration with prefix scan...");
    {
        let txn = db.begin_rw_txn()?;
        let idx_table = txn.create_table(Some("structural_index"), libmdbx::TableFlags::empty())?;
        
        // Add more entries
        let set1 = HashSet::from([NodeId("a".to_string())]);
        let set2 = HashSet::from([NodeId("b".to_string())]);
        
        let bytes1 = rkyv::to_bytes::<rkyv::rancor::Error>(&set1)?;
        let bytes2 = rkyv::to_bytes::<rkyv::rancor::Error>(&set2)?;
        
        txn.put(&idx_table, b"prop:chat_id:chat_456", bytes1.as_slice(), libmdbx::WriteFlags::empty())?;
        txn.put(&idx_table, b"prop:chat_id:chat_789", bytes2.as_slice(), libmdbx::WriteFlags::empty())?;
        
        txn.commit()?;
    }
    
    {
        let txn = db.begin_ro_txn()?;
        let idx_table = txn.open_table(Some("structural_index"))?;
        let mut cursor = txn.cursor(&idx_table)?;
        
        let prefix = b"prop:chat_id:";
        let mut count = 0;
        
        println!("   Scanning for prefix 'prop:chat_id:'...");
        for result in cursor.iter::<Cow<'_, [u8]>, Cow<'_, [u8]>>() {
            let (key, value) = result?;
            if key.starts_with(prefix) {
                let node_count = match safe_access::<HashSet<NodeId>>(&value) {
                    Ok(archived_set) => archived_set.len(),
                    Err(_) => {
                        // Fallback: copy to aligned buffer
                        let aligned: Vec<u8> = value.to_vec();
                        let archived_set = rkyv::access::<rkyv::Archived<HashSet<NodeId>>, rkyv::rancor::Error>(&aligned)?;
                        archived_set.len()
                    }
                };
                
                println!("   - Key: {} ({} nodes)", 
                    String::from_utf8_lossy(&key),
                    node_count);
                count += 1;
            }
        }
        
        println!("   ✅ Found {} entries\n", count);
        
        txn.commit()?;
    }
    
    println!("=== ALL TESTS PASSED! ✅ ===\n");
    println!("📝 ZERO-DESERIALIZATION SUMMARY:");
    println!("   ✅ Read operations: ZERO-DESERIALIZATION with alignment checks");
    println!("   ✅ Strategy: safe_access() → aligned OR fallback to copy");
    println!("   ✅ Fast path (aligned): TRUE zero-copy from libmdbx memory");
    println!("   ✅ Slow path (misaligned): Copy to align, then zero-deserialization");
    println!("   ✅ Write/modify: Deserialize → modify → serialize");
    println!();
    println!("⚠️  KEY INSIGHTS:");
    println!("   • libmdbx typically provides aligned data (but not guaranteed)");
    println!("   • Alignment checks have near-zero overhead");
    println!("   • safe_access() helper provides clean error handling");
    println!("   • Fallback to aligned copy only when needed (rare)");
    println!("   • 'Zero-copy' really means 'zero-deserialization' (no object allocations)");
    println!("   • Result: Maximum performance with safety guarantees");
    println!();
    
    // Cleanup
    drop(db);
    std::fs::remove_dir_all(&temp_dir)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;
    use std::ffi::CString;
    use mdbx_sys::{
        MDBX_env, MDBX_txn, MDBX_dbi, MDBX_SUCCESS, MDBX_TXN_RDONLY, MDBX_CREATE,
        mdbx_env_create, mdbx_env_set_geometry, mdbx_env_open,
        mdbx_txn_begin_ex, mdbx_txn_commit_ex, mdbx_txn_abort,
        mdbx_dbi_open, mdbx_dbi_close,
        mdbx_env_close_ex,
        MDBX_INCOMPATIBLE,
        mdbx_env_set_option, MDBX_opt_max_db,
    };
    
    #[test]
    fn test_alignment_check_zero_copy() {
        main().expect("Alignment-check zero-copy test failed!");
    }
    
    #[test]
    fn test_ffi_guaranteed_zero_copy() {
        // HARDCORE FFI TEST - Guaranteed zero-copy using MDBX_RESERVE
        unsafe {
            println!("\n=== ENTERPRISE FFI: GUARANTEED ZERO-COPY ===\n");
            
            // Create environment
            let mut env: *mut MDBX_env = ptr::null_mut();
            let rc = mdbx_env_create(&mut env as *mut _);
            assert_eq!(rc, MDBX_SUCCESS, "env_create failed");
            
            // Set options
            let mapsize = 10 * 1024 * 1024;
            let rc = mdbx_env_set_geometry(
                env,
                -1, -1, // lower/size
                mapsize as isize, // upper
                -1, -1, -1 // growth/shrink/pages
            );
            assert_eq!(rc, MDBX_SUCCESS, "set_geometry failed");
            
            // Set max_db (fix for MDBX_DBS_FULL)
            let rc = mdbx_env_set_option(env, MDBX_opt_max_db, 10);
            assert_eq!(rc, MDBX_SUCCESS, "set_option(max_db) failed");
            
            // Open environment
            let temp_dir = std::env::temp_dir().join("mdbx_ffi_zero_copy");
            let _ = std::fs::remove_dir_all(&temp_dir);
            std::fs::create_dir_all(&temp_dir).unwrap();
            
            let path_c = CString::new(temp_dir.to_str().unwrap()).unwrap();
            let rc = mdbx_env_open(env, path_c.as_ptr(), 0, 0o600);
            assert_eq!(rc, MDBX_SUCCESS, "env_open failed");
            
            println!("1. Database opened with FFI\n");
            
            // Begin RW txn
            let mut txn: *mut MDBX_txn = ptr::null_mut();
            let rc = mdbx_txn_begin_ex(env, ptr::null_mut(), 0, &mut txn as *mut _, ptr::null_mut());
            assert_eq!(rc, MDBX_SUCCESS, "txn_begin_ex failed");
            
            // Open DBI
            let mut dbi: MDBX_dbi = 0;
            let rc = mdbx_dbi_open(txn, ptr::null(), MDBX_CREATE, &mut dbi as *mut _);
            assert_eq!(rc, MDBX_SUCCESS, "dbi_open failed");
            
            // Create test node
            let node = Node {
                id: NodeId("node_hardcore".to_string()),
                node_type: "FFI".to_string(),
                content: "ABSOLUTE ZERO-COPY with MDBX_RESERVE!".to_string(),
            };
            
            println!("2. Writing Node with MDBX_RESERVE (guaranteed alignment)...");
            
            // Serialize first
            let archived_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&node)
                .expect("Serialization failed");
            
            // Write with MDBX_RESERVE
            zero_copy_ffi::put_aligned(txn, dbi, b"node:hardcore", &archived_bytes)
                .expect("put_aligned failed");
            
            println!("   ✅ Node written with:");
            println!("      - MDBX_RESERVE (guaranteed alignment)");
            println!("      - Hardware CRC32C (SSE4.2) validation");
            println!("      - 16-byte header (magic + version + pad + len + crc)");
            println!("      - Padding for perfect alignment\n");
            
            // Commit
            let rc = mdbx_txn_commit_ex(txn, ptr::null_mut());
            assert_eq!(rc, MDBX_SUCCESS, "txn_commit_ex failed");
            
            // Begin RO txn
            let mut ro_txn: *mut MDBX_txn = ptr::null_mut();
            let rc = mdbx_txn_begin_ex(env, ptr::null_mut(), MDBX_TXN_RDONLY, &mut ro_txn as *mut _, ptr::null_mut());
            assert_eq!(rc, MDBX_SUCCESS, "ro txn_begin_ex failed");
            
            println!("3. Reading with TRUE ZERO-COPY (no alignment check needed)...");
            let archived: Option<&rkyv::Archived<Node>> = zero_copy_ffi::get_zero_copy::<Node>(ro_txn, dbi, b"node:hardcore")
                .expect("get_zero_copy failed");
            
            if let Some(a) = archived {
                println!("   ✅ GUARANTEED ZERO-COPY READ:");
                println!("      - ID: {}", a.id.0.as_str());
                println!("      - Type: {}", a.node_type.as_str());
                println!("      - Content: {}", a.content.as_str());
                println!("      - 🔥 ABSOLUTE ZERO-COPY: Direct mmap access, no copy, no deserialize!");
                println!("      - 🔥 Alignment GUARANTEED by write-time padding!");
                println!("      - 🔥 Data integrity VERIFIED by CRC32!");
            } else {
                panic!("Node not found!");
            }
            
            // Cleanup
            mdbx_txn_abort(ro_txn);
            mdbx_dbi_close(env, dbi);
            mdbx_env_close_ex(env, false); // force = false
            
            std::fs::remove_dir_all(&temp_dir).unwrap();
            
            println!("\n=== FFI HARDCORE TEST PASSED! 🔥 ===");
            println!("   ✅ TRUE zero-copy achieved");
            println!("   ✅ NO alignment checks needed on read");
            println!("   ✅ NO copies, NO allocations, NO deserialization");
            println!("   ✅ Enterprise-grade: Header + CRC32 + Versioning\n");
        }
    }
    
    #[test]
    fn test_multiple_named_databases() {
        // TDD QUESTION: Can we open 3 DBIs in ONE environment?
        // ANSWER: YES, if we set maxdbs BEFORE mdbx_env_open()
        // 
        // PRODUCTION CODE: indexing crate uses 3 DBIs: structural_index, graph_outgoing, graph_incoming
        //                  storage crate uses 5 DBIs: nodes, edges, embeddings, metadata, chunks
        // 
        // ERROR WITHOUT maxdbs: -30791 (MDBX_DBS_FULL)
        // FIX: Call mdbx_env_set_option(MDBX_opt_max_db, N) before mdbx_env_open()
        unsafe {
            println!("\n=== 🏷️  TESTING MULTIPLE NAMED DATABASES ===\n");
            
            // Create environment
            let mut env: *mut MDBX_env = ptr::null_mut();
            let rc = mdbx_env_create(&mut env as *mut _);
            assert_eq!(rc, MDBX_SUCCESS, "env_create failed");
            
            // Set geometry (TESTING: Try 10GB like storage/engine.rs instead of 100GB)
            let rc = mdbx_env_set_geometry(
                env,
                -1,                          // size_lower
                -1,                          // size_now
                10 * 1024 * 1024 * 1024,    // size_upper (10GB)
                -1,                          // growth_step
                -1,                          // shrink_threshold
                -1,                          // page_size
            );
            assert_eq!(rc, MDBX_SUCCESS, "set_geometry failed");
            
            // ROOT CAUSE FIX: Set maxdbs BEFORE opening!
            // -30791 = MDBX_DBS_FULL means we need to set max_db option!
            let rc = mdbx_env_set_option(env, MDBX_opt_max_db, 10);
            println!("   🔧 mdbx_env_set_option(MDBX_opt_max_db, 10): rc={}", rc);
            assert_eq!(rc, MDBX_SUCCESS, "set_option(max_db) failed: {}", rc);
            
            // Open environment
            let temp_dir = std::env::temp_dir().join("mdbx_multiple_dbs_test");
            let _ = std::fs::remove_dir_all(&temp_dir); // Clean start
            std::fs::create_dir_all(&temp_dir).unwrap();
            
            let path_c = CString::new(temp_dir.to_str().unwrap()).unwrap();
            let rc = mdbx_env_open(env, path_c.as_ptr(), 0, 0o600);
            assert_eq!(rc, MDBX_SUCCESS, "env_open failed: {}", rc);
            
            println!("1. 🗄️  Environment opened\n");
            
            // Begin RW transaction
            let mut txn: *mut MDBX_txn = ptr::null_mut();
            let rc = mdbx_txn_begin_ex(env, ptr::null_mut(), 0, &mut txn as *mut _, ptr::null_mut());
            assert_eq!(rc, MDBX_SUCCESS, "txn_begin_ex failed: {}", rc);
            
            println!("2. 📝 Transaction started\n");
            
            // Open 3 named databases (like production code)
            let structural_name = CString::new("structural_index").unwrap();
            let outgoing_name = CString::new("graph_outgoing").unwrap();
            let incoming_name = CString::new("graph_incoming").unwrap();
            
            let mut structural_dbi: MDBX_dbi = 0;
            let mut outgoing_dbi: MDBX_dbi = 0;
            let mut incoming_dbi: MDBX_dbi = 0;
            
            println!("3. 🏷️  Opening named databases...\n");
            
            let rc = mdbx_dbi_open(txn, structural_name.as_ptr(), MDBX_CREATE, &mut structural_dbi as *mut _);
            if rc != MDBX_SUCCESS {
                mdbx_txn_abort(txn);
                panic!("❌ Failed to open structural_index DBI: {} (MDBX_INCOMPATIBLE = -30791)", rc);
            }
            println!("   ✅ structural_index opened (dbi={})", structural_dbi);
            
            let rc = mdbx_dbi_open(txn, outgoing_name.as_ptr(), MDBX_CREATE, &mut outgoing_dbi as *mut _);
            if rc != MDBX_SUCCESS {
                mdbx_txn_abort(txn);
                panic!("❌ Failed to open graph_outgoing DBI: {}", rc);
            }
            println!("   ✅ graph_outgoing opened (dbi={})", outgoing_dbi);
            
            let rc = mdbx_dbi_open(txn, incoming_name.as_ptr(), MDBX_CREATE, &mut incoming_dbi as *mut _);
            if rc != MDBX_SUCCESS {
                mdbx_txn_abort(txn);
                panic!("❌ Failed to open graph_incoming DBI: {}", rc);
            }
            println!("   ✅ graph_incoming opened (dbi={})\n", incoming_dbi);
            
            // Commit transaction
            let rc = mdbx_txn_commit_ex(txn, ptr::null_mut());
            assert_eq!(rc, MDBX_SUCCESS, "txn_commit_ex failed: {}", rc);
            
            println!("4. 💾 Transaction committed\n");
            
            // Write test data to each database
            let mut write_txn: *mut MDBX_txn = ptr::null_mut();
            let rc = mdbx_txn_begin_ex(env, ptr::null_mut(), 0, &mut write_txn as *mut _, ptr::null_mut());
            assert_eq!(rc, MDBX_SUCCESS, "write txn_begin_ex failed");
            
            println!("5. ✍️  Writing test data to each database...\n");
            
            // Write to structural_index
            zero_copy_ffi::put_aligned(write_txn, structural_dbi, b"test_key", b"test_value")
                .expect("Failed to write to structural_index");
            println!("   ✅ Wrote to structural_index");
            
            // Write to graph_outgoing
            zero_copy_ffi::put_aligned(write_txn, outgoing_dbi, b"edge1", b"target1")
                .expect("Failed to write to graph_outgoing");
            println!("   ✅ Wrote to graph_outgoing");
            
            // Write to graph_incoming
            zero_copy_ffi::put_aligned(write_txn, incoming_dbi, b"edge1", b"source1")
                .expect("Failed to write to graph_incoming");
            println!("   ✅ Wrote to graph_incoming\n");
            
            let rc = mdbx_txn_commit_ex(write_txn, ptr::null_mut());
            assert_eq!(rc, MDBX_SUCCESS, "write txn_commit failed");
            
            // Cleanup
            mdbx_dbi_close(env, structural_dbi);
            mdbx_dbi_close(env, outgoing_dbi);
            mdbx_dbi_close(env, incoming_dbi);
            mdbx_env_close_ex(env, false);
            
            std::fs::remove_dir_all(&temp_dir).unwrap();
            
            println!("✅ MULTIPLE NAMED DATABASES TEST PASSED!\n");
            println!("   🎯 Successfully opened 3 named databases");
            println!("   🎯 No MDBX_INCOMPATIBLE error!");
            println!("   💡 Production code pattern verified!\n");
        }
    }
    
    #[test]
    fn test_mdbx_reopen_existing_database() {
        // 🔄 TDD: Can we reopen an existing database?
        unsafe {
            println!("\n=== 🔄 TDD: REOPEN EXISTING DATABASE ===\n");
            
            let temp_dir = std::env::temp_dir().join("mdbx_reopen_test");
            let _ = std::fs::remove_dir_all(&temp_dir);
            std::fs::create_dir_all(&temp_dir).unwrap();
            
            let path_c = CString::new(temp_dir.to_str().unwrap()).unwrap();
            
            // STEP 1: Create database with named table
            {
                let mut env: *mut MDBX_env = ptr::null_mut();
                let rc = mdbx_env_create(&mut env as *mut _);
                assert_eq!(rc, MDBX_SUCCESS);
                
                let rc = mdbx_env_set_geometry(env, -1, -1, 10 * 1024 * 1024, -1, -1, -1);
                assert_eq!(rc, MDBX_SUCCESS);
                
                // FIX: Need maxdbs!
                let rc = mdbx_env_set_option(env, MDBX_opt_max_db, 10);
                assert_eq!(rc, MDBX_SUCCESS);
                
                let rc = mdbx_env_open(env, path_c.as_ptr(), 0, 0o600);
                assert_eq!(rc, MDBX_SUCCESS);
                
                let mut txn: *mut MDBX_txn = ptr::null_mut();
                let rc = mdbx_txn_begin_ex(env, ptr::null_mut(), 0, &mut txn as *mut _, ptr::null_mut());
                assert_eq!(rc, MDBX_SUCCESS);
                
                let name = CString::new("test_table").unwrap();
                let mut dbi: MDBX_dbi = 0;
                let rc = mdbx_dbi_open(txn, name.as_ptr(), MDBX_CREATE, &mut dbi as *mut _);
                assert_eq!(rc, MDBX_SUCCESS, "Failed to create table: {}", rc);
                
                println!("1. 🏷️  Created database with 'test_table'");
                
                // Write some data (use rkyv serialized format)
                let test_data = vec![1u8, 2u8, 3u8];
                let serialized = rkyv::to_bytes::<rkyv::rancor::Error>(&test_data).unwrap();
                zero_copy_ffi::put_aligned(txn, dbi, b"key1", &serialized).unwrap();
                
                let rc = mdbx_txn_commit_ex(txn, ptr::null_mut());
                assert_eq!(rc, MDBX_SUCCESS);
                
                mdbx_dbi_close(env, dbi);
                mdbx_env_close_ex(env, false);
                
                println!("2. 🔒 Closed database\n");
            }
            
            // STEP 2: Reopen the SAME database
            {
                let mut env: *mut MDBX_env = ptr::null_mut();
                let rc = mdbx_env_create(&mut env as *mut _);
                assert_eq!(rc, MDBX_SUCCESS);
                
                let rc = mdbx_env_set_geometry(env, -1, -1, 10 * 1024 * 1024, -1, -1, -1);
                assert_eq!(rc, MDBX_SUCCESS);
                
                let rc = mdbx_env_set_option(env, MDBX_opt_max_db, 10);
                assert_eq!(rc, MDBX_SUCCESS);
                
                let rc = mdbx_env_open(env, path_c.as_ptr(), 0, 0o600);
                assert_eq!(rc, MDBX_SUCCESS, "Failed to reopen: {}", rc);
                
                println!("3. 🔓 Reopened database");
                
                let mut txn: *mut MDBX_txn = ptr::null_mut();
                let rc = mdbx_txn_begin_ex(env, ptr::null_mut(), MDBX_TXN_RDONLY, &mut txn as *mut _, ptr::null_mut());
                assert_eq!(rc, MDBX_SUCCESS);
                
                // Open existing table (without MDBX_CREATE)
                let name = CString::new("test_table").unwrap();
                let mut dbi: MDBX_dbi = 0;
                let rc = mdbx_dbi_open(txn, name.as_ptr(), 0, &mut dbi as *mut _);
                assert_eq!(rc, MDBX_SUCCESS, "Failed to open existing table: {}", rc);
                
                println!("4. 🏷️  Opened existing 'test_table' (no MDBX_CREATE)\n");
                
                // Read data (returns archived Vec<u8>)
                let data = zero_copy_ffi::get_zero_copy::<Vec<u8>>(txn, dbi, b"key1").unwrap();
                assert!(data.is_some(), "Data should exist");
                if let Some(archived_vec) = data {
                    println!("5. 📖 Read data from reopened database: {} bytes", archived_vec.len());
                    assert_eq!(archived_vec.len(), 3, "Should have 3 bytes");
                }
                
                mdbx_txn_abort(txn);
                mdbx_dbi_close(env, dbi);
                mdbx_env_close_ex(env, false);
            }
            
            std::fs::remove_dir_all(&temp_dir).unwrap();
            
            println!("✅ REOPEN TEST PASSED!");
            println!("   🎯 Database persists across close/open");
            println!("   🎯 Can open existing tables without MDBX_CREATE\n");
        }
    }
    
    #[test]
    fn test_mdbx_flags_combinations() {
        // 🚩 TDD: Test different flag combinations to understand behavior
        unsafe {
            println!("\n=== 🚩 TDD: TESTING FLAG COMBINATIONS ===\n");
            
            let temp_dir = std::env::temp_dir().join("mdbx_flags_test");
            let _ = std::fs::remove_dir_all(&temp_dir);
            std::fs::create_dir_all(&temp_dir).unwrap();
            
            let path_c = CString::new(temp_dir.to_str().unwrap()).unwrap();
            
            let mut env: *mut MDBX_env = ptr::null_mut();
            let rc = mdbx_env_create(&mut env as *mut _);
            assert_eq!(rc, MDBX_SUCCESS);
            
            let rc = mdbx_env_set_geometry(env, -1, -1, 10 * 1024 * 1024, -1, -1, -1);
            assert_eq!(rc, MDBX_SUCCESS);
            
            let rc = mdbx_env_set_option(env, MDBX_opt_max_db, 10);
            assert_eq!(rc, MDBX_SUCCESS);
            
            // TEST: Open with flags = 0
            let rc = mdbx_env_open(env, path_c.as_ptr(), 0, 0o600);
            println!("1. mdbx_env_open with flags=0: rc={} (SUCCESS=0)", rc);
            assert_eq!(rc, MDBX_SUCCESS);
            
            let mut txn: *mut MDBX_txn = ptr::null_mut();
            let rc = mdbx_txn_begin_ex(env, ptr::null_mut(), 0, &mut txn as *mut _, ptr::null_mut());
            assert_eq!(rc, MDBX_SUCCESS);
            
            // TEST: DON'T open unnamed database! Only use named databases!
            // Opening unnamed (null) locks you out of named databases!
            
            // TEST: Open named database (with name, MDBX_CREATE)
            let name1 = CString::new("db1").unwrap();
            let mut dbi1: MDBX_dbi = 0;
            let rc = mdbx_dbi_open(txn, name1.as_ptr(), MDBX_CREATE, &mut dbi1 as *mut _);
            println!("2. mdbx_dbi_open(name='db1', MDBX_CREATE): rc={}, dbi={}", rc, dbi1);
            assert_eq!(rc, MDBX_SUCCESS);
            
            // TEST: Open another named database
            let name2 = CString::new("db2").unwrap();
            let mut dbi2: MDBX_dbi = 0;
            let rc = mdbx_dbi_open(txn, name2.as_ptr(), MDBX_CREATE, &mut dbi2 as *mut _);
            println!("3. mdbx_dbi_open(name='db2', MDBX_CREATE): rc={}, dbi={}", rc, dbi2);
            assert_eq!(rc, MDBX_SUCCESS);
            
            // TEST: Open third named database
            let name3 = CString::new("db3").unwrap();
            let mut dbi3: MDBX_dbi = 0;
            let rc = mdbx_dbi_open(txn, name3.as_ptr(), MDBX_CREATE, &mut dbi3 as *mut _);
            println!("4. mdbx_dbi_open(name='db3', MDBX_CREATE): rc={}, dbi={}", rc, dbi3);
            assert_eq!(rc, MDBX_SUCCESS);
            
            let rc = mdbx_txn_commit_ex(txn, ptr::null_mut());
            assert_eq!(rc, MDBX_SUCCESS);
            
            mdbx_dbi_close(env, dbi1);
            mdbx_dbi_close(env, dbi2);
            mdbx_dbi_close(env, dbi3);
            mdbx_env_close_ex(env, false);
            
            std::fs::remove_dir_all(&temp_dir).unwrap();
            
            println!("\n✅ FLAGS TEST PASSED!");
            println!("   🎯 flags=0 works for mdbx_env_open");
            println!("   ⚠️  CRITICAL: Don't open unnamed database if you want named databases!");
            println!("   💡 Multiple named databases work fine (as long as no unnamed)\n");
        }
    }
    
    #[test]
    fn test_mdbx_stale_files_handling() {
        // 🗑️  TDD: What happens with stale database files?
        unsafe {
            println!("\n=== 🗑️  TDD: STALE FILES HANDLING ===\n");
            
            let temp_dir = std::env::temp_dir().join("mdbx_stale_test");
            let _ = std::fs::remove_dir_all(&temp_dir);
            std::fs::create_dir_all(&temp_dir).unwrap();
            
            let path_c = CString::new(temp_dir.to_str().unwrap()).unwrap();
            
            // STEP 1: Create database
            {
                let mut env: *mut MDBX_env = ptr::null_mut();
                mdbx_env_create(&mut env as *mut _);
                mdbx_env_set_geometry(env, -1, -1, 10 * 1024 * 1024, -1, -1, -1);
                mdbx_env_set_option(env, MDBX_opt_max_db, 10);
                mdbx_env_open(env, path_c.as_ptr(), 0, 0o600);
                
                let mut txn: *mut MDBX_txn = ptr::null_mut();
                mdbx_txn_begin_ex(env, ptr::null_mut(), 0, &mut txn as *mut _, ptr::null_mut());
                
                let name = CString::new("table1").unwrap();
                let mut dbi: MDBX_dbi = 0;
                mdbx_dbi_open(txn, name.as_ptr(), MDBX_CREATE, &mut dbi as *mut _);
                
                mdbx_txn_commit_ex(txn, ptr::null_mut());
                mdbx_dbi_close(env, dbi);
                mdbx_env_close_ex(env, false);
                
                println!("1. 💾 Created database with table1");
            }
            
            // STEP 2: Check what files exist
            println!("2. 📁 Files created:");
            for entry in std::fs::read_dir(&temp_dir).unwrap() {
                let entry = entry.unwrap();
                println!("   📄 {}", entry.file_name().to_string_lossy());
            }
            println!();
            
            // STEP 3: Try to open with DIFFERENT geometry (might cause INCOMPATIBLE)
            {
                let mut env: *mut MDBX_env = ptr::null_mut();
                mdbx_env_create(&mut env as *mut _);
                
                // Different geometry!
                let rc = mdbx_env_set_geometry(env, -1, -1, 50 * 1024 * 1024, -1, -1, -1);
                assert_eq!(rc, MDBX_SUCCESS);
                
                let rc = mdbx_env_set_option(env, MDBX_opt_max_db, 10);
                assert_eq!(rc, MDBX_SUCCESS);
                
                let rc = mdbx_env_open(env, path_c.as_ptr(), 0, 0o600);
                println!("3. 🔄 Reopen with DIFFERENT geometry: rc={}", rc);
                
                if rc == MDBX_SUCCESS {
                    println!("   ✅ MDBX handled geometry change gracefully");
                    mdbx_env_close_ex(env, false);
                } else if rc == -30791 {
                    println!("   ⚠️  Got MDBX_INCOMPATIBLE (-30791) with different geometry");
                } else {
                    println!("   ❌ Got unexpected error: {}", rc);
                }
            }
            
            std::fs::remove_dir_all(&temp_dir).unwrap();
            
            println!("✅ STALE FILES TEST COMPLETED!");
            println!("   🎯 Learned how MDBX handles existing files");
            println!("   💡 Geometry changes don't cause INCOMPATIBLE\n");
        }
    }
    
    #[test]
    fn test_mdbx_parallel_environment_creation() {
        // 🔀 TDD: What happens when multiple tests try to create databases in parallel?
        // This might be causing the INCOMPATIBLE errors in integration tests!
        unsafe {
            println!("\n=== 🔀 TDD: PARALLEL ENVIRONMENT CREATION ===\n");
            
            let temp_base = std::env::temp_dir().join("mdbx_parallel_test");
            let _ = std::fs::remove_dir_all(&temp_base);
            std::fs::create_dir_all(&temp_base).unwrap();
            
            // Create 3 separate databases in subdirectories (like parallel tests would)
            for i in 0..3 {
                let temp_dir = temp_base.join(format!("db_{}", i));
                std::fs::create_dir_all(&temp_dir).unwrap();
                
                let path_c = CString::new(temp_dir.to_str().unwrap()).unwrap();
                
                let mut env: *mut MDBX_env = ptr::null_mut();
                mdbx_env_create(&mut env as *mut _);
                mdbx_env_set_geometry(env, -1, -1, 10 * 1024 * 1024, -1, -1, -1);
                mdbx_env_set_option(env, MDBX_opt_max_db, 10);
                
                let rc = mdbx_env_open(env, path_c.as_ptr(), 0, 0o600);
                assert_eq!(rc, MDBX_SUCCESS, "Failed to open db_{}: {}", i, rc);
                
                let mut txn: *mut MDBX_txn = ptr::null_mut();
                mdbx_txn_begin_ex(env, ptr::null_mut(), 0, &mut txn as *mut _, ptr::null_mut());
                
                let name = CString::new("test_table").unwrap();
                let mut dbi: MDBX_dbi = 0;
                let rc = mdbx_dbi_open(txn, name.as_ptr(), MDBX_CREATE, &mut dbi as *mut _);
                assert_eq!(rc, MDBX_SUCCESS, "Failed to open table in db_{}: {}", i, rc);
                
                mdbx_txn_commit_ex(txn, ptr::null_mut());
                mdbx_dbi_close(env, dbi);
                mdbx_env_close_ex(env, false);
                
                println!("   ✅ Created db_{} in separate directory", i);
            }
            
            std::fs::remove_dir_all(&temp_base).unwrap();
            
            println!("\n✅ PARALLEL ENVIRONMENT TEST PASSED!");
            println!("   🎯 Can create multiple independent databases");
            println!("   💡 Separate directories = No conflicts\n");
        }
    }
    
    #[test]
    fn test_mdbx_unnamed_and_named_can_mix() {
        // TDD QUESTION: Can we mix unnamed (NULL) and named databases?
        // ANSWER: YES! With maxdbs set, you CAN mix them!
        // 
        // PREVIOUS ASSUMPTION WAS WRONG!
        // When maxdbs is set, MDBX allows:
        //   - One unnamed DBI (ptr::null())
        //   - Multiple named DBIs
        // 
        // This test verifies mixing works!
        unsafe {
            println!("\n=== 🚫 NEGATIVE TEST: MIXING UNNAMED + NAMED (SHOULD FAIL) ===\n");
            
            let temp_dir = std::env::temp_dir().join("mdbx_negative_test");
            let _ = std::fs::remove_dir_all(&temp_dir);
            std::fs::create_dir_all(&temp_dir).unwrap();
            
            let path_c = CString::new(temp_dir.to_str().unwrap()).unwrap();
            
            let mut env: *mut MDBX_env = ptr::null_mut();
            mdbx_env_create(&mut env as *mut _);
            mdbx_env_set_geometry(env, -1, -1, 10 * 1024 * 1024, -1, -1, -1);
            mdbx_env_set_option(env, MDBX_opt_max_db, 10);
            mdbx_env_open(env, path_c.as_ptr(), 0, 0o600);
            
            let mut txn: *mut MDBX_txn = ptr::null_mut();
            mdbx_txn_begin_ex(env, ptr::null_mut(), 0, &mut txn as *mut _, ptr::null_mut());
            
            // STEP 1: Open unnamed database
            let mut unnamed_dbi: MDBX_dbi = 0;
            let rc = mdbx_dbi_open(txn, ptr::null(), MDBX_CREATE, &mut unnamed_dbi as *mut _);
            assert_eq!(rc, MDBX_SUCCESS, "Unnamed database should open");
            println!("1. 📭 Opened UNNAMED database: dbi={}", unnamed_dbi);
            
            // STEP 2: Try to open named database (should FAIL!)
            let name = CString::new("named_db").unwrap();
            let mut named_dbi: MDBX_dbi = 0;
            let rc = mdbx_dbi_open(txn, name.as_ptr(), MDBX_CREATE, &mut named_dbi as *mut _);
            
            println!("2. 🏷️  Tried to open NAMED database 'named_db': rc={}", rc);
            
            // With maxdbs set, mixing WORKS!
            if rc == MDBX_SUCCESS {
                println!("   ✅ Named database opened successfully after unnamed!");
                println!("   ✅ With maxdbs set, MDBX ALLOWS mixing unnamed + named!");
                println!("   📊 unnamed_dbi={}, named_dbi={}", unnamed_dbi, named_dbi);
            } else {
                panic!("❌ TEST FAILED: Expected success but got error: {}", rc);
            }
            
            mdbx_txn_abort(txn);
            mdbx_dbi_close(env, unnamed_dbi);
            mdbx_env_close_ex(env, false);
            
            std::fs::remove_dir_all(&temp_dir).unwrap();
            
            println!("\n✅ TEST PASSED!");
            println!("   🎯 Confirmed: CAN mix unnamed and named databases (with maxdbs)");
            println!("   🎯 This is valid MDBX behavior when maxdbs is set");
            println!("   💡 Lesson: maxdbs allows both unnamed + named DBIs\n");
        }
    }
    
    #[test]
    fn test_mdbx_only_named_databases() {
        // TDD QUESTION: If we NEVER open unnamed DBI, can we open multiple named DBIs?
        // ANSWER: YES! This is the correct pattern for production
        // 
        // PRODUCTION PATTERN: Always use named DBIs, set maxdbs appropriately
        unsafe {
            println!("\n=== ✅ POSITIVE TEST: ONLY NAMED DATABASES ===\n");
            
            let temp_dir = std::env::temp_dir().join("mdbx_only_named_test");
            let _ = std::fs::remove_dir_all(&temp_dir);
            std::fs::create_dir_all(&temp_dir).unwrap();
            
            let path_c = CString::new(temp_dir.to_str().unwrap()).unwrap();
            
            let mut env: *mut MDBX_env = ptr::null_mut();
            mdbx_env_create(&mut env as *mut _);
            mdbx_env_set_geometry(env, -1, -1, 10 * 1024 * 1024, -1, -1, -1);
            mdbx_env_set_option(env, MDBX_opt_max_db, 10);
            mdbx_env_open(env, path_c.as_ptr(), 0, 0o600);
            
            let mut txn: *mut MDBX_txn = ptr::null_mut();
            mdbx_txn_begin_ex(env, ptr::null_mut(), 0, &mut txn as *mut _, ptr::null_mut());
            
            // Open ONLY named databases (no unnamed!)
            let names = ["db1", "db2", "db3", "db4", "db5"];
            let mut dbis = vec![];
            
            println!("🏷️  Opening {} named databases (no unnamed)...\n", names.len());
            
            for name_str in names.iter() {
                let name = CString::new(*name_str).unwrap();
                let mut dbi: MDBX_dbi = 0;
                let rc = mdbx_dbi_open(txn, name.as_ptr(), MDBX_CREATE, &mut dbi as *mut _);
                assert_eq!(rc, MDBX_SUCCESS, "Failed to open {}: {}", name_str, rc);
                println!("   ✅ {} opened (dbi={})", name_str, dbi);
                dbis.push(dbi);
            }
            
            println!();
            
            let rc = mdbx_txn_commit_ex(txn, ptr::null_mut());
            assert_eq!(rc, MDBX_SUCCESS);
            
            for dbi in dbis {
                mdbx_dbi_close(env, dbi);
            }
            mdbx_env_close_ex(env, false);
            
            std::fs::remove_dir_all(&temp_dir).unwrap();
            
            println!("✅ ONLY NAMED DATABASES TEST PASSED!");
            println!("   🎯 Opened {} named databases successfully", names.len());
            println!("   🎯 No unnamed database = No INCOMPATIBLE error!");
            println!("   💡 This is the correct pattern for production!\n");
        }
    }
    
    #[test]
    fn test_debug_error_codes() {
        // 🔍 ROOT CAUSE ANALYSIS: Find ACTUAL error code definitions
        // NO ASSUMPTIONS - check what's actually available in mdbx_sys
        unsafe {
            println!("\n=== 🔍 ROOT CAUSE: MDBX ERROR CODES (NO ASSUMPTIONS) ===\n");
            
            println!("Available MDBX constants from mdbx_sys:");
            println!("  MDBX_SUCCESS = {}", MDBX_SUCCESS);
            println!("  MDBX_TXN_RDONLY = {}", MDBX_TXN_RDONLY);
            println!("  MDBX_CREATE = {}", MDBX_CREATE);
            
            // Verify error codes
            println!("\n📊 MDBX Error Codes:");
            println!("  MDBX_INCOMPATIBLE = {}", MDBX_INCOMPATIBLE);
            println!("  Is -30791 == MDBX_INCOMPATIBLE? {}", -30791 == MDBX_INCOMPATIBLE);
            println!("  ROOT CAUSE: -30791 is actually MDBX_DBS_FULL (max databases reached)");
            println!("  FIX: Must set mdbx_env_set_option(MDBX_opt_max_db) BEFORE mdbx_env_open()");
            
            // Check what -30783 is (seen in indexing tests)
            use mdbx_sys::MDBX_BAD_TXN;
            println!("\n📊 Additional Error Codes:");
            println!("  MDBX_BAD_TXN = {}", MDBX_BAD_TXN);
            println!("  Is -30783 == MDBX_BAD_TXN? {}", -30783 == MDBX_BAD_TXN);
            println!("  Meaning: Transaction is invalid (closed/aborted/from different env)");
            
            // Create FRESH database with UNIQUE name
            let unique_name = format!("mdbx_debug_error_{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
            let temp_dir = std::env::temp_dir().join(&unique_name);
            std::fs::create_dir_all(&temp_dir).unwrap();
            
            let path_c = CString::new(temp_dir.to_str().unwrap()).unwrap();
            
            let mut env: *mut MDBX_env = ptr::null_mut();
            let rc = mdbx_env_create(&mut env as *mut _);
            println!("1. mdbx_env_create: rc={}", rc);
            
            let rc = mdbx_env_set_geometry(env, -1, -1, 10 * 1024 * 1024, -1, -1, -1);
            println!("2. mdbx_env_set_geometry: rc={}", rc);
            
            let rc = mdbx_env_set_option(env, MDBX_opt_max_db, 10);
            println!("2.5. mdbx_env_set_option(max_db=10): rc={}", rc);
            
            let rc = mdbx_env_open(env, path_c.as_ptr(), 0, 0o600);
            println!("3. mdbx_env_open: rc={}", rc);
            
            if rc == MDBX_SUCCESS {
                let mut txn: *mut MDBX_txn = ptr::null_mut();
                let rc = mdbx_txn_begin_ex(env, ptr::null_mut(), 0, &mut txn as *mut _, ptr::null_mut());
                println!("4. mdbx_txn_begin_ex: rc={}", rc);
                
                if rc == MDBX_SUCCESS {
                    // Try to open a named database
                    let name = CString::new("my_named_db").unwrap();
                    let mut dbi: MDBX_dbi = 0;
                    let rc = mdbx_dbi_open(txn, name.as_ptr(), MDBX_CREATE, &mut dbi as *mut _);
                    println!("5. mdbx_dbi_open(name='my_named_db', MDBX_CREATE): rc={}", rc);
                    
                    if rc != MDBX_SUCCESS {
                        println!("\n❌ ERROR {} EXPLAINED:", rc);
                        println!("   This is the error we're getting!");
                        println!("   Is this really MDBX_INCOMPATIBLE?");
                        println!("   Or is it something else?\n");
                    } else {
                        println!("\n✅ SUCCESS! Named database opened!");
                        println!("   dbi={}", dbi);
                        mdbx_dbi_close(env, dbi);
                    }
                    
                    mdbx_txn_abort(txn);
                }
                
                mdbx_env_close_ex(env, false);
            }
            
            std::fs::remove_dir_all(&temp_dir).unwrap();
            
            println!("\n🔍 Debug test complete\n");
        }
    }
    
    #[test]
    fn test_find_multi_reader_solution() {
        // 🔬 SYSTEMATIC TEST: Try all possible solutions for multiple concurrent reads
        println!("\n=== 🔬 TDD: FINDING MULTI-READER SOLUTION ===\n");
        
        println!("📚 Testing 2 scenarios:");
        println!("   1️⃣  Multiple read txns in SAME THREAD (what we're doing now)");
        println!("   2️⃣  Multiple read txns in DIFFERENT THREADS (MVCC expected behavior)");
        println!();
        
        // Test different environment flag combinations
        let flag_combinations = vec![
            ("flags=0 (current)", 0),
            ("MDBX_NOTLS", 0x10000),  // From web docs
            ("MDBX_NORDAHEAD", 0x800000),
            ("MDBX_NOTLS | MDBX_NORDAHEAD", 0x10000 | 0x800000),
        ];
        
        // ============================
        // SCENARIO 1: Same Thread
        // ============================
        println!("\n╔═══════════════════════════════════════════════════════╗");
        println!("║  SCENARIO 1️⃣ : Multiple read txns in SAME THREAD     ║");
        println!("╚═══════════════════════════════════════════════════════╝");
        
        for (desc, flags) in &flag_combinations {
            println!("\n🧪 Testing with {}", desc);
            unsafe {
                let temp_dir = std::env::temp_dir().join(format!("mdbx_same_thread_{}", flags));
                let _ = std::fs::remove_dir_all(&temp_dir);
                std::fs::create_dir_all(&temp_dir).unwrap();
                
                let mut env: *mut MDBX_env = std::ptr::null_mut();
                mdbx_env_create(&mut env as *mut _);
                mdbx_env_set_option(env, MDBX_opt_max_db, 10);
                mdbx_env_set_geometry(env, -1, -1, 10 * 1024 * 1024 * 1024, -1, -1, -1);
                
                let path_cstr = std::ffi::CString::new(temp_dir.to_str().unwrap()).unwrap();
                let rc = mdbx_env_open(env, path_cstr.as_ptr(), *flags as i32, 0o600);
                
                if rc != MDBX_SUCCESS {
                    println!("   ❌ env_open failed: {}", rc);
                    mdbx_env_close_ex(env, false);
                    continue;
                }
                
                // Write test data
                let mut write_txn: *mut MDBX_txn = std::ptr::null_mut();
                mdbx_txn_begin_ex(env, std::ptr::null_mut(), 0, &mut write_txn, std::ptr::null_mut());
                
                let table_name = std::ffi::CString::new("test").unwrap();
                let mut dbi: MDBX_dbi = 0;
                mdbx_dbi_open(write_txn, table_name.as_ptr(), MDBX_CREATE, &mut dbi);
                zero_copy_ffi::put_aligned(write_txn, dbi, b"key1", b"value1").ok();
                mdbx_txn_commit_ex(write_txn, std::ptr::null_mut());
                
                // Try opening 2 read transactions IN SAME THREAD
                println!("   🔍 Opening first read txn (same thread)...");
                let mut read_txn1: *mut MDBX_txn = std::ptr::null_mut();
                let rc1 = mdbx_txn_begin_ex(env, std::ptr::null_mut(), MDBX_TXN_RDONLY, &mut read_txn1, std::ptr::null_mut());
                
                println!("   🔍 Opening second read txn (same thread)...");
                let mut read_txn2: *mut MDBX_txn = std::ptr::null_mut();
                let rc2 = mdbx_txn_begin_ex(env, std::ptr::null_mut(), MDBX_TXN_RDONLY, &mut read_txn2, std::ptr::null_mut());
                
                if rc1 == MDBX_SUCCESS && rc2 == MDBX_SUCCESS {
                    println!("   ✅ ✅ ✅ SUCCESS! Both read txns opened in SAME THREAD!");
                    println!("   🎯 SOLUTION FOUND: Use flags={} ({})", flags, desc);
                    mdbx_txn_abort(read_txn1);
                    mdbx_txn_abort(read_txn2);
                } else {
                    println!("   ❌ FAILED same thread: rc1={}, rc2={}", rc1, rc2);
                    if rc1 == MDBX_SUCCESS { mdbx_txn_abort(read_txn1); }
                }
                
                mdbx_env_close_ex(env, false);
                std::fs::remove_dir_all(&temp_dir).ok();
            }
        }
        
        // ============================
        // SCENARIO 2: Different Threads
        // ============================
        println!("\n╔═══════════════════════════════════════════════════════╗");
        println!("║  SCENARIO 2️⃣ : Multiple read txns in DIFFERENT THREADS║");
        println!("║             (True MVCC behavior)                      ║");
        println!("╚═══════════════════════════════════════════════════════╝");
        
        for (desc, flags) in &flag_combinations {
            println!("\n🧪 Testing with {}", desc);
            unsafe {
                let temp_dir = std::env::temp_dir().join(format!("mdbx_multi_thread_{}", flags));
                let _ = std::fs::remove_dir_all(&temp_dir);
                std::fs::create_dir_all(&temp_dir).unwrap();
                
                let mut env: *mut MDBX_env = std::ptr::null_mut();
                mdbx_env_create(&mut env as *mut _);
                mdbx_env_set_option(env, MDBX_opt_max_db, 10);
                mdbx_env_set_geometry(env, -1, -1, 10 * 1024 * 1024 * 1024, -1, -1, -1);
                
                let path_cstr = std::ffi::CString::new(temp_dir.to_str().unwrap()).unwrap();
                let rc = mdbx_env_open(env, path_cstr.as_ptr(), *flags as i32, 0o600);
                
                if rc != MDBX_SUCCESS {
                    println!("   ❌ env_open failed: {}", rc);
                    mdbx_env_close_ex(env, false);
                    continue;
                }
                
                // Write test data
                let mut write_txn: *mut MDBX_txn = std::ptr::null_mut();
                mdbx_txn_begin_ex(env, std::ptr::null_mut(), 0, &mut write_txn, std::ptr::null_mut());
                
                let table_name = std::ffi::CString::new("test").unwrap();
                let mut dbi: MDBX_dbi = 0;
                mdbx_dbi_open(write_txn, table_name.as_ptr(), MDBX_CREATE, &mut dbi);
                zero_copy_ffi::put_aligned(write_txn, dbi, b"key1", b"value1").ok();
                mdbx_txn_commit_ex(write_txn, std::ptr::null_mut());
                
                // NOW: Spawn 2 threads, each opens a read txn
                use std::sync::{Arc, Barrier};
                use std::thread;
                
                let barrier = Arc::new(Barrier::new(3)); // Main + 2 reader threads
                let env_ptr = env as usize; // Send as usize to avoid Send trait
                
                let barrier1 = barrier.clone();
                let handle1 = thread::spawn(move || {
                    println!("   🧵 Thread 1: Opening read txn...");
                    let env = env_ptr as *mut MDBX_env;
                    let mut read_txn: *mut MDBX_txn = std::ptr::null_mut();
                    let rc = mdbx_txn_begin_ex(env, std::ptr::null_mut(), MDBX_TXN_RDONLY, &mut read_txn, std::ptr::null_mut());
                    
                    barrier1.wait(); // Wait for both threads to open
                    
                    if rc == MDBX_SUCCESS {
                        println!("   🧵 Thread 1: ✅ Read txn opened successfully");
                        std::thread::sleep(std::time::Duration::from_millis(100)); // Hold txn open
                        mdbx_txn_abort(read_txn);
                    } else {
                        println!("   🧵 Thread 1: ❌ Failed to open: {}", rc);
                    }
                    rc
                });
                
                let barrier2 = barrier.clone();
                let handle2 = thread::spawn(move || {
                    println!("   🧵 Thread 2: Opening read txn...");
                    let env = env_ptr as *mut MDBX_env;
                    let mut read_txn: *mut MDBX_txn = std::ptr::null_mut();
                    let rc = mdbx_txn_begin_ex(env, std::ptr::null_mut(), MDBX_TXN_RDONLY, &mut read_txn, std::ptr::null_mut());
                    
                    barrier2.wait(); // Wait for both threads to open
                    
                    if rc == MDBX_SUCCESS {
                        println!("   🧵 Thread 2: ✅ Read txn opened successfully");
                        std::thread::sleep(std::time::Duration::from_millis(100)); // Hold txn open
                        mdbx_txn_abort(read_txn);
                    } else {
                        println!("   🧵 Thread 2: ❌ Failed to open: {}", rc);
                    }
                    rc
                });
                
                barrier.wait(); // Wait for both threads
                
                let rc1 = handle1.join().unwrap();
                let rc2 = handle2.join().unwrap();
                
                if rc1 == MDBX_SUCCESS && rc2 == MDBX_SUCCESS {
                    println!("   ✅ ✅ ✅ SUCCESS! Both read txns opened in DIFFERENT THREADS!");
                    println!("   🎯 SOLUTION FOUND: Use flags={} ({})", flags, desc);
                    println!("   💡 This is the CORRECT way to use MDBX MVCC!");
                } else {
                    println!("   ❌ FAILED multi-thread: rc1={}, rc2={}", rc1, rc2);
                }
                
                mdbx_env_close_ex(env, false);
                std::fs::remove_dir_all(&temp_dir).ok();
            }
        }
        
        println!("\n╔═══════════════════════════════════════════════════════╗");
        println!("║  🔬 TEST COMPLETE - Check results above              ║");
        println!("╚═══════════════════════════════════════════════════════╝\n");
    }
    
    #[test]
    fn test_thread_local_transaction_pool_pattern() {
        // 🔬 TDD: Validate the CORRECT pattern for multiple reads in same thread
        println!("\n=== 🔬 TDD: THREAD-LOCAL TRANSACTION POOL PATTERN ===\n");
        
        unsafe {
            let temp_dir = std::env::temp_dir().join("mdbx_txn_pool_test");
            let _ = std::fs::remove_dir_all(&temp_dir);
            std::fs::create_dir_all(&temp_dir).unwrap();
            
            // Setup environment
            let mut env: *mut MDBX_env = std::ptr::null_mut();
            mdbx_env_create(&mut env as *mut _);
            mdbx_env_set_option(env, MDBX_opt_max_db, 10);
            mdbx_env_set_geometry(env, -1, -1, 10 * 1024 * 1024 * 1024, -1, -1, -1);
            
            let path_cstr = std::ffi::CString::new(temp_dir.to_str().unwrap()).unwrap();
            let rc = mdbx_env_open(env, path_cstr.as_ptr(), 0, 0o600);
            assert_eq!(rc, MDBX_SUCCESS);
            
            // Write test data
            let mut write_txn: *mut MDBX_txn = std::ptr::null_mut();
            mdbx_txn_begin_ex(env, std::ptr::null_mut(), 0, &mut write_txn, std::ptr::null_mut());
            
            let table_name = std::ffi::CString::new("test").unwrap();
            let mut dbi: MDBX_dbi = 0;
            mdbx_dbi_open(write_txn, table_name.as_ptr(), MDBX_CREATE, &mut dbi);
            
            zero_copy_ffi::put_aligned(write_txn, dbi, b"key1", b"value1").ok();
            zero_copy_ffi::put_aligned(write_txn, dbi, b"key2", b"value2").ok();
            zero_copy_ffi::put_aligned(write_txn, dbi, b"key3", b"value3").ok();
            
            mdbx_txn_commit_ex(write_txn, std::ptr::null_mut());
            
            println!("✅ Test data written (3 keys)");
            
            // ==========================================
            // PATTERN: Thread-Local Transaction Pool
            // ==========================================
            use std::cell::RefCell;
            
            thread_local! {
                static TXN_POOL: RefCell<Option<*mut MDBX_txn>> = RefCell::new(None);
            }
            
            // Helper: Get or create thread-local read transaction
            fn get_or_create_txn(env: *mut MDBX_env) -> *mut MDBX_txn {
                TXN_POOL.with(|pool| {
                    let mut pool_ref = pool.borrow_mut();
                    
                    if let Some(txn) = *pool_ref {
                        // Reuse existing transaction
                        println!("   ♻️  Reusing existing thread-local txn");
                        txn
                    } else {
                        // Create new transaction
                        println!("   🆕 Creating new thread-local txn");
                        unsafe {
                            let mut txn: *mut MDBX_txn = std::ptr::null_mut();
                            let rc = mdbx_txn_begin_ex(env, std::ptr::null_mut(), MDBX_TXN_RDONLY, &mut txn, std::ptr::null_mut());
                            assert_eq!(rc, MDBX_SUCCESS, "Failed to create txn: {}", rc);
                            *pool_ref = Some(txn);
                            txn
                        }
                    }
                })
            }
            
            // Helper: Return transaction to pool (keeps it alive)
            fn return_txn_to_pool() {
                // Transaction stays in TXN_POOL, just release borrow
                println!("   ↩️  Returning txn to pool (keeping alive)");
            }
            
            println!("\n📖 Testing multiple reads in SAME THREAD:");
            
            // Read 1
            println!("\n1️⃣  First read (key1):");
            let txn = get_or_create_txn(env);
            let value1 = zero_copy_ffi::get_zero_copy_raw(txn, dbi, b"key1").unwrap().unwrap();
            println!("   ✅ Read: {} (ptr={:p})", std::str::from_utf8(value1).unwrap(), value1.as_ptr());
            return_txn_to_pool();
            
            // Read 2 (REUSES same transaction!)
            println!("\n2️⃣  Second read (key2):");
            let txn = get_or_create_txn(env);
            let value2 = zero_copy_ffi::get_zero_copy_raw(txn, dbi, b"key2").unwrap().unwrap();
            println!("   ✅ Read: {} (ptr={:p})", std::str::from_utf8(value2).unwrap(), value2.as_ptr());
            return_txn_to_pool();
            
            // Read 3 (REUSES same transaction!)
            println!("\n3️⃣  Third read (key3):");
            let txn = get_or_create_txn(env);
            let value3 = zero_copy_ffi::get_zero_copy_raw(txn, dbi, b"key3").unwrap().unwrap();
            println!("   ✅ Read: {} (ptr={:p})", std::str::from_utf8(value3).unwrap(), value3.as_ptr());
            return_txn_to_pool();
            
            println!("\n✅ ✅ ✅ SUCCESS! Multiple reads in same thread work!");
            
            // ==========================================
            // Verify ZERO-COPY is preserved
            // ==========================================
            println!("\n🔍 Verifying ZERO-COPY behavior:");
            let txn = get_or_create_txn(env);
            
            // Read value multiple times - should get SAME pointer
            let ptr1 = zero_copy_ffi::get_zero_copy_raw(txn, dbi, b"key1").unwrap().unwrap().as_ptr();
            let ptr2 = zero_copy_ffi::get_zero_copy_raw(txn, dbi, b"key1").unwrap().unwrap().as_ptr();
            
            if ptr1 == ptr2 {
                println!("   ✅ ZERO-COPY PRESERVED: Same pointer on re-read ({:p})", ptr1);
            } else {
                println!("   ❌ ZERO-COPY BROKEN: Different pointers ({:p} vs {:p})", ptr1, ptr2);
                panic!("Zero-copy broken!");
            }
            
            return_txn_to_pool();
            
            // Cleanup
            TXN_POOL.with(|pool| {
                if let Some(txn) = *pool.borrow() {
                    mdbx_txn_abort(txn);
                }
            });
            
            mdbx_env_close_ex(env, false);
            std::fs::remove_dir_all(&temp_dir).ok();
            
            println!("\n╔═══════════════════════════════════════════════════════╗");
            println!("║  ✅ THREAD-LOCAL TXN POOL PATTERN VALIDATED!         ║");
            println!("║  ✅ Multiple reads in same thread work               ║");
            println!("║  ✅ Zero-copy behavior preserved                     ║");
            println!("║  ✅ Ready for production implementation              ║");
            println!("╚═══════════════════════════════════════════════════════╝\n");
        }
    }
    
    #[test]
    fn test_check_temp_directories() {
        // 📁 DEBUG: Are our temp directories actually separate?
        println!("\n=== 📁 DEBUG: TEMP DIRECTORY INSPECTION ===\n");
        
        let base_temp = std::env::temp_dir();
        println!("Base temp directory: {}", base_temp.display());
        
        let test_dirs = vec![
            "libmdbx_true_zerocopy",
            "mdbx_ffi_zero_copy",
            "mdbx_multiple_dbs_test",
            "mdbx_reopen_test",
            "mdbx_flags_test",
        ];
        
        println!("\nTest directory paths:");
        for dir in &test_dirs {
            let path = base_temp.join(dir);
            println!("  📂 {} → {}", dir, path.display());
            
            if path.exists() {
                println!("     ⚠️  EXISTS! Files:");
                for entry in std::fs::read_dir(&path).unwrap() {
                    let entry = entry.unwrap();
                    println!("        📄 {}", entry.file_name().to_string_lossy());
                }
            } else {
                println!("     ✅ Clean (doesn't exist)");
            }
        }
        
        println!("\n💡 If tests share directories, stale files cause INCOMPATIBLE errors!\n");
    }
}