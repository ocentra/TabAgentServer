use std::collections::HashSet;

use crate::algo::Measure;
use crate::core::graph::GraphIndex;
use common::DbResult;

/// Maximum weight matching (greedy approximation) with zero-copy MDBX access.
pub fn max_weight_matching_zero_copy<F, K>(
    graph: &GraphIndex,
    nodes: &HashSet<String>,
    mut edge_weight: F,
) -> DbResult<Vec<(String, String, K)>>
where
    F: FnMut(&str, &str) -> K,
    K: Measure + Copy + Ord,
{
    let mut edges = Vec::new();

    for node in nodes {
        if let Some(guard) = graph.get_outgoing(node)? {
            for (edge_id_str, target_str) in guard.iter_edges() {
                let weight = edge_weight(edge_id_str, target_str);
                edges.push((weight, node.clone(), target_str.to_string()));
            }
        }
    }

    edges.sort_by_key(|(w, _, _)| std::cmp::Reverse(*w));

    let mut matching = Vec::new();
    let mut matched: HashSet<String> = HashSet::new();

    for (weight, from, to) in edges {
        if !matched.contains(&from) && !matched.contains(&to) {
            matching.push((from.clone(), to.clone(), weight));
            matched.insert(from);
            matched.insert(to);
        }
    }

    Ok(matching)
}
