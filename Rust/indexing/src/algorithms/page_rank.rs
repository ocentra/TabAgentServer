//! PageRank algorithm with zero-copy MDBX access.
//!
//! # Example
//! ```ignore
//! use indexing::algorithms::page_rank_zero_copy;
//! use std::collections::HashSet;
//!
//! let graph = GraphIndex::open(&env, "my_graph")?;
//! let nodes: HashSet<String> = /* all node IDs */;
//!
//! // Compute PageRank scores
//! let ranks = page_rank_zero_copy(
//!     &graph,
//!     &nodes,
//!     0.85,    // Damping factor
//!     100,     // Max iterations
//!     1e-6     // Convergence tolerance
//! )?;
//!
//! // Get top-ranked nodes
//! let mut ranked: Vec<_> = ranks.iter().collect();
//! ranked.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
//! for (node, score) in ranked.iter().take(10) {
//!     println!("{}: {:.6}", node, score);
//! }
//! ```

use std::collections::HashSet;
use hashbrown::HashMap;

use crate::core::graph::GraphIndex;
use common::DbResult;

/// PageRank algorithm with zero-copy MDBX access.
///
/// # Arguments
/// * `graph`: GraphIndex for zero-copy edge access
/// * `nodes`: set of all node IDs
/// * `damping_factor`: value in range 0.0 to 1.0 (typically 0.85)
/// * `iterations`: number of iterations
///
/// # Returns
/// HashMap mapping each node ID to its PageRank score
pub fn page_rank_zero_copy(
    graph: &GraphIndex,
    nodes: &HashSet<String>,
    damping_factor: f64,
    iterations: usize,
) -> DbResult<HashMap<String, f64>> {
    if nodes.is_empty() {
        return Ok(HashMap::new());
    }

    let n = nodes.len() as f64;
    let init_rank = 1.0 / n;
    let base_rank = (1.0 - damping_factor) / n;

    let mut ranks: HashMap<String, f64> = nodes.iter()
        .map(|node| (node.clone(), init_rank))
        .collect();

    let mut out_degree: HashMap<String, usize> = HashMap::new();
    for node in nodes {
        let degree = graph.count_outgoing(node)?;
        out_degree.insert(node.clone(), degree);
    }

    let mut new_ranks = HashMap::new();

    for _ in 0..iterations {
        new_ranks.clear();

        for node in nodes {
            new_ranks.insert(node.clone(), base_rank);
        }

        for source in nodes {
            let source_rank = ranks[source];
            let out_deg = out_degree[source];
            
            if out_deg == 0 {
                continue;
            }

            let contribution = damping_factor * source_rank / (out_deg as f64);

            if let Some(guard) = graph.get_outgoing(source)? {
                for (_edge_id_str, target_str) in guard.iter_edges() {
                    *new_ranks.entry(target_str.to_string()).or_insert(base_rank) += contribution;
                }
            }
        }

        std::mem::swap(&mut ranks, &mut new_ranks);
    }

    Ok(ranks)
}
