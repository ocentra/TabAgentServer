# Graph Algorithms Module

**Advanced graph analysis algorithms (shortest path, community detection, flow, etc.).**

## What's Here

- `algorithms.rs` - Core algorithms (Dijkstra, A*, MST, etc.)
- `graph_traits.rs` - Generic graph trait definitions
- `community_detection.rs` - Community/clustering algorithms
- `flow_algorithms.rs` - Network flow algorithms

## What This Does

Provides **graph algorithms** that work on top of the core GraphIndex:

- **Shortest Path**: Dijkstra, A*, Bellman-Ford
- **Spanning Trees**: Kruskal's MST, Prim's MST
- **Flow**: Edmonds-Karp, Dinic's algorithm
- **Community**: Louvain, label propagation
- **Traversal**: BFS, DFS, topological sort

## When to Use

**Use when you need:**
- Path finding between nodes
- Analyzing graph structure
- Detecting communities/clusters
- Computing centrality metrics
- Network flow problems

**Don't use:** For simple neighbor lookups (use core GraphIndex instead).

## How to Use

### Shortest Path

```rust
use indexing::algorithms::algorithms::{dijkstra, astar};

// Dijkstra's algorithm
let (path, cost) = dijkstra(&graph, "start_node", "end_node")?;

// A* (with heuristic)
let (path, cost) = astar(&graph, "start_node", "end_node", |node| {
    // Heuristic function
    estimated_distance_to_goal(node)
})?;
```

### Community Detection

```rust
use indexing::algorithms::community_detection::louvain;

// Detect communities
let communities = louvain(&graph)?;

for (community_id, nodes) in communities.iter().enumerate() {
    println!("Community {}: {} nodes", community_id, nodes.len());
}
```

### Maximum Flow

```rust
use indexing::algorithms::flow_algorithms::edmonds_karp;

// Compute max flow
let max_flow = edmonds_karp(
    &graph,
    "source",
    "sink",
    |edge_id| capacity_of_edge(edge_id)
)?;
```

### Minimum Spanning Tree

```rust
use indexing::algorithms::algorithms::kruskal_mst;

// Find MST
let mst_edges = kruskal_mst(&graph, |edge| weight_of_edge(edge))?;
```

## Graph Traits

All algorithms work on any type implementing `GraphTrait`:

```rust
pub trait GraphTrait {
    type NodeId: Clone + Eq + Hash;
    type EdgeId: Clone + Eq + Hash;
    
    fn nodes(&self) -> Vec<Self::NodeId>;
    fn edges(&self) -> Vec<Self::EdgeId>;
    fn neighbors(&self, node: &Self::NodeId) -> Vec<Self::NodeId>;
    // ... more methods
}
```

This means algorithms work on:
- The core GraphIndex
- OptimizedGraphIndex  
- Your custom graph structures

## Performance

| Algorithm | Time Complexity | Space | Use Case |
|-----------|----------------|-------|----------|
| Dijkstra | O((V+E) log V) | O(V) | Weighted shortest path |
| A* | O((V+E) log V) | O(V) | Heuristic shortest path |
| BFS/DFS | O(V+E) | O(V) | Traversal, reachability |
| Louvain | O(V log V) | O(V+E) | Community detection |
| Edmonds-Karp | O(VE²) | O(V+E) | Max flow |
| Kruskal MST | O(E log E) | O(V+E) | Minimum spanning tree |

## Examples

### Find Shortest Path in Chat Graph

```rust
// Find path from user to message
let (path, distance) = dijkstra(
    &idx.get_hot_graph_index().unwrap(),
    "user_5",
    "msg_123"
)?;

println!("Path length: {} hops", path.len());
```

### Detect Message Communities

```rust
// Find clusters of related messages
let communities = louvain(&graph)?;

for community in communities {
    println!("Found {} related messages", community.len());
}
```

