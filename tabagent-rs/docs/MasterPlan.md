# TabAgent Embedded Database: Master Implementation Plan

## 1. Project Vision: The "Humane Brain"

The objective is to build a high-performance, embedded, multi-model database in Rust, designed to serve as the **active cognitive engine** for the TabAgent AI system.

This is not a passive data store. It is an **AI-first architecture** that models the associative and contextual nature of human memory. Its primary user is the AI agent, and its core function is to empower the agent with a deep, interconnected, and rapidly accessible understanding of its knowledge. The system must be enterprise-grade, prioritizing performance, concurrency, and data integrity from day one, with no compromises for "simpler" initial versions.

## 2. Core Architectural Pillars

This architecture is founded on five key decisions derived from extensive analysis of our existing system and industry-leading databases (ArangoDB, Neo4j, Qdrant).

1.  **Core Storage Engine: `sled`**
    *   **Rationale:** The database will operate in a high-concurrency server environment (FastAPI, WebRTC). `sled` provides the necessary thread-safe, transactional, and lock-free primitives to prevent data corruption and performance bottlenecks. It is the only viable choice for an enterprise-grade embedded server backend.

2.  **Graph Engine Architecture: Stateless Rust Core, Stateful Python Facade**
    *   **Rationale:** To ensure data consistency and safety in a concurrent Rust environment, the core engine will be stateless. `Node` and `Edge` structs will be simple data containers. All graph operations will be managed by the central `EmbeddedDB` object. A Python wrapper will be created to provide a convenient, "Active Record" style API for the AI agent, mimicking the feel of the existing TypeScript implementation without compromising the core engine's safety.

3.  **Query Model: Converged Query Pipeline**
    *   **Rationale:** The primary interface for data retrieval will be a converged query model that fuses three facets into a single, multi-stage pipeline. This ensures that context provided to the LLM is both **factually accurate and semantically relevant**.
        *   **Stage 1: Candidate Set Generation:** Fast, exact filtering using `sled` secondary indexes (structural) and graph traversals.
        *   **Stage 2: Semantic Re-ranking:** HNSW search performed *only* on the accurate candidate set from Stage 1.

4.  **Schema & Data Structures: Hybrid Schema Model**
    *   **Rationale:** To enable the high-performance filtering required by the Converged Query Model, core entities (`Message`, `Chat`, etc.) will be defined as strongly-typed Rust structs. This allows for the creation of efficient secondary indexes on critical fields (`sender`, `timestamp`, `chat_id`). Flexibility is retained by including a `metadata: serde_json::Value` field for non-essential or evolving application data. This model directly fixes the primary performance bottleneck of the current IndexedDB implementation.

5.  **Cognitive Engine: The "Knowledge Weaver"**
    *   **Rationale:** The database will be an **active system**. An event-driven, asynchronous engine will run in the background to autonomously enrich the knowledge graph. This engine is responsible for:
        *   **Semantic Indexing:** Generating vector embeddings.
        *   **Entity Extraction & Linking:** Performing NER and creating `MENTIONS` edges across all chats.
        *   **Summarization:** Consolidating conversations into hierarchical memories.
        *   **Associative Linking:** Creating `IS_SEMANTICALLY_SIMILAR_TO` edges between different contexts, enabling "intuitive leaps."

## 3. Modular Specification Documents

This master plan is the entry point. The detailed engineering blueprints are located in the following modular specification documents, each corresponding to a primary Rust crate:

*   **`StorageLayer.md`**: Defines the core data structures, `sled` on-disk layout, and basic CRUD operations. Corresponds to the `storage` crate.
*   **`IndexingLayer.md`**: Defines the architecture for secondary indexes (structural, graph, and the mandatory HNSW vector index). Corresponds to the `indexing` crate.
*   **`QueryEngine.md`**: Details the implementation of the Converged Query Pipeline and the high-level RAG API. Corresponds to the `query` crate.
*   **`KnowledgeWeaver.md`**: Outlines the architecture of the asynchronous cognitive engine. Corresponds to the `weaver` crate.
*   **`APIBindings.md`**: Specifies the public FFI/PyO3 API that will be exposed to clients like Python. Corresponds to the root `lib.rs`.

## 4. Phased Implementation Roadmap

Implementation will proceed in phases, with each phase having clear objectives and success criteria.

---

### **Phase 1: The Foundation (Core Storage & Data Models)**

*   **Objective:** Establish the absolute foundation of the database: the ability to reliably store and retrieve the core data entities according to the Hybrid Schema Model.
*   **Crates Involved:** `storage`
*   **Key Tasks (Checklist):**
    *   [ ] Initialize Rust project with `sled`, `serde`, and `bincode` dependencies.
    *   [ ] Implement the Rust structs for `Node`, `Edge`, `Embedding`, `Chat`, `Message`, etc., as defined in `StorageLayer.md`.
    *   [ ] Implement the `StorageManager` responsible for interacting with `sled`.
    *   [ ] Define and implement the on-disk layout (key formats) for all primary data.
    *   [ ] Implement the basic, single-key CRUD functions: `create_entity`, `get_entity`, `update_entity`, `delete_entity`.
    *   [ ] Write comprehensive unit tests for all CRUD operations to ensure data integrity and serialization correctness.
*   **Success Criteria:**
    *   All unit tests for the `storage` crate pass.
    *   Data can be written to a `sled` database file, the process can be shut down, and the data can be perfectly read back upon restart.
    *   Benchmarks show that basic key-value lookups are in the sub-millisecond range.

---

### **Phase 2: The Connections (Indexing & Graph Primitives)**

*   **Objective:** Build the indexing layer that makes finding data fast and enable basic graph traversal. This includes the mandatory HNSW index.
*   **Crates Involved:** `indexing`, `storage` (updated)
*   **Key Tasks (Checklist):**
    *   [ ] Implement the secondary indexing mechanism for structural properties (e.g., `get_nodes_by_type`).
    *   [ ] Implement the graph indexing mechanism (from/to indexes for edges).
    *   [ ] Implement the HNSW index for vector data. A well-maintained Rust crate (e.g., `hnsw`) will be used, integrated with our storage layer.
    *   [ ] Ensure all `create`, `update`, and `delete` operations in the `storage` crate correctly update all relevant indexes transactionally.
    *   [ ] Implement the primitive graph traversal functions: `get_edges(node_id, direction)`.
*   **Success Criteria:**
    *   Unit tests confirm that creating a node correctly populates the type index.
    *   Unit tests confirm that creating an edge correctly populates both the `from` and `to` graph indexes.
    *   A benchmark that filters 100,000 nodes by an indexed property completes in under 5ms.
    *   A benchmark that adds 10,000 vectors to the HNSW index and performs a k-NN search (k=10) completes in under 2ms.

---

### **Phase 3: The Intelligence (Converged Query Engine)**

*   **Objective:** Implement the core logic of the database: the multi-stage query pipeline.
*   **Crates Involved:** `query`, `indexing`, `storage`
*   **Key Tasks (Checklist):**
    *   [ ] Define the `ConvergedQuery` struct in Rust.
    *   [ ] Implement the Stage 1 logic: fetching and intersecting candidate sets from the `indexing` layer.
    *   [ ] Implement the Stage 2 logic: performing a semantic search *only* on the Stage 1 candidate set.
    *   [ ] Implement the high-level RAG API functions (`find_similar_memories`, etc.) that construct and execute `ConvergedQuery` objects.
    *   [ ] Write integration tests for complex queries that combine semantic, structural, and graph filters.
*   **Success Criteria:**
    *   A query with a structural filter for 10 nodes out of 100,000, followed by a semantic search, is verifiably faster and more accurate than a semantic search over the entire dataset.
    *   All integration tests for the `query` crate pass, confirming that the pipeline produces accurate and relevant results.

---

### **Phase 4: The Bridge (API Bindings)**

*   **Objective:** Expose the powerful Rust engine to the outside world, specifically Python.
*   **Crates Involved:** Root library (`lib.rs`)
*   **Key Tasks (Checklist):**
    *   [ ] Set up the `PyO3` crate and build configuration.
    *   [ ] Create Python-native wrapper structs for all query and data structures.
    *   [ ] Expose the high-level query functions from the `query` crate to Python.
    *   [ ] Handle all type conversions between Python types (lists, dicts) and Rust structs.
    *   [ ] Implement robust error handling that converts Rust `DbError` into Python exceptions.
*   **Success Criteria:**
    *   A Python test script can successfully import the compiled Rust library.
    *   The Python script can create a database, add nodes/edges, and execute a complex converged query.
    *   Errors in the Rust layer (e.g., "node not found") correctly raise exceptions in Python.

---

### **Phase 5: The Mind (Knowledge Weaver)**

*   **Objective:** Implement the autonomous, background engine that enriches the knowledge graph.
*   **Crates Involved:** `weaver`
*   **Key Tasks (Checklist):**
    *   [ ] Set up an asynchronous runtime (e.g., `tokio`) and an in-memory event queue.
    *   [ ] Modify the `storage` crate to emit events (e.g., `NewNodeCreated`) onto the queue after successful transactions.
    *   [ ] Implement the first Weaver module: the Semantic Indexer.
    *   [ ] Implement the Entity Extractor & Linker module.
    *   [ ] Implement the Summarizer and Associative Linker modules.
*   **Success Criteria:**
    *   Adding a new `Message` node via the API correctly triggers a background job that generates an embedding and links entities, which can be verified by a subsequent query.
    *   The system remains responsive to queries even while the Weaver is processing background tasks.