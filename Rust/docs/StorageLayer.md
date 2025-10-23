```markdown
# Specification: Core Storage Layer (`storage` crate)

## 1. Objective & Core Principles

The `storage` crate is the foundational layer of the database. Its sole responsibility is to provide a safe, transactional, and performant interface for **Create, Read, Update, and Delete (CRUD)** operations on the core data entities. It abstracts away the direct interaction with the `sled` key-value store and ensures all data is correctly serialized to and deserialized from disk.

This component is governed by the following core principles:

*   **Storage Engine:** The crate will be built exclusively on **`sled`**. This provides the necessary thread-safe, transactional, and high-performance primitives required for a concurrent server backend.
*   **Schema Model:** A **Hybrid Schema Model** will be enforced. Core entities are defined as strongly-typed Rust structs with first-class fields for critical, queryable data. This enables high-performance secondary indexing. A flexible `metadata: serde_json::Value` field is included in each struct to retain extensibility for application-specific data.
*   **Serialization:** All on-disk data will be serialized using **`bincode`**. This provides a compact and extremely fast binary format, a significant improvement over the JSON stringification used in the current system.
*   **Atomicity:** All individual write operations (`insert`, `delete`) exposed by this crate must be atomic. Complex, multi-entity transactions will be orchestrated by higher-level crates but will leverage the transactional primitives exposed here.

## 2. Architectural Analysis & Rationale (The "Why")

The design of this layer is the result of a comparative analysis of multiple database architectures, synthesized to meet the specific needs of our AI-first "humane brain" system.

### 2.1. Findings from Current System (IndexedDB)

Our current TypeScript implementation (`src/DB/`) provides the foundational data model that this crate must evolve.

*   **Strengths to Preserve:**
    *   The "everything is a node" concept, implemented via the `KnowledgeGraphNode` base class, is a powerful abstraction for a unified graph.
    *   The separation of concerns into different object stores (`DB_CHATS`, `DB_MESSAGES`, `DB_KNOWLEDGE_GRAPH_EDGES`) is a sound design that we will mirror with `sled` Trees.
    *   The use of `ArrayBuffer` for storing embedding vectors (`idbEmbedding.ts`) is efficient for the browser environment and validates our choice of a binary format.

*   **Critical Limitations to Solve:**
    *   **The Indexing Bottleneck:** The single greatest flaw is the heavy reliance on a `properties_json` string blob. Critical query fields like `chat_id`, `sender`, `timestamp`, and `status` are trapped within this string. IndexedDB cannot create indexes on data inside a JSON string. Consequently, any query filtering on these fields requires a **full table scan**, where every node is fetched from disk, its JSON is parsed, and the value is checked. This is an O(n) operation and is unacceptable for an enterprise-grade system.
    *   **Serialization Overhead:** The constant `JSON.stringify` and `JSON.parse` operations for every read and write introduce significant performance overhead.

### 2.2. Findings from Reference Systems

*   **ArangoDB & Neo4j (Multi-Model & Graph):** These systems demonstrate the power of treating relationships (edges) as first-class citizens stored and indexed separately from nodes. This is far more scalable than embedding edge lists within nodes and is a core pattern we will adopt. While Neo4j's Index-Free Adjacency is the gold standard for deep traversals, our system's reliance on converged queries makes a secondary indexing approach (similar to ArangoDB's) a better architectural fit.
*   **Qdrant (Vector Specialist):** Qdrant's model of a Vector paired with an indexable, schemaless JSON `Payload` is highly effective for hybrid search. Our Hybrid Schema Model is a direct, graph-aware evolution of this concept, where our strongly-typed core fields serve the same purpose as Qdrant's indexed payload fields.

### 2.3. Synthesis & Final Architectural Decision

The **Hybrid Schema Model** is the definitive architecture for this crate.

**This decision is a direct solution to the critical flaw in the current system.** By promoting essential fields (`chat_id`, `timestamp`, etc.) out of a JSON blob and into first-class, strongly-typed fields in our Rust structs, we enable the `indexing` crate to build high-performance secondary indexes on them. This transforms filtering operations from slow O(n) table scans into fast O(log n) index lookups.

This approach synthesizes the best patterns from our research:
*   It retains the **flexibility** of ArangoDB and our current system via the `metadata` field.
*   It enables the **high-performance filtering** required for a Qdrant-style hybrid search.
*   It provides the **structural foundation** for a rich, Neo4j-style graph model.

This is the only architecture that provides the necessary performance foundation for the Converged Query Pipeline.

## 3. Detailed Rust Implementation Blueprint (The "What")

### 3.1. Data Structures

File: `storage/src/models.rs`
```rust
use serde::{Deserialize, Serialize};

// --- Core Type Aliases ---
pub type NodeId = String;
pub type EdgeId = String;
pub type EmbeddingId = String;

/// The unifying enum for all types of nodes that can exist in the graph.
/// This allows us to store different node types in the same `sled` Tree
/// while retaining type safety.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Node {
    Chat(Chat),
    Message(Message),
    Summary(Summary),
    Attachment(Attachment),
    Entity(Entity), 
}

// --- Concrete Node Implementations ---

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Chat {
    // --- Core, Indexed Fields ---
    pub id: NodeId,
    pub title: String,
    pub topic: String, // Populated by the Knowledge Weaver
    pub created_at: i64, // Unix timestamp (milliseconds)
    pub updated_at: i64,

    // --- Core, Unindexed Fields ---
    pub message_ids: Vec<NodeId>,
    pub summary_ids: Vec<NodeId>,
    pub embedding_id: Option<EmbeddingId>,

    // --- Flexible, Unindexed "Sidecar" Data ---
    pub metadata: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    // --- Core, Indexed Fields ---
    pub id: NodeId,
    pub chat_id: NodeId,
    pub sender: String, // e.g., "user", "assistant", or a specific user ID
    pub timestamp: i64,

    // --- Core, Unindexed Fields ---
    pub text_content: String,
    pub attachment_ids: Vec<NodeId>,
    pub embedding_id: Option<EmbeddingId>,

    // --- Flexible, Unindexed "Sidecar" Data ---
    pub metadata: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Entity {
    // --- Core, Indexed Fields ---
    pub id: NodeId,
    pub label: String, // The canonical name of the entity, e.g., "Project Phoenix"
    pub entity_type: String, // e.g., "PERSON", "PROJECT", "CONCEPT"
    
    // --- Core, Unindexed Fields ---
    pub embedding_id: Option<EmbeddingId>,

    // --- Flexible, Unindexed "Sidecar" Data ---
    pub metadata: serde_json::Value,
}

// ... other structs like Summary and Attachment will be defined here ...

/// Represents a directed, typed relationship between two nodes.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Edge {
    pub id: EdgeId,
    pub from_node: NodeId,
    pub to_node: NodeId,
    pub edge_type: String, // e.g., "CONTAINS_MESSAGE", "MENTIONS_ENTITY"
    pub created_at: i64,
    pub metadata: serde_json::Value,
}

/// Represents a high-dimensional vector embedding.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Embedding {
    pub id: EmbeddingId,
    pub vector: Vec<f32>,
    pub model: String, // The model used to generate the embedding
}
```

### 3.2. On-Disk Layout (`sled` Implementation)

The `sled::Db` instance will contain multiple `sled::Tree`s, which function as tables.

*   **`nodes` Tree:**
    *   **Purpose:** The primary, canonical store for all `Node` objects.
    *   **Key Format:** `NodeId` (e.g., `"chat_a1b2c3d4"`) as `&[u8]`.
    *   **Value Format:** `bincode`-serialized `Node` enum.

*   **`edges` Tree:**
    *   **Purpose:** The primary, canonical store for all `Edge` objects.
    *   **Key Format:** `EdgeId` (e.g., `"edge_e5f6g7h8"`) as `&[u8]`.
    *   **Value Format:** `bincode`-serialized `Edge` struct.

*   **`embeddings` Tree:**
    *   **Purpose:** The primary, canonical store for all `Embedding` objects.
    *   **Key Format:** `EmbeddingId` (e.g., `"embed_i9j0k1l2"`) as `&[u8]`.
    *   **Value Format:** `bincode`-serialized `Embedding` struct.

*All secondary indexes will be defined and managed by the `indexing` crate, but they will also be stored in their own dedicated `sled::Tree`s with a `idx::` prefix.*

### 3.3. Public API

This defines the public interface of the `storage` crate. The `StorageManager` will be the main struct providing access to the database.

File: `storage/src/lib.rs`
```rust
use sled::Db;
use crate::models::{Node, NodeId, Edge, EdgeId, Embedding, EmbeddingId};

/// Manages all direct interactions with the `sled` database for CRUD operations.
pub struct StorageManager {
    db: Db,
    nodes: sled::Tree,
    edges: sled::Tree,
    embeddings: sled::Tree,
}

impl StorageManager {
    /// Opens or creates a database at the specified path and opens the required trees.
    pub fn new(path: &str) -> Result<Self, DbError> { ... }

    // --- Node Operations ---
    pub fn get_node(&self, id: &NodeId) -> Result<Option<Node>, DbError> { ... }
    pub fn insert_node(&self, node: &Node) -> Result<(), DbError> { ... }
    pub fn delete_node(&self, id: &NodeId) -> Result<Option<Node>, DbError> { ... }

    // --- Edge Operations ---
    pub fn get_edge(&self, id: &EdgeId) -> Result<Option<Edge>, DbError> { ... }
    pub fn insert_edge(&self, edge: &Edge) -> Result<(), DbError> { ... }
    pub fn delete_edge(&self, id: &EdgeId) -> Result<Option<Edge>, DbError> { ... }

    // --- Embedding Operations ---
    pub fn get_embedding(&self, id: &EmbeddingId) -> Result<Option<Embedding>, DbError> { ... }
    pub fn insert_embedding(&self, embedding: &Embedding) -> Result<(), DbError> { ... }
    pub fn delete_embedding(&self, id: &EmbeddingId) -> Result<Option<Embedding>, DbError> { ... }
    
    /// Provides access to the raw `sled::Db` instance for higher-level crates
    /// to orchestrate multi-tree transactions.
    pub fn db(&self) -> &Db {
        &self.db
    }
}

// Custom error type for the storage layer
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("Sled database error: {0}")]
    Sled(#[from] sled::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),
    #[error("Entity not found: {0}")]
    NotFound(String),
}
```

## 4. Implementation Plan & Checklist

*   [ ] **Project Setup:**
    *   [ ] Create the `storage` crate within the Rust workspace.
    *   [ ] Add `sled`, `serde`, `serde_json`, `bincode`, and `thiserror` to `Cargo.toml`.
*   [ ] **Models:**
    *   [ ] Implement all structs and enums in `storage/src/models.rs`.
    *   [ ] Ensure all models derive `Serialize`, `Deserialize`, `Debug`, and `Clone`.
*   [ ] **StorageManager:**
    *   [ ] Implement `StorageManager::new()` to open the `sled::Db` and the `nodes`, `edges`, and `embeddings` trees.
    *   [ ] Implement `get_node`, `insert_node`, and `delete_node`. The `insert_node` function should handle both creation and updates (upsert logic).
    *   [ ] Implement `get_edge`, `insert_edge`, and `delete_edge`.
    *   [ ] Implement `get_embedding`, `insert_embedding`, and `delete_embedding`.
*   [ ] **Error Handling:**
    *   [ ] Implement the `DbError` enum using `thiserror` for ergonomic error handling and propagation.
*   [ ] **Unit Tests:**
    *   [ ] Create a test module in each file.
    *   [ ] Write a test to create a `Node`, save it, close the DB, reopen it, and verify the node can be read back correctly.
    *   [ ] Write tests for updating and deleting each entity type.
    *   [ ] Write tests to ensure errors (e.g., serialization failure, sled I/O errors) are handled gracefully.

## 5. Open Questions & Decisions Needed

*   **Transactional Guarantees:** This crate's API provides single-operation atomicity. The design of multi-operation, cross-tree transactions (e.g., inserting a node and updating an index simultaneously) is the responsibility of the higher-level crates (`indexing`, `weaver`) and will be specified in their respective documents. The `db()` method is provided to facilitate this.
*   **Data Migration:** The strategy for migrating data from the existing IndexedDB store is a higher-level concern and will be addressed in a separate `MigrationPlan.md` document. This crate is only concerned with the new, canonical data format.
```