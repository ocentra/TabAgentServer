# Storage Crate

**Core CRUD operations and optional automatic indexing integration.**

## Purpose

The `storage` crate provides a safe, transactional interface for all database operations. It manages the primary data storage using `sled` and optionally integrates with the `indexing` crate for fast queries.

## Responsibilities

### 1. Primary Data Storage
- **Nodes**: All entity types (chats, messages, entities, etc.)
- **Edges**: Relationships between nodes
- **Embeddings**: Vector embeddings for semantic search

### 2. CRUD Operations
- Create (insert), Read (get), Update (insert), Delete
- Batch operations for efficiency
- Transaction support via `db()` method

### 3. Optional Index Integration
Two operation modes:
- **Basic**: `StorageManager::new(path)` - Fast, no indexing
- **Indexed**: `StorageManager::with_indexing(path)` - Automatic index updates on all mutations

### 4. Data Integrity
- ACID transactions via `sled`
- Binary serialization with `bincode`
- Automatic index synchronization (when enabled)

## Architecture

```
StorageManager
  ├── db: sled::Db
  ├── nodes: sled::Tree
  ├── edges: sled::Tree  
  ├── embeddings: sled::Tree
  └── index_manager: Option<Arc<IndexManager>>  ← Optional indexing
```

## Usage

### Basic Mode (No Indexing)
```rust
use storage::{StorageManager, Node, Chat};
use serde_json::json;

let storage = StorageManager::new("my_database")?;

let chat = Node::Chat(Chat {
    id: "chat_001".to_string(),
    title: "Project Discussion".to_string(),
    // ... other fields
    metadata: json!({}),
});

storage.insert_node(&chat)?;
let retrieved = storage.get_node("chat_001")?;
```

### With Automatic Indexing
```rust
let storage = StorageManager::with_indexing("my_database")?;

// Indexes update automatically!
storage.insert_node(&chat)?;

// Query indexes
if let Some(idx) = storage.index_manager() {
    let messages = idx.get_nodes_by_property("chat_id", "chat_123")?;
    let outgoing = idx.get_outgoing_edges("chat_123")?;
    let similar = idx.search_vectors(&query_vec, 10)?;
}
```

## Performance

| Operation | Complexity | Notes |
|-----------|------------|-------|
| Insert/Get/Delete | O(log n) | sled B-tree |
| Batch operations | Amortized | Single transaction |
| Index updates | O(log n) | Only when indexing enabled |

## Dependencies

- `common` - Shared types and models
- `indexing` - Optional index integration
- `sled` - Embedded database engine
- `bincode` - Binary serialization

## Production Deployment

For production database location strategy (platform-specific paths, user data directories), see [DATABASE_STRATEGY.md](./DATABASE_STRATEGY.md).

## See Also

- Parent: [Main README](../README.md)
- Models: [common/README.md](../common/README.md)
- Indexing: [indexing/README.md](../indexing/README.md)
- Progress: [TODO.md](./TODO.md)
- Deployment: [DATABASE_STRATEGY.md](./DATABASE_STRATEGY.md)

