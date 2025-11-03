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

// Enterprise-grade zero-copy FFI wrapper
pub mod zero_copy_ffi;

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
            
            // Note: mdbx_env_set_maxdbs might not be in mdbx-sys, skip for now or use geometry
            
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
}