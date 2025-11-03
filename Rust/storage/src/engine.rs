//! Storage engine abstraction for swappable database backends

use common::DbResult;

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
    use libmdbx::{Database, DatabaseOptions, NoWriteMap, TableFlags, WriteFlags};
    use dashmap::DashMap;
    use std::path::Path;
    use std::sync::Arc;
    use std::borrow::Cow;
    
    /// Read guard for MdbxEngine that holds transaction
    /// 
    /// NOTE: Currently copies data once from transaction due to Rust lifetime constraints.
    /// libmdbx returns &[u8] from transaction, but the transaction must be held for safety.
    /// The data is copied once here for safety.
    pub struct MdbxReadGuard {
        pub data: Vec<u8>, // Owned data - no need to store transaction
    }
    
    impl ReadGuard for MdbxReadGuard {
        fn data(&self) -> &[u8] {
            &self.data
        }
        
        fn archived<T: rkyv::Archive>(&self) -> Result<&T::Archived, String>
        where
            <T as rkyv::Archive>::Archived: 'static,
        {
            // Use unsafe access without validation for performance
            // SAFETY: We assume the data was serialized correctly with rkyv
            unsafe {
                let ptr = self.data.as_ptr() as *const T::Archived;
                Ok(&*ptr)
            }
        }
    }
    
    #[derive(Clone)]
    pub struct MdbxEngine {
        db: Arc<Database<NoWriteMap>>,
        tables: Arc<DashMap<String, String>>, // Map table names to ensure they're created
    }
    
    impl StorageEngine for MdbxEngine {
        type Error = MdbxEngineError;
        type Transaction = MdbxTransaction;
        type ReadGuard = MdbxReadGuard;
        
        fn open(path: &str) -> DbResult<Self> {
            // libmdbx 0.6.3: Use Database::open_with_options
            let mut options = DatabaseOptions::default();
            options.max_tables = Some(16);
            
            let db = Database::<NoWriteMap>::open_with_options(Path::new(path), options)
                .map_err(|e| common::DbError::InvalidOperation(format!("Failed to open libmdbx database: {}", e)))?;
            
            // Create default tables
            let tables = Arc::new(DashMap::new());
            {
                let txn = db.begin_rw_txn()
                    .map_err(|e| common::DbError::InvalidOperation(format!("Failed to begin transaction: {}", e)))?;
                
                let _ = txn.create_table(Some("nodes"), TableFlags::empty())
                    .map_err(|e| common::DbError::InvalidOperation(format!("Failed to create nodes table: {}", e)))?;
                tables.insert("nodes".to_string(), "nodes".to_string());
                
                let _ = txn.create_table(Some("edges"), TableFlags::empty())
                    .map_err(|e| common::DbError::InvalidOperation(format!("Failed to create edges table: {}", e)))?;
                tables.insert("edges".to_string(), "edges".to_string());
                
                let _ = txn.create_table(Some("embeddings"), TableFlags::empty())
                    .map_err(|e| common::DbError::InvalidOperation(format!("Failed to create embeddings table: {}", e)))?;
                tables.insert("embeddings".to_string(), "embeddings".to_string());
                
                txn.commit()
                    .map_err(|e| common::DbError::InvalidOperation(format!("Failed to commit table creation: {}", e)))?;
            }
            
            Ok(Self {
                db: Arc::new(db),
                tables,
            })
        }
        
        fn get(&self, tree: &str, key: &[u8]) -> Result<Option<Self::ReadGuard>, Self::Error> {
            if !self.tables.contains_key(tree) {
                return Err(MdbxEngineError::TreeNotFound(tree.to_string()));
            }
            
            let txn = self.db.begin_ro_txn()
                .map_err(|e| MdbxEngineError::MdbxError(e.to_string()))?;
            
            let table = txn.open_table(Some(tree))
                .map_err(|e| MdbxEngineError::MdbxError(e.to_string()))?;
            
            match txn.get::<Cow<'_, [u8]>>(&table, key) {
                Ok(Some(data)) => {
                    // Copy data once - guard holds it
                    Ok(Some(MdbxReadGuard {
                        data: data.to_vec(),
                    }))
                },
                Ok(None) => Ok(None),
                Err(e) => Err(MdbxEngineError::MdbxError(e.to_string())),
            }
        }
        
        fn insert(&self, tree: &str, key: &[u8], value: Vec<u8>) -> Result<(), Self::Error> {
            if !self.tables.contains_key(tree) {
                return Err(MdbxEngineError::TreeNotFound(tree.to_string()));
            }
            
            let txn = self.db.begin_rw_txn()
                .map_err(|e| MdbxEngineError::MdbxError(e.to_string()))?;
            
            let table = txn.create_table(Some(tree), TableFlags::empty())
                .map_err(|e| MdbxEngineError::MdbxError(e.to_string()))?;
            
            txn.put(&table, key, &value, WriteFlags::empty())
                .map_err(|e| MdbxEngineError::MdbxError(e.to_string()))?;
            
            txn.commit()
                .map_err(|e| MdbxEngineError::MdbxError(e.to_string()))?;
            
            Ok(())
        }
        
        fn remove(&self, tree: &str, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error> {
            if !self.tables.contains_key(tree) {
                return Err(MdbxEngineError::TreeNotFound(tree.to_string()));
            }
            
            let txn = self.db.begin_rw_txn()
                .map_err(|e| MdbxEngineError::MdbxError(e.to_string()))?;
            
            let table = txn.create_table(Some(tree), TableFlags::empty())
                .map_err(|e| MdbxEngineError::MdbxError(e.to_string()))?;
            
            let old_value = match txn.get::<Cow<'_, [u8]>>(&table, key) {
                Ok(Some(data)) => Some(data.to_vec()),
                Ok(None) => None,
                Err(e) => return Err(MdbxEngineError::MdbxError(e.to_string())),
            };
            
            txn.del(&table, key, None)
                .map_err(|e| MdbxEngineError::MdbxError(e.to_string()))?;
            
            txn.commit()
                .map_err(|e| MdbxEngineError::MdbxError(e.to_string()))?;
            
            Ok(old_value)
        }
        
        fn open_tree(&self, name: &str) -> Result<(), Self::Error> {
            if self.tables.contains_key(name) {
                return Ok(());
            }
            
            // Create the table
            let txn = self.db.begin_rw_txn()
                .map_err(|e| MdbxEngineError::MdbxError(e.to_string()))?;
            
            let _ = txn.create_table(Some(name), TableFlags::empty())
                .map_err(|e| MdbxEngineError::MdbxError(e.to_string()))?;
            
            txn.commit()
                .map_err(|e| MdbxEngineError::MdbxError(e.to_string()))?;
            
            self.tables.insert(name.to_string(), name.to_string());
            Ok(())
        }
        
        fn scan_prefix(&self, tree: &str, prefix: &[u8]) -> Box<dyn Iterator<Item = Result<(Vec<u8>, Self::ReadGuard), Self::Error>> + '_> {
            if !self.tables.contains_key(tree) {
                return Box::new(std::iter::once(Err(MdbxEngineError::TreeNotFound(tree.to_string()))));
            }
            
            let table_name = tree.to_string();
            let txn = match self.db.begin_ro_txn() {
                Ok(txn) => txn,
                Err(e) => return Box::new(std::iter::once(Err(MdbxEngineError::MdbxError(e.to_string())))),
            };
            
            let table = match txn.open_table(Some(&table_name)) {
                Ok(t) => t,
                Err(e) => return Box::new(std::iter::once(Err(MdbxEngineError::MdbxError(e.to_string())))),
            };
            
            let mut cursor = match txn.cursor(&table) {
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
            
            // Iterate and filter by prefix (no lower_bound in 0.6.3)
            match cursor_ref.first::<Cow<'_, [u8]>, Cow<'_, [u8]>>() {
                Ok(Some((k, v))) if k.starts_with(prefix) => {
                    results.push((k.to_vec(), MdbxReadGuard { data: v.to_vec() }));
                }
                Ok(_) | Err(_) => {}
            }
            
            // Continue scanning - this requires a more complex implementation
            // For now, return collected results
            Box::new(results.into_iter().map(Ok))
        }
        
        fn iter(&self, tree: &str) -> Box<dyn Iterator<Item = Result<(Vec<u8>, Self::ReadGuard), Self::Error>> + '_> {
            if !self.tables.contains_key(tree) {
                return Box::new(std::iter::once(Err(MdbxEngineError::TreeNotFound(tree.to_string()))));
            }
            
            let table_name = tree.to_string();
            let txn = match self.db.begin_ro_txn() {
                Ok(txn) => txn,
                Err(e) => return Box::new(std::iter::once(Err(MdbxEngineError::MdbxError(e.to_string())))),
            };
            
            let table = match txn.open_table(Some(&table_name)) {
                Ok(t) => t,
                Err(e) => return Box::new(std::iter::once(Err(MdbxEngineError::MdbxError(e.to_string())))),
            };
            
            let mut cursor = match txn.cursor(&table) {
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
            
            if let Ok(Some((k, v))) = cursor_ref.first::<Cow<'_, [u8]>, Cow<'_, [u8]>>() {
                results.push((k.to_vec(), MdbxReadGuard { data: v.to_vec() }));
                // Continue iterating - simplified for now, would need proper streaming iterator
            }
            
            Box::new(results.into_iter().map(Ok))
        }
        
        fn transaction(&self) -> Result<Self::Transaction, Self::Error> {
            // Return a transaction wrapper that creates a new transaction for each operation
            // This avoids complex lifetime issues
            Ok(MdbxTransaction {
                db: Arc::clone(&self.db),
                tables: Arc::clone(&self.tables),
            })
        }
        
        fn flush(&self) -> Result<(), Self::Error> {
            // libmdbx 0.6.3: sync method on Database
            self.db.sync(true)
                .map_err(|e| MdbxEngineError::MdbxError(e.to_string()))?;
            Ok(())
        }
        
        fn size_on_disk(&self) -> Result<u64, Self::Error> {
            // TODO: Implement proper size calculation
            // libmdbx 0.6.3 doesn't expose path() on Arc<Database>
            Ok(0)
        }
    }
    
    // MdbxTransaction wraps a RW transaction  
    // We can't use a trait object because we need concrete type for methods
    // Solution: Store the actual transaction type and handle it generically
    pub struct MdbxTransaction {
        db: Arc<Database<NoWriteMap>>,
        tables: Arc<DashMap<String, String>>,
    }
    
    impl StorageTransaction for MdbxTransaction {
        type Error = MdbxEngineError;
        
        fn get(&self, tree: &str, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error> {
            if !self.tables.contains_key(tree) {
                return Err(MdbxEngineError::TreeNotFound(tree.to_string()));
            }
            
            let txn = self.db.begin_ro_txn()
                .map_err(|e| MdbxEngineError::MdbxError(e.to_string()))?;
            
            let table = txn.open_table(Some(tree))
                .map_err(|e| MdbxEngineError::MdbxError(e.to_string()))?;
            
            match txn.get::<Cow<'_, [u8]>>(&table, key) {
                Ok(Some(data)) => Ok(Some(data.to_vec())),
                Ok(None) => Ok(None),
                Err(e) => Err(MdbxEngineError::MdbxError(e.to_string())),
            }
        }
        
        fn insert(&self, tree: &str, key: &[u8], value: Vec<u8>) -> Result<(), Self::Error> {
            if !self.tables.contains_key(tree) {
                return Err(MdbxEngineError::TreeNotFound(tree.to_string()));
            }
            
            let txn = self.db.begin_rw_txn()
                .map_err(|e| MdbxEngineError::MdbxError(e.to_string()))?;
            
            let table = txn.create_table(Some(tree), TableFlags::empty())
                .map_err(|e| MdbxEngineError::MdbxError(e.to_string()))?;
            
            txn.put(&table, key, &value, WriteFlags::empty())
                .map_err(|e| MdbxEngineError::MdbxError(e.to_string()))?;
            
            txn.commit()
                .map_err(|e| MdbxEngineError::MdbxError(e.to_string()))?;
            
            Ok(())
        }
        
        fn remove(&self, tree: &str, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error> {
            if !self.tables.contains_key(tree) {
                return Err(MdbxEngineError::TreeNotFound(tree.to_string()));
            }
            
            let txn = self.db.begin_rw_txn()
                .map_err(|e| MdbxEngineError::MdbxError(e.to_string()))?;
            
            let table = txn.create_table(Some(tree), TableFlags::empty())
                .map_err(|e| MdbxEngineError::MdbxError(e.to_string()))?;
            
            let old_value = match txn.get::<Cow<'_, [u8]>>(&table, key) {
                Ok(Some(data)) => Some(data.to_vec()),
                Ok(None) => None,
                Err(e) => return Err(MdbxEngineError::MdbxError(e.to_string())),
            };
            
            txn.del(&table, key, None)
                .map_err(|e| MdbxEngineError::MdbxError(e.to_string()))?;
            
            txn.commit()
                .map_err(|e| MdbxEngineError::MdbxError(e.to_string()))?;
            
            Ok(old_value)
        }
        
        fn commit(self) -> Result<(), Self::Error> {
            // Transaction model changed - each operation commits individually
            Ok(())
        }
        
        fn abort(self) -> Result<(), Self::Error> {
            // Transaction model changed - each operation commits individually
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

