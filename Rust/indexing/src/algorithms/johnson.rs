use std::collections::HashSet;
use hashbrown::HashMap;

use crate::algo::{Measure, NegativeCycle};
use crate::core::graph::GraphIndex;
use common::DbResult;

use super::bellman_ford::bellman_ford_zero_copy;
use super::dijkstra::dijkstra_zero_copy;

/// Johnson's algorithm for all-pairs shortest paths with zero-copy MDBX access.
pub fn johnson_zero_copy<F, K>(
    graph: &GraphIndex,
    nodes: &HashSet<String>,
    mut edge_cost: F,
) -> DbResult<Result<HashMap<(String, String), K>, NegativeCycle>>
where
    F: FnMut(&str, &str) -> K + Clone,
    K: Measure + Copy + std::ops::Sub<Output = K> + std::ops::Add<Output = K>,
{
    let mut extended_nodes = nodes.clone();
    let virtual_node = "__virtual__".to_string();
    extended_nodes.insert(virtual_node.clone());

    let h = match bellman_ford_zero_copy(graph, &extended_nodes, &virtual_node, edge_cost.clone())? {
        Ok(distances) => distances,
        Err(NegativeCycle) => return Ok(Err(NegativeCycle)),
    };

    let mut all_pairs: HashMap<(String, String), K> = HashMap::new();

    for source in nodes {
        let h_source = h.get(source).copied().unwrap_or(K::default());

        let reweighted_cost = |edge_id: &str, target: &str| {
            let cost = edge_cost(edge_id, target);
            let h_target = h.get(target).copied().unwrap_or(K::default());
            cost + h_source - h_target
        };

        let distances = dijkstra_zero_copy(graph, source, None, reweighted_cost)?;

        for (target, dist) in distances {
            let h_target = h.get(&target).copied().unwrap_or(K::default());
            let actual_dist = dist - h_source + h_target;
            all_pairs.insert((source.clone(), target), actual_dist);
        }
    }

    Ok(Ok(all_pairs))
}
