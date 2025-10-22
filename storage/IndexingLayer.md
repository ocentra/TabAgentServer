```markdown
# Specification: Indexing Layer (`indexing` crate)

## 1. Objective & Core Principles

The `indexing` crate is the performance engine of the database. Its purpose is to create and maintain a set of specialized data structures (indexes) that enable the rapid retrieval of data based on properties, relationships, and semantic similarity. This crate is the primary enabler of **Stage 1 (Candidate Set Generation)** and **Stage 2 (Semantic Re-ranking)** of the Converged Query Pipeline.

This component is governed by the following core principles:

*   **Converged Indexing:** The crate will provide three distinct types of indexes—Structural, Graph, and Vector—to power the three facets of the Converged Query Model.
*   **Performance:** All indexing strategies are chosen to transform slow O(n) full-table scans into highly efficient O(log n) or O(1) lookups.
*   **Transactional Integrity:** The indexes must always remain in perfect sync with the primary data stored by the `storage` crate. All index modifications must occur within the same transaction as the data modification.
*   **Separation of Concerns:** This crate is responsible for the *structure* of the indexes and the logic for querying them. The underlying storage of the indexes will be handled by `sled`, and the core data remains the responsibility of the `storage` crate.

## 2. Architectural Analysis & Rationale (The "Why")

The architecture of the indexing layer is a direct response to the limitations of the current system and a synthesis of best-in-class patterns from specialized databases.

### 2.1. Findings from Current System (IndexedDB)

*   **Strengths to Preserve:** The existing schema (`idbSchema.ts`) correctly identifies the need for graph indexes by creating indexes on `from_node_id` and `to_node_id` in the edges store. This is a proven pattern that enables efficient 1-hop traversals, and we will build upon it.
*   **Critical Limitations to Solve:**
    *   **No Property Indexing:** As established in `StorageLayer.md`, the inability to index fields within the `properties_json` blob is the system's primary bottleneck. This crate's structural indexes are the direct solution to this problem.
    *   **No Vector Indexing:** The current system performs a brute-force O(n) scan for every vector search (`vectorUtils.ts`). For an AI-first agentic system where context retrieval must be instantaneous, this is unacceptable. An O(log n) vector index is not a "nice-to-have"; it is a mandatory, enterprise-grade requirement from day one.

### 2.2. Findings from Reference Systems

*   **ArangoDB/Qdrant (Structural Indexing):** These systems demonstrate the power of creating secondary indexes on arbitrary document properties (or "payloads"). This allows for rapid, SQL-like `WHERE` clause filtering. We will adopt this pattern for all core fields defined in our Hybrid Schema Model.
*   **Neo4j/ArangoDB (Graph Indexing):** These native graph databases prove the efficiency of dedicated edge indexes for relationship traversal. Our graph index design, using `sled` Trees to represent adjacency lists, is a direct implementation of this industry-standard pattern.
*   **Qdrant/FAISS (Vector Indexing):** The use of **HNSW (Hierarchical Navigable Small World)** is the undisputed industry standard for high-performance, high-accuracy Approximate Nearest Neighbor (ANN) search. Its O(log n) search complexity is a mandatory requirement. The design of our vector index will be heavily influenced by the successful, production-grade implementation patterns found in Qdrant's Rust codebase, particularly the separation of the HNSW graph structure from the raw vector storage.

### 2.3. Synthesis & Final Architectural Decision

The `indexing` crate will be a **composite indexing engine**. It will not use a single, one-size-fits-all indexing strategy. Instead, it will manage three specialized index types, each optimized for a different facet of the Converged Query Model, all stored within `sled` Trees.

1.  **Structural Indexes:** B-Tree-like indexes (provided by `sled`) on the strongly-typed fields of our Rust structs (e.g., `Message.chat_id`). This powers the "SQL `WHERE` clause" facet of our queries.
2.  **Graph Indexes:** A pair of `sled` Trees that act as adjacency lists, mapping a node ID to the set of its incoming and outgoing edges. This powers the "Neo4j `MATCH` clause" facet.
3.  **Vector (HNSW) Index:** A persisted HNSW graph structure that enables O(log n) semantic search. This powers the "RAG `similarity_search`" facet.

This multi-index architecture is the only way to achieve the performance and accuracy required by the Converged Query Pipeline and the "humane brain" vision.

## 3. Detailed Rust Implementation Blueprint (The "What")

### 3.1. Data Structures & API

The primary interface will be the `IndexManager`, which orchestrates updates and queries across all index types.

File: `indexing/src/lib.rs`
```rust
use storage::{DbError, StorageManager};
use storage::models::{Node, NodeId, Edge, EdgeId, EmbeddingId};
use sled::transaction::TransactionalTree;

/// Manages all secondary indexes for fast data retrieval.
pub struct IndexManager {
    // HNSW index instance
    // hnsw: Hnsw<f32, Cosine>, 
}

impl IndexManager {
    /// Loads existing indexes from the DB or initializes them if they don't exist.
    pub fn new(db: &sled::Db) -> Result<Self, DbError> { ... }

    // --- Index Update API (to be called within a transaction) ---

    /// Transactionally updates all relevant indexes when a node is inserted or updated.
    pub fn handle_node_update_tx(
        &self,
        tx: &TransactionalTree, // The 'nodes' tree transaction
        node: &Node
    ) -> Result<(), DbError> { ... }

    /// Transactionally updates all relevant indexes when a node is deleted.
    pub fn handle_node_delete_tx(
        &self,
        tx: &TransactionalTree,
        node: &Node
    ) -> Result<(), DbError> { ... }

    // ... similar handlers for edges and embeddings ...

    // --- Index Query API ---

    /// Finds all node IDs matching a specific property value.
    pub fn get_nodes_by_property(
        &self,
        property_name: &str,
        property_value: &str
    ) -> Result<Vec<NodeId>, DbError> { ... }

    /// Finds all outgoing edge IDs for a given node.
    pub fn get_outgoing_edge_ids(&self, from_node_id: &NodeId) -> Result<Vec<EdgeId>, DbError> { ... }
    
    /// Finds all incoming edge IDs for a given node.
    pub fn get_incoming_edge_ids(&self, to_node_id: &NodeId) -> Result<Vec<EdgeId>, DbError> { ... }

    /// Performs a k-Nearest Neighbor search using the HNSW index.
    pub fn vector_search(
        &self,
        query_vector: &[f32],
        k: usize
    ) -> Result<Vec<(EmbeddingId, f32)>, DbError> { ... }
}
```

### 3.2. On-Disk Layout (`sled` Implementation)

The `indexing` crate will create and manage its own `sled::Tree`s, separate from the primary data trees.

#### 3.2.1. Structural Indexes

*   **Purpose:** To allow fast lookups of nodes by their core, strongly-typed properties.
*   **`sled` Tree Name Pattern:** `idx::struct::{struct_name}::{property_name}` (e.g., `idx::struct::message::chat_id`)
*   **Key Format:** `{property_value}::{primary_key}` (e.g., `"chat_abc123::message_xyz789"`). This format is crucial. It allows us to use `sled`'s `scan_prefix` method on `{property_value}` to instantly find all matching primary keys.
*   **Value Format:** Empty (`&[]`). The key contains all necessary information.

#### 3.2.2. Graph Indexes

*   **Purpose:** To represent the graph's adjacency lists for fast traversal.
*   **`sled` Tree Names:**
    *   `idx::graph::from`: For outgoing edges.
    *   `idx::graph::to`: For incoming edges.
*   **Key Format (`idx::graph::from`):** `{from_node_id}::{edge_id}`
*   **Key Format (`idx::graph::to`):** `{to_node_id}::{edge_id}`
*   **Value Format:** Empty (`&[]`).

#### 3.2.3. Vector (HNSW) Index

*   **Purpose:** To persist the HNSW graph structure for fast ANN search.
*   **`sled` Tree Name:** `idx::vector::hnsw_graph`
*   **Key Format:** A single, constant key, e.g., `"singleton"`.
*   **Value Format:** A `bincode`-serialized representation of the entire HNSW graph structure (layers, entry points, node connections).
*   **`sled` Tree Name:** `idx::vector::point_map`
*   **Key Format:** Internal HNSW point ID (e.g., a `u64`).
*   **Value Format:** The corresponding `EmbeddingId` (String). This maps the index's internal ID back to our database's public ID.
*   **Note:** The raw vectors themselves are **not** duplicated. They are stored only once in the `embeddings` tree managed by the `storage` crate. The HNSW index operates by loading vectors on demand during search and construction.

## 4. Implementation Plan & Checklist

*   [ ] **Project Setup:**
    *   [ ] Create the `indexing` crate.
    *   [ ] Add a dependency on the `storage` crate.
    *   [ ] Add a dependency on a suitable HNSW crate (e.g., `hnsw`).
*   [ ] **Structural Indexes:**
    *   [ ] Implement the logic to open/create the `idx::struct::*` trees.
    *   [ ] Implement the `get_nodes_by_property` function using `scan_prefix`.
    *   [ ] Implement the update/delete logic within `handle_node_update_tx` and `handle_node_delete_tx`.
*   [ ] **Graph Indexes:**
    *   [ ] Implement the logic to open/create the `idx::graph::*` trees.
    *   [ ] Implement `get_outgoing_edge_ids` and `get_incoming_edge_ids`.
    *   [ ] Implement the update/delete logic for edges within transactional handlers.
*   [ ] **Vector Index:**
    *   [ ] Integrate the chosen HNSW crate.
    *   [ ] Implement the persistence logic: saving the HNSW graph to the `idx::vector::hnsw_graph` tree on shutdown or periodically, and loading it on startup.
    *   [ ] Implement the `point_map` to link HNSW internal IDs to our `EmbeddingId`s.
    *   [ ] Implement the `vector_search` function.
*   [ ] **Transactional Integrity:**
    *   [ ] This is the most critical step. Refactor the `StorageManager`'s `insert/delete` methods to accept an optional `sled::transaction::Transaction` context.
    *   [ ] Create higher-level functions (e.g., in a new `TransactionManager` struct) that orchestrate a full operation:
        1.  Start a `sled` transaction.
        2.  Call the `storage` crate's method within the transaction.
        3.  Call the `indexing` crate's `handle_*_tx` methods within the same transaction.
        4.  Commit the transaction.
        *This ensures that data and all its corresponding indexes are updated atomically.*
*   [ ] **Unit Tests:**
    *   [ ] Test that creating a `Message` with `chat_id="abc"` correctly creates an entry in the `idx::struct::message::chat_id` tree.
    *   [ ] Test that deleting the `Message` correctly removes the index entry.
    *   [ ] Test that `get_nodes_by_property("chat_id", "abc")` returns the correct `NodeId`.
    *   [ ] Write similar tests for graph and vector indexes.

## 5. Open Questions & Decisions Needed

*   **HNSW Crate Selection:** A final decision must be made on which HNSW crate to use. The `hnsw` crate is a strong contender, but a brief evaluation of alternatives should be performed before implementation. The key criteria are performance, maturity, and ease of integration with a custom persistence layer.
*   **Index Backfilling:** The initial implementation will focus on keeping indexes in sync with new data. A strategy for building indexes for existing, un-indexed data (a "backfill" or "re-index" operation) will need to be designed. This can be a lower-priority, offline utility.
```