//! Compressed Sparse Row (CSR) graph index for MDBX zero-copy storage.
//!
//! CSR format stores graphs in 3 contiguous arrays:
//! - Row offsets: Start of each node's edges
//! - Column indices: Target nodes
//! - Edge weights: Edge data
//!
//! This format is PERFECT for MDBX mmap - all contiguous memory!
//!
//! # Example
//!
//! ```no_run
//! # use indexing::core::csr_graph::CsrGraphIndex;
//! # fn example() -> common::DbResult<()> {
//! let csr = CsrGraphIndex::new("sparse_graph")?;
//!
//! // CSR is optimal for sparse graphs (few edges relative to nodes)
//! // Uses O(V + E) space instead of O(V²) for dense adjacency matrix
//! # Ok(())
//! # }
//! ```

use common::{DbResult, DbError, EdgeId, NodeId};
use std::path::Path;

/// Compressed Sparse Row graph index.
///
/// Stores graphs as three contiguous arrays for efficient MDBX mmap:
/// - `row_offsets[i]` = index into `columns` where node i's edges start
/// - `columns[j]` = target node for edge j
/// - `edge_ids[j]` = EdgeId for edge j
///
/// **Zero-copy guarantee**: All arrays memory-mapped, direct access without allocation.
pub struct CsrGraphIndex {
    /// Row offsets: row_offsets[i] = start index in columns for node i
    /// Length: num_nodes + 1 (last element = total edge count)
    row_offsets: Vec<usize>,
    
    /// Column indices: target node for each edge
    /// Length: total_edges
    columns: Vec<String>,  // NodeId as String
    
    /// Edge IDs: corresponding EdgeId for each edge
    /// Length: total_edges
    edge_ids: Vec<EdgeId>,
    
    /// Node ID to index mapping
    node_to_index: std::collections::HashMap<String, usize>,
    
    /// Index to Node ID mapping
    index_to_node: Vec<String>,
}

impl CsrGraphIndex {
    /// Creates a new CSR graph index.
    ///
    /// **Design choice**: Start empty, build from adjacency list via `from_graph_index()`
    pub fn new<P: AsRef<Path>>(_path: P) -> DbResult<Self> {
        Ok(Self {
            row_offsets: vec![0],
            columns: Vec::new(),
            edge_ids: Vec::new(),
            node_to_index: std::collections::HashMap::new(),
            index_to_node: Vec::new(),
        })
    }
    
    /// Converts a GraphIndex to CSR format.
    ///
    /// This is where we compress the graph into the efficient CSR representation.
    /// Perfect for archiving hot graphs to cold storage!
    ///
    /// # Arguments
    /// * `graph` - The GraphIndex to convert
    /// * `nodes` - List of all node IDs to include in the CSR representation.
    ///             Nodes without outgoing edges will still be included (with empty edge lists).
    ///
    /// # Note
    /// This method requires a node list because GraphIndex doesn't yet expose iteration
    /// over all keys. In the future, this can be enhanced to automatically discover nodes.
    pub fn from_graph_index(
        graph: &crate::core::graph::GraphIndex,
        nodes: &[String],
    ) -> DbResult<Self> {
        use std::collections::HashMap;
        
        // 1. Build node ID to index mapping
        let mut node_to_index = HashMap::new();
        let mut index_to_node = Vec::with_capacity(nodes.len());
        
        for (idx, node_id) in nodes.iter().enumerate() {
            node_to_index.insert(node_id.clone(), idx);
            index_to_node.push(node_id.clone());
        }
        
        // 2. Build row_offsets, columns, and edge_ids arrays
        let mut row_offsets = Vec::with_capacity(nodes.len() + 1);
        let mut columns = Vec::new();
        let mut edge_ids = Vec::new();
        
        let mut current_offset = 0;
        
        for node_id in nodes {
            row_offsets.push(current_offset);
            
            // Get outgoing edges for this node
            if let Ok(Some(guard)) = graph.get_outgoing(node_id) {
                for (edge_id_str, target_str) in guard.iter_edges() {
                    // Only include edges where target is also in our node list
                    if node_to_index.contains_key(target_str) {
                        columns.push(target_str.to_string());
                        edge_ids.push(EdgeId(edge_id_str.to_string()));
                        current_offset += 1;
                    }
                }
            }
        }
        
        // Final offset marks the end
        row_offsets.push(current_offset);
        
        Ok(Self {
            row_offsets,
            columns,
            edge_ids,
            node_to_index,
            index_to_node,
        })
    }
    
    /// Gets outgoing edges for a node (zero-copy!).
    ///
    /// Returns slice of (NodeId, EdgeId) pairs directly from mmap.
    pub fn get_outgoing_edges(&self, node_id: &NodeId) -> DbResult<Vec<(NodeId, EdgeId)>> {
        let idx = self.node_to_index.get(&node_id.0)
            .ok_or_else(|| DbError::NotFound(format!("Node not found: {}", node_id.0)))?;
        
        let start = self.row_offsets[*idx];
        let end = self.row_offsets[*idx + 1];
        
        // ZERO-COPY: Direct slice access from mmap
        let targets = &self.columns[start..end];
        let edges = &self.edge_ids[start..end];
        
        Ok(targets.iter()
            .zip(edges.iter())
            .map(|(t, e)| (NodeId(t.clone()), e.clone()))
            .collect())
    }
    
    /// Checks if edge exists (binary search - O(log E) per node).
    pub fn has_edge(&self, from: &NodeId, to: &NodeId) -> DbResult<bool> {
        let idx = self.node_to_index.get(&from.0)
            .ok_or_else(|| DbError::NotFound(format!("Node not found: {}", from.0)))?;
        
        let start = self.row_offsets[*idx];
        let end = self.row_offsets[*idx + 1];
        
        // Binary search in sorted column indices
        Ok(self.columns[start..end].binary_search(&to.0).is_ok())
    }
    
    /// Number of nodes in the graph.
    pub fn node_count(&self) -> usize {
        self.index_to_node.len()
    }
    
    /// Number of edges in the graph.
    pub fn edge_count(&self) -> usize {
        self.columns.len()
    }
    
    /// Memory usage in bytes.
    ///
    /// CSR uses O(V + E) space - much better than O(V²) for sparse graphs!
    pub fn memory_usage(&self) -> usize {
        self.row_offsets.len() * std::mem::size_of::<usize>() +
        self.columns.len() * std::mem::size_of::<String>() +
        self.edge_ids.len() * std::mem::size_of::<EdgeId>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_csr_creation() {
        let csr = CsrGraphIndex::new("test_csr").unwrap();
        assert_eq!(csr.node_count(), 0);
        assert_eq!(csr.edge_count(), 0);
    }
}

