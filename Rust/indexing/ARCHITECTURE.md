# Indexing Crate: Complete Architecture Documentation

**Last Updated**: November 5, 2025  
**Status**: Refactoring in progress - fixing architectural issues

---

## Table of Contents

1. [What IS Indexing?](#what-is-indexing)
2. [The Universal Pattern](#the-universal-pattern)
3. [Multi-Resolution Embedding Strategy](#multi-resolution-embedding-strategy)
4. [Module Organization](#module-organization)
5. [Current Architecture (What Exists)](#current-architecture-what-exists)
6. [The Fucked Up Parts](#the-fucked-up-parts)
7. [Correct Architecture (Target)](#correct-architecture-target)
8. [Dependency Flow](#dependency-flow)
9. [How Indexing Serves 9+ Databases](#how-indexing-serves-9-databases)
10. [Testing Strategy](#testing-strategy)

---

## What IS Indexing?

### NOT a Database!

**Indexing is a UNIVERSAL INDEX BUILDING SERVICE that builds search structures for ANY database.**

Think of a book index at the back:
```
Book (STORAGE):
  Page 1: "Rust is great..."      ← ACTUAL DATA
  Page 5: "MDBX is fast..."       ← ACTUAL DATA
  
Index (INDEXING):
  "MDBX" → See page 5             ← POINTER (no content!)
  "Rust" → See pages 1, 45, 203   ← POINTER (no content!)
```

**The index NEVER owns the book - it just points to pages in the book!**

### Core Principle: Indexes = Metadata, Not Data

```
┌─────────────────────────────────────────┐
│ STORAGE owns: knowledge.mdbx            │
│  ├── nodes → {actual node data}         │
│  ├── edges → {actual edge data}         │
│  ├── structural_index → {pointers}      │ ← INDEXING builds this!
│  ├── graph_outgoing → {adjacency}       │ ← INDEXING builds this!
│  └── graph_incoming → {adjacency}       │ ← INDEXING builds this!
└─────────────────────────────────────────┘
```

**INDEXING builds the index tables, but STORAGE owns the database file!**

---

## The Universal Pattern

### Every Searchable Database Follows This Flow:

```
┌──────────────┐
│  RAW DATA    │  (Messages, entities, actions, etc.)
└──────┬───────┘
       │
       ▼ (Embedding model generates)
┌──────────────┐
│  EMBEDDINGS  │  (Vector representations)
└──────┬───────┘
       │
       ▼ (INDEXING builds)
┌──────────────┐
│  HNSW INDEX  │  (Fast approximate search)
└──────┬───────┘
       │
       ▼ (QUERY uses)
┌──────────────┐
│    RESULTS   │  (Top-k similar items)
└──────────────┘
```

### Applied to ALL MIA Databases:

| Database | Raw Data | Embeddings | Index | Query Use Case |
|----------|----------|------------|-------|----------------|
| `conversations.mdbx` | Messages | → `embeddings.mdbx` | HNSW_0.6b, HNSW_8b | "Find similar messages" |
| `experience.mdbx` | Action outcomes | Self-contained | HNSW_actions | "Similar situations I handled?" |
| `tool-results.mdbx` | Scraped pages | Self-contained | HNSW_pages | "Did we scrape this before?" |
| `knowledge.mdbx` | Entities + edges | Concept embeddings | Graph adjacency | "How is A related to B?" |

**INDEXING serves ALL of them - it's a universal service!**

---

## Multi-Resolution Embedding Strategy

### The Problem: Speed vs Accuracy Tradeoff

```
❌ Tiny model (0.6B params):
   - Fast: <100ms to embed
   - Inaccurate: Misses nuances
   
❌ Huge model (8B params):
   - Accurate: Captures subtle meanings
   - Slow: ~1-2 seconds to embed
```

### The Solution: Two-Stage Retrieval

```
embeddings.mdbx (STORAGE owns this!)
├── 0.6b_vectors → {msg_1: [0.23, -0.15, ...] (384D)}      ← FAST model
├── 8b_vectors → {msg_1: [0.2301, -0.1502, ...] (1536D)}   ← ACCURATE model
├── hnsw_0.6b → Index for fast search     ← INDEXING builds
└── hnsw_8b → Index for accurate search   ← INDEXING builds
```

### Two-Stage RAG Pipeline:

```rust
// STAGE 1: Fast Retrieval (0.6B embeddings)
// =========================================
// Problem: Need to search 1,000,000 messages FAST
// Solution: Use tiny model's index

let candidates = indexing.search_hnsw(
    embeddings_env,
    hnsw_0_6b_dbi,
    query_vec_0_6b,
    k=1000
);  
// Speed: <1ms
// Quality: "In the ballpark" (70% accuracy)
// Returns: 1000 candidate message IDs


// STAGE 2: Accurate Reranking (8B embeddings)
// ============================================
// Problem: 1000 candidates have noise, need best 100
// Solution: Rerank with high-quality model

let reranked = candidates
    .iter()
    .map(|msg_id| {
        // Get 8B embedding for SAME message
        let vec_8b = storage.get_vector(embeddings_env, "8b_vectors", msg_id)?;
        let score = cosine(query_vec_8b, vec_8b);  // High precision!
        (msg_id, score)
    })
    .sort_by_score()
    .take(100);

// Speed: ~10ms (only 1000 comparisons, not 1M!)
// Quality: "Best of the best" (95% accuracy)
// Returns: Top 100 most relevant messages
```

### Why This is Genius:

✅ **Fast response:** Stage 1 returns in <1ms  
✅ **High quality:** Stage 2 finds truly best matches  
✅ **Scalable:** Works with millions of vectors  
✅ **Cost-effective:** Only run expensive model on top candidates

**Like Google Search:**
- Stage 1: Quick index scan (returns 10,000 pages)
- Stage 2: PageRank + relevance (returns top 10)

### Async Embedding Flow:

```
User sends message
    │
    ├─→ [IMMEDIATE] 0.6B model embeds
    │   1. Storage saves to embeddings.mdbx/0.6b_vectors
    │   2. INDEXING updates HNSW_0.6b index
    │   3. User gets response <100ms ✅
    │
    └─→ [BACKGROUND] task-scheduler queues
        1. 8B model embeds (minutes later)
        2. Storage saves to embeddings.mdbx/8b_vectors
        3. INDEXING updates HNSW_8b index
        4. High-quality search available for future queries
```

**User gets IMMEDIATE response with fast model, while accurate model works in background!**

---

## Module Organization

### The Four Pillars of Indexing:

```
indexing/
├── algorithms/   → Pure graph algorithms (NO storage)
├── core/         → Persistent index builders (uses storage's MDBX)
├── lock_free/    → Hot in-memory tier (RAM only)
└── advanced/     → Enterprise features (hybrid, segmentation, etc.)
```

### 1. `algorithms/` - Pure Algorithms Library

**Like petgraph - NO database dependency!**

```
algorithms/
├── dijkstra.rs              → Shortest path
├── page_rank.rs             → Importance scoring
├── scc/tarjan_scc.rs        → Strongly connected components
├── bellman_ford.rs          → Negative edge weights
├── astar.rs                 → Heuristic pathfinding
├── community_detection.rs   → Clustering
└── 20+ more graph algorithms
```

**What it does:**
- ✅ Pure computational algorithms
- ✅ Works on ANY graph structure
- ✅ Zero-copy iteration over MDBX data
- ✅ NO database creation or persistence

**Example:**
```rust
// Works on any graph data source
let path = algorithms::dijkstra(&graph, "start", "end", |edge| 1.0);
let ranks = algorithms::page_rank(&graph, &nodes, 0.85, 100);
```

**Dependencies:** NONE (just data structures)

---

### 2. `core/` - Persistent Index Managers

**Builds indexes that persist to MDBX (via storage's env pointers)**

```
core/
├── structural.rs   → Property indexes (B-tree)
├── graph.rs        → Adjacency lists (relationships)
└── vector.rs       → HNSW (semantic search)
```

**What it does:**
- Takes `(env, dbi)` pointers from storage
- Builds/updates indexes
- Provides zero-copy query APIs

**Example:**
```rust
// CORRECT usage (future):
let structural = StructuralIndex::new(
    storage.env("knowledge.mdbx"),      // From storage!
    storage.dbi("structural_index")     // From storage!
);

structural.add("chat_id", "chat_123", "msg_1")?;
let guard = structural.get("chat_id", "chat_123")?;
for node_id in guard.iter_strs() {  // Zero-copy!
    println!("{}", node_id);
}
```

**Current Issue:** `IndexManager::new(path)` creates its own MDBX environment (WRONG!)  
**Being Fixed:** Will take storage's env pointers instead

---

### 3. `lock_free/` - Hot In-Memory Tier

**Fast concurrent structures for frequently accessed data (RAM only)**

```
lock_free/
├── lock_free_hot_vector.rs  → Concurrent vector index (RAM)
├── lock_free_hot_graph.rs   → Concurrent graph (RAM)
├── lock_free.rs             → LockFreeHashMap
├── lock_free_btree.rs       → Concurrent B-tree
└── lock_free_skiplist.rs    → Skip list
```

**What it does:**
- In-memory only (no persistence)
- Lock-free concurrent access (high performance)
- Eventually synced to cold MDBX tier

**The 3-Tier Pattern:**
```
HOT (RAM)     → LockFreeHotVectorIndex.search()     <1ms
    ↓ miss
WARM (Cache)  → WarmVectorCache.search()           <10ms
    ↓ miss
COLD (MDBX)   → VectorIndex.search()               <100ms
```

**Example:**
```rust
// Hot tier for recent vectors
let hot = LockFreeHotVectorIndex::new();
hot.add_vector("vec_1", vec![0.1, 0.2, 0.3])?;  // Instant!

// On cache miss, fallback to cold
let results = if let Some(r) = hot.search(&query, 10)? {
    r
} else {
    cold_index.search(&query, 10)?  // Slower but comprehensive
};
```

---

### 4. `advanced/` - Enterprise Features

**Production-grade enhancements for large-scale deployments**

```
advanced/
├── hybrid.rs           → Hot/Warm/Cold coordination (2032 lines!)
├── payload_index.rs    → Metadata filtering (759 lines)
├── segment.rs          → Segment-based indexing
├── persistence.rs      → Saving/loading strategies
├── vector_storage.rs   → Mmap vector files
└── optimized_graph.rs  → Memory-efficient graphs
```

**What it does:**
- Coordinates hot/warm/cold tiers
- Adds metadata filtering to vector search
- Manages large indexes via segmentation
- Optimizes memory usage

**Example:**
```rust
// Hybrid index with 3 tiers
let config = HybridIndexConfig {
    hot_layer: HotLayerConfig {
        max_entries: 10_000,  // Keep 10K in RAM
    },
    warm_layer: WarmLayerConfig {
        max_size_mb: 500,     // 500MB cache
    },
    ..Default::default()
};

let idx = IndexManager::with_config("db", config)?;

// Automatically uses hot/warm/cold tiers
let results = idx.search_vectors(&query, 10)?;
```

---

## Current Architecture (What Exists)

### The Broken Pattern (Being Fixed):

```
❌ CURRENT (WRONG):

indexing/src/lib.rs:
  IndexManager::new("path") {
      mdbx_env_create(&mut env);     ← Creates its OWN database!
      mdbx_env_open(env, path, ...);
      // Opens tables: structural_index, graph_outgoing, graph_incoming
  }

Result: SEPARATE database files!
  knowledge.mdbx    ← Storage's data
  indexes.mdbx      ← Indexing's indexes (SEPARATED! BAD!)
```

**Why this is wrong:**
- Indexing owns a database (shouldn't!)
- Indexes separated from data (bad for atomicity)
- Tests use `IndexManager::new(temp_dir)` (convenient but wrong pattern)

---

## The Fucked Up Parts

### Issue #1: Test Convenience Leaked into Production API

```rust
// indexing/tests/common/mod.rs
pub fn setup_real_db() -> (IndexManager, TempDir) {
    let temp_dir = TempDir::new()?;
    let manager = IndexManager::new(temp_dir.path())?;  // ← Convenient for tests!
    (manager, temp_dir)
}
```

**Problem:** Tests needed a quick way to create IndexManager  
**Solution:** Made `IndexManager::new(path)` create its own MDBX environment  
**Result:** This pattern LEAKED into production - now indexing owns a database!

### Issue #2: Dependency Inversion

```
❌ CURRENT (BACKWARDS):
indexing → mdbx-sys (direct MDBX access)
storage  → mdbx-sys (direct MDBX access)

Both crates create databases independently!
```

**Should be:**
```
✅ CORRECT:
indexing → storage (gets env pointers)
storage  → mdbx-base (ONLY storage touches MDBX)
```

### Issue #3: Tests Create 233+ Temporary Databases

Every test creates its own MDBX environment:
```rust
#[test]
fn test_add_and_get_outgoing() {
    let (manager, _temp) = setup_real_db();  // Creates temp MDBX!
    // ...
}
```

**Problem:** 233 tests × 1 MDBX environment = resource waste  
**Should be:** Mock or shared test environment with storage's env pointer

---

## Correct Architecture (Target)

### The Universal Service Pattern:

```
✅ CORRECT:

┌─────────────────────────────────────────┐
│ mdbx-base (ONLY crate with raw MDBX)   │
│  - MdbxEnvBuilder                       │
│  - txn_pool (thread-local)              │
│  - zero_copy_ffi                        │
└────────────┬────────────────────────────┘
             │ uses abstractions
             ▼
┌─────────────────────────────────────────┐
│ storage (Owns ALL 9+ databases)         │
│  - Creates MDBX environments            │
│  - Manages all data tables              │
│  - Gives indexing env+dbi pointers      │
└────────────┬────────────────────────────┘
             │ gives env pointers
             ▼
┌─────────────────────────────────────────┐
│ indexing (Universal index service)      │
│  - Builds indexes IN storage's DBs      │
│  - NEVER creates its own database       │
│  - Pure service: takes pointers, builds │
└─────────────────────────────────────────┘
```

### New API (Target):

```rust
// STORAGE creates and owns all databases
impl KnowledgeDatabase {
    pub fn new(path: &Path) -> DbResult<Self> {
        let env = MdbxEnvBuilder::new(path)
            .with_max_dbs(10)
            .with_size_gb(100)
            .open()?;  // Storage creates environment
        
        let data_dbis = open_multiple_dbis(env, &["nodes", "edges"], true)?;
        let index_dbis = open_multiple_dbis(env, &["structural_index", "graph_outgoing", "graph_incoming"], true)?;
        
        // Give indexing our env and index DBIs
        let graph_index = GraphIndex::new(env, index_dbis[1], index_dbis[2]);
        let structural_index = StructuralIndex::new(env, index_dbis[0]);
        
        Ok(Self { env, data_dbis, graph_index, structural_index })
    }
}

// INDEXING just takes pointers, doesn't create DB!
impl GraphIndex {
    pub fn new(
        env: *mut MDBX_env,      // From storage
        outgoing_dbi: MDBX_dbi,  // From storage
        incoming_dbi: MDBX_dbi,  // From storage
    ) -> Self {
        Self { env, outgoing_dbi, incoming_dbi }
        // NO mdbx_env_create! Just uses what storage gave us!
    }
}
```

---

## The Complete System: Three Crates Working Together

### The Correct Three-Layer Architecture:

```
┌─────────────────────────────────────────────────────────┐
│ mdbx-base (Foundation - ONLY crate with raw MDBX)      │
│  - MdbxEnvBuilder, txn_pool, zero_copy_ffi             │
│  - NO domain logic, just MDBX abstractions             │
└────────────┬────────────────────────────────────────────┘
             │ uses abstractions
             ▼
┌─────────────────────────────────────────────────────────┐
│ storage (Database Owner - Creates & manages ALL DBs)   │
│  - Creates MDBX environments for 9+ databases          │
│  - Manages data tables (nodes, edges, messages, etc.)  │
│  - Gives pointers to embedding & indexing crates       │
└────┬────────────────────────────────┬─────────────────┘
     │ gives env+dbi                  │ gives env+dbi
     ▼                                ▼
┌─────────────────────┐    ┌──────────────────────────┐
│ embedding           │    │ indexing                 │
│ (Embedding Service) │    │ (Index Building Service) │
├─────────────────────┤    ├──────────────────────────┤
│ - 0.6B model (fast) │    │ - HNSW builder           │
│ - 8B model (hi-res) │    │ - Graph index builder    │
│ - Reranking model   │    │ - Structural index       │
│                     │    │ - Graph algorithms       │
│ Does NOT create DB! │    │ Does NOT create DB!      │
│ Takes: (db, data)   │    │ Takes: (env, dbi)        │
│ Returns: embeddings │    │ Returns: indexes         │
└─────────────────────┘    └──────────────────────────┘
```

### Responsibility Separation:

| Crate | Creates DB? | What It Does | What It Takes | What It Returns |
|-------|-------------|--------------|---------------|-----------------|
| **mdbx-base** | ✅ Provides tools | MDBX abstractions | - | Builders, helpers |
| **storage** | ✅ YES! | Creates & owns ALL databases | Path | MDBX env+dbi pointers |
| **embedding** | ❌ NO! | Generates embeddings | (db, data) | Stored embeddings |
| **indexing** | ❌ NO! | Builds search indexes | (env, dbi) | Search structures |

---

## The Embedding Crate (Critical Missing Piece!)

### What embedding IS:

**Embedding = Service that generates vectors and stores them to a database (given by storage).**

```rust
// embedding/src/lib.rs (stub to be created)

pub struct EmbeddingService {
    model_0_6b: FastEmbeddingModel,    // 0.6B params - fast
    model_8b: AccurateEmbeddingModel,  // 8B params - accurate
    reranker: RerankingModel,          // Reranking model
}

impl EmbeddingService {
    /// Embed text with fast model (0.6B)
    pub fn embed_fast(
        &self,
        db_env: *mut MDBX_env,        // From storage!
        vectors_dbi: MDBX_dbi,        // From storage!
        text: &str,
        id: EmbeddingId,
    ) -> DbResult<Vec<f32>> {
        // 1. Chunk text if needed
        let chunks = self.chunk_text(text, max_tokens: 512);
        
        // 2. Embed with 0.6B model
        let embedding = self.model_0_6b.encode(&chunks)?;  // [0.23, -0.15, ...] (384D)
        
        // 3. Store to database (given by storage!)
        storage::save_vector(db_env, vectors_dbi, id, &embedding)?;
        
        Ok(embedding)
    }
    
    /// Embed text with accurate model (8B) - background task
    pub fn embed_accurate(
        &self,
        db_env: *mut MDBX_env,        // From storage!
        vectors_dbi: MDBX_dbi,        // From storage!
        text: &str,
        id: EmbeddingId,
    ) -> DbResult<Vec<f32>> {
        let chunks = self.chunk_text(text, max_tokens: 512);
        let embedding = self.model_8b.encode(&chunks)?;  // [0.2301, -0.1502, ...] (1536D)
        storage::save_vector(db_env, vectors_dbi, id, &embedding)?;
        Ok(embedding)
    }
    
    /// Rerank candidates with high-quality model
    pub fn rerank(
        &self,
        query: &str,
        candidates: &[(EmbeddingId, String)],  // (id, text) pairs
        top_k: usize,
    ) -> DbResult<Vec<(EmbeddingId, f32)>> {
        // Use reranking model for final scoring
        let scores = self.reranker.score(query, &candidates)?;
        let ranked = scores.into_iter()
            .sorted_by(|a, b| b.1.partial_cmp(&a.1).unwrap())
            .take(top_k)
            .collect();
        Ok(ranked)
    }
}
```

### Embedding Crate Responsibilities:

1. **Load Models:**
   - 0.6B model (fast, low quality) - e.g., MiniLM
   - 8B model (slow, high quality) - e.g., Qwen-8B-Instruct
   - Reranking model - e.g., Cross-encoder

2. **Text Processing:**
   - Chunking (split long text into 512-token chunks)
   - Pooling (mean/CLS token pooling)
   - Normalization (L2 normalize vectors)

3. **Embedding Generation:**
   - `embed_fast()` - 0.6B model, returns in <100ms
   - `embed_accurate()` - 8B model, returns in ~1-2s
   - `rerank()` - Cross-encoder for final ranking

4. **Storage:**
   - Takes database env+dbi from storage
   - Stores vectors to given table
   - Does NOT create its own database!

### The Complete Flow:

```
User sends message → "How to optimize Rust code?"
    │
    ▼
[STORAGE] Creates message in conversations.mdbx
    │
    ├─→ [EMBEDDING - IMMEDIATE]
    │   1. embedding.embed_fast(
    │          embeddings_env,          ← From storage
    │          "0.6b_vectors",          ← From storage
    │          "How to optimize...",
    │          msg_1
    │      )
    │   2. Returns: [0.23, -0.15, ...] (384D)
    │   3. Stored in: embeddings.mdbx/0.6b_vectors
    │
    ├─→ [INDEXING - IMMEDIATE]
    │   1. indexing.add_to_hnsw(
    │          embeddings_env,          ← From storage
    │          "hnsw_0.6b",             ← From storage
    │          msg_1,
    │          [0.23, -0.15, ...]
    │      )
    │   2. HNSW index updated
    │   3. Fast search ready! <100ms total ✅
    │
    └─→ [BACKGROUND - task-scheduler queues]
        1. embedding.embed_accurate(
               embeddings_env,
               "8b_vectors",
               "How to optimize...",
               msg_1
           )
        2. Returns: [0.2301, -0.1502, ...] (1536D)
        3. Stored in: embeddings.mdbx/8b_vectors
        4. indexing.add_to_hnsw(embeddings_env, "hnsw_8b", msg_1, ...)
        5. Accurate search ready! (minutes later)
```

### Embedding Crate Does NOT:

❌ Create databases  
❌ Manage MDBX environments  
❌ Build indexes (indexing's job!)  
❌ Store raw data (storage's job!)

### Embedding Crate DOES:

✅ Load 3 models (0.6B, 8B, reranker)  
✅ Chunk text appropriately  
✅ Generate embeddings  
✅ Store to database (given by storage)  
✅ Provide reranking service

---

## Dependency Flow

```
indexing/Cargo.toml:
  mdbx-sys = "13.8.0"    ← WRONG! Direct MDBX access
  libmdbx = "0.6.3"      ← WRONG!
  mdbx-base = { ... }    ← For helpers
```

### Target (Clean):

```
indexing/Cargo.toml:
  mdbx-base = { ... }    ← ONLY this! For abstractions
  # NO mdbx-sys
  # NO libmdbx
  # Storage provides env pointers
```

**Only `mdbx-base` exposes MDBX functionality!**

---

## How Indexing Serves 9+ Databases

### MIA's Database Landscape:

From `MIA_VISION.md` and `mia_memory.md`:

1. **conversations.mdbx** - Chats, messages (SOURCE)
2. **embeddings.mdbx** - 0.6B + 8B vectors + HNSW indexes
3. **knowledge.mdbx** - Entities, edges, graph indexes
4. **experience.mdbx** - Actions, feedback, pattern learning
5. **tool-results.mdbx** - Search cache, scraped pages
6. **summaries.mdbx** - Daily/weekly/monthly summaries
7. **meta.mdbx** - Query routing, performance stats
8. **model-cache.mdbx** - ML model files
9. **logs.mdbx** - System logs

**Each with 3 tiers (active/recent/archive) = 27+ MDBX files!**

### Indexing Service Pattern:

```rust
// Universal HNSW builder (works for ANY database)
pub fn build_hnsw_for_database(
    db_name: &str,           // "embeddings", "experience", "tool-results"
    vectors_table: &str,     // "0.6b_vectors", "action_embeddings", etc.
    index_table: &str,       // "hnsw_0.6b", "hnsw_actions", etc.
    dimension: usize,
) -> DbResult<()> {
    let env = storage.env(db_name)?;
    let vectors_dbi = storage.dbi(db_name, vectors_table)?;
    let index_dbi = storage.dbi(db_name, index_table)?;
    
    // Build HNSW index
    indexing.build_hnsw(env, vectors_dbi, index_dbi, dimension)?;
    Ok(())
}
```

### Usage Across All Databases:

```rust
// 1. Conversations (via embeddings.mdbx)
build_hnsw_for_database("embeddings", "0.6b_vectors", "hnsw_0.6b", 384)?;
build_hnsw_for_database("embeddings", "8b_vectors", "hnsw_8b", 1536)?;

// 2. Experience (self-contained)
build_hnsw_for_database("experience", "action_embeddings", "hnsw_actions", 384)?;

// 3. Tool-Results (self-contained)
build_hnsw_for_database("tool-results", "page_embeddings", "hnsw_pages", 384)?;
build_hnsw_for_database("tool-results", "search_embeddings", "hnsw_searches", 384)?;

// 4. Knowledge (graph indexes, not HNSW)
build_graph_index("knowledge", "edges", "graph_outgoing", "graph_incoming")?;
```

**Same indexing service, different databases!**

---

## Module Details

### algorithms/ - Pure Computation (0 Dependencies)

**Philosophy:** Like petgraph - pure algorithmic library

**What's Inside:**
- 30+ graph algorithms from petgraph (adapted for zero-copy)
- Distance metrics (cosine, euclidean, jaccard, etc.)
- SIMD-optimized distance calculations
- Graph traits for generic operations

**NO Database Code:**
```rust
// Check the dependencies
algorithms/ imports:
  ✅ std::collections
  ✅ petgraph traits (for compatibility)
  ❌ NO mdbx-sys
  ❌ NO storage
  ❌ NO database code
```

**Use Cases:**
- Path finding (Dijkstra, A*, Bellman-Ford)
- Centrality (PageRank, betweenness)
- Community detection (Louvain, modularity)
- Flow problems (Ford-Fulkerson, Dinic's)
- Structural analysis (SCC, bridges, articulation points)

**See:** `src/algorithms/README.md` for complete algorithm list

---

### core/ - Index Builders (Takes Storage Pointers)

**Philosophy:** Build indexes IN storage's databases

**What's Inside:**

#### **StructuralIndex** (Property Queries)
```rust
pub struct StructuralIndex {
    env: *mut MDBX_env,    // From storage!
    dbi: MDBX_dbi,         // From storage!
}

impl StructuralIndex {
    pub fn add(&self, property: &str, value: &str, node_id: &str);
    pub fn get(&self, property: &str, value: &str) -> Option<StructuralIndexGuard>;
    pub fn remove(&self, property: &str, value: &str, node_id: &str);
}
```

**Builds:** `prop:chat_id:chat_123 → [msg_1, msg_2, msg_5]`  
**Persists to:** Storage's database table `structural_index`

#### **GraphIndex** (Adjacency Lists)
```rust
pub struct GraphIndex {
    env: *mut MDBX_env,        // From storage!
    outgoing_dbi: MDBX_dbi,    // From storage!
    incoming_dbi: MDBX_dbi,    // From storage!
}

impl GraphIndex {
    pub fn add_edge(&self, edge: &Edge);
    pub fn get_outgoing(&self, node_id: &str) -> Option<GraphIndexGuard>;
    pub fn get_incoming(&self, node_id: &str) -> Option<GraphIndexGuard>;
}
```

**Builds:** Bidirectional adjacency lists  
**Persists to:** Storage's tables `graph_outgoing`, `graph_incoming`

#### **VectorIndex** (HNSW for ANN)
```rust
pub struct VectorIndex {
    hnsw: Arc<RwLock<Hnsw<f32, DistCosine>>>,  // In-memory graph
    vector_storage: MmapVectorStorage,          // Mmap file for vectors
    // ...
}

impl VectorIndex {
    pub fn add_vector(&self, id: EmbeddingId, vector: Vec<f32>);
    pub fn search(&self, query: &[f32], k: usize) -> Vec<SearchResult>;
}
```

**Builds:** HNSW graph structure  
**Persists to:** Separate mmap file (coordinated by storage)

**See:** `src/core/README.md` for implementation details

---

### lock_free/ - Concurrent Hot Tier

**Philosophy:** Maximum performance for frequently accessed data

**What's Inside:**

#### **LockFreeHotVectorIndex**
```rust
pub struct LockFreeHotVectorIndex {
    vectors: Arc<LockFreeHashMap<EmbeddingId, Vec<f32>>>,
    access_tracker: Arc<LockFreeAccessTracker>,
    stats: Arc<LockFreeStats>,
}
```

**Use Case:**
- Recent vectors that need <1ms search
- High concurrent read/write load
- Eventually synced to cold tier

#### **LockFreeHotGraphIndex**
```rust
pub struct LockFreeHotGraphIndex {
    nodes: Arc<LockFreeHashMap<NodeId, NodeData>>,
    edges: Arc<LockFreeHashMap<NodeId, Vec<EdgeId>>>,
}
```

**Use Case:**
- Recent graph data in RAM
- Lock-free concurrent updates
- Fast neighbor lookups

**Data Structures:**
- `LockFreeHashMap` - Lock-free hash table
- `LockFreeBTree` - Concurrent B-tree
- `LockFreeSkipList` - Concurrent skip list
- `LockFreeAccessTracker` - Track hot items
- `LockFreeStats` - Performance metrics

**See:** `src/lock_free/README.md` for concurrency details

---

### advanced/ - Enterprise Features

**Philosophy:** Production-ready enhancements for scale

**What's Inside:**

#### **Hybrid Tiering** (`hybrid.rs` - 2032 lines!)
```rust
pub struct HotGraphIndex {
    // RAM: Recent 10K nodes
}

pub struct WarmGraphCache {
    // LRU cache: Recent 50K nodes
}

// Cold tier: Full MDBX database
```

**Coordinates:** Hot → Warm → Cold fallback

#### **Payload Filtering** (`payload_index.rs` - 759 lines)
```rust
// Search with metadata filters
let results = index.search_filtered(
    &query_vec,
    10,
    PayloadFilter::must("category", "tech")
        .and(PayloadFilter::range("timestamp", start, end))
)?;
```

**Combines:** Vector similarity + metadata filtering (like Qdrant)

#### **Segmentation** (`segment.rs`)
```rust
// Partition large indexes
let idx = SegmentBasedVectorIndex::new("segments/")?;
idx.add_vector("vec_1", vector, Some("2024-Q4"))?;
```

**Use Case:** Horizontal scaling for millions of vectors

**See:** `src/advanced/README.md` for enterprise features

---

## How Modules Work Together

### Example: Semantic Search Query

```rust
// USER QUERY: "Find Rust database discussions"

// ================================================
// STEP 1: Query Engine (decides strategy)
// ================================================
let query_plan = query_engine.plan(Query {
    semantic: "Rust database discussions",
    time_scope: TimeScope::LastWeek,
    temperature: Temperature::Warm,  // Hot + Warm tiers
});

// ================================================
// STEP 2: Structural Filter (algorithms/)
// ================================================
// No MDBX access! Just pure algorithm
let time_filtered = filter_by_timestamp(
    messages,
    last_week,
    now
);

// ================================================
// STEP 3: Semantic Search (core/ + lock_free/)
// ================================================
// Try hot tier first
if let Some(results) = hot_vector_index.search(&query_vec, 100)? {
    return results;  // <1ms
}

// Fallback to warm cache
if let Some(results) = warm_cache.search(&query_vec, 100)? {
    return results;  // <10ms
}

// Fallback to cold MDBX
let candidates = core::VectorIndex::search(
    storage.env("embeddings.mdbx"),
    storage.dbi("hnsw_0.6b"),
    &query_vec_0_6b,
    1000
)?;  // <100ms, but gets ALL data

// ================================================
// STEP 4: Reranking (advanced/)
// ================================================
let reranked = advanced::hybrid::rerank_with_high_res(
    candidates,
    storage.env("embeddings.mdbx"),
    storage.dbi("8b_vectors"),
    &query_vec_8b,
    100
)?;

// ================================================
// STEP 5: Graph Expansion (algorithms/ + core/)
// ================================================
if query_plan.use_knowledge_graph {
    // Use pure algorithms on graph data
    let expanded = algorithms::dijkstra(
        &knowledge_graph,
        start_nodes: reranked,
        depth: 2
    )?;
    
    results.extend(expanded);
}

// ================================================
// STEP 6: Final Ranking (algorithms/)
// ================================================
let ranked = algorithms::page_rank(&result_graph, 0.85, 100)?;
```

**Notice:**
- `algorithms/` = Pure computation (no DB)
- `core/` = Index building (uses storage's env)
- `lock_free/` = Hot RAM tier (no persistence)
- `advanced/` = Orchestration (coordinates everything)

---

## Testing Strategy

### Current Test Pattern (Being Fixed):

```rust
// CURRENT (creates temp MDBX per test)
#[test]
fn test_something() {
    let (manager, _temp) = setup_real_db();  // Creates temp DB
    // Test uses manager...
}
```

**Problem:**
- 233 tests × temp MDBX environment = slow, resource-heavy
- Tests accidentally validated the WRONG pattern (indexing owning DB)

### Target Test Pattern:

```rust
// CORRECT (uses storage's env)
#[test]
fn test_something() {
    let storage = TestStorage::new()?;  // Storage creates DB
    let env = storage.env("test.mdbx");
    let dbi = storage.create_table("test_index")?;
    
    let index = StructuralIndex::new(env, dbi);  // Takes pointer!
    // Test the index...
}
```

**Benefits:**
- Tests validate CORRECT pattern (indexing takes env from storage)
- Can share storage environment across tests
- Clearer separation of concerns

---

## The Complete Data Flow

### Message Insert → Full Enrichment:

```
1. User sends: "How to optimize Rust code?"
   │
   ├─→ [STORAGE] Saves to conversations.mdbx
   │   └─ conversations/active/messages/{msg_1}
   │
   ├─→ [IMMEDIATE - 0.6B Model]
   │   1. task-scheduler: Queue("Embed 0.6B", URGENT)
   │   2. 0.6B model generates: [0.23, -0.15, 0.88, ...] (384D)
   │   3. STORAGE saves to: embeddings.mdbx/0.6b_vectors/{msg_1}
   │   4. INDEXING updates: embeddings.mdbx/hnsw_0.6b
   │   5. Response ready in <100ms ✅
   │
   ├─→ [BACKGROUND - 8B Model]
   │   1. task-scheduler: Queue("Embed 8B", LOW priority)
   │   2. 8B model generates: [0.2301, -0.1502, ...] (1536D)
   │   3. STORAGE saves to: embeddings.mdbx/8b_vectors/{msg_1}
   │   4. INDEXING updates: embeddings.mdbx/hnsw_8b
   │   5. High-quality search ready (minutes later)
   │
   └─→ [BACKGROUND - Entity Extraction]
       1. task-scheduler: Queue("Extract entities", NORMAL)
       2. weaver extracts: ["Rust", "optimization", "code"]
       3. STORAGE saves to: knowledge.mdbx/entities/{...}
       4. INDEXING updates: knowledge.mdbx/graph_outgoing
```

### Query Execution (Multi-Database):

```
User queries: "Find Rust database info"
   │
   ├─→ [1] EXPERIENCE DB (what worked before?)
   │   - Search: experience.mdbx/hnsw_actions
   │   - Finds: "Similar question 3 days ago, user approved 'embedded DB' refinement"
   │   - Agent learns: Refine query to "Rust embedded databases"
   │
   ├─→ [2] TOOL-RESULTS DB (check cache!)
   │   - Search: tool-results.mdbx/hnsw_searches
   │   - Finds: "We searched 'Rust embedded DB' yesterday"
   │   - Cache HIT! Return cached results (no API call)
   │
   ├─→ [3] EMBEDDINGS DB (semantic search)
   │   - Stage 1: embeddings.mdbx/hnsw_0.6b → 1000 candidates
   │   - Stage 2: embeddings.mdbx/hnsw_8b → Rerank to top 100
   │
   ├─→ [4] KNOWLEDGE DB (graph expansion)
   │   - Traverse: knowledge.mdbx/graph_outgoing
   │   - Find: "Rust" → "sled", "redb", "RocksDB"
   │
   └─→ [5] MERGE & RANK
       - Combine results from all databases
       - Apply algorithms/page_rank for final ranking
       - Return top 10 with confidence scores
```

**INDEXING serves ALL databases in this flow!**

---

## Key Architectural Insights

### 1. Indexing = Service, Not Owner

**WRONG:** Indexing creates and owns a database  
**RIGHT:** Indexing builds indexes IN storage's databases

Like a contractor:
- Storage owns the building (database)
- Indexing builds the elevator (indexes)
- Indexing doesn't own the building!

### 2. Universal Pattern

**WRONG:** Different index logic for each database  
**RIGHT:** Same HNSW builder works for embeddings, experience, tool-results

```rust
// Same code, different databases!
build_hnsw(env, vectors_dbi, index_dbi, dim);
```

### 3. Separation of Concerns

**algorithms/:** Pure computation (NO storage)  
**core/:** Index persistence (USES storage's env)  
**lock_free/:** Hot RAM tier (NO persistence)  
**advanced/:** Orchestration (COORDINATES tiers)

### 4. Zero-Copy Everywhere

All read operations use transaction-backed guards:
```rust
pub struct GraphIndexGuard {
    txn: *mut MDBX_txn,  // Keeps transaction alive
    archived: &'static rkyv::Archived<Vec<(EdgeId, NodeId)>>,
}

// Iterate without allocations!
for (edge_id, target) in guard.iter_edges() {
    // Both &str are borrowed from mmap - ZERO allocations!
}
```

### 5. Thread-Local Transaction Pool

**Problem:** MDBX allows only ONE transaction per thread  
**Solution:** Thread-local pool reuses single transaction

```rust
// First read
let txn = txn_pool::get_or_create_read_txn(env)?;  // Creates new
let guard1 = StructuralIndexGuard::new(txn, ...);

// Second read (SAME thread)
let txn = txn_pool::get_or_create_read_txn(env)?;  // REUSES!
let guard2 = GraphIndexGuard::new(txn, ...);  // No -30783 error!
```

**Validated in:** `mdbx-base/src/lib.rs::test_thread_local_transaction_pool_pattern`

---

## What Needs to Happen (Refactoring Plan)

### Phase 1: Fix Core API (CRITICAL)

**Remove:**
```rust
❌ IndexManager::new(path)  // Creates own DB
```

**Add:**
```rust
✅ IndexManager::new(
    structural_env: *mut MDBX_env,
    structural_dbi: MDBX_dbi,
    graph_env: *mut MDBX_env,
    outgoing_dbi: MDBX_dbi,
    incoming_dbi: MDBX_dbi,
    vector_path: PathBuf,  // For mmap files
)
```

### Phase 2: Update Tests

**Remove:**
```rust
❌ setup_real_db() -> (IndexManager, TempDir)
```

**Add:**
```rust
✅ setup_with_storage() -> TestStorageWithIndexing
```

### Phase 3: Clean Dependencies

**Remove from** `indexing/Cargo.toml`:
```toml
❌ mdbx-sys = "13.8.0"
❌ libmdbx = "0.6.3"
```

**Keep only:**
```toml
✅ mdbx-base = { path = "../mdbx-base" }
```

### Phase 4: Update Storage Integration

Storage creates databases and gives indexing the pointers:
```rust
impl KnowledgeDatabase {
    pub fn with_indexing(path: &Path) -> DbResult<Self> {
        let env = MdbxEnvBuilder::new(path).open()?;
        let data_dbis = ...;
        let index_dbis = ...;
        
        let graph_index = GraphIndex::new(env, index_dbis[1], index_dbis[2]);
        let structural_index = StructuralIndex::new(env, index_dbis[0]);
        
        Ok(Self { env, graph_index, structural_index })
    }
}
```

---

## Summary: What Indexing Actually Is

### In One Sentence:
**Indexing is a universal service that builds search structures (HNSW, B-trees, adjacency lists) in storage's databases to enable fast semantic, structural, and graph queries across all 9+ MIA databases.**

### What It Provides:

1. **Algorithms** (`algorithms/`) - Pure graph algorithms (like petgraph)
2. **Index Builders** (`core/`) - HNSW, graph, structural indexes
3. **Hot Tier** (`lock_free/`) - Fast in-memory concurrent indexes
4. **Orchestration** (`advanced/`) - Hot/warm/cold coordination

### What It Does NOT Do:

❌ Own databases  
❌ Create MDBX environments  
❌ Manage persistence (storage's job!)  
❌ Handle raw data (only indexes!)

### What It DOES Do:

✅ Build indexes IN storage's databases  
✅ Provide fast search algorithms  
✅ Coordinate hot/warm/cold tiers  
✅ Enable multi-resolution retrieval (0.6B → 8B)  
✅ Support ALL MIA databases with same service

---

## Quick Reference

### For Storage Developers:
- Give indexing your `(env, dbi)` pointers
- Call `index.add()` when data changes
- Indexing writes to YOUR database tables

### For Query Developers:
- Use `index.search()` for semantic queries
- Use `graph.get_outgoing()` for relationships
- Use `structural.get()` for property lookups
- Combine with `algorithms/` for graph reasoning

### For ML Pipeline Developers:
- Generate embeddings → Give to storage
- Storage saves embeddings → Calls indexing
- Indexing builds/updates HNSW → Ready for search

---

**Read Next:**
- `README.md` - Quick start guide
- `src/core/README.md` - Core index implementations
- `src/algorithms/README.md` - Graph algorithms
- `src/advanced/README.md` - Enterprise features
- `src/lock_free/README.md` - Concurrent structures

**For full MIA system context:**
- `../../docs/mia_memory.md` - Complete cognitive architecture
- `../../MIA_VISION.md` - System overview

---

**This is the single source of truth for what indexing is and how it fits into MIA.**  
**No more re-explaining - everything is here.**

