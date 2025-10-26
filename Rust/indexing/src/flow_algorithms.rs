//! Flow algorithms for capacity-constrained networks.
//!
//! This module provides implementations of various flow algorithms
//! commonly used in network optimization, including Edmonds-Karp and
//! Dinic's algorithm for maximum flow computation. These implementations
//! follow the Rust Architecture Guidelines for safety, performance, and clarity.

use crate::graph_traits::{GraphBase, DirectedGraph, IntoNodeIdentifiers, IntoEdgeIdentifiers, Data, Direction};
use common::DbResult;
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;

/// Implementation of Edmonds-Karp algorithm for maximum flow.
///
/// The Edmonds-Karp algorithm is an implementation of the Ford-Fulkerson
/// method for computing the maximum flow in a flow network. It uses BFS
/// to find augmenting paths, which ensures polynomial time complexity.
pub fn edmonds_karp<G, F>(
    graph: &G,
    source: G::NodeId,
    sink: G::NodeId,
    capacity_fn: F,
) -> DbResult<f32>
where
    G: DirectedGraph + IntoNodeIdentifiers,
    G::NodeId: Clone + Eq + std::hash::Hash,
    G::EdgeId: Clone + Eq + std::hash::Hash,
    F: Fn(G::EdgeId) -> f32,
{
    // Create residual graph representation
    let mut residual_capacity: HashMap<(G::NodeId, G::NodeId), f32> = HashMap::new();
    
    // Initialize residual capacities using the edges method from DirectedGraph trait
    for node_id in graph.node_identifiers() {
        if let Ok(edges) = graph.edges(node_id.clone()) {
            for (edge_id, from, to) in edges {
                let capacity = capacity_fn(edge_id);
                residual_capacity.insert((from.clone(), to.clone()), capacity);
                // Add reverse edge with 0 capacity
                residual_capacity.entry((to, from)).or_insert(0.0);
            }
        }
    }
    
    let mut max_flow = 0.0;
    
    // Main loop: find augmenting paths until no more exist
    loop {
        // Use BFS to find an augmenting path
        let path = bfs_find_path(graph, &source, &sink, &residual_capacity)?;
        
        if path.is_empty() {
            // No more augmenting paths
            break;
        }
        
        // Find the minimum residual capacity along the path
        let mut min_capacity = f32::INFINITY;
        for window in path.windows(2) {
            let from = &window[0];
            let to = &window[1];
            let capacity = *residual_capacity.get(&(from.clone(), to.clone())).unwrap_or(&0.0);
            min_capacity = min_capacity.min(capacity);
        }
        
        // Update residual capacities
        for window in path.windows(2) {
            let from = window[0].clone();
            let to = window[1].clone();
            
            // Decrease forward edge capacity
            if let Some(cap) = residual_capacity.get_mut(&(from.clone(), to.clone())) {
                *cap -= min_capacity;
            }
            
            // Increase backward edge capacity
            if let Some(cap) = residual_capacity.get_mut(&(to.clone(), from.clone())) {
                *cap += min_capacity;
            }
        }
        
        max_flow += min_capacity;
    }
    
    Ok(max_flow)
}

/// Helper function to find an augmenting path using BFS.
fn bfs_find_path<G>(
    graph: &G,
    source: &G::NodeId,
    sink: &G::NodeId,
    residual_capacity: &HashMap<(G::NodeId, G::NodeId), f32>,
) -> DbResult<Vec<G::NodeId>>
where
    G: DirectedGraph,
    G::NodeId: Clone + Eq + Hash,
{
    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();
    let mut parent: HashMap<G::NodeId, G::NodeId> = HashMap::new();
    
    queue.push_back(source.clone());
    visited.insert(source.clone());
    
    while let Some(current) = queue.pop_front() {
        if current == *sink {
            // Found a path to sink, reconstruct it
            let mut path = Vec::new();
            let mut node = sink.clone();
            
            while node != *source {
                path.push(node.clone());
                node = parent.get(&node).unwrap().clone();
            }
            path.push(source.clone());
            path.reverse();
            
            return Ok(path);
        }
        
        // Explore neighbors with positive residual capacity
        if let Ok(neighbors) = graph.neighbors(current.clone()) {
            for neighbor in neighbors {
                if !visited.contains(&neighbor) {
                    let capacity = *residual_capacity.get(&(current.clone(), neighbor.clone())).unwrap_or(&0.0);
                    if capacity > 0.0 {
                        visited.insert(neighbor.clone());
                        parent.insert(neighbor.clone(), current.clone());
                        queue.push_back(neighbor);
                    }
                }
            }
        }
    }
    
    // No path found
    Ok(Vec::new())
}

/// Implementation of Dinic's algorithm for maximum flow.
///
/// Dinic's algorithm is a strongly polynomial algorithm for computing
/// the maximum flow in a flow network. It uses the concepts of level
/// graphs and blocking flows to achieve better performance than Edmonds-Karp
/// in practice.
pub fn dinic<G, F>(
    graph: &G,
    source: G::NodeId,
    sink: G::NodeId,
    capacity_fn: F,
) -> DbResult<f32>
where
    G: DirectedGraph + IntoNodeIdentifiers,
    G::NodeId: Clone + Eq + Hash,
    G::EdgeId: Clone + Eq + Hash,
    F: Fn(G::EdgeId) -> f32,
{
    // Create residual graph representation
    let mut residual_capacity: HashMap<(G::NodeId, G::NodeId), f32> = HashMap::new();
    
    // Initialize residual capacities
    for node_id in graph.node_identifiers() {
        if let Ok(edges) = graph.edges(node_id.clone()) {
            for (edge_id, from, to) in edges {
                let capacity = capacity_fn(edge_id);
                residual_capacity.insert((from.clone(), to.clone()), capacity);
                // Add reverse edge with 0 capacity
                residual_capacity.entry((to, from)).or_insert(0.0);
            }
        }
    }
    
    let mut max_flow = 0.0;
    
    // Main loop: build level graph and find blocking flow
    loop {
        // Build level graph using BFS
        let level = build_level_graph(graph, &source, &sink, &residual_capacity)?;
        
        if !level.contains_key(&sink) {
            // Sink is not reachable, we're done
            break;
        }
        
        // Find blocking flow in the level graph
        let blocking_flow = find_blocking_flow(
            graph,
            &source,
            &sink,
            &mut residual_capacity,
            &level,
        )?;
        
        if blocking_flow == 0.0 {
            // No more blocking flow can be found
            break;
        }
        
        max_flow += blocking_flow;
    }
    
    Ok(max_flow)
}

/// Helper function to build a level graph using BFS.
fn build_level_graph<G>(
    graph: &G,
    source: &G::NodeId,
    sink: &G::NodeId,
    residual_capacity: &HashMap<(G::NodeId, G::NodeId), f32>,
) -> DbResult<HashMap<G::NodeId, usize>>
where
    G: DirectedGraph,
    G::NodeId: Clone + Eq + Hash,
{
    let mut level: HashMap<G::NodeId, usize> = HashMap::new();
    let mut queue = VecDeque::new();
    
    level.insert(source.clone(), 0);
    queue.push_back(source.clone());
    
    while let Some(current) = queue.pop_front() {
        let current_level = *level.get(&current).unwrap();
        
        // Explore neighbors with positive residual capacity
        if let Ok(neighbors) = graph.neighbors(current.clone()) {
            for neighbor in neighbors {
                if !level.contains_key(&neighbor) {
                    let capacity = *residual_capacity.get(&(current.clone(), neighbor.clone())).unwrap_or(&0.0);
                    if capacity > 0.0 {
                        level.insert(neighbor.clone(), current_level + 1);
                        queue.push_back(neighbor);
                    }
                }
            }
        }
    }
    
    Ok(level)
}

/// Helper function to find a blocking flow in the level graph.
fn find_blocking_flow<G>(
    graph: &G,
    source: &G::NodeId,
    sink: &G::NodeId,
    residual_capacity: &mut HashMap<(G::NodeId, G::NodeId), f32>,
    level: &HashMap<G::NodeId, usize>,
) -> DbResult<f32>
where
    G: DirectedGraph,
    G::NodeId: Clone + Eq + Hash,
{
    let mut total_flow = 0.0;
    let mut visited: HashSet<G::NodeId> = HashSet::new();
    
    // DFS to find paths in the level graph
    while let Some(flow) = dfs_blocking_flow(
        graph,
        source,
        sink,
        residual_capacity,
        level,
        &mut visited,
        f32::INFINITY,
    )? {
        if flow > 0.0 {
            total_flow += flow;
        } else {
            break;
        }
    }
    
    Ok(total_flow)
}

/// Helper function for DFS in blocking flow computation.
fn dfs_blocking_flow<G>(
    graph: &G,
    current: &G::NodeId,
    sink: &G::NodeId,
    residual_capacity: &mut HashMap<(G::NodeId, G::NodeId), f32>,
    level: &HashMap<G::NodeId, usize>,
    visited: &mut HashSet<G::NodeId>,
    min_capacity: f32,
) -> DbResult<Option<f32>>
where
    G: DirectedGraph,
    G::NodeId: Clone + Eq + Hash,
{
    if *current == *sink {
        return Ok(Some(min_capacity));
    }
    
    if visited.contains(current) {
        return Ok(None);
    }
    
    visited.insert(current.clone());
    
    let current_level = *level.get(current).unwrap_or(&0);
    
    // Explore neighbors in the level graph
    if let Ok(neighbors) = graph.neighbors(current.clone()) {
        for neighbor in neighbors {
            let neighbor_level = *level.get(&neighbor).unwrap_or(&0);
            
            // Only consider edges that go to the next level
            if neighbor_level == current_level + 1 {
                let capacity = *residual_capacity.get(&(current.clone(), neighbor.clone())).unwrap_or(&0.0);
                if capacity > 0.0 {
                    let bottleneck = min_capacity.min(capacity);
                    if let Some(flow) = dfs_blocking_flow(
                        graph,
                        &neighbor,
                        sink,
                        residual_capacity,
                        level,
                        visited,
                        bottleneck,
                    )? {
                        if flow > 0.0 {
                            // Update residual capacities
                            *residual_capacity.get_mut(&(current.clone(), neighbor.clone())).unwrap() -= flow;
                            *residual_capacity.get_mut(&(neighbor.clone(), current.clone())).unwrap() += flow;
                            return Ok(Some(flow));
                        }
                    }
                }
            }
        }
    }
    
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph_traits::{GraphBase, DirectedGraph};
    use std::collections::{HashMap, HashSet};
    
    // Simple graph implementation for testing
    struct TestGraph {
        nodes: HashSet<String>,
        edges: HashMap<String, (String, String, f32)>, // edge_id -> (from, to, capacity)
        outgoing_edges: HashMap<String, Vec<String>>, // node -> edge_ids
    }
    
    impl TestGraph {
        fn new() -> Self {
            Self {
                nodes: HashSet::new(),
                edges: HashMap::new(),
                outgoing_edges: HashMap::new(),
            }
        }
        
        fn add_node(&mut self, node: &str) {
            self.nodes.insert(node.to_string());
        }
        
        fn add_edge(&mut self, edge_id: &str, from: &str, to: &str, capacity: f32) {
            self.nodes.insert(from.to_string());
            self.nodes.insert(to.to_string());
            self.edges.insert(edge_id.to_string(), (from.to_string(), to.to_string(), capacity));
            self.outgoing_edges.entry(from.to_string()).or_insert_with(Vec::new).push(edge_id.to_string());
        }
    }
    
    impl GraphBase for TestGraph {
        type NodeId = String;
        type EdgeId = String;
        
        fn node_count(&self) -> usize {
            self.nodes.len()
        }
        
        fn edge_count(&self) -> usize {
            self.edges.len()
        }
    }
    
    impl IntoNodeIdentifiers for TestGraph {
        type NodeIdentifiers = std::vec::IntoIter<Self::NodeId>;
        
        fn node_identifiers(&self) -> Self::NodeIdentifiers {
            self.nodes.iter().cloned().collect::<Vec<_>>().into_iter()
        }
    }
    
    impl IntoEdgeIdentifiers for TestGraph {
        type EdgeIdentifiers = std::vec::IntoIter<Self::EdgeId>;
        
        fn edge_identifiers(&self) -> Self::EdgeIdentifiers {
            self.edges.keys().cloned().collect::<Vec<_>>().into_iter()
        }
    }
    
    impl Data for TestGraph {
        type NodeWeight = String;
        type EdgeWeight = f32;
        
        fn node_weight(&self, node_id: Self::NodeId) -> Option<&Self::NodeWeight> {
            self.nodes.get(&node_id)
        }
        
        fn edge_weight(&self, edge_id: Self::EdgeId) -> Option<&Self::EdgeWeight> {
            self.edges.get(&edge_id).map(|(_, _, weight)| weight)
        }
        
        fn edge_endpoints(&self, edge_id: Self::EdgeId) -> Option<(Self::NodeId, Self::NodeId)> {
            self.edges.get(&edge_id).map(|(from, to, _)| (from.clone(), to.clone()))
        }
    }
    
    impl DirectedGraph for TestGraph {
        fn neighbors(&self, node: Self::NodeId) -> DbResult<Vec<Self::NodeId>> {
            let mut neighbors = Vec::new();
            if let Some(edge_ids) = self.outgoing_edges.get(&node) {
                for edge_id in edge_ids {
                    if let Some((_, to, _)) = self.edges.get(edge_id) {
                        neighbors.push(to.clone());
                    }
                }
            }
            Ok(neighbors)
        }
        
        fn neighbors_directed(&self, node: Self::NodeId, _direction: Direction) -> DbResult<Vec<Self::NodeId>> {
            self.neighbors(node)
        }
        
        fn edges(&self, node: Self::NodeId) -> DbResult<Vec<(Self::EdgeId, Self::NodeId, Self::NodeId)>> {
            let mut result = Vec::new();
            if let Some(edge_ids) = self.outgoing_edges.get(&node) {
                for edge_id in edge_ids {
                    if let Some((from, to, _)) = self.edges.get(edge_id) {
                        result.push((edge_id.clone(), from.clone(), to.clone()));
                    }
                }
            }
            Ok(result)
        }
    }
    
    #[test]
    fn test_edmonds_karp_simple() {
        let mut graph = TestGraph::new();
        
        // Create a simple flow network
        graph.add_node("s"); // source
        graph.add_node("t"); // sink
        graph.add_node("a");
        graph.add_node("b");
        
        // Add edges with capacities
        graph.add_edge("e1", "s", "a", 10.0);
        graph.add_edge("e2", "s", "b", 10.0);
        graph.add_edge("e3", "a", "b", 2.0);
        graph.add_edge("e4", "a", "t", 4.0);
        graph.add_edge("e5", "b", "t", 10.0);
        
        let capacity_fn = |edge_id: String| -> f32 {
            if let Some((_, _, capacity)) = graph.edges.get(&edge_id) {
                *capacity
            } else {
                0.0
            }
        };
        
        let max_flow = edmonds_karp(&graph, "s".to_string(), "t".to_string(), capacity_fn).unwrap();
        assert_eq!(max_flow, 14.0); // Expected maximum flow
    }
    
    #[test]
    fn test_dinic_simple() {
        let mut graph = TestGraph::new();
        
        // Create a simple flow network
        graph.add_node("s"); // source
        graph.add_node("t"); // sink
        graph.add_node("a");
        graph.add_node("b");
        
        // Add edges with capacities
        graph.add_edge("e1", "s", "a", 10.0);
        graph.add_edge("e2", "s", "b", 10.0);
        graph.add_edge("e3", "a", "b", 2.0);
        graph.add_edge("e4", "a", "t", 4.0);
        graph.add_edge("e5", "b", "t", 10.0);
        
        let capacity_fn = |edge_id: String| -> f32 {
            if let Some((_, _, capacity)) = graph.edges.get(&edge_id) {
                *capacity
            } else {
                0.0
            }
        };
        
        let max_flow = dinic(&graph, "s".to_string(), "t".to_string(), capacity_fn).unwrap();
        assert_eq!(max_flow, 14.0); // Expected maximum flow
    }
}