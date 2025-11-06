use std::collections::HashSet;
use hashbrown::HashMap;

use crate::core::graph::GraphIndex;
use common::DbResult;

/// Find bridges (cut edges) with zero-copy MDBX access.
pub fn bridges_zero_copy(
    graph: &GraphIndex,
    nodes: &HashSet<String>,
) -> DbResult<Vec<(String, String)>> {
    let mut state = BridgeState {
        visited: HashSet::new(),
        disc: HashMap::new(),
        low: HashMap::new(),
        parent: HashMap::new(),
        bridges: Vec::new(),
        time: 0,
    };

    for node in nodes {
        if !state.visited.contains(node.as_str()) {
            dfs_bridge(graph, node, &mut state)?;
        }
    }

    Ok(state.bridges)
}

struct BridgeState {
    visited: HashSet<String>,
    disc: HashMap<String, usize>,
    low: HashMap<String, usize>,
    parent: HashMap<String, Option<String>>,
    bridges: Vec<(String, String)>,
    time: usize,
}

fn dfs_bridge(
    graph: &GraphIndex,
    node: &str,
    state: &mut BridgeState,
) -> DbResult<()> {
    state.visited.insert(node.to_string());
    state.disc.insert(node.to_string(), state.time);
    state.low.insert(node.to_string(), state.time);
    state.time += 1;

    if let Some(guard) = graph.get_outgoing(node)? {
        for (_edge_id_str, target_str) in guard.iter_edges() {
            if !state.visited.contains(target_str) {
                state.parent.insert(target_str.to_string(), Some(node.to_string()));
                dfs_bridge(graph, target_str, state)?;

                let target_low = state.low[target_str];
                let node_low = state.low[node];
                if target_low < node_low {
                    state.low.insert(node.to_string(), target_low);
                }

                if state.low[target_str] > state.disc[node] {
                    state.bridges.push((node.to_string(), target_str.to_string()));
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
