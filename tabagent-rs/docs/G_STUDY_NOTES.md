# TabAgent Embedded Database - Architecture Study

## Project Context

### Our Current System (IndexedDB - Extension/Client)

**Location:** `src/DB/`

**Core Files:**
- `idbKnowledgeGraph.ts`: The cornerstone of the data model. It defines the `KnowledgeGraphNode` and `KnowledgeGraphEdge` classes, establishing the "everything is a node" architecture.
- `idbEmbedding.ts`: Manages vector embeddings, including their storage as `ArrayBuffer` and linking to graph nodes.
- `idbChat.ts` & `idbMessage.ts`: These are concrete implementations of `KnowledgeGraphNode`, representing the main entities in the application.
- `idbBase.ts`: Defines the abstract `BaseCRUD` class, which is not heavily used but establishes a basic pattern.
- `vectorUtils.ts`: Contains pure functions for `cosineSimilarity` and a brute-force nearest neighbor search.
- `idbSchema.ts`: Crucially defines the object stores and indexes for the entire IndexedDB database.

**Data Model:**
```typescript
KnowledgeGraphNode {
  id: string;
  type: string;  // e.g., "chat", "message"
  label: string;
  properties_json?: string; // Flexible JSON properties
  embedding_id?: string;
  edgesOut: KnowledgeGraphEdge[]; // Populated at runtime
  edgesIn: KnowledgeGraphEdge[];  // Populated at runtime
  created_at: number;
  updated_at: number;
}

KnowledgeGraphEdge {
  id: string;
  from_node_id: string;
  to_node_id: string;
  edge_type: string;
  metadata_json?: string;
  created_at: number;
}

Embedding {
  id: string;
  input: string; // The text that was embedded
  vector: ArrayBuffer; // Stored as a Float32Array buffer
  model: string;
}
```

**Key Architectural Patterns:**
- **"Everything is a Node":** The system uses inheritance (`Chat extends KnowledgeGraphNode`) to create a unified graph structure. This is a powerful concept that the Rust implementation should preserve, likely using traits.
- **Adjacency List via Indexed Queries:** Edges are not embedded in node documents. Instead, they are stored in a separate `DB_KNOWLEDGE_GRAPH_EDGES` object store. The `fetchEdges` method in `idbKnowledgeGraph.ts` queries this store using IndexedDB's built-in indexes on `from_node_id` and `to_node_id`. This is a performant and scalable pattern for graph representation.
- **Brute-Force Vector Search:** `vectorUtils.ts` provides a `cosineSimilarity` function. Any vector search operation would require fetching all vectors and comparing them one by one, which is an O(n) operation. This is a major performance bottleneck that the new Rust database should address, ideally with an HNSW index.
- **Asynchronous Operations via Web Workers:** All database interactions are asynchronous and funneled through a web worker. This is a standard browser pattern to prevent blocking the UI thread.
- **Normalized Embeddings:** Embeddings are stored separately and linked via an `embedding_id`. This is a good design that allows for embeddings to be reused and managed independently of the nodes they are associated with.

### What We're Building (Rust - Server)

**Goal:** To create an embedded, multi-model database in Rust that mirrors the structure and functionality of the existing IndexedDB implementation but with native performance and scalability.

**Key Requirements:**
1.  **Data Model Parity:** The Rust database must support the same node, edge, and embedding structures.
2.  **Performance:** It must be significantly faster than the IndexedDB implementation, especially for graph traversals and vector searches.
3.  **Zero-Configuration:** It should be a library that can be directly embedded into the Python server with no external dependencies.
4.  **Python API:** A clean Python API must be provided using PyO3.
5.  **Cross-Platform:** It must compile and run on Windows, macOS, and Linux.

**Key Improvements over IndexedDB:**
-   **Efficient Serialization:** Replace `JSON.stringify`/`JSON.parse` with a binary format like `bincode`.
-   **Performant Indexing:** Replace brute-force vector search with an HNSW index for approximate nearest neighbor search.
-   **Advanced Graph Queries:** Provide built-in support for multi-hop traversals, shortest path finding, and other graph algorithms.

---

## 1. Storage Layer Architecture

### 1.1 ArangoDB Approach

**Files Studied:**
- `arangod/StorageEngine/StorageEngine.h`
- `arangod/StorageEngine/PhysicalCollection.h`
- `arangod/RocksDBEngine/RocksDBEngine.h`
- `arangod/RocksDBEngine/RocksDBCollection.h`
- `arangod/VocBase/vocbase.h`
- `arangod/VocBase/LogicalCollection.h`
- `arangod/RocksDBEngine/RocksDBVPackIndex.h`

**Findings:**

#### Serialization Format
ArangoDB's choice of **VelocyPack (VPack)** is a cornerstone of its performance. It's a binary format for a superset of JSON. Unlike JSON, VPack is structured to allow for fast attribute access without parsing the entire document. This is a significant advantage over the `JSON.stringify`/`JSON.parse` approach in the current IndexedDB implementation. The `RocksDBVPackIndex.h` file shows that VPack is even used to construct index keys, demonstrating its deep integration.

#### Storage Structure
- **Layered Architecture:** ArangoDB has a clean separation of concerns through a layered architecture:
    1.  **`LogicalDataSource` / `LogicalCollection`:** The high-level, user-facing representation of a collection. It handles the collection's properties, sharding, and other logical aspects.
    2.  **`PhysicalCollection`:** An abstract interface that defines the physical storage operations for a collection (e.g., insert, update, lookup).
    3.  **`RocksDBCollection`:** The concrete implementation of `PhysicalCollection` for the RocksDB storage engine. It translates the logical operations into key-value operations for RocksDB.
    4.  **`StorageEngine` / `RocksDBEngine`:** The top-level classes that manage the entire storage engine, including database creation, transaction management, and the lifecycle of collections.
- **Key-Value Backend:** Everything is stored in RocksDB, a highly performant key-value store. Documents, indexes, and graph edges are all mapped to key-value pairs. The `RocksDBEngine.h` file shows the direct interaction with the `rocksdb::TransactionDB` instance.

#### Schema Flexibility
- VPack's self-describing nature allows for fully flexible schemas, just like JSON. This is a perfect match for the `properties_json` field in the existing data model.

**Code References:**
```cpp
// from arangod/VocBase/LogicalCollection.h
class LogicalCollection : public LogicalDataSource {
  // ...
  PhysicalCollection* getPhysical() const { return _physical.get(); }
  // ...
  std::unique_ptr<PhysicalCollection> _physical;
};

// from arangod/StorageEngine/PhysicalCollection.h
class PhysicalCollection {
  // ...
  virtual Result insert(transaction::Methods& trx, ..., velocypack::Slice newDocument, ...) = 0;
  // ...
};

// from arangod/RocksDBEngine/RocksDBCollection.h
class RocksDBCollection final : public RocksDBMetaCollection {
  // ...
  Result insert(transaction::Methods& trx, ..., velocypack::Slice newDocument, ...) override;
  // ...
};
```

### 1.2 Our IndexedDB Approach

**Files Analyzed:**
- `src/DB/idbKnowledgeGraph.ts`
- `src/DB/idbBase.ts`

**Findings:**

- **Serialization:** Uses `JSON.stringify` to store object properties as a string (`properties_json`), which is inefficient for querying and requires parsing the entire string to access a single attribute.
- **Storage:** Relies on the browser's IndexedDB, which is a transactional key-value store, but with significant overhead compared to a native solution.

### 1.3 Rust Design Decisions

**Storage Backend:**
- **`redb`:** The choice of `redb` is reinforced. It provides a simple, pure-Rust key-value store that is sufficient for this single-user, embedded application. It avoids the complexity and C++ dependencies of RocksDB while providing a significant performance boost over IndexedDB.

**Serialization:**
- **`serde` + `bincode`:** This combination is the clear winner. It provides the same schema flexibility as VPack and JSON but with the speed of a binary format. It is the idiomatic way to handle serialization in Rust and will be a huge improvement over the current `JSON.stringify` approach.

**Decision:** The new database will use **`redb`** for its key-value storage and **`serde` with `bincode`** for serialization. This provides a modern, performant, and pure-Rust foundation.

---

## 2. Graph Storage & Edges

### 2.1 ArangoDB Edge Collections

**Files Studied:**
- `arangod/RocksDBEngine/RocksDBEdgeIndex.h`
- `arangod/VocBase/LogicalCollection.h`

**Findings:**

#### Edge Storage Model
- ArangoDB models graphs by storing edges in a dedicated **edge collection**. These are special collections that enforce the presence of `_from` and `_to` attributes.
- The `RocksDBEdgeIndex` is a specialized index that is automatically created for edge collections. It indexes both the `_from` and `_to` attributes, allowing for fast lookups of incoming and outgoing edges.

#### _from and _to Indexing
- The `RocksDBEdgeIndex` is not a single index but two. One for `_from` and one for `_to`. This is a classic adjacency list implementation using a key-value store, where the key is the node ID and the value is a list of edge documents.

### 2.2 Our IndexedDB Edge Management

**Files Analyzed:**
- `src/DB/idbKnowledgeGraph.ts`

**Findings:**

- The current implementation is surprisingly similar to ArangoDB's at a high level. Edges are stored in a separate `DB_KNOWLEDGE_GRAPH_EDGES` object store, and the `from_node_id` and `to_node_id` attributes are indexed. This allows for efficient 1-hop traversals.

### 2.3 Rust Design Decisions

**Edge Storage:** **Separate `redb` Table with Secondary Indexes**
- The design of using a separate `redb` table for edges is confirmed as the correct approach.
- **Primary Table:** `edges: Table<&str, &[u8]>` will store `EdgeId -> bincode-serialized Edge struct`.
- **Secondary Indexes:** Two additional `redb` tables will be created to act as secondary indexes:
    - `edges_from_index: Table<&str, &[u8]>` will map `from_node_id -> bincode-serialized Vec<EdgeId>`.
    - `edges_to_index: Table<&str, &[u8]>` will map `to_node_id -> bincode-serialized Vec<EdgeId>`.
- This design is a direct translation of the efficient adjacency list model used by both ArangoDB and the existing IndexedDB implementation, and it will be highly performant in Rust.

---

## 3. Indexing Strategies

### 3.1 ArangoDB Indexes

**Files Studied:**
- `arangod/RocksDBEngine/RocksDBPrimaryIndex.h`
- `arangod/RocksDBEngine/RocksDBEdgeIndex.h`
- `arangod/RocksDBEngine/RocksDBVPackIndex.h`

**Findings:**

#### Primary Index (ID lookups)
- ArangoDB uses a `RocksDBPrimaryIndex` for fast document lookups by `_key`. This is a sorted index that maps the document key to the document's `LocalDocumentId`.

#### Edge Index (from/to lookups)
- The `RocksDBEdgeIndex` is a specialized hash-based index that stores pointers from `_from` and `_to` values to the corresponding edge documents. This allows for very fast traversal from a node to its neighbors.

#### VPack Index (filtering by node properties)
- The `RocksDBVPackIndex` is a general-purpose index that can be created on any combination of document attributes. It allows for fast filtering and sorting on arbitrary fields, which is crucial for a flexible query language like AQL.

### 3.2 Our IndexedDB Indexes

**Files Analyzed:**
- `src/DB/idbSchema.ts`

**Findings:**

#### Current Indexes
- **Nodes:** The `DB_KNOWLEDGE_GRAPH_NODES` store is indexed by `type` and `label`. This allows for basic filtering of nodes.
- **Edges:** The `DB_KNOWLEDGE_GRAPH_EDGES` store is indexed by `from_node_id`, `to_node_id`, and `edge_type`. This is a good design that enables efficient 1-hop traversals.
- **Embeddings:** The `DB_EMBEDDINGS` store is indexed by `input` and `model`, which allows for finding existing embeddings to avoid re-computation.

### 3.3 Rust Design Decisions

**Essential Indexes (MVP):**
1.  **Primary Index (Node ID -> Node):** This will be the main table in `redb` for storing nodes, where the key is the node ID.
2.  **Type Index (Node Type -> Node IDs):** A separate `redb` table mapping a node type (e.g., "chat", "message") to a `Vec<String>` of node IDs. This will allow for fast filtering by type.
3.  **Edge Indexes (from/to -> Edges):** As decided in the Graph Storage section, two separate `redb` tables will be used to index edges by `from_node_id` and `to_node_id`.

**Data Structures:**
- All indexes will be implemented as `redb` tables. The values will be `bincode`-serialized vectors of IDs or other data.

**Rationale:**
- This initial set of indexes mirrors the most important indexes from both ArangoDB and the existing IndexedDB implementation, providing a solid foundation for the most common query patterns.
- Using `redb` tables for indexes is a simple and efficient approach for an embedded database.

---

## 4. Query Processing & Graph Traversal

### 4.1 ArangoDB AQL & Traversal

**Findings:**
- ArangoDB's AQL provides powerful graph traversal capabilities, including BFS, DFS, and shortest path algorithms.
- The `TraversalExecutor.cpp` is responsible for executing these traversals.

### 4.2 petgraph Rust Architecture

**Files Analyzed:**
- `src/visit/traversal.rs`
- `src/algo/dijkstra.rs`

**Findings:**

#### Graph Representation
- `petgraph` provides generic graph data structures (`Graph`, `GraphMap`) and uses traits to abstract over the graph representation.

#### Traversal Algorithms
- **BFS/DFS:** `traversal.rs` provides iterator-based `Bfs` and `Dfs` structs.
- **Dijkstra:** `dijkstra.rs` provides a generic `dijkstra` function for shortest path finding.

### 4.3 Our IndexedDB Query Patterns

**Findings:**
- The most common query is 1-hop traversal. There is no built-in support for multi-hop or shortest path queries.

### 4.4 Rust Design Decisions

**Query API:**
- **Leverage `petgraph`:** The new database will use `petgraph` for all graph algorithms.
- **Internal `petgraph::Graph`:** An in-memory `petgraph::Graph` will be maintained for traversal queries.

**Proposed API:**
```rust
use petgraph::graph::{Graph, NodeIndex};
use std::collections::HashMap;

pub struct EmbeddedDB {
    // ...
    graph: Graph<String, String>,
    node_id_to_index: HashMap<String, NodeIndex>,
}

impl EmbeddedDB {
    pub fn get_edges(&self, node_id: &str, direction: Direction) -> Vec<Edge> { ... }
    pub fn traverse(&self, start_id: &str, max_depth: u32) -> Vec<Path> { ... }
    pub fn shortest_path(&self, start_id: &str, end_id: &str) -> Option<Path> { ... }
}
```

**Rationale:**
- This design provides a powerful and extensible query layer by leveraging the robust and idiomatic algorithms in `petgraph`.

---

## 5. Vector Embeddings & Similarity Search

### 5.1 ArangoDB Vector Search

**Findings:**
- ArangoDB uses the **HNSW** algorithm for vector indexing, integrated with ArangoSearch for hybrid queries.

### 5.2 Qdrant Rust Architecture (Study for Rust Patterns!)

**Files Analyzed:**
- `lib/segment/src/vector_storage/vector_storage_base.rs`
- `lib/segment/src/vector_storage/dense/simple_dense_vector_storage.rs`
- `lib/segment/src/vector_storage/dense/memmap_dense_vector_storage.rs`
- `lib/segment/src/index/hnsw_index/hnsw.rs`
- `lib/segment/src/index/hnsw_index/graph_layers.rs`
- `lib/segment/src/spaces/metric.rs`
- `lib/segment/src/spaces/simple.rs`

**Findings:**

#### Vector Storage
- **`VectorStorage` Trait:** Qdrant's use of a `VectorStorage` trait is a key design pattern, allowing for multiple storage backends (in-memory, memory-mapped).
- **Raw Data Layout:** Vectors are stored as flat arrays of primitive types, which is the most efficient approach.

#### HNSW Index
- **Separation of Concerns:** The `HNSWIndex` is separate from the `VectorStorage`, with the index holding the graph structure and the storage holding the vector data.

#### Distance Metrics
- **`Metric` Trait:** A `Metric` trait is used to abstract distance calculations, making the system extensible.

### 5.3 Our IndexedDB Embedding System

**Findings:**
- The current system uses a brute-force search, which is a major performance bottleneck.

### 5.4 Rust Design Decisions

**Vector Storage:**
- **`redb` Table:** A `redb` table will be used to store embeddings, with the embedding ID as the key and the `bincode`-serialized vector as the value.

**Similarity Algorithm:**
- **Cosine Similarity:** A `cosine_similarity` function will be implemented in Rust.

**Indexing:**
- **Brute-Force MVP:** The initial version will use a brute-force search for simplicity.
- **Future HNSW:** An HNSW index will be implemented in a future phase, following the patterns from Qdrant.

**Decision:** Start with a simple `redb`-based vector store and brute-force cosine similarity, with a clear path to a future HNSW implementation.

---

## 6. Multi-Model Architecture

### 6.1 ArangoDB Unification Strategy

**Findings:**
- ArangoDB's multi-model capability comes from storing all data as documents and using different collection types and indexes to enable different models (e.g., edge collections for graphs).

### 6.2 Our IndexedDB Unification (Everything is a Node!)

**Findings:**
- The use of a `KnowledgeGraphNode` base class creates a unified graph structure, which is a powerful and proven pattern in the existing application.

### 6.3 Rust Design Decisions

**Unification Strategy:** **Traits**
- The "everything is a node" concept will be implemented using a `GraphNode` trait in Rust. This is an idiomatic and extensible approach.

---

## Summary: Rust Implementation Roadmap

### Phase 1: Core Storage (Week 1-2)
- [ ] Integrate `redb` and `bincode`.
- [ ] Implement CRUD operations for nodes, edges, and embeddings.

### Phase 2: Indexing (Week 2-3)
- [ ] Implement primary, type, and edge indexes using `redb` tables.

### Phase 3: Query Layer (Week 3-4)
- [ ] Integrate `petgraph`.
- [ ] Implement 1-hop, multi-hop (BFS), and shortest path (Dijkstra) queries.
- [ ] Implement brute-force vector search.

### Phase 4: PyO3 Bindings (Week 4-5)
- [ ] Expose the Rust API to Python.

### Phase 5: Integration & Testing (Week 5-6)
- [ ] Write comprehensive integration tests.
- [ ] Benchmark performance.

---

## Open Questions & Decisions Needed

1.  **Storage Backend:** Is `redb` the final choice, or should `sled` be reconsidered for specific workloads?
2.  **Serialization:** Is `bincode` the best choice, or should other formats be considered?
3.  **Vector Indexing:** When should the transition from brute-force to HNSW be prioritized?

---

## References

- **ArangoDB:** https://github.com/arangodb/arangodb
- **Qdrant:** https://github.com/qdrant/qdrant
- **petgraph:** https://github.com/petgraph/petgraph
- **Our Codebase:** `src/DB/`
- **Rust Resources:** PyO3 Book, redb, bincode