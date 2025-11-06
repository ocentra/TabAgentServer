use std::collections::BinaryHeap;

use crate::algo::Measure;
use crate::scored::MinScored;
use crate::core::graph::GraphIndex;
use common::DbResult;

/// Find K shortest paths with zero-copy MDBX access.
pub fn k_shortest_paths_zero_copy<F, K>(
    graph: &GraphIndex,
    source: &str,
    target: &str,
    k: usize,
    mut edge_cost: F,
) -> DbResult<Vec<(K, Vec<String>)>>
where
    F: FnMut(&str, &str) -> K,
    K: Measure + Copy,
{
    let mut paths = Vec::new();
    let mut heap = BinaryHeap::new();

    heap.push(MinScored(K::default(), vec![source.to_string()]));

    while let Some(MinScored(cost, path)) = heap.pop() {
        let current = &path[path.len() - 1];

        if current == target {
            paths.push((cost, path.clone()));
            if paths.len() >= k {
                break;
            }
            continue;
        }

        if let Some(guard) = graph.get_outgoing(current)? {
            for (edge_id_str, next_str) in guard.iter_edges() {
                if !path.contains(&next_str.to_string()) {
                    let mut new_path = path.clone();
                    new_path.push(next_str.to_string());
                    let new_cost = cost + edge_cost(edge_id_str, next_str);
                    heap.push(MinScored(new_cost, new_path));
                }
            }
        }
    }

    Ok(paths)
}
