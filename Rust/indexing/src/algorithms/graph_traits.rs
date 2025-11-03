//! Generic graph traits for the TabAgent indexing system.
//!
//! This module defines traits that abstract over different graph implementations,
//! following patterns inspired by petgraph but adapted to our specific needs.
//! These traits enable generic algorithms that work with different graph representations.
//!
//! # Design Principles
//!
//! 1. **Type Safety**: Using newtype patterns for NodeIndex and EdgeIndex to prevent
//!    mixing with other identifier types (RAG Rule 8.1)
//! 2. **Extensibility**: Traits allow multiple graph implementations
//! 3. **Performance**: Designed for zero-cost abstractions (RAG Rule 15.1)
//! 4. **Ergonomics**: Easy to use correctly, hard to misuse (RAG Rule 1.4)

use common::{DbResult, DbError};
use std::hash::Hash;

/// A node identifier in a graph.
///
/// This newtype wrapper prevents accidentally mixing NodeIndex with other types.
/// It's a simple wrapper around usize for efficient storage and indexing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeIndex(pub usize);

impl NodeIndex {
    /// Creates a new NodeIndex from a usize.
    pub fn new(index: usize) -> Self {
        NodeIndex(index)
    }
    
    /// Returns the underlying usize value.
    pub fn index(&self) -> usize {
        self.0
    }
    
    /// Creates a NodeIndex representing an invalid or end index.
    pub fn end() -> Self {
        NodeIndex(std::usize::MAX)
    }
    
    /// Checks if this index represents an invalid or end index.
    pub fn is_end(&self) -> bool {
        self.0 == std::usize::MAX
    }
}

/// An edge identifier in a graph.
///
/// This newtype wrapper prevents accidentally mixing EdgeIndex with other types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EdgeIndex(pub usize);

impl EdgeIndex {
    /// Creates a new EdgeIndex from a usize.
    pub fn new(index: usize) -> Self {
        EdgeIndex(index)
    }
    
    /// Returns the underlying usize value.
    pub fn index(&self) -> usize {
        self.0
    }
    
    /// Creates an EdgeIndex representing an invalid or end index.
    pub fn end() -> Self {
        EdgeIndex(std::usize::MAX)
    }
    
    /// Checks if this index represents an invalid or end index.
    pub fn is_end(&self) -> bool {
        self.0 == std::usize::MAX
    }
}

/// A trait for basic graph operations.
///
/// This trait defines the fundamental operations that any graph implementation
/// must support. It's inspired by petgraph's GraphTrait but simplified for our needs.
pub trait GraphBase {
    /// The type of node identifiers.
    type NodeId: Clone + PartialEq + Eq + Hash;
    
    /// The type of edge identifiers.
    type EdgeId: Clone + PartialEq + Eq + Hash;
    
    /// Returns the number of nodes in the graph.
    fn node_count(&self) -> usize;
    
    /// Returns the number of edges in the graph.
    fn edge_count(&self) -> usize;
}

/// A trait for accessing graph data.
///
/// This trait provides methods for accessing the data associated with nodes and edges.
pub trait Data: GraphBase {
    /// The type of data stored at each node.
    type NodeWeight;
    
    /// The type of data stored at each edge.
    type EdgeWeight;
    
    /// Returns a reference to the data stored at the given node.
    fn node_weight(&self, node_id: Self::NodeId) -> Option<&Self::NodeWeight>;
    
    /// Returns a reference to the data stored at the given edge.
    fn edge_weight(&self, edge_id: Self::EdgeId) -> Option<&Self::EdgeWeight>;
    
    /// Returns the endpoints of the given edge.
    fn edge_endpoints(&self, edge_id: Self::EdgeId) -> Option<(Self::NodeId, Self::NodeId)>;
}

/// A trait for directed graph operations.
///
/// This trait provides methods specific to directed graphs, such as accessing
/// incoming and outgoing neighbors.
pub trait DirectedGraph: Data {
    /// Returns the outgoing neighbors of a node.
    fn neighbors(&self, node_id: Self::NodeId) -> DbResult<Vec<Self::NodeId>>;
    
    /// Returns the incoming neighbors of a node.
    fn neighbors_directed(&self, node_id: Self::NodeId, direction: Direction) -> DbResult<Vec<Self::NodeId>>;
    
    /// Returns the edges connected to a node.
    fn edges(&self, node_id: Self::NodeId) -> DbResult<Vec<(Self::EdgeId, Self::NodeId, Self::NodeId)>>;
}

/// Direction for graph traversal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Outgoing edges from a node.
    Outgoing,
    
    /// Incoming edges to a node.
    Incoming,
}

/// A trait for undirected graph operations.
///
/// This trait provides methods specific to undirected graphs, where edges
/// have no direction and neighbors can be accessed without specifying direction.
pub trait UndirectedGraph: Data {
    /// Returns the neighbors of a node (undirected).
    fn neighbors(&self, node_id: Self::NodeId) -> DbResult<Vec<Self::NodeId>>;
    
    /// Returns the edges connected to a node.
    fn edges(&self, node_id: Self::NodeId) -> DbResult<Vec<(Self::EdgeId, Self::NodeId, Self::NodeId)>>;
}

/// A trait for mutable graph operations.
///
/// This trait provides methods for modifying the graph structure.
pub trait GraphMut: DirectedGraph {
    /// Adds a node to the graph with the given weight.
    fn add_node(&mut self, weight: Self::NodeWeight) -> DbResult<Self::NodeId>;
    
    /// Removes a node from the graph.
    fn remove_node(&mut self, node_id: Self::NodeId) -> DbResult<Option<Self::NodeWeight>>;
    
    /// Adds an edge to the graph with the given weight.
    fn add_edge(&mut self, from: Self::NodeId, to: Self::NodeId, weight: Self::EdgeWeight) -> DbResult<Self::EdgeId>;
    
    /// Removes an edge from the graph.
    fn remove_edge(&mut self, edge_id: Self::EdgeId) -> DbResult<Option<Self::EdgeWeight>>;
    
    /// Updates the weight of a node.
    fn update_node_weight(&mut self, node_id: Self::NodeId, weight: Self::NodeWeight) -> DbResult<()>;
    
    /// Updates the weight of an edge.
    fn update_edge_weight(&mut self, edge_id: Self::EdgeId, weight: Self::EdgeWeight) -> DbResult<()>;
}

/// An iterator over node identifiers.
pub trait IntoNodeIdentifiers: GraphBase {
    /// Type that yields node identifiers.
    type NodeIdentifiers: Iterator<Item = Self::NodeId>;
    
    /// Creates an iterator over all node identifiers.
    fn node_identifiers(&self) -> Self::NodeIdentifiers;
}

/// An iterator over edge identifiers.
pub trait IntoEdgeIdentifiers: GraphBase {
    /// Type that yields edge identifiers.
    type EdgeIdentifiers: Iterator<Item = Self::EdgeId>;
    
    /// Creates an iterator over all edge identifiers.
    fn edge_identifiers(&self) -> Self::EdgeIdentifiers;
}

/// A trait for accessing edges from a node.
pub trait IntoEdges: GraphBase {
    /// Type that yields edges.
    type Edges: Iterator<Item = (Self::EdgeId, Self::NodeId, Self::NodeId)>;
    
    /// Creates an iterator over all edges from a node.
    fn edges(&self, node_id: Self::NodeId) -> DbResult<Self::Edges>;
}

/// A trait for graph algorithms that need to track visited nodes.
pub trait Visitable: GraphBase {
    /// Type used to track visited nodes.
    type Map: VisitMap<Self::NodeId>;
    
    /// Creates a new visit map for tracking visited nodes.
    fn visit_map(&self) -> Self::Map;
    
    /// Resets the visit map.
    fn reset_map(&self, map: &mut Self::Map);
}

/// A map for tracking visited nodes during graph traversal.
pub trait VisitMap<NodeId> {
    /// Marks a node as visited.
    fn visit(&mut self, node_id: &NodeId) -> bool;
    
    /// Checks if a node has been visited.
    fn is_visited(&self, node_id: &NodeId) -> bool;
}

/// A trait for breadth-first search traversal.
pub trait BfsTraversal: GraphBase {
    /// Performs a breadth-first search starting from the given node.
    fn bfs(&self, start: Self::NodeId) -> DbResult<Vec<Self::NodeId>>;
}

/// A trait for depth-first search traversal.
pub trait DfsTraversal: GraphBase {
    /// Performs a depth-first search starting from the given node.
    fn dfs(&self, start: Self::NodeId) -> DbResult<Vec<Self::NodeId>>;
}

/// A trait for shortest path algorithms.
pub trait ShortestPath: DirectedGraph
where
    Self::EdgeWeight: Clone + Into<f32>,
{
    /// Finds the shortest path between two nodes using Dijkstra's algorithm.
    fn dijkstra(&self, start: Self::NodeId, goal: Self::NodeId) -> DbResult<Option<(Vec<Self::NodeId>, f32)>>;
    
    /// Finds the shortest path using A* algorithm with a heuristic function.
    fn astar<F>(
        &self,
        start: Self::NodeId,
        goal: Self::NodeId,
        heuristic: F,
    ) -> DbResult<Option<(Vec<Self::NodeId>, f32)>>
    where
        F: Fn(Self::NodeId) -> f32;
}

/// A trait for centrality algorithms.
pub trait Centrality: GraphBase {
    /// Calculates the betweenness centrality for all nodes.
    fn betweenness_centrality(&self) -> DbResult<std::collections::HashMap<Self::NodeId, f32>>;
    
    /// Calculates the PageRank for all nodes.
    fn pagerank(&self, damping_factor: f32, max_iterations: usize, tolerance: f32) -> DbResult<std::collections::HashMap<Self::NodeId, f32>>;
}

/// A trait for community detection algorithms.
pub trait CommunityDetection: GraphBase {
    /// Finds strongly connected components using Kosaraju's algorithm.
    fn strongly_connected_components(&self) -> DbResult<Vec<Vec<Self::NodeId>>>;
    
    /// Detects communities using the Louvain algorithm.
    fn louvain_communities(&self, max_iterations: usize) -> DbResult<Vec<Vec<Self::NodeId>>>;
}

/// A trait for minimum spanning tree algorithms.
pub trait MinimumSpanningTree: Data
where
    Self::EdgeWeight: Clone + PartialOrd,
{
    /// Finds the minimum spanning tree using Kruskal's algorithm.
    fn kruskal(&self) -> DbResult<Vec<Self::EdgeId>>;
    
    /// Finds the minimum spanning tree using Prim's algorithm.
    fn prim(&self) -> DbResult<Vec<Self::EdgeId>>;
}

/// A trait for flow algorithms.
pub trait Flow: Data + DirectedGraph
where
    Self::EdgeWeight: Clone + Into<f32>,
{
    /// Calculates the maximum flow between source and sink nodes using Edmonds-Karp algorithm.
    fn max_flow(&self, source: Self::NodeId, sink: Self::NodeId) -> DbResult<f32>;
}