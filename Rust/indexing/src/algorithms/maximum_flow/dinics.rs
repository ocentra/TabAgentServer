use std::collections::{HashSet, VecDeque};
use hashbrown::HashMap;

use crate::core::graph::GraphIndex;
use common::DbResult;

/// Dinic's maximum flow with zero-copy MDBX access.
pub fn dinics_zero_copy<F>(
    graph: &GraphIndex,
    source: &str,
    sink: &str,
    mut capacity: F,
) -> DbResult<f64>
where
    F: FnMut(&str, &str) -> f64,
{
    let mut flow: HashMap<(String, String), f64> = HashMap::new();
    let mut total_flow = 0.0;

    loop {
        let level = build_level_graph(graph, source, sink, &flow, &mut capacity)?;
        
        if !level.contains_key(sink) {
            break;
        }

        loop {
            let blocking_flow = dfs_blocking_flow(
                graph,
                source,
                sink,
                f64::MAX,
                &level,
                &mut flow,
                &mut capacity,
                &mut HashSet::new(),
            )?;

            if blocking_flow == 0.0 {
                break;
            }

            total_flow += blocking_flow;
        }
    }

    Ok(total_flow)
}

fn build_level_graph<F>(
    graph: &GraphIndex,
    source: &str,
    sink: &str,
    flow: &HashMap<(String, String), f64>,
    capacity: &mut F,
) -> DbResult<HashMap<String, usize>>
where
    F: FnMut(&str, &str) -> f64,
{
    let mut level = HashMap::new();
    level.insert(source.to_string(), 0);

    let mut queue = VecDeque::new();
    queue.push_back(source.to_string());

    while let Some(node) = queue.pop_front() {
        let current_level = level[&node];

        if let Some(guard) = graph.get_outgoing(&node)? {
            for (_edge_id_str, target_str) in guard.iter_edges() {
                if !level.contains_key(target_str) {
                    let cap = capacity(&node, target_str);
                    let current_flow = flow.get(&(node.clone(), target_str.to_string())).copied().unwrap_or(0.0);

                    if cap > current_flow {
                        level.insert(target_str.to_string(), current_level + 1);
                        queue.push_back(target_str.to_string());

                        if target_str == sink {
                            return Ok(level);
                        }
                    }
                }
            }
        }
    }

    Ok(level)
}

fn dfs_blocking_flow<F>(
    graph: &GraphIndex,
    current: &str,
    sink: &str,
    min_flow: f64,
    level: &HashMap<String, usize>,
    flow: &mut HashMap<(String, String), f64>,
    capacity: &mut F,
    visited: &mut HashSet<String>,
) -> DbResult<f64>
where
    F: FnMut(&str, &str) -> f64,
{
    if current == sink {
        return Ok(min_flow);
    }

    if visited.contains(current) {
        return Ok(0.0);
    }

    visited.insert(current.to_string());

    let current_level = level[current];

    if let Some(guard) = graph.get_outgoing(current)? {
        for (_edge_id_str, target_str) in guard.iter_edges() {
            if let Some(&target_level) = level.get(target_str) {
                if target_level == current_level + 1 {
                    let cap = capacity(current, target_str);
                    let current_flow = flow.get(&(current.to_string(), target_str.to_string())).copied().unwrap_or(0.0);
                    let remaining = cap - current_flow;

                    if remaining > 0.0 {
                        let pushed = dfs_blocking_flow(
                            graph,
                            target_str,
                            sink,
                            min_flow.min(remaining),
                            level,
                            flow,
                            capacity,
                            visited,
                        )?;

                        if pushed > 0.0 {
                            *flow.entry((current.to_string(), target_str.to_string())).or_insert(0.0) += pushed;
                            *flow.entry((target_str.to_string(), current.to_string())).or_insert(0.0) -= pushed;
                            return Ok(pushed);
                        }
                    }
                }
            }
        }
    }

    Ok(0.0)
}
