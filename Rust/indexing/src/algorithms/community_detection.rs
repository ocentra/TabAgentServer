use std::collections::HashSet;
use hashbrown::HashMap;

use crate::core::graph::GraphIndex;
use common::DbResult;

/// Louvain community detection with zero-copy MDBX access.
pub fn louvain_zero_copy(
    graph: &GraphIndex,
    nodes: &HashSet<String>,
    iterations: usize,
) -> DbResult<HashMap<String, usize>> {
    let mut communities: HashMap<String, usize> = nodes.iter()
        .enumerate()
        .map(|(i, n)| (n.clone(), i))
        .collect();

    for _ in 0..iterations {
        let mut changed = false;

        for node in nodes {
            let current_comm = communities[node];
            let mut best_comm = current_comm;
            let mut best_gain = 0.0;

            let neighbors = get_neighbors(graph, node)?;

            let neighbor_comms: HashSet<usize> = neighbors.iter()
                .filter_map(|n| communities.get(n).copied())
                .collect();

            for &comm in &neighbor_comms {
                let gain = calculate_modularity_gain(node, comm, &neighbors, &communities);
                if gain > best_gain {
                    best_gain = gain;
                    best_comm = comm;
                }
            }

            if best_comm != current_comm {
                communities.insert(node.clone(), best_comm);
                changed = true;
            }
        }

        if !changed {
            break;
        }
    }

    Ok(communities)
}

fn get_neighbors(graph: &GraphIndex, node: &str) -> DbResult<HashSet<String>> {
    let mut neighbors = HashSet::new();

    if let Some(guard) = graph.get_outgoing(node)? {
        for (_edge_id_str, target_str) in guard.iter_edges() {
            neighbors.insert(target_str.to_string());
        }
    }

    if let Some(guard) = graph.get_incoming(node)? {
        for (_edge_id_str, source_str) in guard.iter_edges() {
            neighbors.insert(source_str.to_string());
        }
    }

    Ok(neighbors)
}

fn calculate_modularity_gain(
    _node: &str,
    _comm: usize,
    neighbors: &HashSet<String>,
    communities: &HashMap<String, usize>,
) -> f64 {
    let same_comm_neighbors = neighbors.iter()
        .filter(|n| communities.get(n.as_str()).copied() == Some(_comm))
        .count();

    same_comm_neighbors as f64 / neighbors.len().max(1) as f64
}
