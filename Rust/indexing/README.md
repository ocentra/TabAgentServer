# Indexing Crate

**Universal index building service for MIA's 9+ databases - zero-copy, multi-resolution, semantically aware.**

> **📚 For complete architecture details, see [ARCHITECTURE.md](ARCHITECTURE.md)**  
> *Explains: What indexing IS, multi-resolution embeddings, how modules work together, the refactoring plan*

## What This Does

Builds search structures for ALL MIA databases:

1. **Semantic search** → HNSW indexes for vector similarity (O(log n))
2. **Graph traversal** → Adjacency lists for relationships (O(1))
3. **Property lookups** → B-tree indexes for metadata (O(log n))
4. **Multi-resolution** → Fast (0.6B) + Accurate (8B) embeddings

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

- **[ARCHITECTURE.md](ARCHITECTURE.md)** - **START HERE!** Complete explanation of what indexing is
- **Module READMEs** - `src/*/README.md` for each module
- **TODO.md** - Implementation status

## Key Concepts

### Indexing is a SERVICE, not a database owner!

```
storage owns → knowledge.mdbx
               ├── nodes (data)
               ├── edges (data)
               ├── structural_index ← INDEXING builds this
               └── graph_outgoing   ← INDEXING builds this

indexing builds indexes IN storage's database!
```

### Multi-Resolution Embeddings:

```
embeddings.mdbx
├── 0.6b_vectors → FAST search (Stage 1: 1M→1K candidates, <1ms)
└── 8b_vectors   → ACCURATE rerank (Stage 2: 1K→100 best, <10ms)
```

## Summary

**What indexing is:**
1. Universal index builder for ALL MIA databases
2. Provides: HNSW (vectors), adjacency lists (graph), B-trees (properties)
3. Takes storage's env pointers (doesn't create DBs)
4. Serves 9+ databases with same service

**Start reading:**
1. [ARCHITECTURE.md](ARCHITECTURE.md) - Full explanation
2. `src/core/README.md` - Core implementations
3. `src/algorithms/README.md` - Graph algorithms
4. `src/advanced/README.md` - Enterprise features
