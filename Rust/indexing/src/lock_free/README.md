# Lock-Free Concurrency Module

**High-performance concurrent data structures for multi-threaded workloads.**

## What's Here

- `lock_free.rs` - Core lock-free utilities (HashMap, AccessTracker, etc.)
- `lock_free_hot_vector.rs` - Lock-free vector index
- `lock_free_hot_graph.rs` - Lock-free graph index
- `lock_free_btree.rs` - Lock-free B-tree (experimental)
- `lock_free_skiplist.rs` - Lock-free skiplist (experimental)
- `lock_free_benchmark.rs` - Performance benchmarks
- `lock_free_stress_tests.rs` - Concurrency stress tests
- `LOCK_FREE_CONCURRENCY.md` - Design documentation

## What This Does

Provides **lock-free alternatives** to the core indexes for scenarios with:
- High concurrent read/write load
- Many threads accessing the same data
- Need to avoid lock contention

## When to Use

**Use when:** You have high-concurrency workloads with many threads.

**Don't use:** For single-threaded or low-concurrency scenarios (core indexes are simpler and sufficient).

## How to Use

### Enable Hot Indexes

```rust
use indexing::IndexManager;

let mut idx = IndexManager::new("db_path")?;

// Enable lock-free hot indexes
idx.enable_hot_indexes()?;

// Now queries automatically use lock-free structures when beneficial
let nodes = idx.get_nodes_by_property("chat_id", "chat_123")?;
```

### Direct Access

```rust
use indexing::{LockFreeHotVectorIndex, LockFreeHotGraphIndex};

// Lock-free vector index
let hot_vector = LockFreeHotVectorIndex::new();
hot_vector.add_vector("vec_1", vec![0.1; 384])?;
let results = hot_vector.search(&query, 10)?;

// Lock-free graph index
let hot_graph = LockFreeHotGraphIndex::new();
hot_graph.add_edge("node_1", "node_2", "edge_1")?;
let neighbors = hot_graph.get_neighbors("node_1")?;
```

## Key Features

### Lock-Free HashMap
- Uses crossbeam epoch-based reclamation
- Concurrent reads and writes without locks
- Automatic memory reclamation

### Access Tracking
- Tracks access frequency per key
- Useful for hot/warm/cold tiering decisions
- Thread-safe without locks

### Performance Stats
- Real-time performance metrics
- Average operation latency
- Success/failure rates

## Performance

**Benchmark Results:**
- 10K concurrent reads: 5x faster than Mutex-based
- 10K concurrent writes: 3x faster than RwLock
- Mixed workload (70% read, 30% write): 4x faster

## Tradeoffs

**Pros:**
- No lock contention
- Better scalability under high concurrency
- Consistent performance regardless of thread count

**Cons:**
- More complex implementation
- Slightly higher memory overhead
- Overkill for low-concurrency scenarios

## Testing

Run stress tests:
```bash
cargo test --test lock_free_stress_tests
```

Run benchmarks:
```bash
cargo bench --bench lock_free_benchmark
```

