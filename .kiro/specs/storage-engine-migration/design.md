# Storage Engine Migration Design Document

## Overview

This design document outlines the architectural approach for migrating MIA's storage layer from `sled`/`bincode` to `libmdbx`/`rkyv`. The migration follows a phased approach prioritizing system stability through abstraction layers, allowing for safe transition and rollback capabilities.

**Design Philosophy:**
- **Abstraction First**: Create storage engine abstraction before implementing new engine
- **Zero Disruption**: Maintain identical API surface during transition
- **Incremental Migration**: Phase implementation to minimize risk
- **Performance Focus**: Optimize for zero-copy reads and MVCC concurrency

## Architecture

### Current Architecture Analysis

```
StorageManager (storage/src/storage_manager.rs)
├── sled::Db (direct dependency)
├── sled::Tree (nodes, edges, embeddings)
├── bincode serialization (serde derives)
├── Optional IndexManager integration
└── Multi-tier support (DatabaseType, TemperatureTier)

DatabaseCoordinator (storage/src/coordinator.rs)
├── Multiple StorageManager instances
├── Tier management (Active, Recent, Archive)
└── Cross-tier query operations
```

### Target Architecture

```
StorageManager<E: StorageEngine>
├── E: StorageEngine (generic abstraction)
├── Optional IndexManager integration (unchanged)
└── Multi-tier support (unchanged)

StorageEngine Trait
├── SledEngine (compatibility wrapper)
└── MdbxEngine (new implementation)

DatabaseCoordinator
├── StorageManager<SledEngine> (transition phase)
├── StorageManager<MdbxEngine> (target phase)
└── Tier management (unchanged)
```

## Components and Interfaces

### Phase 1: Storage Engine Abstraction

#### StorageEngine Trait Design

```rust
pub trait StorageEngine: Send + Sync + Clone {
    type Error: std::error::Error + Send + Sync + 'static;
    type Transaction: Transaction<Error = Self::Error>;
    
    // Core database operations
    fn get(&self, tree: &str, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error>;
    fn insert(&self, tree: &str, key: &[u8], value: Vec<u8>) -> Result<(), Self::Error>;
    fn remove(&self, tree: &str, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error>;
    
    // Tree operations
    fn open_tree(&self, name: &str) -> Result<(), Self::Error>;
    fn scan_prefix(&self, tree: &str, prefix: &[u8]) -> Box<dyn Iterator<Item = Result<(Vec<u8>, Vec<u8>), Self::Error>>>;
    fn iter(&self, tree: &str) -> Box<dyn Iterator<Item = Result<(Vec<u8>, Vec<u8>), Self::Error>>>;
    
    // Transaction support
    fn transaction(&self) -> Result<Self::Transaction, Self::Error>;
    
    // Maintenance operations
    fn flush(&self) -> Result<(), Self::Error>;
    fn size_on_disk(&self) -> Result<u64, Self::Error>;
}

pub trait Transaction {
    type Error: std::error::Error + Send + Sync + 'static;
    
    fn get(&self, tree: &str, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error>;
    fn insert(&self, tree: &str, key: &[u8], value: Vec<u8>) -> Result<(), Self::Error>;
    fn remove(&self, tree: &str, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error>;
    fn commit(self) -> Result<(), Self::Error>;
    fn abort(self) -> Result<(), Self::Error>;
}
```

#### SledEngine Implementation

```rust
#[derive(Clone)]
pub struct SledEngine {
    db: sled::Db,
    trees: Arc<DashMap<String, sled::Tree>>,
}

impl StorageEngine for SledEngine {
    type Error = SledEngineError;
    type Transaction = SledTransaction;
    
    fn get(&self, tree: &str, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error> {
        let tree = self.get_tree(tree)?;
        Ok(tree.get(key)?.map(|v| v.to_vec()))
    }
    
    // ... other implementations wrap existing sled operations
}
```

#### StorageManager Refactoring

```rust
pub struct StorageManager<E: StorageEngine> {
    engine: E,
    index_manager: Option<Arc<IndexManager>>,
    db_type: DatabaseType,
    tier: Option<TemperatureTier>,
}

impl<E: StorageEngine> StorageManager<E> {
    pub fn new_with_engine(engine: E, db_type: DatabaseType, tier: Option<TemperatureTier>) -> DbResult<Self> {
        // Initialize trees
        engine.open_tree("nodes")?;
        engine.open_tree("edges")?;
        engine.open_tree("embeddings")?;
        
        Ok(Self {
            engine,
            index_manager: None,
            db_type,
            tier,
        })
    }
    
    pub fn get_node(&self, id: &str) -> DbResult<Option<Node>> {
        match self.engine.get("nodes", id.as_bytes())? {
            Some(bytes) => {
                // Deserialize based on engine type
                let node = self.deserialize_node(&bytes)?;
                Ok(Some(node))
            }
            None => Ok(None),
        }
    }
    
    // ... other methods use self.engine instead of direct sled calls
}
```

### Phase 2: Serialization Migration

#### Data Model Conversion Strategy

**Challenge**: `serde_json::Value` is not zero-copy compatible with `rkyv`

**Solution**: Hybrid approach with conversion layers

```rust
// New rkyv-compatible metadata type
#[derive(Archive, Deserialize, Serialize)]
pub struct RkyvMetadata {
    data: ArchivedHashMap<ArchivedString, RkyvValue>,
}

#[derive(Archive, Deserialize, Serialize)]
pub enum RkyvValue {
    Null,
    Bool(bool),
    Number(f64),
    String(ArchivedString),
    Array(ArchivedVec<RkyvValue>),
    Object(ArchivedHashMap<ArchivedString, RkyvValue>),
}

// Conversion utilities
impl From<serde_json::Value> for RkyvMetadata {
    fn from(value: serde_json::Value) -> Self {
        // Convert serde_json::Value to RkyvMetadata
    }
}

impl From<&ArchivedRkyvMetadata> for serde_json::Value {
    fn from(archived: &ArchivedRkyvMetadata) -> Self {
        // Convert archived metadata back to serde_json::Value for compatibility
    }
}
```

#### Node Model Migration

```rust
// Dual-derive approach during transition
#[derive(Serialize, Deserialize, Archive, rkyv::Deserialize, rkyv::Serialize, Debug, Clone)]
pub struct Chat {
    pub id: NodeId,
    pub title: String,
    pub topic: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub message_ids: Vec<NodeId>,
    pub summary_ids: Vec<NodeId>,
    pub embedding_id: Option<EmbeddingId>,
    
    // Transition: support both formats
    #[serde(with = "crate::json_metadata")]
    #[with(RkyvMetadataResolver)]
    pub metadata: RkyvMetadata,
}
```

### Phase 3: MdbxEngine Implementation

#### LMDB/libmdbx Integration

```rust
use libmdbx::{Environment, Database, Transaction as MdbxTxn};

#[derive(Clone)]
pub struct MdbxEngine {
    env: Arc<Environment>,
    databases: Arc<DashMap<String, Database>>,
}

impl StorageEngine for MdbxEngine {
    type Error = MdbxEngineError;
    type Transaction = MdbxTransaction;
    
    fn get(&self, tree: &str, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error> {
        let db = self.get_database(tree)?;
        let txn = self.env.begin_ro_txn()?;
        
        match txn.get(&db, key)? {
            Some(data) => {
                // Zero-copy read: return reference to memory-mapped data
                Ok(Some(data.to_vec()))
            }
            None => Ok(None),
        }
    }
    
    fn insert(&self, tree: &str, key: &[u8], value: Vec<u8>) -> Result<(), Self::Error> {
        let db = self.get_database(tree)?;
        let mut txn = self.env.begin_rw_txn()?;
        
        txn.put(&db, key, &value, WriteFlags::empty())?;
        txn.commit()?;
        
        Ok(())
    }
}
```

#### Zero-Copy Deserialization

```rust
impl<E: StorageEngine> StorageManager<E> {
    fn deserialize_node(&self, bytes: &[u8]) -> DbResult<Node> {
        match E::supports_zero_copy() {
            true => {
                // rkyv zero-copy deserialization
                let archived = rkyv::check_archived_root::<Node>(bytes)
                    .map_err(|e| DbError::Serialization(e.to_string()))?;
                
                // Direct access to archived data without copying
                Ok(archived.deserialize(&mut rkyv::Infallible)?)
            }
            false => {
                // Fallback to bincode for SledEngine
                let (node, _) = bincode::serde::decode_from_slice(bytes, bincode::config::standard())
                    .map_err(|e| DbError::Serialization(e.to_string()))?;
                Ok(node)
            }
        }
    }
}
```

## Data Models

### Serialization Format Evolution

**Current Format (bincode/serde):**
```
Node -> serde::Serialize -> bincode -> Vec<u8> -> sled storage
sled storage -> Vec<u8> -> bincode -> serde::Deserialize -> Node
```

**Target Format (rkyv):**
```
Node -> rkyv::Serialize -> AlignedVec<u8> -> libmdbx storage
libmdbx storage -> &[u8] -> rkyv::check_archived_root -> &ArchivedNode (zero-copy)
```

### Metadata Handling Strategy

1. **Phase 1**: Maintain `serde_json::Value` with conversion layer
2. **Phase 2**: Introduce `RkyvMetadata` with bidirectional conversion
3. **Phase 3**: Full migration to `RkyvMetadata` with optional serde compatibility

## Error Handling

### Unified Error Types

```rust
#[derive(thiserror::Error, Debug)]
pub enum StorageEngineError {
    #[error("Sled error: {0}")]
    Sled(#[from] sled::Error),
    
    #[error("MDBX error: {0}")]
    Mdbx(#[from] libmdbx::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Transaction error: {0}")]
    Transaction(String),
}

// Convert to existing DbError for compatibility
impl From<StorageEngineError> for common::DbError {
    fn from(err: StorageEngineError) -> Self {
        match err {
            StorageEngineError::Sled(e) => common::DbError::Sled(e),
            StorageEngineError::Mdbx(e) => common::DbError::InvalidOperation(e.to_string()),
            StorageEngineError::Serialization(e) => common::DbError::Serialization(e),
            StorageEngineError::Transaction(e) => common::DbError::InvalidOperation(e),
        }
    }
}
```

## Testing Strategy

### Compatibility Testing

1. **Engine Abstraction Tests**: Verify SledEngine maintains identical behavior
2. **Cross-Engine Tests**: Run same test suite against both engines
3. **Migration Tests**: Validate data integrity during engine switching
4. **Performance Tests**: Benchmark zero-copy vs traditional deserialization

### Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    // Generic tests that work with any StorageEngine
    fn test_crud_operations<E: StorageEngine>(engine: E) {
        let storage = StorageManager::new_with_engine(engine, DatabaseType::Conversations, None).unwrap();
        // ... test CRUD operations
    }
    
    #[test]
    fn test_sled_engine_crud() {
        let engine = SledEngine::new("test_db").unwrap();
        test_crud_operations(engine);
    }
    
    #[test]
    fn test_mdbx_engine_crud() {
        let engine = MdbxEngine::new("test_db").unwrap();
        test_crud_operations(engine);
    }
}
```

### Migration Validation

```rust
#[test]
fn test_engine_migration() {
    // 1. Create data with SledEngine
    let sled_storage = StorageManager::new_with_engine(
        SledEngine::new("migration_test").unwrap(),
        DatabaseType::Conversations,
        None
    ).unwrap();
    
    // Insert test data
    let chat = create_test_chat();
    sled_storage.insert_node(&Node::Chat(chat.clone())).unwrap();
    
    // 2. Migrate to MdbxEngine
    let migration_tool = MigrationTool::new();
    migration_tool.migrate_sled_to_mdbx("migration_test", "migration_test_mdbx").unwrap();
    
    // 3. Verify data integrity
    let mdbx_storage = StorageManager::new_with_engine(
        MdbxEngine::new("migration_test_mdbx").unwrap(),
        DatabaseType::Conversations,
        None
    ).unwrap();
    
    let retrieved_chat = mdbx_storage.get_node(&chat.id).unwrap().unwrap();
    assert_eq!(Node::Chat(chat), retrieved_chat);
}
```

## Performance Considerations

### Zero-Copy Benefits

1. **Memory Efficiency**: Eliminate deserialization allocations
2. **CPU Efficiency**: Remove parsing overhead for read operations
3. **Cache Efficiency**: Direct memory access improves cache locality

### MVCC Advantages

1. **Read Scalability**: Multiple concurrent readers without blocking
2. **Write Isolation**: Writers don't block readers
3. **Consistency**: Snapshot isolation for transactions

### Benchmarking Strategy

```rust
#[bench]
fn bench_node_read_sled(b: &mut Bencher) {
    let storage = create_sled_storage();
    b.iter(|| {
        storage.get_node("test_node").unwrap()
    });
}

#[bench]
fn bench_node_read_mdbx_zero_copy(b: &mut Bencher) {
    let storage = create_mdbx_storage();
    b.iter(|| {
        storage.get_node("test_node").unwrap()
    });
}
```

## Migration Path

### Phase 1: Abstraction Layer (Weeks 1-2)
1. Define `StorageEngine` trait
2. Implement `SledEngine` wrapper
3. Refactor `StorageManager` to be generic
4. Update `DatabaseCoordinator` for generic engines
5. Validate all existing tests pass

### Phase 2: Serialization Migration (Weeks 3-4)
1. Add `rkyv` dependencies
2. Create `RkyvMetadata` conversion layer
3. Add dual derives to data models
4. Implement serialization detection in `StorageManager`
5. Test bidirectional compatibility

### Phase 3: MdbxEngine Implementation (Weeks 5-6)
1. Add `libmdbx-rs` dependency
2. Implement `MdbxEngine` and `MdbxTransaction`
3. Create comprehensive test suite
4. Implement zero-copy deserialization
5. Performance benchmarking

### Phase 4: Integration & Migration (Weeks 7-8)
1. Create data migration tool
2. A/B testing framework
3. Production migration scripts
4. Performance validation
5. Rollback procedures

### Phase 5: Cleanup (Week 9)
1. Remove `sled` dependencies
2. Delete `SledEngine` wrapper
3. Update documentation
4. Final performance validation