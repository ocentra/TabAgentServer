use std::collections::HashSet;

use crate::algo::Measure;
use crate::core::graph::GraphIndex;
use common::DbResult;

use super::min_spanning_tree::prim_mst_zero_copy;

/// Steiner tree approximation with zero-copy MDBX access.
pub fn steiner_tree_zero_copy<F, K>(
    graph: &GraphIndex,
    nodes: &HashSet<String>,
    terminals: &HashSet<String>,
    edge_cost: F,
) -> DbResult<Vec<(String, String, K)>>
where
    F: FnMut(&str, &str) -> K,
    K: Measure + Copy,
{
    if terminals.is_empty() {
        return Ok(Vec::new());
    }

    let start = terminals.iter().next().unwrap();
    let mst = prim_mst_zero_copy(graph, nodes, start, edge_cost)?;

    let steiner: Vec<(String, String, K)> = mst.into_iter()
        .filter(|(from, to, _)| {
            terminals.contains(from) || terminals.contains(to)
        })
        .collect();

    Ok(steiner)
}
