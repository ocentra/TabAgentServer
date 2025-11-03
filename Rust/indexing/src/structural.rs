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
use libmdbx::{Database, NoWriteMap, TableFlags};
use std::sync::Arc;
use std::borrow::Cow;

/// Structural index for fast property-based queries.
pub struct StructuralIndex {
    db: Arc<Database<NoWriteMap>>,
    table_name: String,
}

impl StructuralIndex {
    /// Creates a new structural index using the given libmdbx database and table name.
    pub fn new(db: Arc<Database<NoWriteMap>>, table_name: String) -> Self {
        Self { db, table_name }
    }
    
    /// Adds a node ID to an index for a specific property value.
    pub fn add(&self, property: &str, value: &str, node_id: &str) -> DbResult<()> {
        let key = format!("prop:{}:{}", property, value);
        
        // Get existing set or create new
        let mut id_set: HashSet<NodeId> = self.get_raw(key.as_bytes())?
            .map(|bytes| {
                // rkyv 0.8: Use from_bytes (with error type)
                rkyv::from_bytes::<HashSet<NodeId>, rkyv::rancor::Error>(&bytes)
                    .map_err(|e| DbError::Serialization(format!("rkyv deserialize error: {}", e)))
            })
            .transpose()?
            .unwrap_or_default();
        
        // Add node ID
        id_set.insert(NodeId::from(node_id));
        
        // Store back with rkyv (0.8: specify error type, returns AlignedVec)
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&id_set)
            .map_err(|e| DbError::Serialization(format!("rkyv serialize error: {}", e)))?;
        self.set_raw(key.as_bytes(), bytes.to_vec())?;
        
        Ok(())
    }
    
    /// Removes a node ID from an index for a specific property value.
    pub fn remove(&self, property: &str, value: &str, node_id: &str) -> DbResult<()> {
        let key = format!("prop:{}:{}", property, value);
        
        // Get existing set
        if let Some(bytes) = self.get_raw(key.as_bytes())? {
            // rkyv 0.8: Use from_bytes (with error type)
            let mut id_set = rkyv::from_bytes::<HashSet<NodeId>, rkyv::rancor::Error>(&bytes)
                .map_err(|e| DbError::Serialization(format!("rkyv deserialize error: {}", e)))?;
            
            // Remove node ID
            id_set.remove(&NodeId::from(node_id));
            
            if id_set.is_empty() {
                // Remove key if set is now empty
                self.remove_raw(key.as_bytes())?;
            } else {
                // Store back with rkyv (0.8: specify error type, returns AlignedVec)
                let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&id_set)
                    .map_err(|e| DbError::Serialization(format!("rkyv serialize error: {}", e)))?;
                self.set_raw(key.as_bytes(), bytes.to_vec())?;
            }
        }
        
        Ok(())
    }
    
    /// Retrieves all node IDs for a specific property value.
    pub fn get(&self, property: &str, value: &str) -> DbResult<Vec<NodeId>> {
        let key = format!("prop:{}:{}", property, value);
        
        let id_set: HashSet<NodeId> = self.get_raw(key.as_bytes())?
            .map(|bytes| {
                // rkyv 0.8: Use from_bytes (with error type)
                rkyv::from_bytes::<HashSet<NodeId>, rkyv::rancor::Error>(&bytes)
                    .map_err(|e| DbError::Serialization(format!("rkyv deserialize error: {}", e)))
            })
            .transpose()?
            .unwrap_or_default();
        
        Ok(id_set.into_iter().collect())
    }
    
    /// Returns the number of nodes indexed for a property value.
    pub fn count(&self, property: &str, value: &str) -> DbResult<usize> {
        let key = format!("prop:{}:{}", property, value);
        
        let id_set: HashSet<NodeId> = self.get_raw(key.as_bytes())?
            .map(|bytes| {
                // rkyv 0.8: Use from_bytes (with error type)
                rkyv::from_bytes::<HashSet<NodeId>, rkyv::rancor::Error>(&bytes)
                    .map_err(|e| DbError::Serialization(format!("rkyv deserialize error: {}", e)))
            })
            .transpose()?
            .unwrap_or_default();
        
        Ok(id_set.len())
    }
    
    /// Clears all indexes for a specific property name.
    pub fn clear_property(&self, property: &str) -> DbResult<usize> {
        let prefix = format!("prop:{}:", property);
        let mut removed = 0;
        
        for item in self.scan_prefix(prefix.as_bytes()) {
            let (key, _) = item?;
            self.remove_raw(&key)?;
            removed += 1;
        }
        
        Ok(removed)
    }
    
    // Internal raw operations using libmdbx 0.6.3 API
    fn get_raw(&self, key: &[u8]) -> Result<Option<Vec<u8>>, DbError> {
        let txn = self.db.begin_ro_txn()
            .map_err(|e| DbError::InvalidOperation(format!("libmdbx ro txn error: {}", e)))?;
        
        let table = txn.open_table(Some(&self.table_name))
            .map_err(|e| DbError::InvalidOperation(format!("libmdbx open table error: {}", e)))?;
        
        match txn.get::<Cow<'_, [u8]>>(&table, key) {
            Ok(Some(data)) => Ok(Some(data.to_vec())),
            Ok(None) => Ok(None),
            Err(e) => Err(DbError::InvalidOperation(format!("libmdbx get error: {}", e))),
        }
    }
    
    fn set_raw(&self, key: &[u8], value: Vec<u8>) -> Result<(), DbError> {
        let txn = self.db.begin_rw_txn()
            .map_err(|e| DbError::InvalidOperation(format!("libmdbx rw txn error: {}", e)))?;
        
        let table = txn.create_table(Some(&self.table_name), TableFlags::empty())
            .map_err(|e| DbError::InvalidOperation(format!("libmdbx create table error: {}", e)))?;
        
        txn.put(&table, key, &value, libmdbx::WriteFlags::empty())
            .map_err(|e| DbError::InvalidOperation(format!("libmdbx put error: {}", e)))?;
        
        txn.commit()
            .map_err(|e| DbError::InvalidOperation(format!("libmdbx commit error: {}", e)))?;
        
        Ok(())
    }
    
    fn remove_raw(&self, key: &[u8]) -> Result<(), DbError> {
        let txn = self.db.begin_rw_txn()
            .map_err(|e| DbError::InvalidOperation(format!("libmdbx rw txn error: {}", e)))?;
        
        let table = txn.create_table(Some(&self.table_name), TableFlags::empty())
            .map_err(|e| DbError::InvalidOperation(format!("libmdbx create table error: {}", e)))?;
        
        txn.del(&table, key, None)
            .map_err(|e| DbError::InvalidOperation(format!("libmdbx del error: {}", e)))?;
        
        txn.commit()
            .map_err(|e| DbError::InvalidOperation(format!("libmdbx commit error: {}", e)))?;
        
        Ok(())
    }
    
    fn scan_prefix(&self, prefix: &[u8]) -> Box<dyn Iterator<Item = Result<(Vec<u8>, Vec<u8>), DbError>>> {
        // Simplified: collect all results upfront to avoid complex lifetime issues
        match self.collect_prefix_results(prefix) {
            Ok(results) => Box::new(results.into_iter().map(Ok)),
            Err(e) => Box::new(std::iter::once(Err(e))),
        }
    }
    
    fn collect_prefix_results(&self, prefix: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>, DbError> {
        let txn = self.db.begin_ro_txn()
            .map_err(|e| DbError::InvalidOperation(format!("libmdbx ro txn error: {}", e)))?;
        
        let table = txn.open_table(Some(&self.table_name))
            .map_err(|e| DbError::InvalidOperation(format!("libmdbx open table error: {}", e)))?;
        
        let mut cursor = txn.cursor(&table)
            .map_err(|e| DbError::InvalidOperation(format!("libmdbx cursor error: {}", e)))?;
        
        let mut results = Vec::new();
        
        // Iterate through all entries and filter by prefix
        for item in cursor.iter::<Cow<'_, [u8]>, Cow<'_, [u8]>>() {
            let (k, v) = item.map_err(|e| DbError::InvalidOperation(format!("libmdbx cursor iter error: {}", e)))?;
            if k.starts_with(prefix) {
                results.push((k.to_vec(), v.to_vec()));
            }
        }
        
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    fn create_test_index() -> (StructuralIndex, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        
        // libmdbx 0.6.3: Use Database::open_with_options
        let mut options = libmdbx::DatabaseOptions::default();
        options.max_tables = Some(10);
        
        let db = Database::<NoWriteMap>::open_with_options(temp_dir.path(), options).unwrap();
        
        // Create the table
        let txn = db.begin_rw_txn().unwrap();
        let _ = txn.create_table(Some("test_structural"), TableFlags::empty()).unwrap();
        txn.commit().unwrap();
        
        (StructuralIndex::new(Arc::new(db), "test_structural".to_string()), temp_dir)
    }
    
    #[test]
    fn test_add_and_get() {
        let (index, _temp) = create_test_index();
        
        index.add("chat_id", "chat_123", "msg_1").unwrap();
        index.add("chat_id", "chat_123", "msg_2").unwrap();
        index.add("chat_id", "chat_456", "msg_3").unwrap();
        
        let results = index.get("chat_id", "chat_123").unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.contains(&NodeId::from("msg_1")));
        assert!(results.contains(&NodeId::from("msg_2")));
        
        let results = index.get("chat_id", "chat_456").unwrap();
        assert_eq!(results.len(), 1);
        assert!(results.contains(&NodeId::from("msg_3")));
    }
    
    #[test]
    fn test_remove() {
        let (index, _temp) = create_test_index();
        
        index.add("chat_id", "chat_123", "msg_1").unwrap();
        index.add("chat_id", "chat_123", "msg_2").unwrap();
        
        index.remove("chat_id", "chat_123", "msg_1").unwrap();
        
        let results = index.get("chat_id", "chat_123").unwrap();
        assert_eq!(results.len(), 1);
        assert!(results.contains(&NodeId::from("msg_2")));
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

