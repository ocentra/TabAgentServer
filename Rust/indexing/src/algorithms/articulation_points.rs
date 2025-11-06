use std::collections::HashSet;
use hashbrown::HashMap;

use crate::core::graph::GraphIndex;
use common::DbResult;

/// Find articulation points (cut vertices) with zero-copy MDBX access.
pub fn articulation_points_zero_copy(
    graph: &GraphIndex,
    nodes: &HashSet<String>,
) -> DbResult<Vec<String>> {
    let mut state = ArticulationState {
        visited: HashSet::new(),
        disc: HashMap::new(),
        low: HashMap::new(),
        parent: HashMap::new(),
        ap: HashSet::new(),
        time: 0,
    };

    for node in nodes {
        if !state.visited.contains(node.as_str()) {
            dfs_ap(graph, node, &mut state)?;
        }
    }

    Ok(state.ap.into_iter().collect())
}

struct ArticulationState {
    visited: HashSet<String>,
    disc: HashMap<String, usize>,
    low: HashMap<String, usize>,
    parent: HashMap<String, Option<String>>,
    ap: HashSet<String>,
    time: usize,
}

fn dfs_ap(
    graph: &GraphIndex,
    node: &str,
    state: &mut ArticulationState,
) -> DbResult<()> {
    let mut children = 0;
    state.visited.insert(node.to_string());
    state.disc.insert(node.to_string(), state.time);
    state.low.insert(node.to_string(), state.time);
    state.time += 1;

    if let Some(guard) = graph.get_outgoing(node)? {
        for (_edge_id_str, target_str) in guard.iter_edges() {
            if !state.visited.contains(target_str) {
                children += 1;
                state.parent.insert(target_str.to_string(), Some(node.to_string()));
                dfs_ap(graph, target_str, state)?;

                let target_low = state.low[target_str];
                let node_low = state.low[node];
                if target_low < node_low {
                    state.low.insert(node.to_string(), target_low);
                }

                if state.parent.get(node).unwrap_or(&None).is_none() && children > 1 {
                    state.ap.insert(node.to_string());
                }

                if state.parent.get(node).unwrap_or(&None).is_some() && state.low[target_str] >= state.disc[node] {
                    state.ap.insert(node.to_string());
                }
            } else if state.parent.get(node) != Some(&Some(target_str.to_string())) {
                let target_disc = state.disc[target_str];
                let node_low = state.low[node];
                if target_disc < node_low {
                    state.low.insert(node.to_string(), target_disc);
                }
            }
        }
    }

    Ok(())
}
