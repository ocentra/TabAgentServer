//! Flow algorithms for capacity-constrained networks.
//!
//! This module provides implementations of various flow algorithms
//! commonly used in network optimization, including Edmonds-Karp and
//! Dinic's algorithm for maximum flow computation. These implementations
//! follow the Rust Architecture Guidelines for safety, performance, and clarity.

use crate::algorithms::graph_traits::{GraphBase, DirectedGraph, IntoNodeIdentifiers, IntoEdgeIdentifiers, Data, Direction};
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
