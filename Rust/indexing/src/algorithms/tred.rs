use std::collections::HashSet;
use hashbrown::HashMap;

use crate::core::graph::GraphIndex;
use common::DbResult;

/// Transitive reduction with zero-copy MDBX access.
pub fn transitive_reduction_zero_copy(
    graph: &GraphIndex,
    nodes: &HashSet<String>,
) -> DbResult<Vec<(String, String)>> {
    let mut reachable: HashMap<String, HashSet<String>> = HashMap::new();

    for node in nodes {
        let mut visited = HashSet::new();
        dfs_reachable(graph, node, node, &mut visited)?;
        reachable.insert(node.clone(), visited);
    }

    let mut reduced_edges = Vec::new();

    for node in nodes {
        if let Some(guard) = graph.get_outgoing(node)? {
            for (_edge_id_str, target_str) in guard.iter_edges() {
                let mut is_transitive = false;

                if let Some(node_reachable) = reachable.get(node) {
                    for intermediate in node_reachable {
                        if intermediate != node && intermediate != target_str {
                            if let Some(intermediate_reachable) = reachable.get(intermediate) {
                                if intermediate_reachable.contains(target_str) {
                                    is_transitive = true;
                                    break;
                                }
                            }
                        }
                    }
                }

                if !is_transitive {
                    reduced_edges.push((node.clone(), target_str.to_string()));
                }
            }
        }
    }

    Ok(reduced_edges)
}

fn dfs_reachable(
    graph: &GraphIndex,
    start: &str,
    current: &str,
    visited: &mut HashSet<String>,
) -> DbResult<()> {
    if let Some(guard) = graph.get_outgoing(current)? {
        for (_edge_id_str, target_str) in guard.iter_edges() {
            if target_str != start && visited.insert(target_str.to_string()) {
                dfs_reachable(graph, start, target_str, visited)?;
            }
        }
    }
    Ok(())
}

/// Transitive closure with zero-copy MDBX access.
pub fn transitive_closure_zero_copy(
    graph: &GraphIndex,
    nodes: &HashSet<String>,
) -> DbResult<Vec<(String, String)>> {
    let mut closure = Vec::new();

    for node in nodes {
        let mut reachable = HashSet::new();
        dfs_reachable(graph, node, node, &mut reachable)?;

        for target in reachable {
            closure.push((node.clone(), target));
        }
    }

    Ok(closure)
}
