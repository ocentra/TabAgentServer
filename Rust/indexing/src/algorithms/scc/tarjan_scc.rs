//! Tarjan's algorithm for strongly connected components with zero-copy MDBX access.
//!
//! # Example
//! ```ignore
//! use indexing::algorithms::tarjan_scc_zero_copy;
//! use std::collections::HashSet;
//!
//! // Find all strongly connected components
//! let sccs = tarjan_scc_zero_copy(&graph, &nodes)?;
//! println!("Found {} SCCs", sccs.len());
//! ```

use std::collections::HashSet;
use hashbrown::HashMap;

use crate::core::graph::GraphIndex;
use common::DbResult;

/// Tarjan's strongly connected components with zero-copy MDBX access.
pub fn tarjan_scc_zero_copy(
    graph: &GraphIndex,
    nodes: &HashSet<String>,
) -> DbResult<Vec<Vec<String>>> {
    let mut state = TarjanState {
        index: 0,
        stack: Vec::new(),
        on_stack: HashSet::new(),
        indices: HashMap::new(),
        low_links: HashMap::new(),
        components: Vec::new(),
    };

    for node in nodes {
        if !state.indices.contains_key(node.as_str()) {
            strongconnect(graph, node, &mut state)?;
        }
    }

    Ok(state.components)
}

struct TarjanState {
    index: usize,
    stack: Vec<String>,
    on_stack: HashSet<String>,
    indices: HashMap<String, usize>,
    low_links: HashMap<String, usize>,
    components: Vec<Vec<String>>,
}

fn strongconnect(
    graph: &GraphIndex,
    node: &str,
    state: &mut TarjanState,
) -> DbResult<()> {
    state.indices.insert(node.to_string(), state.index);
    state.low_links.insert(node.to_string(), state.index);
    state.index += 1;
    state.stack.push(node.to_string());
    state.on_stack.insert(node.to_string());

    if let Some(guard) = graph.get_outgoing(node)? {
        for (_edge_id_str, target_str) in guard.iter_edges() {
            if !state.indices.contains_key(target_str) {
                strongconnect(graph, target_str, state)?;
                let target_low = state.low_links[target_str];
                let node_low = state.low_links[node];
                if target_low < node_low {
                    state.low_links.insert(node.to_string(), target_low);
                }
            } else if state.on_stack.contains(target_str) {
                let target_index = state.indices[target_str];
                let node_low = state.low_links[node];
                if target_index < node_low {
                    state.low_links.insert(node.to_string(), target_index);
                }
            }
        }
    }

    if state.low_links[node] == state.indices[node] {
        let mut component = Vec::new();
        loop {
            let w = state.stack.pop().unwrap();
            state.on_stack.remove(&w);
            component.push(w.clone());
            if w == node {
                break;
            }
        }
        state.components.push(component);
    }

    Ok(())
}
