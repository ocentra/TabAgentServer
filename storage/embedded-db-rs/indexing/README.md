# Indexing Crate

**Three-tier indexing system for fast multi-dimensional queries.**

## Purpose

The `indexing` crate provides high-performance indexes that enable different query patterns: property-based filtering, graph traversal, and semantic similarity search.

## Responsibilities

### 1. Structural Index (B-tree)
- Fast property-based lookups: O(log n)
- Indexes 12+ properties across all node types
- Examples: `chat_id`, `sender`, `url`, `mime_type`, `node_type`

### 2. Graph Index (Adjacency Lists)
- Bidirectional edge tracking
- O(1) neighbor lookup (outgoing/incoming)
- Essential for relationship traversal

### 3. Vector Index (HNSW)
- Hierarchical Navigable Small World algorithm
- O(log n) Approximate Nearest Neighbor search
- Optimized for 384/768/1536 dimensional embeddings
- Uses `hnsw_rs` library

## Architecture

```
IndexManager
  ├── structural: StructuralIndex
  │   └── sled::Tree (property → [node_ids])
  ├── graph: GraphIndex
  │   ├── outgoing: sled::Tree (from_node → [edge_ids])
  │   └── incoming: sled::Tree (to_node → [edge_ids])
  └── vector: VectorIndex
      ├── hnsw: Hnsw<f32> (in-memory graph)
      └── embeddings: HashMap (id → vector)
```

## Automatic Synchronization

Indexes are **automatically updated** by the `storage` crate on every mutation:
- `insert_node` → `index_node`
- `delete_node` → `unindex_node`
- `insert_edge` → `index_edge`
- `delete_edge` → `unindex_edge`
- `insert_embedding` → `index_embedding`
- `delete_embedding` → `unindex_embedding`

## Usage

### Initialization (via Storage)
```rust
use storage::StorageManager;

// IndexManager is created automatically
let storage = StorageManager::with_indexing("my_db")?;
let idx = storage.index_manager().unwrap();
```

### Structural Queries
```rust
// Find all messages in a chat
let messages = idx.get_nodes_by_property("chat_id", "chat_123")?;

// Find all chats about "Rust"
let rust_chats = idx.get_nodes_by_property("topic", "Rust")?;
```

### Graph Traversal
```rust
// Get all outgoing edges from a node
let outgoing = idx.get_outgoing_edges("chat_123")?;

// Get all incoming edges to a node
let incoming = idx.get_incoming_edges("msg_456")?;
```

### Semantic Search
```rust
// Find similar embeddings (vector from ML model)
let query_vector = vec![0.1; 384];
let similar = idx.search_vectors(&query_vector, 10)?;

for result in similar {
    println!("ID: {}, Distance: {}", result.id, result.distance);
}
```

## Performance

| Operation | Complexity | Implementation |
|-----------|------------|---------------|
| Property Query | O(log n) | sled B-tree |
| Graph Neighbor | O(1) | Adjacency list |
| Vector Search | O(log n) | HNSW ANN |
| Index Update | O(log n) | Automatic |

## Dependencies

- `common` - Shared types and models
- `sled` - For structural and graph indexes
- `hnsw_rs` - HNSW algorithm implementation

## See Also

- Parent: [Main README](../README.md)
- Integration: [storage/README.md](../storage/README.md)
- Progress: [TODO.md](./TODO.md)

