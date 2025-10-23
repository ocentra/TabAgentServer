```markdown
# Specification: Query Engine (`query` crate)

## 1. Objective & Core Principles

The `query` crate is the intelligence of the database. It provides the high-level, expressive API that the AI agent will use to interact with its memory. This crate's primary responsibility is to translate a multi-faceted query into a series of optimized operations against the `storage` and `indexing` layers, delivering a single, coherent, and relevant result set.

This component is governed by the following core principles:

*   **Converged Query Model:** This crate is the concrete implementation of the Converged Query Model. Its central `query` function is designed to seamlessly fuse **structural filters**, **graph traversals**, and **semantic search** into a single, atomic operation.
*   **Accuracy First, Relevance Second:** The engine will execute a **Multi-Stage Query Pipeline**. It will always prioritize factual and relational accuracy (Stage 1) before re-ranking the accurate results by semantic relevance (Stage 2). This is critical to prevent feeding factually incorrect but semantically similar context to the LLM.
*   **AI-First API:** The public API will be high-level and declarative. It is designed to be intuitive for an AI agent to use, allowing it to ask complex, human-like questions about its memory without needing to write complex, procedural database code.
*   **Performance:** The engine must be highly performant, leveraging the indexes provided by the `indexing` crate to avoid full data scans at all costs.

## 2. Architectural Analysis & Rationale (The "Why")

The decision to build a dedicated, multi-stage query engine is a direct solution to the limitations of the current system and a synthesis of the most powerful concepts from leading database paradigms.

### 2.1. Findings from Current System (IndexedDB)

*   **Critical Limitations to Solve:** The current system has **no unified query engine**. Data retrieval is handled by ad-hoc, procedural functions scattered throughout the application (e.g., `getEdgesByNodeId`, `getAllChats`). A developer (or AI agent) wanting to perform a complex query (e.g., "find messages from user X containing entity Y") would have to manually chain these calls together, fetching large amounts of data into the application layer and filtering it there. This is:
    *   **Inefficient:** It transfers excessive data and pushes computational load onto the client.
    *   **Complex:** It forces the application to manage the query logic.
    *   **Unscalable:** It cannot support the fusion of graph, structural, and semantic queries.

### 2.2. Findings from Reference Systems

*   **SQL Databases (e.g., PostgreSQL):** The power of SQL lies in its declarative `WHERE` clause and the query planner that translates it into an efficient execution plan using indexes. We adopt this principle for our **structural filters**, allowing the user to state *what* they want, not *how* to get it.
*   **Graph Databases (e.g., Neo4j's Cypher, ArangoDB's AQL):** The power of graph query languages is their ability to express complex relationship patterns declaratively. While we are not implementing a full query parser, our **graph filters** are designed to capture the essence of these patterns (e.g., "find nodes connected to X via a 'MENTIONS' edge"), providing the same expressive power through a structured API.
*   **Vector Databases (e.g., Qdrant):** The core feature of a modern vector DB is not just vector search, but **hybrid search**—the ability to pre-filter by metadata *before* performing the vector search. This is the exact pattern our Multi-Stage Query Pipeline implements, ensuring both accuracy and performance.

### 2.3. Synthesis & Final Architectural Decision

The **Multi-Stage Converged Query Pipeline** is the definitive architecture for this crate.

This design is a direct solution to the fragmented and inefficient query patterns of the current system. By creating a single, powerful `query` entry point in Rust, we centralize all query logic and optimization within the high-performance native core.

This architecture synthesizes the best of all paradigms:
*   It provides the **declarative filtering of SQL**.
*   It offers the **relational expressiveness of a Graph DB**.
*   It incorporates the **hybrid search capability of a Vector DB**.

By executing these facets in a specific order (filter first, then re-rank), we create a system that is uniquely suited to the needs of a RAG-powered AI agent, delivering context that is guaranteed to be factually correct and optimally relevant.

## 3. Detailed Rust Implementation Blueprint (The "What")

### 3.1. Data Structures

These structs define the public API contract for a converged query. They are designed to be easily constructed by a client (like the Python wrapper).

File: `query/src/models.rs`
```rust
use storage::models::{Node, NodeId, Edge};
use serde::{Deserialize, Serialize};

/// The top-level structure for a converged query.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConvergedQuery {
    pub semantic_query: Option<SemanticQuery>,
    pub structural_filters: Option<Vec<StructuralFilter>>,
    pub graph_filter: Option<GraphFilter>,
    pub limit: usize,
    pub offset: usize,
}

/// Defines a semantic search component.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SemanticQuery {
    pub vector: Vec<f32>,
    // A threshold can be used to filter results by similarity score post-ranking.
    pub similarity_threshold: Option<f32>,
}

/// Defines a filter on a core, indexed property of a node.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StructuralFilter {
    pub property_name: String, // e.g., "chat_id", "sender", "node_type"
    pub operator: FilterOperator,
    pub value: serde_json::Value, // The value to compare against
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum FilterOperator {
    Equals,
    NotEquals,
    GreaterThan,
    // ... other operators as needed
}

/// Defines a filter based on graph relationships.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GraphFilter {
    pub start_node_id: NodeId,
    pub direction: EdgeDirection,
    pub edge_type: Option<String>,
    pub depth: u32, // 1 for direct neighbors, >1 for multi-hop traversal
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum EdgeDirection {
    Outbound,
    Inbound,
    Both,
}

// --- Result Structures ---

/// Represents a single item in the query result set.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QueryResult {
    pub node: Node,
    pub similarity_score: Option<f32>,
}

/// Represents a path found during a graph traversal.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Path {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}
```

### 3.2. The Query Pipeline (Core Logic)

The central `QueryManager` will execute the query as follows:

File: `query/src/lib.rs`
```rust
use storage::StorageManager;
use indexing::IndexManager;
use std::collections::HashSet;
// ... other imports

pub struct QueryManager<'a> {
    storage: &'a StorageManager,
    indexing: &'a IndexManager,
}

impl<'a> QueryManager<'a> {
    pub fn new(storage: &'a StorageManager, indexing: &'a IndexManager) -> Self {
        Self { storage, indexing }
    }

    /// The main entry point for all queries in the database.
    pub fn query(&self, query: &ConvergedQuery) -> Result<Vec<QueryResult>, QueryError> {
        // --- STAGE 1: Candidate Set Generation ---

        // 1a. Apply structural filters.
        let structural_candidates: Option<HashSet<NodeId>> = self.apply_structural_filters(&query.structural_filters)?;

        // 1b. Apply graph filters.
        let graph_candidates: Option<HashSet<NodeId>> = self.apply_graph_filter(&query.graph_filter)?;

        // 1c. Intersect candidate sets to get the final, accurate set.
        // If a filter type exists, it narrows the set. If not, it's ignored.
        let final_candidates = self.intersect_candidates(structural_candidates, graph_candidates);

        // --- STAGE 2: Fetching & Semantic Re-ranking ---

        let results = if let Some(semantic_query) = &query.semantic_query {
            // If there's a semantic component, search within the candidate set and re-rank.
            self.fetch_and_rank_by_similarity(
                &semantic_query.vector,
                final_candidates, // Can be None, meaning search all
                query.limit,
                query.offset
            )?
        } else {
            // If no semantic component, just fetch the filtered candidates.
            self.fetch_nodes(
                final_candidates, // Can be None, meaning fetch all
                query.limit,
                query.offset
            )?
        };

        Ok(results)
    }

    // ... private helper methods for apply_structural_filters, apply_graph_filter, etc. ...

    // --- High-Level Convenience APIs ---

    /// Finds the shortest path between two nodes.
    pub fn find_shortest_path(&self, start_node_id: &NodeId, end_node_id: &NodeId) -> Result<Option<Path>, QueryError> {
        // This will use a BFS traversal algorithm from the `indexing` or a graph crate.
        // ... implementation ...
    }
}

// Custom error type for the query layer
#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("Storage error: {0}")]
    Storage(#[from] storage::DbError),
    #[error("Indexing error: {0}")]
    Indexing(#[from] indexing::IndexError), // Assuming IndexError exists
    #[error("Invalid query: {0}")]
    InvalidQuery(String),
}
```

## 4. Implementation Plan & Checklist

*   [ ] **Project Setup:**
    *   [ ] Create the `query` crate.
    *   [ ] Add dependencies on the `storage` and `indexing` crates, `serde`, `serde_json`, and `thiserror`.
*   [ ] **Models:**
    *   [ ] Implement all query and result structs in `query/src/models.rs`.
*   [ ] **QueryManager & Pipeline:**
    *   [ ] Implement the `QueryManager` struct and its `new` constructor.
    *   [ ] Implement the private helper for `apply_structural_filters`. This will loop through the filters and call the appropriate `IndexManager` methods.
    *   [ ] Implement the private helper for `apply_graph_filter`. This will involve a traversal (e.g., BFS up to `depth`) using the graph indexes.
    *   [ ] Implement the `intersect_candidates` logic to correctly combine the results of the filters.
    *   [ ] Implement the main `query` function, orchestrating the full pipeline.
*   [ ] **High-Level APIs:**
    *   [ ] Implement `find_shortest_path` using a graph traversal algorithm.
*   [ ] **Integration Tests:**
    *   [ ] Write a test for a purely structural query.
    *   [ ] Write a test for a purely graph-based query.
    *   [ ] Write a test for a purely semantic query.
    *   [ ] **Crucially, write tests for converged queries:**
        *   Test a structural + semantic query.
        *   Test a graph + semantic query.
        *   Test a query that combines all three facets.
    *   [ ] Test pagination (`limit` and `offset`).

## 5. Open Questions & Decisions Needed

*   **Complex Filter Logic:** The initial implementation of `apply_structural_filters` will treat the list of filters as a logical **AND**. Supporting complex nested `AND`/`OR` logic would require a more complex filter structure (e.g., a tree) and is deferred to a future version.
*   **Pagination Strategy:** The initial `limit`/`offset` implementation will be simple. For very large result sets, more advanced cursor-based pagination might be needed in the future to improve performance.
*   **Graph Algorithm Crate:** While traversals can be implemented using the `indexing` crate's primitives, for more advanced algorithms like shortest path, a dedicated graph crate like `petgraph` should be integrated. The decision to add this dependency will be made during the implementation of `find_shortest_path`.
```