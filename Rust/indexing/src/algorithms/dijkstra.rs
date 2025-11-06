//! Dijkstra's shortest path algorithm with zero-copy MDBX access.
//!
//! # Example
//! ```ignore
//! use indexing::algorithms::dijkstra_zero_copy;
//!
//! // Edge cost function receives &str from mmap - zero allocations!
//! let edge_cost = |_edge_id: &str, _target: &str| 1.0f64;
//!
//! // Compute distances from "start" to all reachable nodes
//! let distances = dijkstra_zero_copy(&graph, "start", None, edge_cost)?;
//! ```

use std::collections::BinaryHeap;
use std::collections::HashSet;
use hashbrown::HashMap;

use crate::algo::Measure;
use crate::scored::MinScored;
use crate::core::graph::GraphIndex;
use common::DbResult;

/// Result of Dijkstra algorithm.
#[derive(Debug, Clone)]
pub struct AlgoResult<N, K> {
    /// Maps node ID to shortest path cost.
    pub scores: HashMap<N, K>,
    /// Goal node that terminated the search, if any.
    pub goal_node: Option<N>,
}

/// Dijkstra's shortest path algorithm with zero-copy MDBX access.
///
/// Fetches edges on-demand per node via short-lived transaction guards.
/// Edge IDs and target nodes are borrowed from mmap - no allocations during traversal.
///
/// # Arguments
/// * `graph`: GraphIndex for zero-copy edge access
/// * `start`: start node ID
/// * `goal`: optional goal node ID
/// * `edge_cost`: closure that returns cost for an edge (edge_id, target_node_id)
pub fn dijkstra_zero_copy<F, K>(
    graph: &GraphIndex,
    start: &str,
    goal: Option<&str>,
    edge_cost: F,
) -> DbResult<HashMap<String, K>>
where
    F: FnMut(&str, &str) -> K,
    K: Measure + Copy,
{
    with_dynamic_goal_zero_copy(graph, start, |node| goal == Some(node), edge_cost)
        .map(|result| result.scores)
}

/// Dijkstra's algorithm with dynamic goal function.
pub fn with_dynamic_goal_zero_copy<GoalFn, CostFn, K>(
    graph: &GraphIndex,
    start: &str,
    mut goal_fn: GoalFn,
    mut edge_cost: CostFn,
) -> DbResult<AlgoResult<String, K>>
where
    GoalFn: FnMut(&str) -> bool,
    CostFn: FnMut(&str, &str) -> K,
    K: Measure + Copy,
{
    let mut visited = HashSet::new();
    let mut scores = HashMap::new();
    let mut visit_next = BinaryHeap::new();
    let zero_score = K::default();
    
    scores.insert(start.to_string(), zero_score);
    visit_next.push(MinScored(zero_score, start.to_string()));
    let mut goal_node = None;
    
    while let Some(MinScored(node_score, node)) = visit_next.pop() {
        if visited.contains(&node) {
            continue;
        }
        
        if goal_fn(&node) {
            goal_node = Some(node.clone());
            break;
        }
        
        visited.insert(node.clone());
        
        if let Some(guard) = graph.get_outgoing(&node)? {
            for (edge_id_str, target_str) in guard.iter_edges() {
                if visited.contains(target_str) {
                    continue;
                }
                
                let next_score = node_score + edge_cost(edge_id_str, target_str);
                
                match scores.entry(target_str.to_string()) {
                    hashbrown::hash_map::Entry::Occupied(mut ent) => {
                        if next_score < *ent.get() {
                            *ent.get_mut() = next_score;
                            visit_next.push(MinScored(next_score, target_str.to_string()));
                        }
                    }
                    hashbrown::hash_map::Entry::Vacant(ent) => {
                        ent.insert(next_score);
                        visit_next.push(MinScored(next_score, target_str.to_string()));
                    }
                }
            }
        }
    }
    
    Ok(AlgoResult { scores, goal_node })
}

/// Bidirectional Dijkstra searching from both start and goal.
pub fn bidirectional_dijkstra_zero_copy<F, K>(
    graph: &GraphIndex,
    start: &str,
    goal: &str,
    mut edge_cost: F,
) -> DbResult<Option<K>>
where
    F: FnMut(&str, &str) -> K,
    K: Measure + Copy,
{
    let mut forward_visited = HashSet::new();
    let mut forward_distance = HashMap::new();
    forward_distance.insert(start.to_string(), K::default());
    
    let mut backward_visited = HashSet::new();
    let mut backward_distance = HashMap::new();
    backward_distance.insert(goal.to_string(), K::default());
    
    let mut forward_heap = BinaryHeap::new();
    let mut backward_heap = BinaryHeap::new();
    
    forward_heap.push(MinScored(K::default(), start.to_string()));
    backward_heap.push(MinScored(K::default(), goal.to_string()));
    
    let mut best_value = None;
    
    while !forward_heap.is_empty() && !backward_heap.is_empty() {
        let MinScored(_, u) = forward_heap.pop().unwrap();
        let MinScored(_, v) = backward_heap.pop().unwrap();
        
        forward_visited.insert(u.clone());
        backward_visited.insert(v.clone());
        
        let distance_to_u = forward_distance[&u];
        let distance_to_v = backward_distance[&v];
        
        if let Some(guard) = graph.get_outgoing(&u)? {
            for (edge_id_str, target_str) in guard.iter_edges() {
                let current_edge_cost = edge_cost(edge_id_str, target_str);
                
                if !forward_visited.contains(target_str) {
                    let next_score = distance_to_u + current_edge_cost;
                    
                    match forward_distance.entry(target_str.to_string()) {
                        hashbrown::hash_map::Entry::Occupied(mut ent) => {
                            if next_score < *ent.get() {
                                *ent.get_mut() = next_score;
                                forward_heap.push(MinScored(next_score, target_str.to_string()));
                            }
                        }
                        hashbrown::hash_map::Entry::Vacant(ent) => {
                            ent.insert(next_score);
                            forward_heap.push(MinScored(next_score, target_str.to_string()));
                        }
                    }
                }
                
                if backward_visited.contains(target_str) {
                    let potential_best = distance_to_u + current_edge_cost 
                        + backward_distance.get(target_str).copied().unwrap_or(K::default());
                    best_value = match best_value {
                        None => Some(potential_best),
                        Some(best) if potential_best < best => Some(potential_best),
                        _ => best_value,
                    };
                }
            }
        }
        
        if let Some(guard) = graph.get_incoming(&v)? {
            for (edge_id_str, source_str) in guard.iter_edges() {
                let current_edge_cost = edge_cost(edge_id_str, &v);
                
                if !backward_visited.contains(source_str) {
                    let next_score = distance_to_v + current_edge_cost;
                    
                    match backward_distance.entry(source_str.to_string()) {
                        hashbrown::hash_map::Entry::Occupied(mut ent) => {
                            if next_score < *ent.get() {
                                *ent.get_mut() = next_score;
                                backward_heap.push(MinScored(next_score, source_str.to_string()));
                            }
                        }
                        hashbrown::hash_map::Entry::Vacant(ent) => {
                            ent.insert(next_score);
                            backward_heap.push(MinScored(next_score, source_str.to_string()));
                        }
                    }
                }
                
                if forward_visited.contains(source_str) {
                    let potential_best = distance_to_v + current_edge_cost 
                        + forward_distance.get(source_str).copied().unwrap_or(K::default());
                    best_value = match best_value {
                        None => Some(potential_best),
                        Some(best) if potential_best < best => Some(potential_best),
                        _ => best_value,
                    };
                }
            }
        }
        
        if let Some(best) = best_value {
            if distance_to_u + distance_to_v >= best {
                return Ok(Some(best));
            }
        }
    }
    
    Ok(None)
}
