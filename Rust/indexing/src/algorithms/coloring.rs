use std::collections::HashSet;
use hashbrown::HashMap;

use crate::core::graph::GraphIndex;
use common::DbResult;

/// DSATUR graph coloring with zero-copy MDBX access.
pub fn dsatur_coloring_zero_copy(
    graph: &GraphIndex,
    nodes: &HashSet<String>,
) -> DbResult<HashMap<String, usize>> {
    let mut colors: HashMap<String, usize> = HashMap::new();
    let mut saturation: HashMap<String, HashSet<usize>> = HashMap::new();

    for node in nodes {
        saturation.insert(node.clone(), HashSet::new());
    }

    while colors.len() < nodes.len() {
        let next_node = nodes.iter()
            .filter(|n| !colors.contains_key(n.as_str()))
            .max_by_key(|n| saturation[n.as_str()].len())
            .unwrap();

        let mut color = 0;
        while saturation[next_node].contains(&color) {
            color += 1;
        }

        colors.insert(next_node.clone(), color);

        if let Some(guard) = graph.get_outgoing(next_node)? {
            for (_edge_id_str, neighbor_str) in guard.iter_edges() {
                if let Some(sat) = saturation.get_mut(neighbor_str) {
                    sat.insert(color);
                }
            }
        }

        if let Some(guard) = graph.get_incoming(next_node)? {
            for (_edge_id_str, neighbor_str) in guard.iter_edges() {
                if let Some(sat) = saturation.get_mut(neighbor_str) {
                    sat.insert(color);
                }
            }
        }
    }

    Ok(colors)
}
