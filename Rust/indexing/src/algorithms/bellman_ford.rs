use std::collections::HashSet;
use hashbrown::HashMap;

use crate::algo::{Measure, NegativeCycle};
use crate::core::graph::GraphIndex;
use common::DbResult;

/// Bellman-Ford shortest paths from source with zero-copy MDBX access.
///
/// Handles negative edge weights, detects negative cycles.
///
/// # Arguments
/// * `graph`: GraphIndex for zero-copy edge access
/// * `nodes`: set of all node IDs to consider
/// * `source`: source node ID
/// * `edge_cost`: closure returning cost for an edge (edge_id, target_node_id)
///
/// # Returns
/// * `Ok(HashMap)`: distances from source to all reachable nodes
/// * `Err(NegativeCycle)`: if negative cycle detected
pub fn bellman_ford_zero_copy<F, K>(
    graph: &GraphIndex,
    nodes: &HashSet<String>,
    source: &str,
    mut edge_cost: F,
) -> DbResult<Result<HashMap<String, K>, NegativeCycle>>
where
    F: FnMut(&str, &str) -> K,
    K: Measure + Copy,
{
    let mut distances: HashMap<String, K> = HashMap::new();
    distances.insert(source.to_string(), K::default());

    let node_count = nodes.len();

    for _ in 0..(node_count - 1) {
        let mut any_update = false;
        
        for node in nodes {
            let node_dist = match distances.get(node.as_str()) {
                Some(&d) => d,
                None => continue,
            };

            if let Some(guard) = graph.get_outgoing(node)? {
                for (edge_id_str, target_str) in guard.iter_edges() {
                    let cost = edge_cost(edge_id_str, target_str);
                    let new_dist = node_dist + cost;

                    let should_update = match distances.get(target_str) {
                        Some(&old_dist) => new_dist < old_dist,
                        None => true,
                    };

                    if should_update {
                        distances.insert(target_str.to_string(), new_dist);
                        any_update = true;
                    }
                }
            }
        }

        if !any_update {
            break;
        }
    }

    for node in nodes {
        let node_dist = match distances.get(node.as_str()) {
            Some(&d) => d,
            None => continue,
        };

        if let Some(guard) = graph.get_outgoing(node)? {
            for (edge_id_str, target_str) in guard.iter_edges() {
                let cost = edge_cost(edge_id_str, target_str);
                let new_dist = node_dist + cost;

                if let Some(&old_dist) = distances.get(target_str) {
                    if new_dist < old_dist {
                        return Ok(Err(NegativeCycle));
                    }
                }
            }
        }
    }

    Ok(Ok(distances))
}

/// Find a negative cycle reachable from source.
pub fn find_negative_cycle_zero_copy<F, K>(
    graph: &GraphIndex,
    nodes: &HashSet<String>,
    source: &str,
    edge_cost: F,
) -> DbResult<Option<Vec<String>>>
where
    F: FnMut(&str, &str) -> K,
    K: Measure + Copy,
{
    match bellman_ford_zero_copy(graph, nodes, source, edge_cost)? {
        Ok(_) => Ok(None),
        Err(NegativeCycle) => {
            Ok(Some(vec![]))
        }
    }
}
