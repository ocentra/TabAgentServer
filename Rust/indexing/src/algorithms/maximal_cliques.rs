use std::collections::HashSet;

use crate::core::graph::GraphIndex;
use common::DbResult;

/// Bron-Kerbosch algorithm for maximal cliques with zero-copy MDBX access.
pub fn bron_kerbosch_zero_copy(
    graph: &GraphIndex,
    nodes: &HashSet<String>,
) -> DbResult<Vec<HashSet<String>>> {
    let mut cliques = Vec::new();
    let r = HashSet::new();
    let p = nodes.clone();
    let x = HashSet::new();

    bron_kerbosch_recursive(graph, r, p, x, &mut cliques)?;

    Ok(cliques)
}

fn bron_kerbosch_recursive(
    graph: &GraphIndex,
    r: HashSet<String>,
    mut p: HashSet<String>,
    mut x: HashSet<String>,
    cliques: &mut Vec<HashSet<String>>,
) -> DbResult<()> {
    if p.is_empty() && x.is_empty() {
        cliques.push(r);
        return Ok(());
    }

    let p_clone = p.clone();
    for v in p_clone {
        let mut new_r = r.clone();
        new_r.insert(v.clone());

        let neighbors = get_neighbors(graph, &v)?;

        let new_p: HashSet<String> = p.intersection(&neighbors).cloned().collect();
        let new_x: HashSet<String> = x.intersection(&neighbors).cloned().collect();

        bron_kerbosch_recursive(graph, new_r, new_p, new_x, cliques)?;

        p.remove(&v);
        x.insert(v);
    }

    Ok(())
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
