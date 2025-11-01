# Storage Crate

**Core CRUD operations with zero-copy serialization and optional automatic indexing.**

## Purpose

The `storage` crate provides a safe, transactional interface for all database operations in TabAgent's MIA (Multi-tier Intelligent Architecture). It manages primary data storage using `libmdbx` with **zero-copy** `rkyv` serialization and optionally integrates with the `indexing` crate for fast queries.

## Key Features

- **Zero-copy serialization** with `rkyv` for maximum performance
- **MVCC concurrency** via `libmdbx` - multiple readers, lock-free
- **Multi-tier architecture** - HOT/WARM/COLD temperature tiers
- **Automatic indexing** - optional integration with indexing crate
- **ACID transactions** - crash recovery and data integrity
- **Platform-specific paths** - seamless OS integration

## System Architecture

```text
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           APPLICATION LAYER                                      │
│                    (Server, API, Weaver, etc.)                                   │
└─────────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────────┐
│                      DATABASE COORDINATOR (Single Entry Point)                   │
│                                                                                  │
│  Manages 7 database types across temperature tiers:                              │
│  ├─ Conversations (Active, Recent, Archive)                                     │
│  ├─ Knowledge (Active, Stable, Inferred)                                        │
│  ├─ Embeddings (Active, Recent, Archive)                                        │
│  ├─ ToolResults (single tier)                                                   │
│  ├─ Experience (single tier)                                                    │
│  ├─ Summaries (Session, Daily, Weekly, Monthly)                                │
│  └─ Meta (query routing, stats)                                                 │
│                                                                                  │
│  HOT tiers: Always loaded in RAM                                                │
│  WARM/COLD tiers: Lazy-loaded on demand                                         │
└─────────────────────────────────────────────────────────────────────────────────┘
                                      │
                    ┌─────────────────┼─────────────────┐
                    ▼                 ▼                 ▼
        ┌───────────────────┐  ┌───────────────────┐  ┌───────────────────┐
        │ ConversationMgr   │  │ KnowledgeMgr      │  │ EmbeddingMgr      │
        │ ExperienceMgr     │  │ ToolResultMgr     │  │ SummaryMgr        │
        └───────────────────┘  └───────────────────┘  └───────────────────┘
                    │                 │                 │
                    └─────────────────┼─────────────────┘
                                      ▼
                    ┌─────────────────────────────────────┐
                    │  STORAGE MANAGER (Per Database)     │
                    │                                     │
                    │  StorageManager<MdbxEngine>         │
                    │    ├─ engine: MdbxEngine           │
                    │    ├─ index_manager: Option        │
                    │    ├─ db_type: DatabaseType        │
                    │    └─ tier: TemperatureTier        │
                    └─────────────────────────────────────┘
                                      │
                    ┌─────────────────┼─────────────────┐
                    ▼                 │                 ▼
        ┌───────────────────────┐    │    ┌───────────────────────┐
        │   CRUD OPERATIONS     │    │    │   INDEXING (Auto)     │
        │                       │    │    ├─ StructuralIndex     │
        │ insert_node(node)    │◄───┼───► ├─ GraphIndex          │
        │ get_node(id)         │    │    └─ VectorIndex          │
        │ delete_node(id)      │◄───┼───►                          │
        │ insert_edge(edge)    │    │                              │
        └───────────────────────┘    │                              │
                                      ▼                              │
        ┌─────────────────────────────────────────────────────────┐  │
        │  RKYV SERIALIZATION (Zero-Copy!)                       │  │
        │                                                         │  │
        │  WRITE: Node → rkyv::to_bytes() → serialized_bytes   │  │
        │  READ:  raw_bytes → check_archived_root()           │  │
        │         → deserialize() → Node (zero-copy!)         │  │
        └─────────────────────────────────────────────────────────┘  │
                                      │                              │
                                      ▼                              │
        ┌─────────────────────────────────────────────────────────────┐
        │                        MdbxEngine                            │
        │  ├─ env: Arc<Environment<NoWriteMap>>                       │
        │  └─ databases: Arc<DashMap<String, Database>>              │
        │                                                              │
        │  Operations:                                                │
        │    get()    → RO transaction → txn.get()                   │
        │    insert() → RW transaction → txn.put()                   │
        │    scan_prefix() → cursor.lower_bound()                    │
        └─────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
        ┌─────────────────────────────────────────────────────────────┐
        │                      LIBMDBX (Disk)                          │
        │  Each StorageManager = one .db file:                        │
        │  ├─ conversations/active.db                                 │
        │  ├─ knowledge/stable.db                                     │
        │  ├─ embeddings/active.db                                    │
        │  └─ ... (one file per tier)                                 │
        │                                                              │
        │  Inside each file:                                          │
        │  ├─ "nodes"     database (namespace)                        │
        │  ├─ "edges"     database                                    │
        │  └─ "embeddings" database                                   │
        │                                                              │
        │  MVCC: Concurrent reads, ACID writes                       │
        └─────────────────────────────────────────────────────────────┘
```

## Responsibilities

### 1. Primary Data Storage

- **Nodes**: All entity types (chats, messages, entities, etc.)
- **Edges**: Relationships between nodes
- **Embeddings**: Vector embeddings for semantic search

### 2. CRUD Operations

- Create (insert), Read (get), Update (insert), Delete
- Batch operations for efficiency
- Transaction support via engine abstraction

### 3. Optional Index Integration

Two operation modes:

- **Basic**: `StorageManager::new(path)` - Fast, no indexing
- **Indexed**: `StorageManager::with_indexing(path)` - Automatic index updates on all mutations

### 4. Data Integrity

- **ACID transactions** via libmdbx
- **Zero-copy serialization** with rkyv
- **MVCC concurrency** - lock-free reads
- **Automatic index synchronization** (when enabled)

## Usage

### Using DatabaseCoordinator (Recommended)

```rust
use storage::DatabaseCoordinator;

// Initialize all databases
let coordinator = DatabaseCoordinator::new()?;

// Insert a message (goes to conversations/active automatically)
coordinator.insert_message(Message {
    id: MessageId::new("msg_001"),
    content: "Hello, world!".to_string(),
    // ...
})?;

// Get across all tiers automatically
let msg = coordinator.get_message("msg_001")?;
```

### Direct StorageManager Usage

```rust
use storage::{StorageManager, Node, Chat};
use serde_json::json;

// Create storage with indexing enabled
let storage = StorageManager::with_indexing("my_database")?;

let chat = Node::Chat(Chat {
    id: NodeId::new("chat_001"),
    title: "Project Discussion".to_string(),
    // ... other fields
    metadata: json!({}).to_string(),
});

// Insert and auto-index
storage.insert_node(&chat)?;

// Get with zero-copy deserialization
let retrieved = storage.get_node("chat_001")?;

// Query indexes
if let Some(idx) = storage.index_manager() {
    let messages = idx.get_nodes_by_property("chat_id", "chat_123")?;
    let outgoing = idx.get_outgoing_edges("chat_123")?;
}
```

### Multi-tier Temperature Architecture

```rust
use storage::{DatabaseCoordinator, TemperatureTier};

let coordinator = DatabaseCoordinator::new()?;

// HOT tier: Always loaded, <1ms queries
coordinator.conversations_active.insert_message(msg)?;

// WARM tier: Lazy-loaded on first access, <10ms queries
let recent_manager = coordinator.get_or_load_conversations_recent()?;
recent_manager?.insert_message(old_msg)?;

// COLD tier: On-demand per quarter, 100ms queries (acceptable)
let archive = coordinator.get_or_load_conversations_archive("2024-Q1")?;
archive?.insert_message(very_old_msg)?;
```

## Performance

| Operation | Complexity | Notes |
|-----------|------------|-------|
| **Insert/Get/Delete** | O(log n) | libmdbx B+ tree |
| **Reads** | **Lock-free** | MVCC, no blocking |
| **Batch operations** | Amortized | Single transaction |
| **Index updates** | O(log n) | Only when indexing enabled |
| **Deserialization** | **Zero-copy** | rkyv archived reads |
| **Concurrent reads** | **Unlimited** | MVCC isolation |

## Dependencies

- `common` - Shared types and models with rkyv derives
- `indexing` - Optional index integration using libmdbx
- `libmdbx` - Embedded database engine (LMDB fork)
- `rkyv` - Zero-copy serialization framework
- `dashmap` - Concurrent HashMap for database handles

## Production Deployment

For production database location strategy (platform-specific paths, user data directories), see [DATABASE_STRATEGY.md](./DATABASE_STRATEGY.md).

## Migration from sled/bincode

✅ **COMPLETE**: All storage operations now use `libmdbx` + `rkyv`

**Benefits:**

- Zero-copy reads - no deserialization overhead
- MVCC concurrency - unlimited concurrent readers
- Better write performance - copy-on-write semantics
- Lower memory usage - memory-mapped files
- Cross-platform compatibility

## Testing

The storage crate includes comprehensive tests:

- 10 unit tests covering core functionality
- 18 integration tests covering database operations
- 15 documentation tests covering all public APIs
- All tests validate real functionality with temporary databases

## See Also

- Parent: [Main README](../README.md)
- Models: [common/README.md](../common/README.md)
- Indexing: [indexing/README.md](../indexing/README.md)
- Progress: [TODO.md](./TODO.md)
- Deployment: [DATABASE_STRATEGY.md](./DATABASE_STRATEGY.md)
