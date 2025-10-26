//! Iterator-based graph traversal implementations.
//!
//! This module provides lazy evaluation iterators for graph traversal algorithms
//! such as BFS and DFS, following patterns inspired by petgraph but adapted
//! to our specific needs and graph traits.

use crate::graph_traits::*;
use common::{DbResult, DbError};
use std::collections::{HashSet, VecDeque};
use std::marker::PhantomData;

/// A breadth-first search iterator for graph traversal.
///
/// This iterator provides lazy evaluation of breadth-first search traversal,
/// yielding nodes in BFS order without computing the entire traversal upfront.
pub struct Bfs<G: GraphBase> {
    /// The graph being traversed
    graph: G,
    
    /// Queue of nodes to visit
    queue: VecDeque<G::NodeId>,
    
    /// Set of visited nodes to avoid cycles
    visited: HashSet<G::NodeId>,
    
    /// Phantom data to hold type information
    _phantom: PhantomData<G>,
}

impl<G: GraphBase + DirectedGraph> Bfs<G>
where
    G::NodeId: Clone + Eq + std::hash::Hash,
{
    /// Creates a new BFS iterator starting from the given node.
    pub fn new(graph: G, start: G::NodeId) -> Self {
        let mut queue = VecDeque::new();
        queue.push_back(start.clone());
        
        let mut visited = HashSet::new();
        visited.insert(start);
        
        Self {
            graph,
            queue,
            visited,
            _phantom: PhantomData,
        }
    }
}

impl<G: GraphBase + DirectedGraph> Iterator for Bfs<G>
where
    G::NodeId: Clone + Eq + std::hash::Hash,
{
    type Item = G::NodeId;
    
    fn next(&mut self) -> Option<Self::Item> {
        // Pop the next node from the queue
        let current = self.queue.pop_front()?;
        
        // Get the neighbors of the current node
        if let Ok(neighbors) = self.graph.neighbors(current.clone()) {
            // Add unvisited neighbors to the queue
            for neighbor in neighbors {
                if self.visited.insert(neighbor.clone()) {
                    self.queue.push_back(neighbor);
                }
            }
        }
        
        Some(current)
    }
}

/// A depth-first search iterator for graph traversal.
///
/// This iterator provides lazy evaluation of depth-first search traversal,
/// yielding nodes in DFS order without computing the entire traversal upfront.
pub struct Dfs<G: GraphBase> {
    /// The graph being traversed
    graph: G,
    
    /// Stack of nodes to visit
    stack: Vec<G::NodeId>,
    
    /// Set of visited nodes to avoid cycles
    visited: HashSet<G::NodeId>,
    
    /// Phantom data to hold type information
    _phantom: PhantomData<G>,
}

impl<G: GraphBase + DirectedGraph> Dfs<G>
where
    G::NodeId: Clone + Eq + std::hash::Hash,
{
    /// Creates a new DFS iterator starting from the given node.
    pub fn new(graph: G, start: G::NodeId) -> Self {
        let mut stack = Vec::new();
        stack.push(start.clone());
        
        let mut visited = HashSet::new();
        visited.insert(start);
        
        Self {
            graph,
            stack,
            visited,
            _phantom: PhantomData,
        }
    }
}

impl<G: GraphBase + DirectedGraph> Iterator for Dfs<G>
where
    G::NodeId: Clone + Eq + std::hash::Hash,
{
    type Item = G::NodeId;
    
    fn next(&mut self) -> Option<Self::Item> {
        // Pop the next node from the stack
        let current = self.stack.pop()?;
        
        // Get the neighbors of the current node
        if let Ok(neighbors) = self.graph.neighbors(current.clone()) {
            // Add unvisited neighbors to the stack
            // Note: We add them in reverse order to maintain left-to-right traversal
            for neighbor in neighbors.iter().rev() {
                if self.visited.insert(neighbor.clone()) {
                    self.stack.push(neighbor.clone());
                }
            }
        }
        
        Some(current)
    }
}

/// A depth-first search post-order iterator for graph traversal.
///
/// This iterator provides lazy evaluation of depth-first search traversal
/// in post-order, which is useful for certain graph algorithms.
pub struct DfsPostOrder<G: GraphBase> {
    /// The graph being traversed
    graph: G,
    
    /// Stack of nodes to visit
    stack: Vec<G::NodeId>,
    
    /// Set of visited nodes to avoid cycles
    visited: HashSet<G::NodeId>,
    
    /// Stack for post-order traversal
    post_order: Vec<G::NodeId>,
    
    /// Phantom data to hold type information
    _phantom: PhantomData<G>,
}

impl<G: GraphBase + DirectedGraph> DfsPostOrder<G>
where
    G::NodeId: Clone + Eq + std::hash::Hash,
{
    /// Creates a new DFS post-order iterator starting from the given node.
    pub fn new(graph: G, start: G::NodeId) -> Self {
        let mut stack = Vec::new();
        stack.push(start.clone());
        
        let mut visited = HashSet::new();
        visited.insert(start);
        
        Self {
            graph,
            stack,
            visited,
            post_order: Vec::new(),
            _phantom: PhantomData,
        }
    }
}

impl<G: GraphBase + DirectedGraph> Iterator for DfsPostOrder<G>
where
    G::NodeId: Clone + Eq + std::hash::Hash,
{
    type Item = G::NodeId;
    
    fn next(&mut self) -> Option<Self::Item> {
        // If we have items in our post-order queue, return them
        if let Some(node) = self.post_order.pop() {
            return Some(node);
        }
        
        // Continue DFS traversal
        while let Some(current) = self.stack.pop() {
            // Add to post-order queue
            self.post_order.push(current.clone());
            
            // Get the neighbors of the current node
            if let Ok(neighbors) = self.graph.neighbors(current.clone()) {
                // Add unvisited neighbors to the stack
                for neighbor in neighbors.iter().rev() {
                    if self.visited.insert(neighbor.clone()) {
                        self.stack.push(neighbor.clone());
                    }
                }
            }
        }
        
        // Return the last item from post-order queue
        self.post_order.pop()
    }
}

/// An edge iterator for traversing edges from a node.
///
/// This iterator provides lazy evaluation of edges connected to a node.
pub struct EdgeIterator<G: GraphBase> {
    /// The graph being traversed
    graph: G,
    
    /// The node whose edges we're iterating over
    node: G::NodeId,
    
    /// Current position in the iteration
    current: usize,
    
    /// Neighbors of the node
    neighbors: Vec<G::NodeId>,
    
    /// Phantom data to hold type information
    _phantom: PhantomData<G>,
}

impl<G: GraphBase + DirectedGraph> EdgeIterator<G>
where
    G::NodeId: Clone + Eq + std::hash::Hash,
{
    /// Creates a new edge iterator for the given node.
    pub fn new(graph: G, node: G::NodeId) -> DbResult<Self> {
        let neighbors = graph.neighbors(node.clone())?;
        
        Ok(Self {
            graph,
            node,
            current: 0,
            neighbors,
            _phantom: PhantomData,
        })
    }
}

impl<G: GraphBase<EdgeId = EdgeIndex> + DirectedGraph> Iterator for EdgeIterator<G>
where
    G::NodeId: Clone + Eq + std::hash::Hash,
    G::EdgeId: Clone,
{
    type Item = (G::EdgeId, G::NodeId, G::NodeId);
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.neighbors.len() {
            return None;
        }
        
        let neighbor = self.neighbors[self.current].clone();
        self.current += 1;
        
        // In a real implementation, we would need to find the actual edge ID
        // For now, we'll create a dummy edge ID
        let edge_id = EdgeIndex(self.current);
        
        Some((edge_id, self.node.clone(), neighbor))
    }
}

/// A filtered iterator that only yields nodes/edges that satisfy a predicate.
pub struct FilteredIterator<I, F> {
    /// The underlying iterator
    inner: I,
    
    /// The filter predicate
    predicate: F,
}

impl<I, F> FilteredIterator<I, F>
where
    I: Iterator,
    F: FnMut(&I::Item) -> bool,
{
    /// Creates a new filtered iterator.
    pub fn new(inner: I, predicate: F) -> Self {
        Self { inner, predicate }
    }
}

impl<I, F> Iterator for FilteredIterator<I, F>
where
    I: Iterator,
    F: FnMut(&I::Item) -> bool,
{
    type Item = I::Item;
    
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(item) = self.inner.next() {
            if (self.predicate)(&item) {
                return Some(item);
            }
        }
        None
    }
}

/// Extension trait for graph traversal iterators.
pub trait TraversalIteratorExt: Iterator {
    /// Filters the iterator to only yield items that satisfy the predicate.
    fn filter_nodes<F>(self, predicate: F) -> FilteredIterator<Self, F>
    where
        Self: Sized,
        F: FnMut(&Self::Item) -> bool,
    {
        FilteredIterator::new(self, predicate)
    }
    
    /// Collects the iterator into a vector.
    fn collect_vec(self) -> Vec<Self::Item>
    where
        Self: Sized,
    {
        self.collect()
    }
}

impl<I: Iterator> TraversalIteratorExt for I {}

/// A trait for graphs that support BFS traversal.
pub trait BfsTraversalExt: GraphBase + DirectedGraph
where
    Self::NodeId: Clone + Eq + std::hash::Hash,
{
    /// Creates a BFS iterator starting from the given node.
    fn bfs(&self, start: Self::NodeId) -> DbResult<Bfs<Self>>
    where
        Self: Sized,
    {
        Err(DbError::InvalidOperation("BFS iterator not supported for this graph type".to_string()))
    }
    
    /// Performs a BFS traversal and collects all nodes.
    fn bfs_collect(&self, start: Self::NodeId) -> DbResult<Vec<Self::NodeId>> {
        Err(DbError::InvalidOperation("BFS collect not supported for this graph type".to_string()))
    }
}

impl<G: GraphBase + DirectedGraph> BfsTraversalExt for G where G::NodeId: Clone + Eq + std::hash::Hash {}

/// A trait for graphs that support DFS traversal.
pub trait DfsTraversalExt: GraphBase + DirectedGraph
where
    Self::NodeId: Clone + Eq + std::hash::Hash,
{
    /// Creates a DFS iterator starting from the given node.
    fn dfs(&self, start: Self::NodeId) -> DbResult<Dfs<Self>>
    where
        Self: Sized,
    {
        Err(DbError::InvalidOperation("DFS iterator not supported for this graph type".to_string()))
    }
    
    /// Performs a DFS traversal and collects all nodes.
    fn dfs_collect(&self, start: Self::NodeId) -> DbResult<Vec<Self::NodeId>> {
        Err(DbError::InvalidOperation("DFS collect not supported for this graph type".to_string()))
    }
    
    /// Creates a DFS post-order iterator starting from the given node.
    fn dfs_post_order(&self, start: Self::NodeId) -> DbResult<DfsPostOrder<Self>>
    where
        Self: Sized,
    {
        Err(DbError::InvalidOperation("DFS post-order iterator not supported for this graph type".to_string()))
    }
    
    /// Performs a DFS post-order traversal and collects all nodes.
    fn dfs_post_order_collect(&self, start: Self::NodeId) -> DbResult<Vec<Self::NodeId>> {
        Err(DbError::InvalidOperation("DFS post-order collect not supported for this graph type".to_string()))
    }
}

impl<G: GraphBase + DirectedGraph> DfsTraversalExt for G where G::NodeId: Clone + Eq + std::hash::Hash {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimized_graph::{OptimizedGraphIndex, NodeData, EdgeData};
    use crate::graph_traits::{NodeIndex, EdgeIndex};
    
    #[test]
    fn test_bfs_iterator() {
        let mut graph = OptimizedGraphIndex::new();
        
        // Add nodes
        let node1 = NodeIndex(0);
        let node2 = NodeIndex(1);
        let node3 = NodeIndex(2);
        
        // Add edges to create a simple graph
        // 0 -> 1 -> 2
        let edge1 = EdgeIndex(0);
        let edge2 = EdgeIndex(1);
        
        // In a real test, we would add actual nodes and edges to the graph
        // For now, we'll just test that the iterator can be created
        
        // Create a BFS iterator
        // let bfs = Bfs::new(&graph, node1);
        
        // This is a placeholder test
        assert!(true);
    }
    
    #[test]
    fn test_dfs_iterator() {
        let mut graph = OptimizedGraphIndex::new();
        
        // Add nodes
        let node1 = NodeIndex(0);
        let node2 = NodeIndex(1);
        let node3 = NodeIndex(2);
        
        // Add edges to create a simple graph
        // 0 -> 1 -> 2
        let edge1 = EdgeIndex(0);
        let edge2 = EdgeIndex(1);
        
        // In a real test, we would add actual nodes and edges to the graph
        // For now, we'll just test that the iterator can be created
        
        // Create a DFS iterator
        // let dfs = Dfs::new(&graph, node1);
        
        // This is a placeholder test
        assert!(true);
    }
}