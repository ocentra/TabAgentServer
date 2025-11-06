# Graph Algorithms Module

**Zero-copy graph algorithms operating directly on MDBX memory-mapped storage.**

## 🔥 Key Innovation: Tuple Guard Pattern

All algorithms use **zero-copy iteration** via `GraphIndexGuard`:

```rust
// GraphIndex stores Vec<(EdgeId, NodeId)> in MDBX
pub struct GraphIndexGuard {
    archived: &'static rkyv::Archived<Vec<(EdgeId, NodeId)>>,
}

impl GraphIndexGuard {
    // Zero-copy: iterate (edge, target) pairs from mmap
    pub fn iter_edges(&self) -> impl Iterator<Item = (&str, &str)>;
    pub fn iter_edge_ids(&self) -> impl Iterator<Item = &str>;
    pub fn iter_targets(&self) -> impl Iterator<Item = &str>;
}
```

**Benefits:**
- ✅ **Zero allocations** during graph traversal
- ✅ **Direct mmap access** - strings borrowed from disk
- ✅ **Multiple iteration modes** - choose what you need
- ✅ **No deserialization** - rkyv archived types accessed in-place

## Available Algorithms

### Shortest Path (Single-Source)
- **Dijkstra** - Non-negative weights, optimal for sparse graphs
- **Bellman-Ford** - Handles negative weights, detects negative cycles
- **A*** - Heuristic-guided, optimal with admissible heuristic
- **SPFA** - Shortest Path Faster Algorithm, queue-based Bellman-Ford

### All-Pairs Shortest Path
- **Floyd-Warshall** - Dense graphs, all pairs
- **Johnson** - Sparse graphs, reweighting technique

### Traversal
- **BFS** - Breadth-first search, unweighted shortest path
- **DFS** - Depth-first search, cycle detection

### Strongly Connected Components
- **Tarjan SCC** - Linear time, single-pass
- **Kosaraju SCC** - Two-pass, simpler to understand

### Spanning Trees
- **Prim MST** - Minimum spanning tree, greedy
- **Kruskal MST** - Edge-based, union-find
- **Steiner Tree** - Approximation for terminal nodes

### Network Flow
- **Ford-Fulkerson** - Augmenting paths
- **Dinics** - Blocking flow, faster for unit capacities
- **Min-Cost Max-Flow** - Optimize both flow and cost

### Graph Structure
- **Bridges** - Cut edges (removing disconnects graph)
- **Articulation Points** - Cut vertices
- **Feedback Arc Set** - Edges to remove for DAG
- **Transitive Reduction** - Minimal equivalent graph
- **Transitive Closure** - All reachable pairs

### Centrality & Ranking
- **PageRank** - Node importance, random walk
- **Dominators** - Control flow analysis

### Clustering & Coloring
- **Louvain** - Community detection, modularity optimization
- **DSATUR** - Graph coloring, degree saturation
- **Bron-Kerbosch** - Maximal cliques

### Other
- **Simple Paths** - All paths from source to target
- **K-Shortest Paths** - Top-K paths by weight
- **Isomorphism** - Structural equivalence check
- **Max Weight Matching** - Greedy approximation

## Usage Examples

### Example 1: Zero-Copy Dijkstra

```rust
use indexing::algorithms::dijkstra_zero_copy;
use std::collections::HashSet;

let graph = GraphIndex::open(&env, "my_graph")?;
let nodes: HashSet<String> = /* ... */;

// Edge cost function - receives &str from mmap!
let edge_cost = |edge_id: &str, _target: &str| -> f64 {
    // Parse edge ID or lookup weight - no allocation during iteration!
    1.0  // Unweighted
};

// Returns HashMap<String, f64> of distances
let distances = dijkstra_zero_copy(
    &graph,
    "start_node",
    Some("end_node"),  // Or None for all distances
    edge_cost
)?;

println!("Distance: {}", distances.get("end_node").unwrap());
```

### Example 2: PageRank with Zero-Copy

```rust
use indexing::algorithms::page_rank_zero_copy;

let ranks = page_rank_zero_copy(
    &graph,
    &nodes,
    0.85,      // Damping factor
    100,       // Max iterations
    1e-6       // Convergence tolerance
)?;

for (node, rank) in ranks.iter().take(10) {
    println!("{}: {:.6}", node, rank);
}
```

### Example 3: BFS Traversal

```rust
use indexing::algorithms::bfs_zero_copy;

// BFS from node, visitor called for each node
bfs_zero_copy(&graph, "start", |node_str| {
    println!("Visited: {}", node_str);  // node_str is &str from mmap!
})?;
```

### Example 4: Strongly Connected Components

```rust
use indexing::algorithms::tarjan_scc;

// Returns Vec<Vec<String>> - each inner vec is an SCC
let sccs = tarjan_scc(&graph, &nodes)?;

println!("Found {} strongly connected components", sccs.len());
for (i, scc) in sccs.iter().enumerate() {
    println!("SCC {}: {} nodes", i, scc.len());
}
```

## Architecture

All algorithms operate on `GraphIndex` which stores edges in MDBX:

```
IndexManager
  ├── structural: StructuralIndex
  │   └── libmdbx DBI (property → Vec<NodeId>)
  │
  ├── graph: GraphIndex
  │   ├── outgoing: libmdbx DBI (node_id → Vec<(EdgeId, NodeId)>)
  │   └── incoming: libmdbx DBI (node_id → Vec<(EdgeId, NodeId)>)
  │
  └── vector: VectorIndex
      └── libmdbx DBI (embedding_id → Vector)
```

### Zero-Copy Read Path

1. **MDBX Transaction** - Opens read transaction (mmap access)
2. **GraphIndexGuard** - Wraps `&'static Archived<Vec<(EdgeId, NodeId)>>`
3. **iter_edges()** - Returns iterator of `(&str, &str)` from mmap
4. **Algorithm** - Processes strings directly, allocates only for results

### Write Path (Justified Allocation)

When adding/removing edges:
1. Deserialize `Archived<Vec<(EdgeId, NodeId)>>` → owned `Vec<(EdgeId, NodeId)>`
2. Modify owned vector
3. Serialize back with rkyv → MDBX

**Trade-off:** Write-heavy for modifications, but algorithms stay zero-copy!

## Performance Characteristics

| Algorithm | Time | Space | Zero-Copy? | Notes |
|-----------|------|-------|------------|-------|
| Dijkstra | O((V+E) log V) | O(V) | ✅ | Priority queue, min-heap |
| Bellman-Ford | O(VE) | O(V) | ✅ | Detects negative cycles |
| Floyd-Warshall | O(V³) | O(V²) | ✅ | All-pairs dense graphs |
| BFS/DFS | O(V+E) | O(V) | ✅ | Linear traversal |
| Tarjan SCC | O(V+E) | O(V) | ✅ | Single-pass DFS |
| PageRank | O(kE) | O(V) | ✅ | k = iterations |
| Prim MST | O((V+E) log V) | O(V) | ✅ | Greedy, priority queue |
| Dinic Flow | O(V²E) | O(V+E) | ✅ | Blocking flow layers |

**All algorithms avoid copying graph data during traversal!**

## Attribution

These algorithms are **adapted from petgraph** (MIT/Apache-2.0) to work directly with our MDBX+rkyv stack:

- Original: Generic trait-based implementations
- Our version: Direct `GraphIndex` integration for zero-copy

See `ATTRIBUTION.md` for full licensing details.

## When NOT to Use

❌ **Don't use algorithms for:**
- Simple neighbor lookups → Use `graph.get_outgoing()` directly
- Single edge checks → Use `graph.get_outgoing()` and iterate
- Full graph materialization → These are zero-copy, no need!

✅ **DO use algorithms for:**
- Complex graph analysis
- Path finding
- Ranking/centrality
- Community detection
- Flow problems
