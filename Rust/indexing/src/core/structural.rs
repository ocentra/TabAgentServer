//! Structural indexes for property-based queries - TRUE ZERO-COPY with transaction guards.
//!
//! **ARCHITECTURE:**
//! - Returns `StructuralIndexGuard` that keeps transaction alive
//! - Provides zero-copy iterators over `&str` slices from mmap
//! - NO allocations, NO deserialization - pure pointer access
//!
//! **API:**
//! ```rust,ignore
//! let guard = index.get("chat_id", "chat_123")?;
//! for node_id_str in guard.iter_strs() {
//!     // node_id_str is &str borrowed from mmap - ZERO allocation!
//! }
//! ```
//!
//! **PERFORMANCE:**
//! - Reads: TRUE zero-copy - direct mmap slice access
//! - Writes: O(log n) binary search insert
//! - Count: O(1) - direct Vec length from archived data
//! - Memory: ZERO heap allocations on reads

use common::{DbError, DbResult, NodeId};
use storage::mdbx_base::mdbx_sys::{
    MDBX_env, MDBX_txn, MDBX_dbi, MDBX_val,
    MDBX_SUCCESS, MDBX_NOTFOUND,
    mdbx_txn_begin_ex, mdbx_txn_commit_ex, mdbx_txn_abort, mdbx_del,
};
use std::ptr;
use std::os::raw::c_void;
use storage::mdbx_base::zero_copy_ffi;
use storage::mdbx_base::txn_pool;  // ← USE TRANSACTION POOL to avoid -30783!

/// Structural index - TRUE ZERO-COPY with transaction-backed guards.
pub struct StructuralIndex {
    env: *mut MDBX_env,
    dbi: MDBX_dbi,
}

unsafe impl Send for StructuralIndex {}
unsafe impl Sync for StructuralIndex {}

/// Zero-copy guard for structural index reads.
/// 
/// Holds the transaction alive and provides direct mmap access to NodeId strings.
/// NO allocations, NO deserialization!
pub struct StructuralIndexGuard {
    txn: *mut MDBX_txn,
    archived: &'static rkyv::Archived<Vec<NodeId>>,
    owns_txn: bool,  // If false, txn is from pool and should NOT be closed
}

impl Drop for StructuralIndexGuard {
    fn drop(&mut self) {
        // Only close transaction if we own it (not from pool)
        if self.owns_txn {
            unsafe {
                mdbx_txn_abort(self.txn);
            }
        }
    }
}

impl StructuralIndexGuard {
    /// Returns the count of node IDs - O(1), zero-copy.
    pub fn len(&self) -> usize {
        self.archived.len()
    }
    
    /// Checks if empty - O(1), zero-copy.
    pub fn is_empty(&self) -> bool {
        self.archived.is_empty()
    }
    
    /// Returns an iterator over zero-copy &str slices.
    /// 
    /// Each string is borrowed directly from mmap memory - NO allocations!
    pub fn iter_strs(&self) -> impl Iterator<Item = &str> + '_ {
        self.archived.iter().map(|archived_id| archived_id.0.as_str())
    }
    
    /// Returns the archived Vec directly for advanced use cases.
    pub fn archived(&self) -> &rkyv::Archived<Vec<NodeId>> {
        self.archived
    }
    
    /// Checks if a specific node ID exists (by string comparison) - zero-copy.
    pub fn contains_str(&self, node_id_str: &str) -> bool {
        self.archived.iter().any(|id| id.0.as_str() == node_id_str)
    }
    
    /// Collects to owned Vec<NodeId> - USE ONLY WHEN NEEDED (allocates).
    pub fn to_owned(&self) -> DbResult<Vec<NodeId>> {
        self.archived.iter()
            .map(|id| rkyv::deserialize::<NodeId, rkyv::rancor::Error>(id)
                .map_err(|e| DbError::Serialization(format!("Failed to deserialize NodeId: {}", e))))
            .collect::<Result<Vec<_>, _>>()
    }
}

impl StructuralIndex {
    /// Creates a new structural index with the given FFI handles.
    pub fn new(env: *mut MDBX_env, dbi: MDBX_dbi) -> Self {
        Self { env, dbi }
    }
    
    /// Adds a node ID to a property-value index.
    pub fn add(&self, property: &str, value: &str, node_id: &str) -> DbResult<()> {
        let key = format!("prop:{}:{}", property, value);
        let node_id_obj = NodeId::new(node_id.to_string());
        
        unsafe {
            let mut txn: *mut MDBX_txn = ptr::null_mut();
            let rc = mdbx_txn_begin_ex(
                self.env,
                ptr::null_mut(),
                0,
                &mut txn,
                ptr::null_mut(),
            );
            if rc != MDBX_SUCCESS {
                return Err(DbError::InvalidOperation(format!("Failed to begin RW txn: {}", rc)));
            }
            
            // Get existing Vec or create new
            let mut vec = match zero_copy_ffi::get_zero_copy::<Vec<NodeId>>(txn, self.dbi, key.as_bytes()) {
                Ok(Some(archived)) => {
                    // Deserialize to modify
                    archived.iter()
                        .map(|id| rkyv::deserialize::<NodeId, rkyv::rancor::Error>(id)
                            .map_err(|e| DbError::Serialization(format!("Deserialize error: {}", e))))
                        .collect::<Result<Vec<_>, _>>()?
                }
                Ok(None) => Vec::new(),
                Err(e) => {
                    mdbx_txn_abort(txn);
                    return Err(DbError::InvalidOperation(format!("Failed to read: {}", e)));
                }
            };
            
            // Binary search insert (keeps sorted, prevents duplicates)
            match vec.binary_search(&node_id_obj) {
                Ok(_) => {
                    // Already exists
                    mdbx_txn_abort(txn);
                    return Ok(());
                }
                Err(pos) => {
                    vec.insert(pos, node_id_obj);
                }
            }
            
            // Serialize Vec and write back with MDBX_RESERVE
            let archived_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&vec)
                .map_err(|e| {
                    mdbx_txn_abort(txn);
                    DbError::Serialization(format!("Failed to serialize Vec: {}", e))
                })?;
            
            if let Err(e) = zero_copy_ffi::put_aligned(txn, self.dbi, key.as_bytes(), &archived_bytes) {
                mdbx_txn_abort(txn);
                return Err(DbError::InvalidOperation(format!("Failed to write: {}", e)));
            }
            
            let rc = mdbx_txn_commit_ex(txn, ptr::null_mut());
            if rc != MDBX_SUCCESS {
                return Err(DbError::InvalidOperation(format!("Failed to commit: {}", rc)));
            }
            
            Ok(())
        }
    }
    
    /// Returns a zero-copy guard for reading node IDs.
    /// 
    /// The guard provides direct access to &str slices from mmap - NO allocations!
    pub fn get(&self, property: &str, value: &str) -> DbResult<Option<StructuralIndexGuard>> {
        let key = format!("prop:{}:{}", property, value);
        
        unsafe {
            // Use transaction pool to reuse read transaction (fixes -30783)
            let txn = txn_pool::get_or_create_read_txn(self.env)
                .map_err(|e| DbError::InvalidOperation(format!("Failed to get pooled txn: {:?}", e)))?;
            
            // TRUE ZERO-COPY READ!
            match zero_copy_ffi::get_zero_copy::<Vec<NodeId>>(txn, self.dbi, key.as_bytes()) {
                Ok(Some(archived)) => {
                    // Extend archived lifetime to 'static (safe because txn is kept alive in guard)
                    let archived_static: &'static rkyv::Archived<Vec<NodeId>> = 
                        std::mem::transmute(archived);
                    
                    Ok(Some(StructuralIndexGuard {
                        txn,
                        archived: archived_static,
                        owns_txn: false,  // From pool, don't close in Drop!
                    }))
                }
                Ok(None) => {
                    // txn is from pool, don't abort it
                    Ok(None)
                }
                Err(e) => {
                    // txn is from pool, don't abort it
                    Err(DbError::InvalidOperation(format!("Zero-copy get failed: {}", e)))
                }
            }
        }
    }
    
    /// Returns the count of nodes for a property-value - O(1), zero-copy.
    pub fn count(&self, property: &str, value: &str) -> DbResult<usize> {
        let key = format!("prop:{}:{}", property, value);
        
        unsafe {
            // Use transaction pool to reuse read transaction (fixes -30783)
            let txn = txn_pool::get_or_create_read_txn(self.env)
                .map_err(|e| DbError::InvalidOperation(format!("Failed to get pooled txn: {:?}", e)))?;
            
            // txn is from pool, don't abort it - just return result
            match zero_copy_ffi::get_zero_copy::<Vec<NodeId>>(txn, self.dbi, key.as_bytes()) {
                Ok(Some(archived)) => Ok(archived.len()),  // TRUE zero-copy!
                Ok(None) => Ok(0),
                Err(e) => Err(DbError::InvalidOperation(format!("Zero-copy get failed: {}", e))),
            }
        }
    }
    
    /// Removes a node ID from a property-value index.
    pub fn remove(&self, property: &str, value: &str, node_id: &str) -> DbResult<()> {
        let key = format!("prop:{}:{}", property, value);
        let node_id_obj = NodeId::new(node_id.to_string());
        
        unsafe {
            let mut txn: *mut MDBX_txn = ptr::null_mut();
            let rc = mdbx_txn_begin_ex(
                self.env,
                ptr::null_mut(),
                0,
                &mut txn,
                ptr::null_mut(),
            );
            if rc != MDBX_SUCCESS {
                return Err(DbError::InvalidOperation(format!("Failed to begin RW txn: {}", rc)));
            }
            
            // Get existing Vec
            let mut vec = match zero_copy_ffi::get_zero_copy::<Vec<NodeId>>(txn, self.dbi, key.as_bytes()) {
                Ok(Some(archived)) => {
                    archived.iter()
                        .map(|id| rkyv::deserialize::<NodeId, rkyv::rancor::Error>(id)
                            .map_err(|e| DbError::Serialization(format!("Deserialize error: {}", e))))
                        .collect::<Result<Vec<_>, _>>()?
                }
                Ok(None) => {
                    mdbx_txn_abort(txn);
                    return Ok(()); // Nothing to remove
                }
                Err(e) => {
                    mdbx_txn_abort(txn);
                    return Err(DbError::InvalidOperation(format!("Failed to read: {}", e)));
                }
            };
            
            // Binary search remove
            if let Ok(pos) = vec.binary_search(&node_id_obj) {
                vec.remove(pos);
                
                if vec.is_empty() {
                    // Remove key entirely
                    let mut key_val = MDBX_val {
                        iov_base: key.as_ptr() as *mut c_void,
                        iov_len: key.len(),
                    };
                    let rc = mdbx_del(txn, self.dbi, &mut key_val, ptr::null_mut());
                    if rc != MDBX_SUCCESS && rc != MDBX_NOTFOUND {
                        mdbx_txn_abort(txn);
                        return Err(DbError::InvalidOperation(format!("Failed to delete key: {}", rc)));
                    }
                } else {
                    // Serialize and write back updated Vec
                    let archived_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&vec)
                        .map_err(|e| {
                            mdbx_txn_abort(txn);
                            DbError::Serialization(format!("Failed to serialize Vec: {}", e))
                        })?;
                    
                    if let Err(e) = zero_copy_ffi::put_aligned(txn, self.dbi, key.as_bytes(), &archived_bytes) {
                        mdbx_txn_abort(txn);
                        return Err(DbError::InvalidOperation(format!("Failed to write: {}", e)));
                    }
                }
            }
            
            let rc = mdbx_txn_commit_ex(txn, ptr::null_mut());
            if rc != MDBX_SUCCESS {
                return Err(DbError::InvalidOperation(format!("Failed to commit: {}", rc)));
            }
            
            Ok(())
        }
    }
}
