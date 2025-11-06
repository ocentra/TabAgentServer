use std::collections::HashSet;
use hashbrown::HashMap;

use crate::core::graph::GraphIndex;
use common::DbResult;

/// Compute dominators with zero-copy MDBX access.
pub fn dominators_zero_copy(
    graph: &GraphIndex,
    nodes: &HashSet<String>,
    start: &str,
) -> DbResult<HashMap<String, String>> {
    let mut dom: HashMap<String, HashSet<String>> = HashMap::new();

    for node in nodes {
        if node == start {
            let mut start_set = HashSet::new();
            start_set.insert(start.to_string());
            dom.insert(start.to_string(), start_set);
        } else {
            dom.insert(node.clone(), nodes.clone());
        }
    }

    let mut changed = true;
    while changed {
        changed = false;

        for node in nodes {
            if node == start {
                continue;
            }

            let predecessors = get_predecessors(graph, node)?;

            if predecessors.is_empty() {
                continue;
            }

            let mut new_dom: Option<HashSet<String>> = None;

            for pred in predecessors {
                if let Some(pred_dom) = dom.get(&pred) {
                    match &mut new_dom {
                        None => new_dom = Some(pred_dom.clone()),
                        Some(set) => {
                            set.retain(|x| pred_dom.contains(x));
                        }
                    }
                }
            }

            if let Some(mut new_dom) = new_dom {
                new_dom.insert(node.clone());

                if dom.get(node) != Some(&new_dom) {
                    dom.insert(node.clone(), new_dom);
                    changed = true;
                }
            }
        }
    }

    let mut idom: HashMap<String, String> = HashMap::new();
    for (node, dominators) in &dom {
        let immediate: Vec<String> = dominators.iter()
            .filter(|d| *d != node)
            .filter(|d| {
                let d_dom = &dom[*d];
                dominators.iter().all(|x| x == *d || x == node || !d_dom.contains(x))
            })
            .cloned()
            .collect();

        if let Some(idom_node) = immediate.first() {
            idom.insert(node.clone(), idom_node.clone());
        }
    }

    Ok(idom)
}

fn get_predecessors(graph: &GraphIndex, node: &str) -> DbResult<Vec<String>> {
    let mut predecessors = Vec::new();

    if let Some(guard) = graph.get_incoming(node)? {
        for (_edge_id_str, source_str) in guard.iter_edges() {
            predecessors.push(source_str.to_string());
        }
    }

    Ok(predecessors)
}
