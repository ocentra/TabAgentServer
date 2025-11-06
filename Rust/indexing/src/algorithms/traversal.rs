//! Graph traversal algorithms (BFS/DFS) with zero-copy MDBX access.
//!
//! # Example
//! ```ignore
//! use indexing::algorithms::{bfs_zero_copy, dfs_zero_copy};
//!
//! // BFS from "start" node
//! bfs_zero_copy(&graph, "start", |node_str| {
//!     println!("Visited: {}", node_str);  // node_str is &str from mmap!
//! })?;
//! ```

use std::collections::{HashSet, VecDeque};

use crate::core::graph::GraphIndex;
use common::DbResult;

/// BFS traversal with zero-copy MDBX access.
pub fn bfs_zero_copy<F>(
    graph: &GraphIndex,
    start: &str,
    mut visit: F,
) -> DbResult<()>
where
    F: FnMut(&str),
{
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();

    queue.push_back(start.to_string());
    visited.insert(start.to_string());

    while let Some(node) = queue.pop_front() {
        visit(&node);

        if let Some(guard) = graph.get_outgoing(&node)? {
            for (_edge_id_str, target_str) in guard.iter_edges() {
                if visited.insert(target_str.to_string()) {
                    queue.push_back(target_str.to_string());
                }
            }
        }
    }

    Ok(())
}

/// DFS traversal with zero-copy MDBX access.
pub fn dfs_zero_copy<F>(
    graph: &GraphIndex,
    start: &str,
    mut visit: F,
) -> DbResult<()>
where
    F: FnMut(&str),
{
    let mut visited = HashSet::new();
    let mut stack = vec![start.to_string()];

    while let Some(node) = stack.pop() {
        if !visited.insert(node.clone()) {
            continue;
        }

        visit(&node);

        if let Some(guard) = graph.get_outgoing(&node)? {
            for (_edge_id_str, target_str) in guard.iter_edges() {
                if !visited.contains(target_str) {
                    stack.push(target_str.to_string());
                }
            }
        }
    }

    Ok(())
}

/// DFS with pre/post-order callbacks.
pub fn dfs_with_callbacks_zero_copy<PreF, PostF>(
    graph: &GraphIndex,
    start: &str,
    mut pre_visit: PreF,
    mut post_visit: PostF,
) -> DbResult<()>
where
    PreF: FnMut(&str),
    PostF: FnMut(&str),
{
    let mut visited = HashSet::new();
    dfs_recursive(graph, start, &mut visited, &mut pre_visit, &mut post_visit)
}

fn dfs_recursive<PreF, PostF>(
    graph: &GraphIndex,
    node: &str,
    visited: &mut HashSet<String>,
    pre_visit: &mut PreF,
    post_visit: &mut PostF,
) -> DbResult<()>
where
    PreF: FnMut(&str),
    PostF: FnMut(&str),
{
    if !visited.insert(node.to_string()) {
        return Ok(());
    }

    pre_visit(node);

    if let Some(guard) = graph.get_outgoing(node)? {
        for (_edge_id_str, target_str) in guard.iter_edges() {
            dfs_recursive(graph, target_str, visited, pre_visit, post_visit)?;
        }
    }

    post_visit(node);
    Ok(())
}

