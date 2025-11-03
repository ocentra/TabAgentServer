# Core Indexing Module

**Essential implementations - the foundation of the indexing system.**

## What's Here

- `structural.rs` - Property-based B+ tree indexing
- `graph.rs` - Graph adjacency list indexing  
- `vector.rs` - HNSW semantic search
- `zero_copy_ffi.rs` - Low-level libmdbx FFI
- `errors.rs` - Error types

## What This Does

These are the **three core indexes** that power the system:

1. **StructuralIndex** → Fast property lookups (O(log n))
2. **GraphIndex** → Fast relationship traversal (O(1))
3. **VectorIndex** → Semantic similarity search (O(log n))

## When to Use

**You don't use these directly.** The `IndexManager` in `lib.rs` orchestrates all three.

```rust
use indexing::IndexManager;

let idx = IndexManager::new("db_path")?;

// IndexManager uses all three core indexes internally
let nodes = idx.get_nodes_by_property("chat_id", "chat_123")?;  // → StructuralIndex
let edges = idx.get_outgoing_edges("user_5")?;                   // → GraphIndex
let similar = idx.search_vectors(&query, 10)?;                   // → VectorIndex
```

## Key Innovation: Zero-Copy

**All reads return guards, not owned data:**

```rust
// ✅ Zero-copy (fast)
if let Some(guard) = idx.get_nodes_by_property("sender", "user_5")? {
    for id in guard.iter_strs() {  // &str borrowed from mmap
        process(id);  // No allocations!
    }
}

// ⚠️ Owned data (only when needed)
let owned = guard.to_owned()?;  // Explicit allocation
```

## Implementation Details

### StructuralIndex
- Uses libmdbx B+ tree
- Keys: `"property:value"` → Values: `Vec<NodeId>`
- CRC32C validation on all reads
- MDBX_RESERVE for zero-copy writes

### GraphIndex  
- Two adjacency lists: outgoing + incoming
- Keys: `"out:node_id"` or `"in:node_id"` → Values: `Vec<EdgeId>`
- Bidirectional for fast traversal in both directions

### VectorIndex
- HNSW algorithm (hnsw_rs crate)
- In-memory graph structure
- Disk persistence via serialization
- Supports 384/768/1536 dimensional embeddings

## Performance

| Operation | Time | Space | Notes |
|-----------|------|-------|-------|
| Property query | O(log n) | 0 bytes | Zero-copy guard |
| Graph neighbor | O(1) | 0 bytes | Direct lookup |
| Vector search | O(log n) | N vectors | Approximate NN |

**Benchmark:** 10K node query = 100x faster than traditional deserialization.

