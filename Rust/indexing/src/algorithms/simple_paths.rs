use std::collections::HashSet;

use crate::core::graph::GraphIndex;
use common::DbResult;

/// Find all simple paths from source to target with zero-copy MDBX access.
pub fn all_simple_paths_zero_copy(
    graph: &GraphIndex,
    source: &str,
    target: &str,
    max_length: Option<usize>,
) -> DbResult<Vec<Vec<String>>> {
    let mut paths = Vec::new();
    let mut current_path = vec![source.to_string()];
    let mut visited = HashSet::new();
    visited.insert(source.to_string());

    dfs_paths(graph, source, target, max_length, &mut current_path, &mut visited, &mut paths)?;

    Ok(paths)
}

fn dfs_paths(
    graph: &GraphIndex,
    current: &str,
    target: &str,
    max_length: Option<usize>,
    path: &mut Vec<String>,
    visited: &mut HashSet<String>,
    paths: &mut Vec<Vec<String>>,
) -> DbResult<()> {
    if current == target {
        paths.push(path.clone());
        return Ok(());
    }

    if let Some(max_len) = max_length {
        if path.len() >= max_len {
            return Ok(());
        }
    }

    if let Some(guard) = graph.get_outgoing(current)? {
        for (_edge_id_str, next_str) in guard.iter_edges() {
            if !visited.contains(next_str) {
                visited.insert(next_str.to_string());
                path.push(next_str.to_string());

                dfs_paths(graph, next_str, target, max_length, path, visited, paths)?;

                path.pop();
                visited.remove(next_str);
            }
        }
    }

    Ok(())
}
