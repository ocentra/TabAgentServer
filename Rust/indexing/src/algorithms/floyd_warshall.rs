use std::collections::HashSet;
use hashbrown::HashMap;

use crate::algo::{Measure, NegativeCycle};
use crate::core::graph::GraphIndex;
use common::DbResult;

/// Floyd-Warshall all-pairs shortest paths with zero-copy MDBX access.
pub fn floyd_warshall_zero_copy<F, K>(
    graph: &GraphIndex,
    nodes: &HashSet<String>,
    mut edge_cost: F,
) -> DbResult<Result<HashMap<(String, String), K>, NegativeCycle>>
where
    F: FnMut(&str, &str) -> K,
    K: Measure + Copy,
{
    let node_vec: Vec<String> = nodes.iter().cloned().collect();
    let mut dist: HashMap<(String, String), K> = HashMap::new();

    for i in &node_vec {
        dist.insert((i.clone(), i.clone()), K::default());
    }

    for node in &node_vec {
        if let Some(guard) = graph.get_outgoing(node)? {
            for (edge_id_str, target_str) in guard.iter_edges() {
                let cost = edge_cost(edge_id_str, target_str);
                dist.insert((node.clone(), target_str.to_string()), cost);
            }
        }
    }

    for k in &node_vec {
        for i in &node_vec {
            for j in &node_vec {
                let ik = dist.get(&(i.clone(), k.clone())).copied();
                let kj = dist.get(&(k.clone(), j.clone())).copied();

                if let (Some(d_ik), Some(d_kj)) = (ik, kj) {
                    let new_dist = d_ik + d_kj;
                    let key = (i.clone(), j.clone());
                    
                    let should_update = dist.get(&key)
                        .map_or(true, |&old| new_dist < old);

                    if should_update {
                        dist.insert(key, new_dist);
                    }
                }
            }
        }
    }

    for node in &node_vec {
        if let Some(&d) = dist.get(&(node.clone(), node.clone())) {
            if d < K::default() {
                return Ok(Err(NegativeCycle));
            }
        }
    }

    Ok(Ok(dist))
}
