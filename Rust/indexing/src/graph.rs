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

/// Graph index for fast relationship traversal.
pub struct GraphIndex {
    outgoing_tree: sled::Tree,
    incoming_tree: sled::Tree,
}

impl GraphIndex {
    /// Creates a new graph index using the given sled trees.
    pub fn new(outgoing_tree: sled::Tree, incoming_tree: sled::Tree) -> Self {
        Self {
            outgoing_tree,
            incoming_tree,
        }
    }
    
    /// Adds an edge to both the outgoing and incoming indexes.
    ///
    /// # Example
    ///
    /// ```
    /// # use indexing::graph::GraphIndex;
    /// # use common::models::Edge;
    /// # use serde_json::json;
    /// # fn example(index: &GraphIndex) -> Result<(), Box<dyn std::error::Error>> {
    /// let edge = Edge {
    ///     id: "edge_1".to_string(),
    ///     from_node: "chat_1".to_string(),
    ///     to_node: "msg_1".to_string(),
    ///     edge_type: "CONTAINS_MESSAGE".to_string(),
    ///     created_at: 1697500000000,
    ///     metadata: json!({}),
    /// };
    /// index.add_edge(&edge)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn add_edge(&self, edge: &Edge) -> DbResult<()> {
        // Add to outgoing index (convert newtypes to &str)
        self.add_to_index(&self.outgoing_tree, edge.from_node.as_str(), edge.id.as_str())?;
        
        // Add to incoming index
        self.add_to_index(&self.incoming_tree, edge.to_node.as_str(), edge.id.as_str())?;
        
        Ok(())
    }
    
    /// Removes an edge from both indexes.
    pub fn remove_edge(&self, edge: &Edge) -> DbResult<()> {
        // Remove from outgoing index (convert newtypes to &str)
        self.remove_from_index(&self.outgoing_tree, edge.from_node.as_str(), edge.id.as_str())?;
        
        // Remove from incoming index
        self.remove_from_index(&self.incoming_tree, edge.to_node.as_str(), edge.id.as_str())?;
        
        Ok(())
    }
    
    /// Retrieves all outgoing edge IDs from a node.
    ///
    /// # Example
    ///
    /// ```
    /// # use indexing::graph::GraphIndex;
    /// # fn example(index: &GraphIndex) -> Result<(), Box<dyn std::error::Error>> {
    /// let outgoing = index.get_outgoing("chat_1")?;
    /// println!("Chat has {} outgoing edges", outgoing.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_outgoing(&self, node_id: &str) -> DbResult<Vec<EdgeId>> {
        self.get_from_index(&self.outgoing_tree, node_id)
    }
    
    /// Retrieves all incoming edge IDs to a node.
    pub fn get_incoming(&self, node_id: &str) -> DbResult<Vec<EdgeId>> {
        self.get_from_index(&self.incoming_tree, node_id)
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
    ///
    /// Useful when deleting a node from the graph.
    pub fn remove_all_edges(&self, node_id: &str) -> DbResult<usize> {
        let mut removed = 0;
        
        // Remove outgoing
        let out_key = format!("out:{}", node_id);
        if self.outgoing_tree.remove(out_key.as_bytes())?.is_some() {
            removed += 1;
        }
        
        // Remove incoming
        let in_key = format!("in:{}", node_id);
        if self.incoming_tree.remove(in_key.as_bytes())?.is_some() {
            removed += 1;
        }
        
        Ok(removed)
    }
    
    // Helper methods
    
    fn add_to_index(&self, tree: &sled::Tree, node_id: &str, edge_id: &str) -> DbResult<()> {
        // Use tree name to determine if it's outgoing or incoming
        let tree_name = tree.name();
        let key = if tree_name.ends_with(b"_out") {
            format!("out:{}", node_id)
        } else {
            format!("in:{}", node_id)
        };
        
        // Get existing set or create new
        let mut edge_set: HashSet<EdgeId> = tree
            .get(&key)?
            .map(|bytes| bincode::deserialize(&bytes))
            .transpose()
            .map_err(|e| DbError::Serialization(e.to_string()))?
            .unwrap_or_default();
        
        // Add edge ID (convert &str to EdgeId)
        edge_set.insert(EdgeId::from(edge_id));
        
        // Store back
        let serialized = bincode::serialize(&edge_set)
            .map_err(|e| DbError::Serialization(e.to_string()))?;
        tree.insert(key.as_bytes(), serialized)?;
        
        Ok(())
    }
    
    fn remove_from_index(&self, tree: &sled::Tree, node_id: &str, edge_id: &str) -> DbResult<()> {
        let tree_name = tree.name();
        let key = if tree_name.ends_with(b"_out") {
            format!("out:{}", node_id)
        } else {
            format!("in:{}", node_id)
        };
        
        if let Some(bytes) = tree.get(&key)? {
            let mut edge_set: HashSet<EdgeId> = bincode::deserialize(&bytes)
                .map_err(|e| DbError::Serialization(e.to_string()))?;
            
            edge_set.remove(&EdgeId::from(edge_id));
            
            if edge_set.is_empty() {
                tree.remove(key.as_bytes())?;
            } else {
                let serialized = bincode::serialize(&edge_set)
                    .map_err(|e| DbError::Serialization(e.to_string()))?;
                tree.insert(key.as_bytes(), serialized)?;
            }
        }
        
        Ok(())
    }
    
    fn get_from_index(&self, tree: &sled::Tree, node_id: &str) -> DbResult<Vec<EdgeId>> {
        let tree_name = tree.name();
        let key = if tree_name.ends_with(b"_out") {
            format!("out:{}", node_id)
        } else {
            format!("in:{}", node_id)
        };
        
        let edge_set: HashSet<EdgeId> = tree
            .get(&key)?
            .map(|bytes| bincode::deserialize(&bytes))
            .transpose()
            .map_err(|e| DbError::Serialization(e.to_string()))?
            .unwrap_or_default();
        
        Ok(edge_set.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use serde_json::json;
    use common::{NodeId, EdgeId};
    use common::models::Edge;
    
    fn create_test_index() -> (GraphIndex, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db = sled::open(temp_dir.path()).unwrap();
        let outgoing = db.open_tree("test_graph_out").unwrap();
        let incoming = db.open_tree("test_graph_in").unwrap();
        (GraphIndex::new(outgoing, incoming), temp_dir)
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

