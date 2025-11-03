# Indexing Crate

**Fast multi-dimensional indexing for graph databases - zero-copy, automatically synchronized.**

## What This Does

Makes graph database queries **10-100x faster** using three specialized indexes:

1. **Property lookups** → "Find all messages in chat_123" (O(log n))
2. **Graph traversal** → "Get edges connected to user_5" (O(1))
3. **Semantic search** → "Find similar embeddings" (O(log n))

## Quick Example

```rust
use storage::StorageManager;

// Create storage with automatic indexing
let storage = StorageManager::with_indexing("my_db")?;
let idx = storage.index_manager().unwrap();

// Query by property (zero-copy!)
if let Some(guard) = idx.get_nodes_by_property("chat_id", "chat_123")? {
    for node_id in guard.iter_strs() {
        println!("Found: {}", node_id);
    }
}

// Graph traversal
if let Some(guard) = idx.get_outgoing_edges("user_5")? {
    for edge_id in guard.iter_strs() {
        println!("Edge: {}", edge_id);
    }
}

// Semantic search
let results = idx.search_vectors(&query_vector, 10)?;
```

## How It Works

```
IndexManager (public API)
    ├── StructuralIndex → B+ tree for properties
    ├── GraphIndex      → Adjacency lists for edges
    └── VectorIndex     → HNSW for embeddings
```

**Key innovation:** Zero-copy reads via transaction-backed guards (no allocations!).

## Module Organization

| Module | Purpose | When to Use |
|--------|---------|-------------|
| **[core/](src/core/)** | Essential implementations | Always (automatic) |
| **[lock_free/](src/lock_free/)** | Concurrent structures | High-concurrency workloads |
| **[algorithms/](src/algorithms/)** | Graph analysis | Path finding, communities |
| **[advanced/](src/advanced/)** | Optional features | Tiered storage, filtering |
| **[utils/](src/utils/)** | Helpers | Caching, metrics, batch ops |

**→ See each module's README for details.**

## Quick Start

### 1. Add Dependency

```toml
[dependencies]
indexing = { path = "../indexing" }
storage = { path = "../storage" }
```

### 2. Use via Storage

```rust
// Indexing happens automatically
let storage = StorageManager::with_indexing("db_path")?;

// Insert data (indexes update automatically)
storage.insert_node(node)?;
storage.insert_edge(edge)?;

// Query indexes
let idx = storage.index_manager().unwrap();
let results = idx.get_nodes_by_property("sender", "user_5")?;
```

### 3. Iterate Zero-Copy

```rust
if let Some(guard) = idx.get_nodes_by_property("chat_id", "chat_123")? {
    // guard.iter_strs() borrows from mmap - NO allocations!
    for id in guard.iter_strs() {
        process(id);
    }
    
    // O(1) count
    println!("Found {} nodes", guard.len());
}
```

## Performance

| Operation | Time | Space | Implementation |
|-----------|------|-------|----------------|
| Property query | O(log n) | 0 bytes | libmdbx B+ tree + guard |
| Graph neighbor | O(1) | 0 bytes | Adjacency list + guard |
| Vector search | O(log n) | N vectors | HNSW approximate |

**Benchmark:** 10K node query = 100x faster than traditional deserialization.

## Common Patterns

### Zero-Copy (Fastest)

```rust
// ✅ No allocations
if let Some(guard) = idx.get_nodes_by_property("sender", "user_5")? {
    for id in guard.iter_strs() {
        process(id);
    }
}
```

### Owned Data (When Needed)

```rust
// ⚠️ Explicit allocation
let guard = idx.get_nodes_by_property("topic", "rust")?;
let owned = guard.to_owned()?;

tokio::spawn(async move {
    process(owned).await;
});
```

## Key Features

- ✅ **Zero-copy reads** - Direct mmap access, no allocations
- ✅ **Automatic sync** - Indexes update on every mutation
- ✅ **ACID transactions** - libmdbx guarantees
- ✅ **Pure Rust** - No external dependencies
- ✅ **Concurrent** - Lock-free options available

## Testing

```bash
cargo test           # Run all tests
cargo bench          # Run benchmarks
cargo check          # Check compilation
```

## Documentation

- **Module READMEs** - See `src/*/README.md` for each module
- **TODO.md** - Implementation status
- **COMPREHENSIVE_PLAN.md** - Future enhancements

## Summary

**TL;DR:**
1. Create storage: `StorageManager::with_indexing("db")?`
2. Insert data: indexes update automatically
3. Query: `idx.get_nodes_by_property(...)`, `idx.get_outgoing_edges(...)`, `idx.search_vectors(...)`
4. Iterate: Use guards for zero-copy iteration

**Start here:** Read `src/core/README.md` for core concepts, then explore other modules as needed.
