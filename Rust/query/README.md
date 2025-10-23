# Query Crate

The converged query engine for the TabAgent embedded database.

## Purpose

The `query` crate implements the **Converged Query Pipeline**, a multi-stage query execution system that fuses three distinct query facets into a unified interface:

1. **Structural Filters** - Fast exact matching on indexed properties
2. **Graph Traversals** - Relationship-based filtering using BFS
3. **Semantic Search** - Vector similarity ranking

## Architecture

### The Two-Stage Pipeline

```
┌─────────────────────────────────────────────────┐
│          STAGE 1: Candidate Generation          │
│                                                  │
│  ┌─────────────────┐    ┌─────────────────┐   │
│  │  Structural     │    │  Graph          │   │
│  │  Filters        │───▶│  Traversal      │   │
│  │  (Exact Match)  │    │  (BFS)          │   │
│  └─────────────────┘    └─────────────────┘   │
│                  │              │               │
│                  └──────┬───────┘               │
│                         ▼                       │
│                 Intersection                    │
│                 (Accurate Candidate Set)        │
└─────────────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────┐
│       STAGE 2: Semantic Re-ranking               │
│                                                  │
│  ┌─────────────────────────────────────┐       │
│  │  Vector Search on Candidates         │       │
│  │  (HNSW ANN)                          │       │
│  └─────────────────────────────────────┘       │
│                     │                            │
│                     ▼                            │
│           Ranked Results                         │
└─────────────────────────────────────────────────┘
```

### Key Design Principles

1. **Accuracy First, Relevance Second**  
   Structural and graph filters ensure factual accuracy before semantic ranking ensures relevance.

2. **Efficient Filtering**  
   By filtering first using fast indexes, semantic search only operates on a small candidate set, dramatically improving performance.

3. **Composable Queries**  
   Each query facet is optional and can be combined in any way, providing maximum flexibility.

## Core Components

### Models (`models.rs`)

- **`ConvergedQuery`** - The top-level query specification
- **`SemanticQuery`** - Vector search parameters
- **`StructuralFilter`** - Property-based filters with operators
- **`GraphFilter`** - Relationship traversal specification
- **`QueryResult`** - Result containing node and optional similarity score
- **`Path`** - Ordered nodes and edges for graph traversals

### QueryManager (`lib.rs`)

The central orchestrator that:
- Executes the multi-stage pipeline
- Manages candidate set intersection
- Coordinates storage and indexing layers
- Provides high-level convenience APIs

## Usage Examples

### 1. Structural Query Only

Find all messages in a specific chat:

```rust
use query::{QueryManager, models::*};
use serde_json::json;

let query = ConvergedQuery {
    structural_filters: Some(vec![
        StructuralFilter {
            property_name: "chat_id".to_string(),
            operator: FilterOperator::Equals,
            value: json!("chat_123"),
        },
        StructuralFilter {
            property_name: "node_type".to_string(),
            operator: FilterOperator::Equals,
            value: json!("Message"),
        },
    ]),
    semantic_query: None,
    graph_filter: None,
    limit: 10,
    offset: 0,
};

let results = query_mgr.query(&query)?;
```

### 2. Graph Traversal

Find all nodes within 2 hops of a starting node:

```rust
let query = ConvergedQuery {
    structural_filters: None,
    semantic_query: None,
    graph_filter: Some(GraphFilter {
        start_node_id: "entity_abc".to_string(),
        direction: EdgeDirection::Outbound,
        edge_type: Some("MENTIONS".to_string()),
        depth: 2,
    }),
    limit: 50,
    offset: 0,
};

let results = query_mgr.query(&query)?;
```

### 3. Converged Query (All Three Facets)

Find semantically similar messages in a specific chat that mention a particular entity:

```rust
let query = ConvergedQuery {
    structural_filters: Some(vec![
        StructuralFilter {
            property_name: "chat_id".to_string(),
            operator: FilterOperator::Equals,
            value: json!("chat_123"),
        },
    ]),
    graph_filter: Some(GraphFilter {
        start_node_id: "entity_project_phoenix".to_string(),
        direction: EdgeDirection::Inbound,
        edge_type: Some("MENTIONS".to_string()),
        depth: 1,
    }),
    semantic_query: Some(SemanticQuery {
        vector: embedding_vector,
        similarity_threshold: Some(0.7),
    }),
    limit: 5,
    offset: 0,
};

let results = query_mgr.query(&query)?;
```

### 4. Shortest Path

Find the shortest path between two nodes:

```rust
let path = query_mgr.find_shortest_path("node_a", "node_b")?;

if let Some(path) = path {
    println!("Path length: {}", path.nodes.len());
    println!("Edges: {}", path.edges.len());
}
```

## Performance Characteristics

| Operation | Complexity | Notes |
|-----------|-----------|-------|
| Structural Filter | O(1) to O(log n) | Uses secondary indexes |
| Graph Traversal | O(E + V) | BFS, limited by depth |
| Semantic Search | O(log n) | HNSW ANN on candidate set |
| Converged Query | O(C * log C) | C = candidate set size |

**Key Insight**: By filtering to a small candidate set (C) before semantic search, we achieve sub-linear performance even on large datasets.

## Integration with Other Crates

```
query
├── Depends on: storage (CRUD operations)
├── Depends on: indexing (secondary indexes, graph, vector search)
├── Depends on: common (types, errors)
└── Used by: Python bindings (via PyO3)
```

## Testing

Run tests:
```bash
cargo test -p query
```

Current test coverage:
- ✅ Structural filtering
- ✅ Empty result sets
- ✅ QueryManager initialization
- ✅ Doc test examples

## Future Enhancements

- **Complex Filter Logic**: Support nested AND/OR expressions
- **Range Queries**: `GreaterThan`, `LessThan` operators on indexed fields
- **Multi-Hop Graph Patterns**: Cypher-like pattern matching
- **Cursor-Based Pagination**: For efficient large result sets
- **Query Optimization**: Cost-based query planning
- **Caching**: Frequently-used query result caching

## See Also

- [StorageLayer.md](../StorageLayer.md) - CRUD operations
- [IndexingLayer.md](../IndexingLayer.md) - Secondary indexes
- [QueryEngine.md](../QueryEngine.md) - Full specification

