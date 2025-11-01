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
use libmdbx::{Environment, NoWriteMap, Database, Transaction as MdbxTxn};
use dashmap::DashMap;
use std::sync::Arc;

/// Structural index for fast property-based queries.
pub struct StructuralIndex {
    env: Arc<Environment<NoWriteMap>>,
    db: Database,
    _databases: Arc<DashMap<String, Database>>, // Keep the databases map alive
}

impl StructuralIndex {
    /// Creates a new structural index using the given libmdbx environment and database.
    pub fn new(env: Arc<Environment<NoWriteMap>>, db: Database, databases: Arc<DashMap<String, Database>>) -> Self {
        Self { env, db, _databases: databases }
    }
    
    /// Adds a node ID to an index for a specific property value.
    pub fn add(&self, property: &str, value: &str, node_id: &str) -> DbResult<()> {
        let key = format!("prop:{}:{}", property, value);
        
        // Get existing set or create new
        let mut id_set: HashSet<NodeId> = self.get_raw(key.as_bytes())?
            .map(|bytes| {
                let archived = rkyv::check_archived_root::<HashSet<NodeId>>(&bytes)
                    .map_err(|e| DbError::Serialization(format!("rkyv check error: {}", e)))?;
                archived.deserialize(&mut rkyv::Infallible)
                    .map_err(|e| DbError::Serialization(format!("rkyv deserialize error: {}", e)))
            })
            .transpose()?
            .unwrap_or_default();
        
        // Add node ID
        id_set.insert(NodeId::from(node_id));
        
        // Store back with rkyv
        let bytes = rkyv::to_bytes::<_, 256>(&id_set)
            .map_err(|e| DbError::Serialization(format!("rkyv serialize error: {}", e)))?;
        self.set_raw(key.as_bytes(), bytes)?;
        
        Ok(())
    }
    
    /// Removes a node ID from an index for a specific property value.
    pub fn remove(&self, property: &str, value: &str, node_id: &str) -> DbResult<()> {
        let key = format!("prop:{}:{}", property, value);
        
        // Get existing set
        if let Some(bytes) = self.get_raw(key.as_bytes())? {
            let archived = rkyv::check_archived_root::<HashSet<NodeId>>(&bytes)
                .map_err(|e| DbError::Serialization(format!("rkyv check error: {}", e)))?;
            let mut id_set = archived.deserialize(&mut rkyv::Infallible)
                .map_err(|e| DbError::Serialization(format!("rkyv deserialize error: {}", e)))?;
            
            // Remove node ID
            id_set.remove(&NodeId::from(node_id));
            
            if id_set.is_empty() {
                // Remove key if set is now empty
                self.remove_raw(key.as_bytes())?;
            } else {
                // Store back with rkyv
                let bytes = rkyv::to_bytes::<_, 256>(&id_set)
                    .map_err(|e| DbError::Serialization(format!("rkyv serialize error: {}", e)))?;
                self.set_raw(key.as_bytes(), bytes)?;
            }
        }
        
        Ok(())
    }
    
    /// Retrieves all node IDs for a specific property value.
    pub fn get(&self, property: &str, value: &str) -> DbResult<Vec<NodeId>> {
        let key = format!("prop:{}:{}", property, value);
        
        let id_set: HashSet<NodeId> = self.get_raw(key.as_bytes())?
            .map(|bytes| {
                let archived = rkyv::check_archived_root::<HashSet<NodeId>>(&bytes)
                    .map_err(|e| DbError::Serialization(format!("rkyv check error: {}", e)))?;
                archived.deserialize(&mut rkyv::Infallible)
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
                let archived = rkyv::check_archived_root::<HashSet<NodeId>>(&bytes)
                    .map_err(|e| DbError::Serialization(format!("rkyv check error: {}", e)))?;
                archived.deserialize(&mut rkyv::Infallible)
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
    
    // Internal raw operations using libmdbx
    fn get_raw(&self, key: &[u8]) -> Result<Option<Vec<u8>>, DbError> {
        let txn = self.env.begin_ro_txn()
            .map_err(|e| DbError::InvalidOperation(format!("libmdbx ro txn error: {}", e)))?;
        
        match txn.get(&self.db, key) {
            Ok(Some(data)) => Ok(Some(data.to_vec())),
            Ok(None) => Ok(None),
            Err(e) => Err(DbError::InvalidOperation(format!("libmdbx get error: {}", e))),
        }
    }
    
    fn set_raw(&self, key: &[u8], value: Vec<u8>) -> Result<(), DbError> {
        let mut txn = self.env.begin_rw_txn()
            .map_err(|e| DbError::InvalidOperation(format!("libmdbx rw txn error: {}", e)))?;
        
        txn.put(&self.db, key, &value, libmdbx::WriteFlags::empty())
            .map_err(|e| DbError::InvalidOperation(format!("libmdbx put error: {}", e)))?;
        
        txn.commit()
            .map_err(|e| DbError::InvalidOperation(format!("libmdbx commit error: {}", e)))?;
        
        Ok(())
    }
    
    fn remove_raw(&self, key: &[u8]) -> Result<(), DbError> {
        let mut txn = self.env.begin_rw_txn()
            .map_err(|e| DbError::InvalidOperation(format!("libmdbx rw txn error: {}", e)))?;
        
        txn.del(&self.db, key, None)
            .map_err(|e| DbError::InvalidOperation(format!("libmdbx del error: {}", e)))?;
        
        txn.commit()
            .map_err(|e| DbError::InvalidOperation(format!("libmdbx commit error: {}", e)))?;
        
        Ok(())
    }
    
    fn scan_prefix(&self, prefix: &[u8]) -> Box<dyn Iterator<Item = Result<(Vec<u8>, Vec<u8>), DbError>>> {
        let txn = match self.env.begin_ro_txn() {
            Ok(txn) => txn,
            Err(e) => return Box::new(std::iter::once(Err(DbError::InvalidOperation(format!("libmdbx ro txn error: {}", e))))),
        };
        
        let mut cursor = match txn.cursor(&self.db) {
            Ok(c) => c,
            Err(e) => return Box::new(std::iter::once(Err(DbError::InvalidOperation(format!("libmdbx cursor error: {}", e))))),
        };
        
        // Position cursor at prefix
        let _ = cursor.lower_bound::<[u8]>(Some(prefix));
        
        struct ScanIterator {
            _txn: libmdbx::Transaction<libmdbx::RO>,
            cursor: libmdbx::Cursor<'static, libmdbx::RO>,
            prefix: Vec<u8>,
            started: bool,
        }
        
        impl Iterator for ScanIterator {
            type Item = Result<(Vec<u8>, Vec<u8>), DbError>;
            
            fn next(&mut self) -> Option<Self::Item> {
                let result = if !self.started {
                    self.started = true;
                    // SAFETY: Transmute is safe here because:
                    // 1. The cursor and cursor_ref both have the same memory layout (libmdbx::Cursor)
                    // 2. The lifetime is constrained to this iterator scope - cursor_ref cannot outlive cursor
                    // 3. The transaction (self.txn) is owned by the iterator, ensuring the cursor's underlying data remains valid
                    // 4. This is a read-only cursor (RO), so no mutation safety concerns
                    // 5. The cursor is created from a valid transaction that lives as long as the iterator
                    unsafe {
                        let cursor: &mut libmdbx::Cursor<'_, libmdbx::RO> = std::mem::transmute(&mut self.cursor);
                        cursor.first()
                    }
                } else {
                    // SAFETY: Transmute is safe here because:
                    // 1. The cursor and cursor_ref both have the same memory layout (libmdbx::Cursor)
                    // 2. The lifetime is constrained to this iterator scope - cursor_ref cannot outlive cursor
                    // 3. The transaction (self.txn) is owned by the iterator, ensuring the cursor's underlying data remains valid
                    // 4. This is a read-only cursor (RO), so no mutation safety concerns
                    // 5. The cursor is created from a valid transaction that lives as long as the iterator
                    unsafe {
                        let cursor: &mut libmdbx::Cursor<'_, libmdbx::RO> = std::mem::transmute(&mut self.cursor);
                        cursor.next()
                    }
                };
                
                match result {
                    Ok(Some((k, v))) => {
                        if k.starts_with(&self.prefix) {
                            Some(Ok((k.to_vec(), v.to_vec())))
                        } else {
                            None // Beyond prefix range
                        }
                    }
                    Ok(None) => None, // End of iteration
                    Err(e) => Some(Err(DbError::InvalidOperation(format!("libmdbx cursor error: {}", e)))),
                }
            }
        }
        
        Box::new(ScanIterator {
            _txn: txn,
            cursor,
            prefix: prefix.to_vec(),
            started: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    fn create_test_index() -> (StructuralIndex, TempDir) {
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
        
        let db = env.create_db(Some("test_structural"), libmdbx::DatabaseFlags::empty()).unwrap();
        let databases = Arc::new(DashMap::new());
        
        (StructuralIndex::new(Arc::new(env), db, databases), temp_dir)
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

