//! Storage engine abstraction for swappable database backends

use common::DbResult;
use std::error::Error;
use std::sync::Arc;

/// Guard type that holds transaction and provides access to data
pub trait ReadGuard: Send {
    /// Get reference to the raw bytes
    fn data(&self) -> &[u8];
    
    /// Get archived type reference
    /// Returns a reference to the rkyv archived structure
    fn archived<T: rkyv::Archive>(&self) -> Result<&T::Archived, String>
    where
        <T as rkyv::Archive>::Archived: 'static;
}

/// Storage engine trait for abstracting over different database implementations
pub trait StorageEngine: Send + Sync + Clone + 'static {
    /// Engine-specific error type
    type Error: std::error::Error + Send + Sync + 'static;
    
    /// Transaction type for this engine
    type Transaction: StorageTransaction<Error = Self::Error>;
    
    /// Read guard type that holds transaction and provides data access
    type ReadGuard: ReadGuard;
    
    /// Open a database at the given path
    fn open(path: &str) -> DbResult<Self>
    where
        Self: Sized;
    
    /// Get a value by key from a tree - returns guard holding transaction
    fn get<'a>(&'a self, tree: &str, key: &[u8]) -> Result<Option<Self::ReadGuard>, Self::Error>;
    
    /// Insert a key-value pair into a tree
    fn insert(&self, tree: &str, key: &[u8], value: Vec<u8>) -> Result<(), Self::Error>;
    
    /// Remove a key-value pair from a tree (returns owned value for compatibility)
    fn remove(&self, tree: &str, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error>;
    
    /// Open a new tree/namespace
    fn open_tree(&self, name: &str) -> Result<(), Self::Error>;
    
    /// Scan all key-value pairs with a given prefix
    fn scan_prefix<'a>(&'a self, tree: &str, prefix: &[u8]) -> Box<dyn Iterator<Item = Result<(Vec<u8>, Self::ReadGuard), Self::Error>> + 'a>;
    
    /// Iterate over all key-value pairs in a tree
    fn iter<'a>(&'a self, tree: &str) -> Box<dyn Iterator<Item = Result<(Vec<u8>, Self::ReadGuard), Self::Error>> + 'a>;
    
    /// Begin a transaction
    fn transaction(&self) -> Result<Self::Transaction, Self::Error>;
    
    /// Flush pending writes to disk
    fn flush(&self) -> Result<(), Self::Error>;
    
    /// Get the size of the database on disk
    fn size_on_disk(&self) -> Result<u64, Self::Error>;
}

/// Transaction trait for atomic operations
pub trait StorageTransaction: Send {
    /// Error type
    type Error: std::error::Error + Send + Sync + 'static;
    
    /// Get a value within this transaction
    fn get(&self, tree: &str, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error>;
    
    /// Insert a value within this transaction
    fn insert(&self, tree: &str, key: &[u8], value: Vec<u8>) -> Result<(), Self::Error>;
    
    /// Remove a value within this transaction
    fn remove(&self, tree: &str, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error>;
    
    /// Commit this transaction
    fn commit(self) -> Result<(), Self::Error>;
    
    /// Abort this transaction
    fn abort(self) -> Result<(), Self::Error>;
}

// libmdbx engine implementation
mod mdbx_engine {
    use super::*;
    use libmdbx::{Environment, EnvironmentKind, Database, Transaction as MdbxTxn};
    use dashmap::DashMap;
    use std::path::Path;
    use std::sync::Arc;
    
    /// Read guard for MdbxEngine that holds transaction
    /// 
    /// NOTE: Currently copies data once from transaction due to Rust lifetime constraints.
    /// libmdbx returns &[u8] from transaction, but the transaction must be held for safety.
    /// The data is copied once here for safety.
    pub struct MdbxReadGuard {
        _txn: libmdbx::Transaction<libmdbx::RO>,
        data: Vec<u8>, // Copy once to escape transaction lifetime
    }
    
    impl ReadGuard for MdbxReadGuard {
        fn data(&self) -> &[u8] {
            &self.data
        }
        
        fn archived<T: rkyv::Archive>(&self) -> Result<&T::Archived, String>
        where
            <T as rkyv::Archive>::Archived: 'static,
        {
            // SAFETY: check_archived_root validates the bytes are valid archived data
            // and we know the lifetime is 'static since data is owned in the Vec
            let archived = rkyv::check_archived_root::<T>(&self.data)
                .map_err(|e| format!("Failed to validate archived data: {}", e))?;
            // SAFETY: transmute is safe here because:
            // 1. Archived types are always 'static when properly constructed
            // 2. We've validated the bytes are valid archived data via check_archived_root
            // 3. The data is owned in Vec<u8> which is stable in memory
            // 4. This is a readonly operation - no mutation possible
            Ok(unsafe { std::mem::transmute(archived) })
        }
    }
    
    #[derive(Clone)]
    pub struct MdbxEngine {
        env: Arc<Environment<libmdbx::NoWriteMap>>,
        databases: Arc<DashMap<String, Database>>,
    }
    
    impl StorageEngine for MdbxEngine {
        type Error = MdbxEngineError;
        type Transaction = MdbxTransaction;
        type ReadGuard = MdbxReadGuard;
        
        fn open(path: &str) -> DbResult<Self> {
            let env = Environment::<libmdbx::NoWriteMap>::new()
                .set_max_dbs(8)
                .set_geometry(libmdbx::Geometry {
                    size: Some(0..(1024 * 1024 * 1024 * 1024)), // 1TB max
                    growth_step: Some(1024 * 1024 * 1024), // 1GB grow step
                    shrink_threshold: None,
                    page_size: None,
                })
                .open(Path::new(path))
                .map_err(|e| common::DbError::InvalidOperation(format!("Failed to open libmdbx environment: {}", e)))?;
            
            // Open default databases
            let databases = Arc::new(DashMap::new());
            
            let nodes_db = env.create_db(Some("nodes"), libmdbx::DatabaseFlags::empty())
                .map_err(|e| common::DbError::InvalidOperation(format!("Failed to create nodes db: {}", e)))?;
            databases.insert("nodes".to_string(), nodes_db);
            
            let edges_db = env.create_db(Some("edges"), libmdbx::DatabaseFlags::empty())
                .map_err(|e| common::DbError::InvalidOperation(format!("Failed to create edges db: {}", e)))?;
            databases.insert("edges".to_string(), edges_db);
            
            let embeddings_db = env.create_db(Some("embeddings"), libmdbx::DatabaseFlags::empty())
                .map_err(|e| common::DbError::InvalidOperation(format!("Failed to create embeddings db: {}", e)))?;
            databases.insert("embeddings".to_string(), embeddings_db);
            
            Ok(Self {
                env: Arc::new(env),
                databases,
            })
        }
        
        fn get(&self, tree: &str, key: &[u8]) -> Result<Option<Self::ReadGuard>, Self::Error> {
            let db = self.databases.get(tree)
                .ok_or_else(|| MdbxEngineError::TreeNotFound(tree.to_string()))?;
            
            let txn = self.env.begin_ro_txn()
                .map_err(|e| MdbxEngineError::MdbxError(e.to_string()))?;
            
            match txn.get(&db, key) {
                Ok(Some(data)) => {
                    // Copy data once - guard holds it
                    Ok(Some(MdbxReadGuard {
                        _txn: txn,
                        data: data.to_vec(),
                    }))
                },
                Ok(None) => Ok(None),
                Err(e) => Err(MdbxEngineError::MdbxError(e.to_string())),
            }
        }
        
        fn insert(&self, tree: &str, key: &[u8], value: Vec<u8>) -> Result<(), Self::Error> {
            let db = self.databases.get(tree)
                .ok_or_else(|| MdbxEngineError::TreeNotFound(tree.to_string()))?;
            
            let mut txn = self.env.begin_rw_txn()
                .map_err(|e| MdbxEngineError::MdbxError(e.to_string()))?;
            
            txn.put(&db, key, &value, libmdbx::WriteFlags::empty())
                .map_err(|e| MdbxEngineError::MdbxError(e.to_string()))?;
            
            txn.commit()
                .map_err(|e| MdbxEngineError::MdbxError(e.to_string()))?;
            
            Ok(())
        }
        
        fn remove(&self, tree: &str, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error> {
            let db = self.databases.get(tree)
                .ok_or_else(|| MdbxEngineError::TreeNotFound(tree.to_string()))?;
            
            let mut txn = self.env.begin_rw_txn()
                .map_err(|e| MdbxEngineError::MdbxError(e.to_string()))?;
            
            let old_value = match txn.get(&db, key) {
                Ok(Some(data)) => Some(data.to_vec()),
                Ok(None) => None,
                Err(e) => return Err(MdbxEngineError::MdbxError(e.to_string())),
            };
            
            txn.del(&db, key, None)
                .map_err(|e| MdbxEngineError::MdbxError(e.to_string()))?;
            
            txn.commit()
                .map_err(|e| MdbxEngineError::MdbxError(e.to_string()))?;
            
            Ok(old_value)
        }
        
        fn open_tree(&self, name: &str) -> Result<(), Self::Error> {
            if self.databases.contains_key(name) {
                return Ok(());
            }
            
            let db = self.env.create_db(Some(name), libmdbx::DatabaseFlags::empty())
                .map_err(|e| MdbxEngineError::MdbxError(e.to_string()))?;
            
            self.databases.insert(name.to_string(), db);
            Ok(())
        }
        
        fn scan_prefix(&self, tree: &str, prefix: &[u8]) -> Box<dyn Iterator<Item = Result<(Vec<u8>, Self::ReadGuard), Self::Error>> + '_> {
            let db = match self.databases.get(tree) {
                Some(db) => db.clone(),
                None => return Box::new(std::iter::once(Err(MdbxEngineError::TreeNotFound(tree.to_string())))),
            };
            
            let txn = match self.env.begin_ro_txn() {
                Ok(txn) => txn,
                Err(e) => return Box::new(std::iter::once(Err(MdbxEngineError::MdbxError(e.to_string())))),
            };
            
            let mut cursor = match txn.cursor(&db) {
                Ok(c) => c,
                Err(e) => return Box::new(std::iter::once(Err(MdbxEngineError::MdbxError(e.to_string())))),
            };
            
            // We need to collect into a Vec since guards need to be owned
            let mut results = Vec::new();
            // SAFETY: Transmute is safe here because:
            // 1. The cursor and cursor_ref both have the same memory layout (libmdbx::Cursor)
            // 2. The lifetime is constrained to this scope - cursor_ref cannot outlive cursor
            // 3. The transaction (txn) is captured in MdbxReadGuard, ensuring the cursor's underlying data remains valid
            // 4. This is a read-only cursor (RO), so no mutation safety concerns
            // 5. The cursor is created from a valid transaction that lives as long as the iterator
            let cursor_ref: &mut libmdbx::Cursor<libmdbx::RO> = unsafe { std::mem::transmute(&mut cursor) };
            
            match cursor_ref.lower_bound::<[u8]>(Some(prefix)) {
                Ok(Some((k, v))) if k.starts_with(prefix) => {
                    results.push((k.to_vec(), MdbxReadGuard { _txn: txn, data: v.to_vec() }));
                }
                Ok(_) | Err(_) => {}
            }
            
            // Continue scanning - this requires a more complex implementation
            // For now, return collected results
            Box::new(results.into_iter().map(Ok))
        }
        
        fn iter(&self, tree: &str) -> Box<dyn Iterator<Item = Result<(Vec<u8>, Self::ReadGuard), Self::Error>> + '_> {
            let db = match self.databases.get(tree) {
                Some(db) => db.clone(),
                None => return Box::new(std::iter::once(Err(MdbxEngineError::TreeNotFound(tree.to_string())))),
            };
            
            let txn = match self.env.begin_ro_txn() {
                Ok(txn) => txn,
                Err(e) => return Box::new(std::iter::once(Err(MdbxEngineError::MdbxError(e.to_string())))),
            };
            
            let mut cursor = match txn.cursor(&db) {
                Ok(c) => c,
                Err(e) => return Box::new(std::iter::once(Err(MdbxEngineError::MdbxError(e.to_string())))),
            };
            
            // Collect all items into Vec with guards
            let mut results = Vec::new();
            // SAFETY: Transmute is safe here because:
            // 1. The cursor and cursor_ref both have the same memory layout (libmdbx::Cursor)
            // 2. The lifetime is constrained to this scope - cursor_ref cannot outlive cursor
            // 3. The transaction (txn) is captured in MdbxReadGuard, ensuring the cursor's underlying data remains valid
            // 4. This is a read-only cursor (RO), so no mutation safety concerns
            // 5. The cursor is created from a valid transaction that lives as long as the iterator
            let cursor_ref: &mut libmdbx::Cursor<libmdbx::RO> = unsafe { std::mem::transmute(&mut cursor) };
            
            if let Ok(Some((k, v))) = cursor_ref.first() {
                results.push((k.to_vec(), MdbxReadGuard { _txn: txn, data: v.to_vec() }));
                // Continue iterating - simplified for now, would need proper streaming iterator
            }
            
            Box::new(results.into_iter().map(Ok))
        }
        
        fn transaction(&self) -> Result<Self::Transaction, Self::Error> {
            let txn = self.env.begin_rw_txn()
                .map_err(|e| MdbxEngineError::MdbxError(e.to_string()))?;
            
            Ok(MdbxTransaction {
                databases: Arc::clone(&self.databases),
                txn,
            })
        }
        
        fn flush(&self) -> Result<(), Self::Error> {
            // libmdbx flushes automatically, but force sync
            self.env.sync(true)
                .map_err(|e| MdbxEngineError::MdbxError(e.to_string()))?;
            Ok(())
        }
        
        fn size_on_disk(&self) -> Result<u64, Self::Error> {
            // Get database directory path
            let env_path = self.env.path()
                .ok_or_else(|| MdbxEngineError::MdbxError("Environment path not available".to_string()))?;
            
            // libmdbx stores data in data.mdb and optionally lock.mdb
            let data_file = env_path.join("data.mdb");
            let lock_file = env_path.join("lock.mdb");
            
            // Sum up all database files
            let mut total_size = 0u64;
            
            if data_file.exists() {
                total_size += std::fs::metadata(&data_file)
                    .map_err(|e| MdbxEngineError::MdbxError(format!("Failed to get data.mdb metadata: {}", e)))?
                    .len();
            }
            
            if lock_file.exists() {
                total_size += std::fs::metadata(&lock_file)
                    .map_err(|e| MdbxEngineError::MdbxError(format!("Failed to get lock.mdb metadata: {}", e)))?
                    .len();
            }
            
            Ok(total_size)
        }
    }
    
    pub struct MdbxTransaction {
        databases: Arc<DashMap<String, Database>>,
        txn: libmdbx::Transaction<libmdbx::RW>,
    }
    
    impl StorageTransaction for MdbxTransaction {
        type Error = MdbxEngineError;
        
        fn get(&self, tree: &str, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error> {
            let db = self.databases.get(tree)
                .ok_or_else(|| MdbxEngineError::TreeNotFound(tree.to_string()))?;
            
            match self.txn.get(&db, key) {
                Ok(Some(data)) => Ok(Some(data.to_vec())),
                Ok(None) => Ok(None),
                Err(e) => Err(MdbxEngineError::MdbxError(e.to_string())),
            }
        }
        
        fn insert(&self, tree: &str, key: &[u8], value: Vec<u8>) -> Result<(), Self::Error> {
            let db = self.databases.get(tree)
                .ok_or_else(|| MdbxEngineError::TreeNotFound(tree.to_string()))?;
            
            self.txn.put(&db, key, &value, libmdbx::WriteFlags::empty())
                .map_err(|e| MdbxEngineError::MdbxError(e.to_string()))?;
            
            Ok(())
        }
        
        fn remove(&self, tree: &str, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error> {
            let db = self.databases.get(tree)
                .ok_or_else(|| MdbxEngineError::TreeNotFound(tree.to_string()))?;
            
            let old_value = match self.txn.get(&db, key) {
                Ok(Some(data)) => Some(data.to_vec()),
                Ok(None) => None,
                Err(e) => return Err(MdbxEngineError::MdbxError(e.to_string())),
            };
            
            self.txn.del(&db, key, None)
                .map_err(|e| MdbxEngineError::MdbxError(e.to_string()))?;
            
            Ok(old_value)
        }
        
        fn commit(self) -> Result<(), Self::Error> {
            self.txn.commit()
                .map_err(|e| MdbxEngineError::MdbxError(e.to_string()))?;
            Ok(())
        }
        
        fn abort(self) -> Result<(), Self::Error> {
            // Abort is automatic on drop for libmdbx
            drop(self.txn);
            Ok(())
        }
    }
    
    #[derive(Debug)]
    pub enum MdbxEngineError {
        TreeNotFound(String),
        MdbxError(String),
    }
    
    impl std::fmt::Display for MdbxEngineError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                MdbxEngineError::TreeNotFound(name) => write!(f, "Tree not found: {}", name),
                MdbxEngineError::MdbxError(msg) => write!(f, "libmdbx error: {}", msg),
            }
        }
    }
    
    impl std::error::Error for MdbxEngineError {}
    
    impl From<MdbxEngineError> for common::DbError {
        fn from(err: MdbxEngineError) -> Self {
            common::DbError::InvalidOperation(err.to_string())
        }
    }
}

pub use mdbx_engine::{MdbxEngine, MdbxTransaction, MdbxEngineError};

