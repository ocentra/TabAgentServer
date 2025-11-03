//! Graph indexes for relationship traversal - TRUE ZERO-COPY with transaction guards.
//!
//! **ARCHITECTURE:**
//! - Returns `GraphIndexGuard` that keeps transaction alive
//! - Provides zero-copy iterators over `&str` slices from mmap
//! - NO allocations, NO deserialization - pure pointer access
//!
//! **API:**
//! ```rust
//! let guard = index.get_outgoing("chat_123")?;
//! for edge_id_str in guard.iter_strs() {
//!     // edge_id_str is &str borrowed from mmap - ZERO allocation!
//! }
//! ```
//!
//! **KEY FORMATS:**
//! ```text
//! Outgoing: "out:{node_id}" → Vec<EdgeId> (sorted)
//! Incoming: "in:{node_id}" → Vec<EdgeId> (sorted)
//! ```

use common::{DbError, DbResult, EdgeId};
use common::models::Edge;
use mdbx_sys::{
    MDBX_env, MDBX_txn, MDBX_dbi, MDBX_val,
    MDBX_SUCCESS, MDBX_NOTFOUND,
    mdbx_txn_begin_ex, mdbx_txn_commit_ex, mdbx_txn_abort, mdbx_del,
    MDBX_TXN_RDONLY,
};
use std::ptr;
use std::os::raw::c_void;
use crate::zero_copy_ffi;

/// Graph index - TRUE ZERO-COPY with transaction-backed guards.
pub struct GraphIndex {
    env: *mut MDBX_env,
    outgoing_dbi: MDBX_dbi,
    incoming_dbi: MDBX_dbi,
}

unsafe impl Send for GraphIndex {}
unsafe impl Sync for GraphIndex {}

/// Zero-copy guard for graph index reads.
/// 
/// Holds the transaction alive and provides direct mmap access to EdgeId strings.
/// NO allocations, NO deserialization!
pub struct GraphIndexGuard {
    txn: *mut MDBX_txn,
    archived: &'static rkyv::Archived<Vec<EdgeId>>,
}

impl Drop for GraphIndexGuard {
    fn drop(&mut self) {
        unsafe {
            mdbx_txn_abort(self.txn);
        }
    }
}

impl GraphIndexGuard {
    /// Returns the count of edge IDs - O(1), zero-copy.
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
    pub fn archived(&self) -> &rkyv::Archived<Vec<EdgeId>> {
        self.archived
    }
    
    /// Checks if a specific edge ID exists (by string comparison) - zero-copy.
    pub fn contains_str(&self, edge_id_str: &str) -> bool {
        self.archived.iter().any(|id| id.0.as_str() == edge_id_str)
    }
    
    /// Collects to owned Vec<EdgeId> - USE ONLY WHEN NEEDED (allocates).
    pub fn to_owned(&self) -> DbResult<Vec<EdgeId>> {
        self.archived.iter()
            .map(|id| rkyv::deserialize::<EdgeId, rkyv::rancor::Error>(id)
                .map_err(|e| DbError::Serialization(format!("Failed to deserialize EdgeId: {}", e))))
            .collect::<Result<Vec<_>, _>>()
    }
}

impl GraphIndex {
    /// Creates a new graph index with the given FFI handles.
    pub fn new(env: *mut MDBX_env, outgoing_dbi: MDBX_dbi, incoming_dbi: MDBX_dbi) -> Self {
        Self { env, outgoing_dbi, incoming_dbi }
    }
    
    /// Adds an edge to both outgoing and incoming indexes.
    pub fn add_edge(&self, edge: &Edge) -> DbResult<()> {
        let from_key = format!("out:{}", edge.from_node.0);
        let to_key = format!("in:{}", edge.to_node.0);
        let edge_id = edge.id.clone();
        
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
            
            // Add to outgoing
            if let Err(e) = self.add_to_index(txn, self.outgoing_dbi, &from_key, edge_id.clone()) {
                mdbx_txn_abort(txn);
                return Err(e);
            }
            
            // Add to incoming
            if let Err(e) = self.add_to_index(txn, self.incoming_dbi, &to_key, edge_id) {
                mdbx_txn_abort(txn);
                return Err(e);
            }
            
            let rc = mdbx_txn_commit_ex(txn, ptr::null_mut());
            if rc != MDBX_SUCCESS {
                return Err(DbError::InvalidOperation(format!("Failed to commit: {}", rc)));
            }
            
            Ok(())
        }
    }
    
    /// Removes an edge from both outgoing and incoming indexes.
    pub fn remove_edge(&self, edge: &Edge) -> DbResult<()> {
        let from_key = format!("out:{}", edge.from_node.0);
        let to_key = format!("in:{}", edge.to_node.0);
        let edge_id = edge.id.clone();
        
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
            
            // Remove from outgoing
            if let Err(e) = self.remove_from_index(txn, self.outgoing_dbi, &from_key, edge_id.clone()) {
                mdbx_txn_abort(txn);
                return Err(e);
            }
            
            // Remove from incoming
            if let Err(e) = self.remove_from_index(txn, self.incoming_dbi, &to_key, edge_id) {
                mdbx_txn_abort(txn);
                return Err(e);
            }
            
            let rc = mdbx_txn_commit_ex(txn, ptr::null_mut());
            if rc != MDBX_SUCCESS {
                return Err(DbError::InvalidOperation(format!("Failed to commit: {}", rc)));
            }
            
            Ok(())
        }
    }
    
    /// Returns a zero-copy guard for outgoing edges.
    pub fn get_outgoing(&self, node_id: &str) -> DbResult<Option<GraphIndexGuard>> {
        let key = format!("out:{}", node_id);
        self.get_from_index(true, &key)
    }
    
    /// Returns a zero-copy guard for incoming edges.
    pub fn get_incoming(&self, node_id: &str) -> DbResult<Option<GraphIndexGuard>> {
        let key = format!("in:{}", node_id);
        self.get_from_index(false, &key)
    }
    
    /// Returns the count of outgoing edges - O(1), zero-copy.
    pub fn count_outgoing(&self, node_id: &str) -> DbResult<usize> {
        let key = format!("out:{}", node_id);
        self.count_from_index(true, &key)
    }
    
    /// Returns the count of incoming edges - O(1), zero-copy.
    pub fn count_incoming(&self, node_id: &str) -> DbResult<usize> {
        let key = format!("in:{}", node_id);
        self.count_from_index(false, &key)
    }
    
    // === PRIVATE HELPERS ===
    
    fn add_to_index(&self, txn: *mut MDBX_txn, dbi: MDBX_dbi, key: &str, edge_id: EdgeId) -> DbResult<()> {
        unsafe {
            // Get existing Vec or create new
            let mut vec = match zero_copy_ffi::get_zero_copy::<Vec<EdgeId>>(txn, dbi, key.as_bytes()) {
                Ok(Some(archived)) => {
                    archived.iter()
                        .map(|id| rkyv::deserialize::<EdgeId, rkyv::rancor::Error>(id)
                            .map_err(|e| DbError::Serialization(format!("Deserialize error: {}", e))))
                        .collect::<Result<Vec<_>, _>>()?
                }
                Ok(None) => Vec::new(),
                Err(e) => return Err(DbError::InvalidOperation(format!("Failed to read: {}", e))),
            };
            
            // Binary search insert
            match vec.binary_search(&edge_id) {
                Ok(_) => return Ok(()), // Already exists
                Err(pos) => vec.insert(pos, edge_id),
            }
            
            // Serialize and write back
            let archived_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&vec)
                .map_err(|e| DbError::Serialization(format!("Failed to serialize Vec: {}", e)))?;
            
            zero_copy_ffi::put_aligned(txn, dbi, key.as_bytes(), &archived_bytes)
                .map_err(|e| DbError::InvalidOperation(format!("Failed to write: {}", e)))
        }
    }
    
    fn remove_from_index(&self, txn: *mut MDBX_txn, dbi: MDBX_dbi, key: &str, edge_id: EdgeId) -> DbResult<()> {
        unsafe {
            // Get existing Vec
            let mut vec = match zero_copy_ffi::get_zero_copy::<Vec<EdgeId>>(txn, dbi, key.as_bytes()) {
                Ok(Some(archived)) => {
                    archived.iter()
                        .map(|id| rkyv::deserialize::<EdgeId, rkyv::rancor::Error>(id)
                            .map_err(|e| DbError::Serialization(format!("Deserialize error: {}", e))))
                        .collect::<Result<Vec<_>, _>>()?
                }
                Ok(None) => return Ok(()), // Nothing to remove
                Err(e) => return Err(DbError::InvalidOperation(format!("Failed to read: {}", e))),
            };
            
            // Binary search remove
            if let Ok(pos) = vec.binary_search(&edge_id) {
                vec.remove(pos);
                
                if vec.is_empty() {
                    // Remove key entirely
                    let mut key_val = MDBX_val {
                        iov_base: key.as_ptr() as *mut c_void,
                        iov_len: key.len(),
                    };
                    let rc = mdbx_del(txn, dbi, &mut key_val, ptr::null_mut());
                    if rc != MDBX_SUCCESS && rc != MDBX_NOTFOUND {
                        return Err(DbError::InvalidOperation(format!("Failed to delete key: {}", rc)));
                    }
                } else {
                    // Serialize and write back
                    let archived_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&vec)
                        .map_err(|e| DbError::Serialization(format!("Failed to serialize Vec: {}", e)))?;
                    
                    zero_copy_ffi::put_aligned(txn, dbi, key.as_bytes(), &archived_bytes)
                        .map_err(|e| DbError::InvalidOperation(format!("Failed to write: {}", e)))?;
                }
            }
            
            Ok(())
        }
    }
    
    fn get_from_index(&self, is_outgoing: bool, key: &str) -> DbResult<Option<GraphIndexGuard>> {
        let dbi = if is_outgoing { self.outgoing_dbi } else { self.incoming_dbi };
        
        unsafe {
            let mut txn: *mut MDBX_txn = ptr::null_mut();
            let rc = mdbx_txn_begin_ex(
                self.env,
                ptr::null_mut(),
                MDBX_TXN_RDONLY,
                &mut txn,
                ptr::null_mut(),
            );
            if rc != MDBX_SUCCESS {
                return Err(DbError::InvalidOperation(format!("Failed to begin RO txn: {}", rc)));
            }
            
            // TRUE ZERO-COPY READ!
            match zero_copy_ffi::get_zero_copy::<Vec<EdgeId>>(txn, dbi, key.as_bytes()) {
                Ok(Some(archived)) => {
                    // Extend archived lifetime to 'static (safe because txn is kept alive in guard)
                    let archived_static: &'static rkyv::Archived<Vec<EdgeId>> = 
                        std::mem::transmute(archived);
                    
                    Ok(Some(GraphIndexGuard {
                        txn,
                        archived: archived_static,
                    }))
                }
                Ok(None) => {
                    mdbx_txn_abort(txn);
                    Ok(None)
                }
                Err(e) => {
                    mdbx_txn_abort(txn);
                    Err(DbError::InvalidOperation(format!("Zero-copy get failed: {}", e)))
                }
            }
        }
    }
    
    fn count_from_index(&self, is_outgoing: bool, key: &str) -> DbResult<usize> {
        let dbi = if is_outgoing { self.outgoing_dbi } else { self.incoming_dbi };
        
        unsafe {
            let mut txn: *mut MDBX_txn = ptr::null_mut();
            let rc = mdbx_txn_begin_ex(
                self.env,
                ptr::null_mut(),
                MDBX_TXN_RDONLY,
                &mut txn,
                ptr::null_mut(),
            );
            if rc != MDBX_SUCCESS {
                return Err(DbError::InvalidOperation(format!("Failed to begin RO txn: {}", rc)));
            }
            
            let result = match zero_copy_ffi::get_zero_copy::<Vec<EdgeId>>(txn, dbi, key.as_bytes()) {
                Ok(Some(archived)) => Ok(archived.len()),  // TRUE zero-copy!
                Ok(None) => Ok(0),
                Err(e) => Err(DbError::InvalidOperation(format!("Zero-copy get failed: {}", e))),
            };
            
            mdbx_txn_abort(txn);
            result
        }
    }
}
