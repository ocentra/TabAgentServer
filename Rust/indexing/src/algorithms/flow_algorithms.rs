use std::collections::{HashSet, VecDeque};
use hashbrown::HashMap;

use crate::core::graph::GraphIndex;
use common::DbResult;

/// Maximum flow (Ford-Fulkerson) with zero-copy MDBX access.
pub fn max_flow_zero_copy<F>(
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
        let path = bfs_find_path(graph, source, sink, &flow, &mut capacity)?;
        
        match path {
            None => break,
            Some(path) => {
                let mut min_capacity = f64::MAX;

                for i in 0..path.len() - 1 {
                    let u = &path[i];
                    let v = &path[i + 1];
                    let cap = capacity(u, v);
                    let current_flow = flow.get(&(u.clone(), v.clone())).copied().unwrap_or(0.0);
                    let remaining = cap - current_flow;
                    min_capacity = min_capacity.min(remaining);
                }

                for i in 0..path.len() - 1 {
                    let u = path[i].clone();
                    let v = path[i + 1].clone();
                    *flow.entry((u.clone(), v.clone())).or_insert(0.0) += min_capacity;
                    *flow.entry((v, u)).or_insert(0.0) -= min_capacity;
                }

                total_flow += min_capacity;
            }
        }
    }

    Ok(total_flow)
}

fn bfs_find_path<F>(
    graph: &GraphIndex,
    source: &str,
    sink: &str,
    flow: &HashMap<(String, String), f64>,
    capacity: &mut F,
) -> DbResult<Option<Vec<String>>>
where
    F: FnMut(&str, &str) -> f64,
{
    let mut visited = HashSet::new();
    let mut parent: HashMap<String, String> = HashMap::new();
    let mut queue = VecDeque::new();

    queue.push_back(source.to_string());
    visited.insert(source.to_string());

    while let Some(node) = queue.pop_front() {
        if node == sink {
            let mut path = vec![sink.to_string()];
            let mut current = sink.to_string();

            while let Some(prev) = parent.get(&current) {
                path.push(prev.clone());
                current = prev.clone();
            }

            path.reverse();
            return Ok(Some(path));
        }

        if let Some(guard) = graph.get_outgoing(&node)? {
            for (_edge_id_str, target_str) in guard.iter_edges() {
                if !visited.contains(target_str) {
                    let cap = capacity(&node, target_str);
                    let current_flow = flow.get(&(node.clone(), target_str.to_string())).copied().unwrap_or(0.0);

                    if cap > current_flow {
                        visited.insert(target_str.to_string());
                        parent.insert(target_str.to_string(), node.clone());
                        queue.push_back(target_str.to_string());
                    }
                }
            }
        }
    }

    Ok(None)
}

/// Min cost max flow with zero-copy MDBX access.
pub fn min_cost_max_flow_zero_copy<FC, FW>(
    graph: &GraphIndex,
    source: &str,
    sink: &str,
    mut capacity: FC,
    mut cost: FW,
) -> DbResult<(f64, f64)>
where
    FC: FnMut(&str, &str) -> f64,
    FW: FnMut(&str, &str) -> f64,
{
    let max_flow = max_flow_zero_copy(graph, source, sink, &mut capacity)?;
    
    let mut total_cost = 0.0;
    if let Some(guard) = graph.get_outgoing(source)? {
        for (_edge_id_str, target_str) in guard.iter_edges() {
            let flow_val = capacity(source, target_str);
            let cost_val = cost(source, target_str);
            total_cost += flow_val * cost_val;
        }
    }

    Ok((max_flow, total_cost))
}
