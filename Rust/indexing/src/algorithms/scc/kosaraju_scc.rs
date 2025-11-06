use std::collections::HashSet;

use crate::core::graph::GraphIndex;
use common::DbResult;

/// Kosaraju's strongly connected components with zero-copy MDBX access.
pub fn kosaraju_scc_zero_copy(
    graph: &GraphIndex,
    nodes: &HashSet<String>,
) -> DbResult<Vec<Vec<String>>> {
    let mut visited = HashSet::new();
    let mut finish_order = Vec::new();

    for node in nodes {
        if !visited.contains(node.as_str()) {
            dfs_first_pass(graph, node, &mut visited, &mut finish_order)?;
        }
    }

    visited.clear();
    let mut components = Vec::new();

    while let Some(node) = finish_order.pop() {
        if !visited.contains(node.as_str()) {
            let mut component = Vec::new();
            dfs_second_pass(graph, &node, &mut visited, &mut component)?;
            components.push(component);
        }
    }

    Ok(components)
}

fn dfs_first_pass(
    graph: &GraphIndex,
    node: &str,
    visited: &mut HashSet<String>,
    finish_order: &mut Vec<String>,
) -> DbResult<()> {
    visited.insert(node.to_string());

    if let Some(guard) = graph.get_outgoing(node)? {
        for (_edge_id_str, target_str) in guard.iter_edges() {
            if !visited.contains(target_str) {
                dfs_first_pass(graph, target_str, visited, finish_order)?;
            }
        }
    }

    finish_order.push(node.to_string());
    Ok(())
}

fn dfs_second_pass(
    graph: &GraphIndex,
    node: &str,
    visited: &mut HashSet<String>,
    component: &mut Vec<String>,
) -> DbResult<()> {
    visited.insert(node.to_string());
    component.push(node.to_string());

    if let Some(guard) = graph.get_incoming(node)? {
        for (_edge_id_str, source_str) in guard.iter_edges() {
            if !visited.contains(source_str) {
                dfs_second_pass(graph, source_str, visited, component)?;
            }
        }
    }

    Ok(())
}
