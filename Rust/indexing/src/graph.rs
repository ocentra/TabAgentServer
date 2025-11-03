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
use libmdbx::{Database, NoWriteMap, TableFlags};
use std::sync::Arc;
use std::borrow::Cow;

/// Graph index for fast relationship traversal.
pub struct GraphIndex {
    db: Arc<Database<NoWriteMap>>,
    outgoing_table: String,
    incoming_table: String,
}

impl GraphIndex {
    /// Creates a new graph index using the given libmdbx database and table names.
    pub fn new(db: Arc<Database<NoWriteMap>>, outgoing_table: String, incoming_table: String) -> Self {
        Self {
            db,
            outgoing_table,
            incoming_table,
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
                // rkyv 0.8: Use from_bytes (with error type)
                rkyv::from_bytes::<HashSet<EdgeId>, rkyv::rancor::Error>(&bytes)
                    .map_err(|e| DbError::Serialization(format!("rkyv deserialize error: {}", e)))
            })
            .transpose()?
            .unwrap_or_default();
        
        // Add edge ID
        edge_set.insert(EdgeId::from(edge_id));
        
        // Store back with rkyv (0.8: specify error type, returns AlignedVec)
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&edge_set)
            .map_err(|e| DbError::Serialization(format!("rkyv serialize error: {}", e)))?;
        self.set_raw(is_outgoing, key.as_bytes(), bytes.to_vec())?;
        
        Ok(())
    }
    
    fn remove_from_index(&self, is_outgoing: bool, node_id: &str, edge_id: &str) -> DbResult<()> {
        let key = if is_outgoing {
            format!("out:{}", node_id)
        } else {
            format!("in:{}", node_id)
        };
        
        if let Some(bytes) = self.get_raw(is_outgoing, key.as_bytes())? {
            // rkyv 0.8: Use from_bytes (with error type)
            let mut edge_set = rkyv::from_bytes::<HashSet<EdgeId>, rkyv::rancor::Error>(&bytes)
                .map_err(|e| DbError::Serialization(format!("rkyv deserialize error: {}", e)))?;
            
            edge_set.remove(&EdgeId::from(edge_id));
            
            if edge_set.is_empty() {
                self.remove_raw(is_outgoing, key.as_bytes())?;
            } else {
                // rkyv 0.8: specify error type, returns AlignedVec
                let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&edge_set)
                    .map_err(|e| DbError::Serialization(format!("rkyv serialize error: {}", e)))?;
                self.set_raw(is_outgoing, key.as_bytes(), bytes.to_vec())?;
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
                // rkyv 0.8: Use from_bytes (with error type)
                rkyv::from_bytes::<HashSet<EdgeId>, rkyv::rancor::Error>(&bytes)
                    .map_err(|e| DbError::Serialization(format!("rkyv deserialize error: {}", e)))
            })
            .transpose()?
            .unwrap_or_default();
        
        Ok(edge_set.into_iter().collect())
    }
    
    // Internal raw operations using libmdbx 0.6.3 API
    fn get_raw(&self, is_outgoing: bool, key: &[u8]) -> Result<Option<Vec<u8>>, DbError> {
        let table_name = if is_outgoing { &self.outgoing_table } else { &self.incoming_table };
        
        let txn = self.db.begin_ro_txn()
            .map_err(|e| DbError::InvalidOperation(format!("libmdbx ro txn error: {}", e)))?;
        
        let table = txn.open_table(Some(table_name))
            .map_err(|e| DbError::InvalidOperation(format!("libmdbx open table error: {}", e)))?;
        
        match txn.get::<Cow<'_, [u8]>>(&table, key) {
            Ok(Some(data)) => Ok(Some(data.to_vec())),
            Ok(None) => Ok(None),
            Err(e) => Err(DbError::InvalidOperation(format!("libmdbx get error: {}", e))),
        }
    }
    
    fn set_raw(&self, is_outgoing: bool, key: &[u8], value: Vec<u8>) -> Result<(), DbError> {
        let table_name = if is_outgoing { &self.outgoing_table } else { &self.incoming_table };
        
        let txn = self.db.begin_rw_txn()
            .map_err(|e| DbError::InvalidOperation(format!("libmdbx rw txn error: {}", e)))?;
        
        let table = txn.create_table(Some(table_name), TableFlags::empty())
            .map_err(|e| DbError::InvalidOperation(format!("libmdbx create table error: {}", e)))?;
        
        txn.put(&table, key, &value, libmdbx::WriteFlags::empty())
            .map_err(|e| DbError::InvalidOperation(format!("libmdbx put error: {}", e)))?;
        
        txn.commit()
            .map_err(|e| DbError::InvalidOperation(format!("libmdbx commit error: {}", e)))?;
        
        Ok(())
    }
    
    fn remove_raw(&self, is_outgoing: bool, key: &[u8]) -> Result<bool, DbError> {
        let table_name = if is_outgoing { &self.outgoing_table } else { &self.incoming_table };
        
        let txn = self.db.begin_rw_txn()
            .map_err(|e| DbError::InvalidOperation(format!("libmdbx rw txn error: {}", e)))?;
        
        let table = txn.create_table(Some(table_name), TableFlags::empty())
            .map_err(|e| DbError::InvalidOperation(format!("libmdbx create table error: {}", e)))?;
        
        let existed = txn.get::<Cow<'_, [u8]>>(&table, key)
            .map_err(|e| DbError::InvalidOperation(format!("libmdbx get error: {}", e)))?
            .is_some();
        
        txn.del(&table, key, None)
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
        
        // libmdbx 0.6.3: Use Database::open_with_options
        let mut options = libmdbx::DatabaseOptions::default();
        options.max_tables = Some(10);
        
        let db = Database::<NoWriteMap>::open_with_options(temp_dir.path(), options).unwrap();
        
        // Create the tables
        let txn = db.begin_rw_txn().unwrap();
        let _ = txn.create_table(Some("test_graph_out"), TableFlags::empty()).unwrap();
        let _ = txn.create_table(Some("test_graph_in"), TableFlags::empty()).unwrap();
        txn.commit().unwrap();
        
        (GraphIndex::new(Arc::new(db), "test_graph_out".to_string(), "test_graph_in".to_string()), temp_dir)
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
