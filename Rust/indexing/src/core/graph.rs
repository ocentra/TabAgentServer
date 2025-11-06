//! Graph indexes for relationship traversal with zero-copy MDBX access.
//!
//! Returns `GraphIndexGuard` that keeps transaction alive and provides
//! zero-copy iterators over (EdgeId, target_node) pairs from mmap.
//!
//! ```rust,ignore
//! let guard = index.get_outgoing("chat_123")?;
//! for (edge_id_str, target_node_str) in guard.iter_edges() {
//!     // Both &str borrowed from mmap
//! }
//! ```

use common::{DbError, DbResult, EdgeId, NodeId};
use common::models::Edge;
use storage::mdbx_base::mdbx_sys::{
    MDBX_env, MDBX_txn, MDBX_dbi, MDBX_val,
    mdbx_txn_begin_ex, mdbx_txn_commit_ex, mdbx_txn_abort,
    MDBX_SUCCESS,
};
use std::ffi::c_void;
use std::ptr;
use storage::mdbx_base::zero_copy_ffi;
use storage::mdbx_base::txn_pool;  // ← USE TRANSACTION POOL to avoid -30783!

pub struct GraphIndex {
    env: *mut MDBX_env,
    outgoing_dbi: MDBX_dbi,
    incoming_dbi: MDBX_dbi,
}

unsafe impl Send for GraphIndex {}
unsafe impl Sync for GraphIndex {}

/// Zero-copy guard for graph index reads.
/// 
/// Holds transaction alive and provides mmap access to (EdgeId, NodeId) pairs.
pub struct GraphIndexGuard {
    txn: *mut MDBX_txn,
    archived: &'static rkyv::Archived<Vec<(EdgeId, NodeId)>>,
    owns_txn: bool,  // If false, txn is from pool and should NOT be closed
}

impl Drop for GraphIndexGuard {
    fn drop(&mut self) {
        // Only close transaction if we own it (not from pool)
        if self.owns_txn {
            unsafe {
                mdbx_txn_abort(self.txn);
            }
        }
    }
}

impl GraphIndexGuard {
    /// Returns the count of edges.
    pub fn len(&self) -> usize {
        self.archived.len()
    }
    
    /// Checks if empty.
    pub fn is_empty(&self) -> bool {
        self.archived.is_empty()
    }
    
    /// Returns an iterator over (edge_id, target_node) pairs from mmap.
    pub fn iter_edges(&self) -> impl Iterator<Item = (&str, &str)> + '_ {
        self.archived.iter().map(|tuple| {
            (tuple.0.0.as_str(), tuple.1.0.as_str())
        })
    }
    
    /// Returns an iterator over edge IDs.
    pub fn iter_edge_ids(&self) -> impl Iterator<Item = &str> + '_ {
        self.archived.iter().map(|tuple| tuple.0.0.as_str())
    }
    
    /// Returns an iterator over just target nodes.
    pub fn iter_targets(&self) -> impl Iterator<Item = &str> + '_ {
        self.archived.iter().map(|tuple| tuple.1.0.as_str())
    }
    
    /// Returns the archived Vec directly for advanced use cases.
    pub fn archived(&self) -> &rkyv::Archived<Vec<(EdgeId, NodeId)>> {
        self.archived
    }
    
    /// Checks if a specific edge ID exists.
    pub fn contains_edge(&self, edge_id_str: &str) -> bool {
        self.archived.iter().any(|tuple| tuple.0.0.as_str() == edge_id_str)
    }
    
    /// Collects to owned Vec<EdgeId> for WARM cache.
    pub fn to_owned_edge_ids(&self) -> DbResult<Vec<EdgeId>> {
        self.archived.iter()
            .map(|tuple| rkyv::deserialize::<EdgeId, rkyv::rancor::Error>(&tuple.0)
                .map_err(|e| DbError::Serialization(format!("Failed to deserialize EdgeId: {}", e))))
            .collect::<Result<Vec<_>, _>>()
    }
}

impl GraphIndex {
    /// Creates a new graph index with the given FFI handles.
    pub fn new(env: *mut MDBX_env, outgoing_dbi: MDBX_dbi, incoming_dbi: MDBX_dbi) -> Self {
        Self { env, outgoing_dbi, incoming_dbi }
    }
    
    /// Adds an edge with just node IDs (convenience method).
    ///
    /// Generates an edge ID automatically and creates the edge.
    /// For more control, use `add_edge_with_struct()`.
    pub fn add_edge(&self, from_id: &str, to_id: &str) -> DbResult<EdgeId> {
        // Generate simple edge ID
        let edge_id = EdgeId(format!("{}:{}", from_id, to_id));
        
        let edge = Edge {
            id: edge_id.clone(),
            from_node: NodeId(from_id.to_string()),
            to_node: NodeId(to_id.to_string()),
            edge_type: "generic".to_string(),
            metadata: "{}".to_string(),
            created_at: 0, // Timestamp can be added if needed
        };
        
        self.add_edge_with_struct(&edge)?;
        Ok(edge_id)
    }
    
    /// Adds an edge to both outgoing and incoming indexes.
    pub fn add_edge_with_struct(&self, edge: &Edge) -> DbResult<()> {
        let from_key = format!("out:{}", edge.from_node.0);
        let to_key = format!("in:{}", edge.to_node.0);
        
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
            
            // Add to outgoing: (edge_id, to_node)
            if let Err(e) = self.add_to_index(
                txn, 
                self.outgoing_dbi, 
                &from_key, 
                edge.id.clone(), 
                edge.to_node.clone()
            ) {
                mdbx_txn_abort(txn);
                return Err(e);
            }
            
            // Add to incoming: (edge_id, from_node)
            if let Err(e) = self.add_to_index(
                txn, 
                self.incoming_dbi, 
                &to_key, 
                edge.id.clone(), 
                edge.from_node.clone()
            ) {
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
            if let Err(e) = self.remove_from_index(txn, self.outgoing_dbi, &from_key, edge.id.clone()) {
                mdbx_txn_abort(txn);
                return Err(e);
            }
            
            // Remove from incoming
            if let Err(e) = self.remove_from_index(txn, self.incoming_dbi, &to_key, edge.id.clone()) {
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
    
    /// Returns the count of outgoing edges.
    pub fn count_outgoing(&self, node_id: &str) -> DbResult<usize> {
        let key = format!("out:{}", node_id);
        self.count_from_index(true, &key)
    }
    
    /// Returns the count of incoming edges.
    pub fn count_incoming(&self, node_id: &str) -> DbResult<usize> {
        let key = format!("in:{}", node_id);
        self.count_from_index(false, &key)
    }
    
    /// Adds a node to the graph (no-op for GraphIndex as nodes are implicit).
    ///
    /// In GraphIndex, nodes are implicitly created when edges are added.
    /// This method exists for API compatibility with graph generators/operators.
    pub fn add_node(&self, _node_id: &str) -> DbResult<()> {
        // Nodes are implicit - they exist when they have edges
        // No storage needed for isolated nodes in this design
        Ok(())
    }
    
    /// Gets all node IDs in the graph by iterating outgoing edge keys.
    ///
    /// Returns all unique node IDs that have outgoing edges.
    /// **Note:** Isolated nodes (no edges) are not tracked.
    pub fn get_all_nodes(&self) -> DbResult<Vec<String>> {
        use storage::mdbx_base::mdbx_sys::{mdbx_cursor_open, mdbx_cursor_get, mdbx_cursor_close, MDBX_cursor};
        use storage::mdbx_base::mdbx_sys::{MDBX_FIRST, MDBX_NEXT};
        use std::collections::HashSet;
        
        unsafe {
            // Use transaction pool to reuse read transaction (fixes -30783)
            let txn = txn_pool::get_or_create_read_txn(self.env)
                .map_err(|e| DbError::InvalidOperation(format!("Failed to get pooled txn: {:?}", e)))?;
            
            let mut cursor: *mut MDBX_cursor = ptr::null_mut();
            let rc = mdbx_cursor_open(txn, self.outgoing_dbi, &mut cursor);
            if rc != MDBX_SUCCESS {
                // txn is from pool, don't abort it
                return Err(DbError::InvalidOperation(format!("Failed to open cursor: {}", rc)));
            }
            
            let mut nodes = HashSet::new();
            let mut key = MDBX_val { iov_base: ptr::null_mut(), iov_len: 0 };
            let mut data = MDBX_val { iov_base: ptr::null_mut(), iov_len: 0 };
            
            // Iterate all keys in outgoing_dbi
            let mut rc = mdbx_cursor_get(cursor, &mut key, &mut data, MDBX_FIRST);
            while rc == MDBX_SUCCESS {
                // Extract key as string
                let key_slice = std::slice::from_raw_parts(
                    key.iov_base as *const u8,
                    key.iov_len
                );
                
                if let Ok(key_str) = std::str::from_utf8(key_slice) {
                    // Keys are "out:node_id" format
                    if let Some(node_id) = key_str.strip_prefix("out:") {
                        nodes.insert(node_id.to_string());
                    }
                }
                
                rc = mdbx_cursor_get(cursor, &mut key, &mut data, MDBX_NEXT);
            }
            
            mdbx_cursor_close(cursor);
            // txn is from pool, don't abort it
            
            Ok(nodes.into_iter().collect())
        }
    }
    
    /// Gets outgoing edges for a node as owned Vec.
    ///
    /// This is a convenience wrapper that deserializes the edges.
    /// For zero-copy access, use `get_outgoing()` instead.
    pub fn get_outgoing_edges(&self, node_id: &str) -> DbResult<Vec<(NodeId, EdgeId)>> {
        if let Some(guard) = self.get_outgoing(node_id)? {
            let edges: Vec<(NodeId, EdgeId)> = guard.iter_edges()
                .map(|(edge_id_str, target_str)| {
                    (NodeId(target_str.to_string()), EdgeId(edge_id_str.to_string()))
                })
                .collect();
            Ok(edges)
        } else {
            Ok(Vec::new())
        }
    }
    
    /// Gets incoming edges for a node as owned Vec.
    ///
    /// This is a convenience wrapper that deserializes the edges.
    /// For zero-copy access, use `get_incoming()` instead.
    pub fn get_incoming_edges(&self, node_id: &str) -> DbResult<Vec<(NodeId, EdgeId)>> {
        if let Some(guard) = self.get_incoming(node_id)? {
            let edges: Vec<(NodeId, EdgeId)> = guard.iter_edges()
                .map(|(edge_id_str, source_str)| {
                    (NodeId(source_str.to_string()), EdgeId(edge_id_str.to_string()))
                })
                .collect();
            Ok(edges)
        } else {
            Ok(Vec::new())
        }
    }
    
    // === PRIVATE HELPERS ===
    
    fn add_to_index(
        &self, 
        txn: *mut MDBX_txn, 
        dbi: MDBX_dbi, 
        key: &str, 
        edge_id: EdgeId,
        target_node: NodeId,
    ) -> DbResult<()> {
        unsafe {
            let mut vec = match zero_copy_ffi::get_zero_copy::<Vec<(EdgeId, NodeId)>>(txn, dbi, key.as_bytes()) {
                Ok(Some(archived)) => {
                    rkyv::deserialize::<Vec<(EdgeId, NodeId)>, rkyv::rancor::Error>(archived)
                        .map_err(|e| DbError::Serialization(format!("Deserialize error: {}", e)))?
                }
                Ok(None) => Vec::new(),
                Err(e) => return Err(DbError::InvalidOperation(format!("Failed to read: {}", e))),
            };
            
            match vec.binary_search_by_key(&&edge_id, |(id, _)| id) {
                Ok(_) => return Ok(()),
                Err(pos) => vec.insert(pos, (edge_id, target_node)),
            }
            
            let archived_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&vec)
                .map_err(|e| DbError::Serialization(format!("Failed to serialize Vec: {}", e)))?;
            
            zero_copy_ffi::put_aligned(txn, dbi, key.as_bytes(), &archived_bytes)
                .map_err(|e| DbError::InvalidOperation(format!("Failed to write: {}", e)))
        }
    }
    
    fn remove_from_index(&self, txn: *mut MDBX_txn, dbi: MDBX_dbi, key: &str, edge_id: EdgeId) -> DbResult<()> {
        unsafe {
            let mut vec = match zero_copy_ffi::get_zero_copy::<Vec<(EdgeId, NodeId)>>(txn, dbi, key.as_bytes()) {
                Ok(Some(archived)) => {
                    rkyv::deserialize::<Vec<(EdgeId, NodeId)>, rkyv::rancor::Error>(archived)
                        .map_err(|e| DbError::Serialization(format!("Deserialize error: {}", e)))?
                }
                Ok(None) => return Ok(()),
                Err(e) => return Err(DbError::InvalidOperation(format!("Failed to read: {}", e))),
            };
            
            if let Ok(pos) = vec.binary_search_by_key(&&edge_id, |(id, _)| id) {
                vec.remove(pos);
                
                if vec.is_empty() {
                    let mut key_val = MDBX_val {
                        iov_base: key.as_ptr() as *mut c_void,
                        iov_len: key.len(),
                    };
                    let rc = storage::mdbx_base::mdbx_sys::mdbx_del(txn, dbi, &mut key_val, ptr::null_mut());
                    if rc != MDBX_SUCCESS && rc != storage::mdbx_base::mdbx_sys::MDBX_NOTFOUND {
                        return Err(DbError::InvalidOperation(format!("Failed to delete key: {}", rc)));
                    }
                } else {
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
            // Use transaction pool to reuse read transaction (fixes -30783)
            let txn = txn_pool::get_or_create_read_txn(self.env)
                .map_err(|e| DbError::InvalidOperation(format!("Failed to get pooled txn: {:?}", e)))?;
            
            match zero_copy_ffi::get_zero_copy::<Vec<(EdgeId, NodeId)>>(txn, dbi, key.as_bytes()) {
                Ok(Some(archived)) => {
                    let archived_static: &'static rkyv::Archived<Vec<(EdgeId, NodeId)>> = 
                        std::mem::transmute(archived);
                    
                    Ok(Some(GraphIndexGuard {
                        txn,
                        archived: archived_static,
                        owns_txn: false,  // From pool, don't close in Drop!
                    }))
                }
                Ok(None) => {
                    // txn is from pool, don't abort it - just return None
                    Ok(None)
                }
                Err(e) => {
                    // txn is from pool, don't abort it - just return error
                    Err(DbError::InvalidOperation(format!("Zero-copy get failed: {}", e)))
                }
            }
        }
    }
    
    fn count_from_index(&self, is_outgoing: bool, key: &str) -> DbResult<usize> {
        let dbi = if is_outgoing { self.outgoing_dbi } else { self.incoming_dbi };
        
        unsafe {
            // Use transaction pool to reuse read transaction (fixes -30783)
            let txn = txn_pool::get_or_create_read_txn(self.env)
                .map_err(|e| DbError::InvalidOperation(format!("Failed to get pooled txn: {:?}", e)))?;
            
            // txn is from pool, don't abort it - just return result
            match zero_copy_ffi::get_zero_copy::<Vec<(EdgeId, NodeId)>>(txn, dbi, key.as_bytes()) {
                Ok(Some(archived)) => Ok(archived.len()),
                Ok(None) => Ok(0),
                Err(e) => Err(DbError::InvalidOperation(format!("Zero-copy get failed: {}", e))),
            }
        }
    }
}
