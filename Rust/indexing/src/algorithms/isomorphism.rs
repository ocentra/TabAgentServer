use std::collections::HashSet;

use crate::core::graph::GraphIndex;
use common::DbResult;

/// Check if two graphs are isomorphic (simple heuristic) with zero-copy MDBX access.
pub fn is_isomorphic_zero_copy(
    graph1: &GraphIndex,
    nodes1: &HashSet<String>,
    graph2: &GraphIndex,
    nodes2: &HashSet<String>,
) -> DbResult<bool> {
    if nodes1.len() != nodes2.len() {
        return Ok(false);
    }

    let mut degree1: Vec<usize> = Vec::new();
    for node in nodes1 {
        let out_deg = graph1.count_outgoing(node)?;
        let in_deg = graph1.count_incoming(node)?;
        degree1.push(out_deg + in_deg);
    }

    let mut degree2: Vec<usize> = Vec::new();
    for node in nodes2 {
        let out_deg = graph2.count_outgoing(node)?;
        let in_deg = graph2.count_incoming(node)?;
        degree2.push(out_deg + in_deg);
    }

    degree1.sort_unstable();
    degree2.sort_unstable();

    Ok(degree1 == degree2)
}
