use std::collections::BinaryHeap;
use hashbrown::HashMap;

use crate::algo::Measure;
use crate::scored::MinScored;
use crate::core::graph::GraphIndex;
use common::DbResult;

/// A* shortest path algorithm with zero-copy MDBX access.
///
/// # Arguments
/// * `graph`: GraphIndex for zero-copy edge access
/// * `start`: start node ID
/// * `is_goal`: callback that returns true if node is the goal
/// * `edge_cost`: closure returning cost for an edge (edge_id, target_node_id)
/// * `estimate_cost`: heuristic estimating cost to goal from a node
///
/// # Returns
/// * `Some((cost, path))`: total cost and path from start to goal
/// * `None`: no path found
pub fn astar_zero_copy<F, H, K, IsGoal>(
    graph: &GraphIndex,
    start: &str,
    mut is_goal: IsGoal,
    mut edge_cost: F,
    mut estimate_cost: H,
) -> DbResult<Option<(K, Vec<String>)>>
where
    IsGoal: FnMut(&str) -> bool,
    F: FnMut(&str, &str) -> K,
    H: FnMut(&str) -> K,
    K: Measure + Copy,
{
    let mut visit_next = BinaryHeap::new();
    let mut scores = HashMap::new();
    let mut came_from = HashMap::new();

    let zero: K = K::default();
    let g: K = zero;
    let h: K = estimate_cost(start);
    let f: K = g + h;
    scores.insert(start.to_string(), (f, h, g));
    visit_next.push(MinScored((f, h, g), start.to_string()));

    while let Some(MinScored((f, h, g), node)) = visit_next.pop() {
        if is_goal(&node) {
            let path = reconstruct_path(&came_from, &node);
            let (goal_f, _goal_h, _goal_g) = scores[&node];
            return Ok(Some((goal_f, path)));
        }

        match scores.get(&node) {
            Some(&(_, _, old_g)) => {
                if old_g < g {
                    continue;
                }
            }
            None => {}
        }
        scores.insert(node.clone(), (f, h, g));

        if let Some(guard) = graph.get_outgoing(&node)? {
            for (edge_id_str, target_str) in guard.iter_edges() {
                let neigh_g = g + edge_cost(edge_id_str, target_str);
                let neigh_h = estimate_cost(target_str);
                let neigh_f = neigh_g + neigh_h;
                let neigh_score = (neigh_f, neigh_h, neigh_g);

                let should_update = match scores.get(target_str) {
                    Some(&(_, _, old_neigh_g)) => neigh_g < old_neigh_g,
                    None => true,
                };

                if should_update {
                    scores.insert(target_str.to_string(), neigh_score);
                    came_from.insert(target_str.to_string(), node.clone());
                    visit_next.push(MinScored(neigh_score, target_str.to_string()));
                }
            }
        }
    }

    Ok(None)
}

fn reconstruct_path(came_from: &HashMap<String, String>, last: &str) -> Vec<String> {
    let mut path = vec![last.to_string()];
    let mut current = last.to_string();
    
    while let Some(previous) = came_from.get(&current) {
        path.push(previous.clone());
        current = previous.clone();
    }
    
    path.reverse();
    path
}
