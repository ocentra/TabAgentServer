//! Graph indexes for relationship traversal.
//!
//! This module provides adjacency list indexes for efficient graph traversal.
//! Two separate trees maintain outgoing and incoming edges for each node.
//!
//! # Key Formats
//!
//! ```text
//! Outgoing: "out:{node_id}" → Set<EdgeId>
//! Incoming: "in:{node_id}" → Set<EdgeId>
//! ```
//!
//! # Example
//!
//! ```text
//! Edge: chat_1 --[CONTAINS_MESSAGE]--> msg_1
//!
//! outgoing["out:chat_1"] = ["edge_1"]
//! incoming["in:msg_1"] = ["edge_1"]
//! ```

use common::{DbError, DbResult, EdgeId};
use std::collections::HashSet;
use common::models::Edge;
use libmdbx::{Environment, NoWriteMap, Database, Transaction as MdbxTxn};
use dashmap::DashMap;
use std::sync::Arc;

/// Graph index for fast relationship traversal.
pub struct GraphIndex {
    env: Arc<Environment<NoWriteMap>>,
    outgoing_db: Database,
    incoming_db: Database,
    _databases: Arc<DashMap<String, Database>>,
}

impl GraphIndex {
    /// Creates a new graph index using the given libmdbx environment and databases.
    pub fn new(env: Arc<Environment<NoWriteMap>>, outgoing_db: Database, incoming_db: Database, databases: Arc<DashMap<String, Database>>) -> Self {
        Self {
            env,
            outgoing_db,
            incoming_db,
            _databases: databases,
        }
    }
    
    /// Adds an edge to both the outgoing and incoming indexes.
    pub fn add_edge(&self, edge: &Edge) -> DbResult<()> {
        // Add to outgoing index
        self.add_to_index(true, edge.from_node.as_str(), edge.id.as_str())?;
        
        // Add to incoming index
        self.add_to_index(false, edge.to_node.as_str(), edge.id.as_str())?;
        
        Ok(())
    }
    
    /// Removes an edge from both indexes.
    pub fn remove_edge(&self, edge: &Edge) -> DbResult<()> {
        // Remove from outgoing index
        self.remove_from_index(true, edge.from_node.as_str(), edge.id.as_str())?;
        
        // Remove from incoming index
        self.remove_from_index(false, edge.to_node.as_str(), edge.id.as_str())?;
        
        Ok(())
    }
    
    /// Retrieves all outgoing edge IDs from a node.
    pub fn get_outgoing(&self, node_id: &str) -> DbResult<Vec<EdgeId>> {
        self.get_from_index(true, node_id)
    }
    
    /// Retrieves all incoming edge IDs to a node.
    pub fn get_incoming(&self, node_id: &str) -> DbResult<Vec<EdgeId>> {
        self.get_from_index(false, node_id)
    }
    
    /// Returns the number of outgoing edges from a node.
    pub fn count_outgoing(&self, node_id: &str) -> DbResult<usize> {
        Ok(self.get_outgoing(node_id)?.len())
    }
    
    /// Returns the number of incoming edges to a node.
    pub fn count_incoming(&self, node_id: &str) -> DbResult<usize> {
        Ok(self.get_incoming(node_id)?.len())
    }
    
    /// Removes all edges (both incoming and outgoing) for a node.
    pub fn remove_all_edges(&self, node_id: &str) -> DbResult<usize> {
        let mut removed = 0;
        
        // Remove outgoing
        let out_key = format!("out:{}", node_id);
        if self.remove_raw(true, out_key.as_bytes())? {
            removed += 1;
        }
        
        // Remove incoming
        let in_key = format!("in:{}", node_id);
        if self.remove_raw(false, in_key.as_bytes())? {
            removed += 1;
        }
        
        Ok(removed)
    }
    
    // Helper methods
    
    fn add_to_index(&self, is_outgoing: bool, node_id: &str, edge_id: &str) -> DbResult<()> {
        let key = if is_outgoing {
            format!("out:{}", node_id)
        } else {
            format!("in:{}", node_id)
        };
        
        // Get existing set or create new
        let mut edge_set: HashSet<EdgeId> = self.get_raw(is_outgoing, key.as_bytes())?
            .map(|bytes| {
                let archived = rkyv::check_archived_root::<HashSet<EdgeId>>(&bytes)
                    .map_err(|e| DbError::Serialization(format!("rkyv check error: {}", e)))?;
                archived.deserialize(&mut rkyv::Infallible)
                    .map_err(|e| DbError::Serialization(format!("rkyv deserialize error: {}", e)))
            })
            .transpose()?
            .unwrap_or_default();
        
        // Add edge ID
        edge_set.insert(EdgeId::from(edge_id));
        
        // Store back with rkyv
        let bytes = rkyv::to_bytes::<_, 256>(&edge_set)
            .map_err(|e| DbError::Serialization(format!("rkyv serialize error: {}", e)))?;
        self.set_raw(is_outgoing, key.as_bytes(), bytes)?;
        
        Ok(())
    }
    
    fn remove_from_index(&self, is_outgoing: bool, node_id: &str, edge_id: &str) -> DbResult<()> {
        let key = if is_outgoing {
            format!("out:{}", node_id)
        } else {
            format!("in:{}", node_id)
        };
        
        if let Some(bytes) = self.get_raw(is_outgoing, key.as_bytes())? {
            let archived = rkyv::check_archived_root::<HashSet<EdgeId>>(&bytes)
                .map_err(|e| DbError::Serialization(format!("rkyv check error: {}", e)))?;
            let mut edge_set = archived.deserialize(&mut rkyv::Infallible)
                .map_err(|e| DbError::Serialization(format!("rkyv deserialize error: {}", e)))?;
            
            edge_set.remove(&EdgeId::from(edge_id));
            
            if edge_set.is_empty() {
                self.remove_raw(is_outgoing, key.as_bytes())?;
            } else {
                let bytes = rkyv::to_bytes::<_, 256>(&edge_set)
                    .map_err(|e| DbError::Serialization(format!("rkyv serialize error: {}", e)))?;
                self.set_raw(is_outgoing, key.as_bytes(), bytes)?;
            }
        }
        
        Ok(())
    }
    
    fn get_from_index(&self, is_outgoing: bool, node_id: &str) -> DbResult<Vec<EdgeId>> {
        let key = if is_outgoing {
            format!("out:{}", node_id)
        } else {
            format!("in:{}", node_id)
        };
        
        let edge_set: HashSet<EdgeId> = self.get_raw(is_outgoing, key.as_bytes())?
            .map(|bytes| {
                let archived = rkyv::check_archived_root::<HashSet<EdgeId>>(&bytes)
                    .map_err(|e| DbError::Serialization(format!("rkyv check error: {}", e)))?;
                archived.deserialize(&mut rkyv::Infallible)
                    .map_err(|e| DbError::Serialization(format!("rkyv deserialize error: {}", e)))
            })
            .transpose()?
            .unwrap_or_default();
        
        Ok(edge_set.into_iter().collect())
    }
    
    // Internal raw operations using libmdbx
    fn get_raw(&self, is_outgoing: bool, key: &[u8]) -> Result<Option<Vec<u8>>, DbError> {
        let db = if is_outgoing { &self.outgoing_db } else { &self.incoming_db };
        let txn = self.env.begin_ro_txn()
            .map_err(|e| DbError::InvalidOperation(format!("libmdbx ro txn error: {}", e)))?;
        
        match txn.get(db, key) {
            Ok(Some(data)) => Ok(Some(data.to_vec())),
            Ok(None) => Ok(None),
            Err(e) => Err(DbError::InvalidOperation(format!("libmdbx get error: {}", e))),
        }
    }
    
    fn set_raw(&self, is_outgoing: bool, key: &[u8], value: Vec<u8>) -> Result<(), DbError> {
        let db = if is_outgoing { &self.outgoing_db } else { &self.incoming_db };
        let mut txn = self.env.begin_rw_txn()
            .map_err(|e| DbError::InvalidOperation(format!("libmdbx rw txn error: {}", e)))?;
        
        txn.put(db, key, &value, libmdbx::WriteFlags::empty())
            .map_err(|e| DbError::InvalidOperation(format!("libmdbx put error: {}", e)))?;
        
        txn.commit()
            .map_err(|e| DbError::InvalidOperation(format!("libmdbx commit error: {}", e)))?;
        
        Ok(())
    }
    
    fn remove_raw(&self, is_outgoing: bool, key: &[u8]) -> Result<bool, DbError> {
        let db = if is_outgoing { &self.outgoing_db } else { &self.incoming_db };
        let mut txn = self.env.begin_rw_txn()
            .map_err(|e| DbError::InvalidOperation(format!("libmdbx rw txn error: {}", e)))?;
        
        let existed = txn.get(db, key)
            .map_err(|e| DbError::InvalidOperation(format!("libmdbx get error: {}", e)))?
            .is_some();
        
        txn.del(db, key, None)
            .map_err(|e| DbError::InvalidOperation(format!("libmdbx del error: {}", e)))?;
        
        txn.commit()
            .map_err(|e| DbError::InvalidOperation(format!("libmdbx commit error: {}", e)))?;
        
        Ok(existed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use serde_json::json;
    use common::{NodeId, EdgeId};
    
    fn create_test_index() -> (GraphIndex, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let env = Environment::<NoWriteMap>::new()
            .set_max_dbs(8)
            .set_geometry(libmdbx::Geometry {
                size: Some(0..(1024 * 1024 * 1024)),
                growth_step: Some(1024 * 1024 * 1024),
                shrink_threshold: None,
                page_size: None,
            })
            .open(temp_dir.path())
            .unwrap();
        
        let outgoing_db = env.create_db(Some("test_graph_out"), libmdbx::DatabaseFlags::empty()).unwrap();
        let incoming_db = env.create_db(Some("test_graph_in"), libmdbx::DatabaseFlags::empty()).unwrap();
        let databases = Arc::new(DashMap::new());
        
        (GraphIndex::new(Arc::new(env), outgoing_db, incoming_db, databases), temp_dir)
    }
    
    fn create_test_edge(id: &str, from: &str, to: &str) -> Edge {
        Edge {
            id: EdgeId::from(id),
            from_node: NodeId::from(from),
            to_node: NodeId::from(to),
            edge_type: "TEST".to_string(),
            created_at: 1697500000000,
            metadata: json!({}),
        }
    }
    
    #[test]
    fn test_add_and_get_outgoing() {
        let (index, _temp) = create_test_index();
        
        let edge1 = create_test_edge("e1", "chat_1", "msg_1");
        let edge2 = create_test_edge("e2", "chat_1", "msg_2");
        
        index.add_edge(&edge1).unwrap();
        index.add_edge(&edge2).unwrap();
        
        let outgoing = index.get_outgoing("chat_1").unwrap();
        assert_eq!(outgoing.len(), 2);
        assert!(outgoing.contains(&EdgeId::from("e1")));
        assert!(outgoing.contains(&EdgeId::from("e2")));
    }
    
    #[test]
    fn test_add_and_get_incoming() {
        let (index, _temp) = create_test_index();
        
        let edge1 = create_test_edge("e1", "chat_1", "msg_1");
        let edge2 = create_test_edge("e2", "chat_2", "msg_1");
        
        index.add_edge(&edge1).unwrap();
        index.add_edge(&edge2).unwrap();
        
        let incoming = index.get_incoming("msg_1").unwrap();
        assert_eq!(incoming.len(), 2);
        assert!(incoming.contains(&EdgeId::from("e1")));
        assert!(incoming.contains(&EdgeId::from("e2")));
    }
    
    #[test]
    fn test_bidirectional_index() {
        let (index, _temp) = create_test_index();
        
        let edge = create_test_edge("e1", "chat_1", "msg_1");
        index.add_edge(&edge).unwrap();
        
        // Check outgoing from chat
        let outgoing = index.get_outgoing("chat_1").unwrap();
        assert_eq!(outgoing.len(), 1);
        
        // Check incoming to message
        let incoming = index.get_incoming("msg_1").unwrap();
        assert_eq!(incoming.len(), 1);
    }
    
    #[test]
    fn test_remove_edge() {
        let (index, _temp) = create_test_index();
        
        let edge = create_test_edge("e1", "chat_1", "msg_1");
        index.add_edge(&edge).unwrap();
        index.remove_edge(&edge).unwrap();
        
        let outgoing = index.get_outgoing("chat_1").unwrap();
        assert_eq!(outgoing.len(), 0);
        
        let incoming = index.get_incoming("msg_1").unwrap();
        assert_eq!(incoming.len(), 0);
    }
    
    #[test]
    fn test_count() {
        let (index, _temp) = create_test_index();
        
        let edge1 = create_test_edge("e1", "chat_1", "msg_1");
        let edge2 = create_test_edge("e2", "chat_1", "msg_2");
        
        index.add_edge(&edge1).unwrap();
        index.add_edge(&edge2).unwrap();
        
        assert_eq!(index.count_outgoing("chat_1").unwrap(), 2);
        assert_eq!(index.count_incoming("msg_1").unwrap(), 1);
    }
}
