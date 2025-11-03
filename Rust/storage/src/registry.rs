use std::path::{Path, PathBuf};
use std::sync::Arc;
use dashmap::DashMap;

use crate::config::DbConfig;
use crate::engine::{MdbxEngine, StorageEngine, ReadGuard};

/// Error type for storage registry operations
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("Database '{0}' already exists")]
    DatabaseExists(String),
    
    #[error("Database '{0}' not found")]
    DatabaseNotFound(String),
    
    #[error("Failed to open database: {0}")]
    OpenError(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Operation failed: {0}")]
    OperationFailed(String),
}

/// Multi-database orchestrator that manages named storage instances
/// 
/// This is the main entry point for the storage system. It allows:
/// - Registering multiple named databases
/// - Routing CRUD operations to specific databases
/// - Discovering available databases
/// - Generic read-only access across all databases
pub struct StorageRegistry {
    /// Map of database name -> MdbxEngine
    databases: Arc<DashMap<String, Arc<MdbxEngine>>>,
    
    /// Base path for all databases
    base_path: PathBuf,
}

impl StorageRegistry {
    /// Create a new storage registry with the given base path
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            databases: Arc::new(DashMap::new()),
            base_path: base_path.into(),
        }
    }
    
    /// Register a new database with the given name and configuration
    /// 
    /// # Example
    /// ```no_run
    /// use storage::{StorageRegistry, DbConfig};
    /// 
    /// let mut registry = StorageRegistry::new("/data");
    /// 
    /// let config = DbConfig::new("knowledge_graph.mdbx")
    ///     .with_collection("nodes")
    ///     .with_collection("edges");
    /// 
    /// registry.add_storage("knowledge_graph", config).unwrap();
    /// ```
    pub fn add_storage(&self, name: &str, config: DbConfig) -> Result<(), RegistryError> {
        // Check if database already exists
        if self.databases.contains_key(name) {
            return Err(RegistryError::DatabaseExists(name.to_string()));
        }
        
        // Resolve database path
        let db_path = if Path::new(&config.path).is_absolute() {
            PathBuf::from(&config.path)
        } else {
            self.base_path.join(&config.path)
        };
        
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| RegistryError::OpenError(format!("Failed to create directory: {}", e)))?;
        }
        
        // Ensure the path is a string
        let path_str = db_path.to_str()
            .ok_or_else(|| RegistryError::InvalidConfig("Invalid UTF-8 in path".to_string()))?;
        
        // Open the database engine
        let engine = MdbxEngine::open(path_str)
            .map_err(|e| RegistryError::OpenError(format!("{}", e)))?;
        
        // Open requested collections (tables/trees)
        for collection in &config.collections {
            engine.open_tree(collection)
                .map_err(|e| RegistryError::OpenError(format!("Failed to open collection '{}': {}", collection, e)))?;
        }
        
        // Insert into registry
        self.databases.insert(name.to_string(), Arc::new(engine));
        
        Ok(())
    }
    
    /// Remove a database from the registry
    /// 
    /// Note: This does not delete the database file, only removes it from the registry
    pub fn remove_storage(&self, name: &str) -> Result<(), RegistryError> {
        self.databases
            .remove(name)
            .ok_or_else(|| RegistryError::DatabaseNotFound(name.to_string()))?;
        Ok(())
    }
    
    /// List all registered database names
    pub fn list_storages(&self) -> Vec<String> {
        self.databases.iter().map(|entry| entry.key().clone()).collect()
    }
    
    /// Check if a database is registered
    pub fn has_storage(&self, name: &str) -> bool {
        self.databases.contains_key(name)
    }
    
    /// Get a reference to a specific database's engine
    pub fn get_storage(&self, name: &str) -> Result<Arc<MdbxEngine>, RegistryError> {
        self.databases
            .get(name)
            .map(|entry| Arc::clone(entry.value()))
            .ok_or_else(|| RegistryError::DatabaseNotFound(name.to_string()))
    }
    
    /// Insert a key-value pair into a specific database and collection
    /// 
    /// # Example
    /// ```no_run
    /// # use storage::StorageRegistry;
    /// # let registry = StorageRegistry::new("/data");
    /// registry.insert("knowledge_graph", "nodes", b"node_123", b"data").unwrap();
    /// ```
    pub fn insert(
        &self,
        db_name: &str,
        collection: &str,
        key: &[u8],
        value: &[u8],
    ) -> Result<(), RegistryError> {
        let engine = self.get_storage(db_name)?;
        engine
            .insert(collection, key, value.to_vec())
            .map_err(|e| RegistryError::OperationFailed(format!("Insert failed: {}", e)))
    }
    
    /// Get a value from a specific database and collection
    /// 
    /// Returns `Ok(None)` if the key doesn't exist
    pub fn get(
        &self,
        db_name: &str,
        collection: &str,
        key: &[u8],
    ) -> Result<Option<Vec<u8>>, RegistryError> {
        let engine = self.get_storage(db_name)?;
        let result = engine
            .get(collection, key)
            .map_err(|e| RegistryError::OperationFailed(format!("Get failed: {}", e)))?;
        
        // Convert ReadGuard to Vec<u8>
        Ok(result.map(|guard| guard.data().to_vec()))
    }
    
    /// Remove a key from a specific database and collection
    pub fn remove(
        &self,
        db_name: &str,
        collection: &str,
        key: &[u8],
    ) -> Result<(), RegistryError> {
        let engine = self.get_storage(db_name)?;
        engine
            .remove(collection, key)
            .map_err(|e| RegistryError::OperationFailed(format!("Remove failed: {}", e)))?;
        Ok(())
    }
    
    /// Scan all keys in a collection with a given prefix
    pub fn scan_prefix(
        &self,
        db_name: &str,
        collection: &str,
        prefix: &[u8],
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, RegistryError> {
        let engine = self.get_storage(db_name)?;
        let iter = engine.scan_prefix(collection, prefix);
        
        // Collect results and convert ReadGuard to Vec<u8>
        let results: Result<Vec<_>, _> = iter
            .map(|result| {
                result.map(|(key, guard)| (key, guard.data().to_vec()))
            })
            .collect();
        
        results.map_err(|e| RegistryError::OperationFailed(format!("Scan failed: {}", e)))
    }
    
    /// Find a key across all databases and collections
    /// 
    /// Returns the first match found as (database_name, collection_name, value)
    /// 
    /// Note: This is a slow operation that scans all databases. Use sparingly.
    /// 
    /// TODO: Currently not implemented - requires tracking collections per database
    pub fn find_key_anywhere(&self, _key: &[u8]) -> Option<(String, String, Vec<u8>)> {
        // TODO: Implement collection tracking per database
        // For now, return None
        None
    }
    
    /// Get the base path for all databases
    pub fn base_path(&self) -> &Path {
        &self.base_path
    }
    
    /// Get the number of registered databases
    pub fn database_count(&self) -> usize {
        self.databases.len()
    }
    
    /// Clear all databases from the registry
    /// 
    /// This does not delete the database files
    pub fn clear(&self) {
        self.databases.clear();
    }
    
    /// Flush all databases to disk
    pub fn flush_all(&self) -> Result<(), RegistryError> {
        for entry in self.databases.iter() {
            let engine = entry.value();
            engine
                .flush()
                .map_err(|e| RegistryError::OperationFailed(format!("Flush failed for {}: {}", entry.key(), e)))?;
        }
        Ok(())
    }
}

impl Clone for StorageRegistry {
    fn clone(&self) -> Self {
        Self {
            databases: Arc::clone(&self.databases),
            base_path: self.base_path.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_add_and_list_storages() {
        let temp_dir = TempDir::new().unwrap();
        let registry = StorageRegistry::new(temp_dir.path());
        
        let config = DbConfig::new("test_db.mdbx").with_collection("test_collection");
        registry.add_storage("test_db", config).unwrap();
        
        let storages = registry.list_storages();
        assert_eq!(storages.len(), 1);
        assert!(storages.contains(&"test_db".to_string()));
    }
    
    #[test]
    fn test_add_duplicate_storage_fails() {
        let temp_dir = TempDir::new().unwrap();
        let registry = StorageRegistry::new(temp_dir.path());
        
        let config = DbConfig::new("test_db.mdbx");
        registry.add_storage("test_db", config.clone()).unwrap();
        
        let result = registry.add_storage("test_db", config);
        assert!(matches!(result, Err(RegistryError::DatabaseExists(_))));
    }
    
    #[test]
    fn test_insert_and_get() {
        let temp_dir = TempDir::new().unwrap();
        let registry = StorageRegistry::new(temp_dir.path());
        
        let config = DbConfig::new("test_db.mdbx").with_collection("test_collection");
        registry.add_storage("test_db", config).unwrap();
        
        registry
            .insert("test_db", "test_collection", b"key1", b"value1")
            .unwrap();
        
        let value = registry
            .get("test_db", "test_collection", b"key1")
            .unwrap();
        
        assert_eq!(value, Some(b"value1".to_vec()));
    }
    
    #[test]
    fn test_remove_storage() {
        let temp_dir = TempDir::new().unwrap();
        let registry = StorageRegistry::new(temp_dir.path());
        
        let config = DbConfig::new("test_db.mdbx");
        registry.add_storage("test_db", config).unwrap();
        
        assert!(registry.has_storage("test_db"));
        
        registry.remove_storage("test_db").unwrap();
        
        assert!(!registry.has_storage("test_db"));
    }
}

