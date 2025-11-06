use std::collections::HashSet;

use crate::core::graph::GraphIndex;
use common::DbResult;

/// Greedy feedback arc set with zero-copy MDBX access.
pub fn feedback_arc_set_zero_copy(
    graph: &GraphIndex,
    nodes: &HashSet<String>,
) -> DbResult<Vec<(String, String)>> {
    let mut fas = Vec::new();
    let mut remaining_nodes = nodes.clone();
    let mut ordered = Vec::new();

    while !remaining_nodes.is_empty() {
        let sinks: Vec<String> = remaining_nodes.iter()
            .filter(|n| {
                graph.get_outgoing(n).ok().flatten()
                    .map(|g| g.is_empty())
                    .unwrap_or(true)
            })
            .cloned()
            .collect();

        for sink in &sinks {
            remaining_nodes.remove(sink);
            ordered.push(sink.clone());
        }

        if remaining_nodes.is_empty() {
            break;
        }

        let sources: Vec<String> = remaining_nodes.iter()
            .filter(|n| {
                graph.get_incoming(n).ok().flatten()
                    .map(|g| g.is_empty())
                    .unwrap_or(true)
            })
            .cloned()
            .collect();

        for source in &sources {
            remaining_nodes.remove(source);
            ordered.insert(0, source.clone());
        }

        if remaining_nodes.is_empty() {
            break;
        }

        if let Some(node) = remaining_nodes.iter().next().cloned() {
            remaining_nodes.remove(&node);
            ordered.insert(0, node);
        }
    }

    let position: hashbrown::HashMap<String, usize> = ordered.iter()
        .enumerate()
        .map(|(i, n)| (n.clone(), i))
        .collect();

    for node in nodes {
        if let Some(guard) = graph.get_outgoing(node)? {
            for (_edge_id_str, target_str) in guard.iter_edges() {
                let from_pos = position.get(node).copied().unwrap_or(usize::MAX);
                let to_pos = position.get(target_str).copied().unwrap_or(usize::MAX);

                if from_pos > to_pos {
                    fas.push((node.clone(), target_str.to_string()));
                }
            }
        }
    }

    Ok(fas)
}
