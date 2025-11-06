use std::collections::{HashSet, VecDeque};
use hashbrown::HashMap;

use crate::algo::{Measure, NegativeCycle};
use crate::core::graph::GraphIndex;
use common::DbResult;

/// SPFA (Shortest Path Faster Algorithm) with zero-copy MDBX access.
pub fn spfa_zero_copy<F, K>(
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

    let mut queue = VecDeque::new();
    queue.push_back(source.to_string());

    let mut in_queue = HashSet::new();
    in_queue.insert(source.to_string());

    let mut count: HashMap<String, usize> = HashMap::new();

    while let Some(node) = queue.pop_front() {
        in_queue.remove(&node);

        let node_dist = distances[&node];

        if let Some(guard) = graph.get_outgoing(&node)? {
            for (edge_id_str, target_str) in guard.iter_edges() {
                let cost = edge_cost(edge_id_str, target_str);
                let new_dist = node_dist + cost;

                let should_update = match distances.get(target_str) {
                    Some(&old_dist) => new_dist < old_dist,
                    None => true,
                };

                if should_update {
                    distances.insert(target_str.to_string(), new_dist);

                    if !in_queue.contains(target_str) {
                        queue.push_back(target_str.to_string());
                        in_queue.insert(target_str.to_string());

                        let c = count.entry(target_str.to_string()).or_insert(0);
                        *c += 1;

                        if *c >= nodes.len() {
                            return Ok(Err(NegativeCycle));
                        }
                    }
                }
            }
        }
    }

    Ok(Ok(distances))
}
