use std::collections::HashSet;
use hashbrown::HashMap;

use crate::algo::Measure;
use crate::core::graph::GraphIndex;
use common::DbResult;

/// Prim's MST with zero-copy MDBX access.
pub fn prim_mst_zero_copy<F, K>(
    graph: &GraphIndex,
    nodes: &HashSet<String>,
    start: &str,
    mut edge_cost: F,
) -> DbResult<Vec<(String, String, K)>>
where
    F: FnMut(&str, &str) -> K,
    K: Measure + Copy,
{
    let mut mst = Vec::new();
    let mut visited = HashSet::new();
    let mut min_edge: HashMap<String, (K, String)> = HashMap::new();

    visited.insert(start.to_string());

    if let Some(guard) = graph.get_outgoing(start)? {
        for (edge_id_str, target_str) in guard.iter_edges() {
            let cost = edge_cost(edge_id_str, target_str);
            min_edge.insert(target_str.to_string(), (cost, start.to_string()));
        }
    }

    while visited.len() < nodes.len() {
        let next_node = min_edge.iter()
            .filter(|(n, _)| !visited.contains(n.as_str()))
            .min_by(|(_, (cost1, _)), (_, (cost2, _))| {
                cost1.partial_cmp(cost2).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(n, _)| n.clone());

        let next_node = match next_node {
            Some(n) => n,
            None => break,
        };

        let (cost, from) = min_edge[&next_node].clone();
        mst.push((from, next_node.clone(), cost));
        visited.insert(next_node.clone());

        if let Some(guard) = graph.get_outgoing(&next_node)? {
            for (edge_id_str, target_str) in guard.iter_edges() {
                if !visited.contains(target_str) {
                    let cost = edge_cost(edge_id_str, target_str);
                    
                    let should_update = min_edge.get(target_str)
                        .map_or(true, |(old_cost, _)| cost < *old_cost);

                    if should_update {
                        min_edge.insert(target_str.to_string(), (cost, next_node.clone()));
                    }
                }
            }
        }
    }

    Ok(mst)
}

/// Kruskal's MST with zero-copy MDBX access.
pub fn kruskal_mst_zero_copy<F, K>(
    graph: &GraphIndex,
    nodes: &HashSet<String>,
    mut edge_cost: F,
) -> DbResult<Vec<(String, String, K)>>
where
    F: FnMut(&str, &str) -> K,
    K: Measure + Copy + Ord,
{
    let mut edges = Vec::new();

    for node in nodes {
        if let Some(guard) = graph.get_outgoing(node)? {
            for (edge_id_str, target_str) in guard.iter_edges() {
                let cost = edge_cost(edge_id_str, target_str);
                edges.push((cost, node.clone(), target_str.to_string()));
            }
        }
    }

    edges.sort_by(|(cost1, _, _), (cost2, _, _)| {
        cost1.partial_cmp(cost2).unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut mst = Vec::new();
    let mut parent: HashMap<String, String> = nodes.iter()
        .map(|n| (n.clone(), n.clone()))
        .collect();

    for (cost, from, to) in edges {
        let root_from = find_root(&parent, &from);
        let root_to = find_root(&parent, &to);

        if root_from != root_to {
            mst.push((from.clone(), to.clone(), cost));
            parent.insert(root_from, root_to);
        }
    }

    Ok(mst)
}

fn find_root(parent: &HashMap<String, String>, node: &str) -> String {
    let mut current = node.to_string();
    while parent[&current] != current {
        current = parent[&current].clone();
    }
    current
}
