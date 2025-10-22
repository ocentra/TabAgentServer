//! Structural indexes for property-based queries.
//!
//! This module provides B-tree-based indexes on entity properties, enabling
//! fast filtering without full table scans. Each index maps property values
//! to sets of entity IDs.
//!
//! # Key Format
//!
//! ```text
//! "prop:{property_name}:{value}" → Set<NodeId>
//! ```
//!
//! # Example
//!
//! ```text
//! "prop:chat_id:chat_123" → ["msg_1", "msg_2", "msg_3"]
//! "prop:node_type:Message" → ["msg_1", "msg_2", ...]
//! "prop:sender:user" → ["msg_1", "msg_5", ...]
//! ```

use common::{DbError, DbResult, NodeId};
use std::collections::HashSet;

/// Structural index for fast property-based queries.
pub struct StructuralIndex {
    tree: sled::Tree,
}

impl StructuralIndex {
    /// Creates a new structural index using the given sled tree.
    pub fn new(tree: sled::Tree) -> Self {
        Self { tree }
    }
    
    /// Adds a node ID to an index for a specific property value.
    ///
    /// # Example
    ///
    /// ```
    /// # use indexing::structural::StructuralIndex;
    /// # fn example(index: &StructuralIndex) -> Result<(), Box<dyn std::error::Error>> {
    /// index.add("chat_id", "chat_123", "msg_1")?;
    /// index.add("chat_id", "chat_123", "msg_2")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn add(&self, property: &str, value: &str, node_id: &str) -> DbResult<()> {
        let key = format!("prop:{}:{}", property, value);
        
        // Get existing set or create new
        let mut id_set: HashSet<NodeId> = self.tree
            .get(&key)?
            .map(|bytes| bincode::deserialize(&bytes))
            .transpose()
            .map_err(|e| DbError::Serialization(e.to_string()))?
            .unwrap_or_default();
        
        // Add node ID
        id_set.insert(node_id.to_string());
        
        // Store back
        let serialized = bincode::serialize(&id_set)
            .map_err(|e| DbError::Serialization(e.to_string()))?;
        self.tree.insert(key.as_bytes(), serialized)?;
        
        Ok(())
    }
    
    /// Removes a node ID from an index for a specific property value.
    pub fn remove(&self, property: &str, value: &str, node_id: &str) -> DbResult<()> {
        let key = format!("prop:{}:{}", property, value);
        
        // Get existing set
        if let Some(bytes) = self.tree.get(&key)? {
            let mut id_set: HashSet<NodeId> = bincode::deserialize(&bytes)
                .map_err(|e| DbError::Serialization(e.to_string()))?;
            
            // Remove node ID
            id_set.remove(node_id);
            
            if id_set.is_empty() {
                // Remove key if set is now empty
                self.tree.remove(key.as_bytes())?;
            } else {
                // Store back
                let serialized = bincode::serialize(&id_set)
                    .map_err(|e| DbError::Serialization(e.to_string()))?;
                self.tree.insert(key.as_bytes(), serialized)?;
            }
        }
        
        Ok(())
    }
    
    /// Retrieves all node IDs for a specific property value.
    ///
    /// # Example
    ///
    /// ```
    /// # use indexing::structural::StructuralIndex;
    /// # fn example(index: &StructuralIndex) -> Result<(), Box<dyn std::error::Error>> {
    /// let message_ids = index.get("chat_id", "chat_123")?;
    /// println!("Found {} messages", message_ids.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn get(&self, property: &str, value: &str) -> DbResult<Vec<NodeId>> {
        let key = format!("prop:{}:{}", property, value);
        
        let id_set: HashSet<NodeId> = self.tree
            .get(&key)?
            .map(|bytes| bincode::deserialize(&bytes))
            .transpose()
            .map_err(|e| DbError::Serialization(e.to_string()))?
            .unwrap_or_default();
        
        Ok(id_set.into_iter().collect())
    }
    
    /// Returns the number of nodes indexed for a property value.
    pub fn count(&self, property: &str, value: &str) -> DbResult<usize> {
        let key = format!("prop:{}:{}", property, value);
        
        let id_set: HashSet<NodeId> = self.tree
            .get(&key)?
            .map(|bytes| bincode::deserialize(&bytes))
            .transpose()
            .map_err(|e| DbError::Serialization(e.to_string()))?
            .unwrap_or_default();
        
        Ok(id_set.len())
    }
    
    /// Clears all indexes for a specific property name.
    ///
    /// Useful for rebuilding indexes.
    pub fn clear_property(&self, property: &str) -> DbResult<usize> {
        let prefix = format!("prop:{}:", property);
        let mut removed = 0;
        
        for item in self.tree.scan_prefix(prefix.as_bytes()) {
            let (key, _) = item?;
            self.tree.remove(&key)?;
            removed += 1;
        }
        
        Ok(removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    fn create_test_index() -> (StructuralIndex, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db = sled::open(temp_dir.path()).unwrap();
        let tree = db.open_tree("test_structural").unwrap();
        (StructuralIndex::new(tree), temp_dir)
    }
    
    #[test]
    fn test_add_and_get() {
        let (index, _temp) = create_test_index();
        
        index.add("chat_id", "chat_123", "msg_1").unwrap();
        index.add("chat_id", "chat_123", "msg_2").unwrap();
        index.add("chat_id", "chat_456", "msg_3").unwrap();
        
        let results = index.get("chat_id", "chat_123").unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.contains(&"msg_1".to_string()));
        assert!(results.contains(&"msg_2".to_string()));
        
        let results = index.get("chat_id", "chat_456").unwrap();
        assert_eq!(results.len(), 1);
        assert!(results.contains(&"msg_3".to_string()));
    }
    
    #[test]
    fn test_remove() {
        let (index, _temp) = create_test_index();
        
        index.add("chat_id", "chat_123", "msg_1").unwrap();
        index.add("chat_id", "chat_123", "msg_2").unwrap();
        
        index.remove("chat_id", "chat_123", "msg_1").unwrap();
        
        let results = index.get("chat_id", "chat_123").unwrap();
        assert_eq!(results.len(), 1);
        assert!(results.contains(&"msg_2".to_string()));
    }
    
    #[test]
    fn test_count() {
        let (index, _temp) = create_test_index();
        
        index.add("node_type", "Message", "msg_1").unwrap();
        index.add("node_type", "Message", "msg_2").unwrap();
        index.add("node_type", "Message", "msg_3").unwrap();
        
        let count = index.count("node_type", "Message").unwrap();
        assert_eq!(count, 3);
    }
    
    #[test]
    fn test_nonexistent_property() {
        let (index, _temp) = create_test_index();
        
        let results = index.get("nonexistent", "value").unwrap();
        assert_eq!(results.len(), 0);
    }
    
    #[test]
    fn test_duplicate_add() {
        let (index, _temp) = create_test_index();
        
        index.add("chat_id", "chat_123", "msg_1").unwrap();
        index.add("chat_id", "chat_123", "msg_1").unwrap(); // Duplicate
        
        let results = index.get("chat_id", "chat_123").unwrap();
        assert_eq!(results.len(), 1); // Set deduplicates
    }
}

