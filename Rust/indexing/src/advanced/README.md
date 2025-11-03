# Advanced Features Module

**Optional performance enhancements and experimental features.**

## What's Here

- `hybrid.rs` - Hot/warm/cold tiered storage
- `payload_index.rs` - Metadata filtering for vectors
- `segment.rs` - Segment-based vector indexing
- `vector_storage.rs` - Alternative vector storage backends
- `memory_mapping.rs` - Custom memory mapping utilities
- `persistence.rs` - Alternative persistence strategies
- `optimized_graph.rs` - Memory-optimized graph structures

## What This Does

Provides **advanced features** for specific use cases:

- **Tiered storage** - Keep hot data in RAM, warm in cache, cold on disk
- **Metadata filtering** - Filter vectors by payload (age, category, etc.)
- **Segmentation** - Partition large indexes into segments
- **Custom storage** - Alternative backends for vectors
- **Optimized structures** - Memory-efficient graph representations

## When to Use

### Hybrid Tiered Storage (`hybrid.rs`)

**Use when:**
- You have millions of nodes/vectors
- Access patterns are skewed (80/20 rule)
- Memory is limited but you need speed

```rust
use indexing::advanced::hybrid::HybridIndexConfig;

let config = HybridIndexConfig {
    enabled: true,
    hot_layer: HotLayerConfig {
        max_entries: 10_000,  // Keep 10K hottest items in RAM
        ..Default::default()
    },
    ..Default::default()
};

let idx = IndexManager::with_config("db", config)?;
```

### Payload Filtering (`payload_index.rs`)

**Use when:**
- You need to filter vectors by metadata
- "Find similar embeddings from last week"
- "Search only in category X"

```rust
use indexing::advanced::payload_index::{PayloadIndex, PayloadFilter};

let mut idx = PayloadIndex::new();

// Add vector with metadata
idx.add_with_payload("vec_1", vector, payload! {
    "category" => "tech",
    "timestamp" => 1699999999,
    "author" => "user_5"
})?;

// Search with filter
let results = idx.search_filtered(
    &query,
    10,
    PayloadFilter::must("category", "tech")
        .and(PayloadFilter::range("timestamp", Some(start), Some(end)))
)?;
```

### Segment-Based Indexing (`segment.rs`)

**Use when:**
- You have very large vector indexes (>1M vectors)
- Need to scale horizontally
- Want to partition by time/category

```rust
use indexing::advanced::segment::SegmentBasedVectorIndex;

let mut idx = SegmentBasedVectorIndex::new("segments/")?;

// Vectors automatically partitioned into segments
idx.add_vector("vec_1", vector, Some("segment_2023_11"))?;
```

### Optimized Graph (`optimized_graph.rs`)

**Use when:**
- Memory usage is a concern
- Graph structure is mostly read-only
- Can sacrifice some write performance for memory

```rust
use indexing::advanced::optimized_graph::OptimizedGraphIndex;

let graph = OptimizedGraphIndex::new();
// Uses compact representations internally
```

## Features Overview

| Feature | Use Case | Memory Impact | Performance Impact |
|---------|----------|---------------|-------------------|
| Hybrid storage | Large datasets | -40% | +10-50% (queries) |
| Payload filtering | Metadata search | +20% | Depends on filter |
| Segmentation | Horizontal scaling | Neutral | Parallel queries |
| Vector storage | Custom backends | Varies | Varies |
| Optimized graph | Memory-constrained | -30% | -10% (writes) |

## Performance Tips

### Tiered Storage

```rust
// Configure tier thresholds
config.hot_layer.access_threshold = 10;  // 10+ accesses → hot
config.warm_layer.max_size_mb = 500;     // 500MB warm cache
```

### Payload Indexes

```rust
// Index frequently queried fields
payload_idx.create_index("category")?;
payload_idx.create_index("timestamp")?;

// Queries on indexed fields are 10-100x faster
```

## Experimental Warning

⚠️ Some modules in this folder are **experimental**:
- `vector_storage.rs` - Alternative backends (incomplete)
- `persistence.rs` - Custom persistence (in development)
- `memory_mapping.rs` - Low-level mmap (expert use only)

Use these only if you understand the tradeoffs!

## Examples

See tests in each file for usage examples:

```bash
cargo test --test advanced -- --nocapture
```

