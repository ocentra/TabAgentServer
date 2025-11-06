//! Batch operations for optimized insertions and searches.
//!
//! This module provides batch processing capabilities for vector and graph operations,
//! allowing for more efficient bulk operations compared to individual operations.
//! These implementations follow the Rust Architecture Guidelines for safety,
//! performance, and clarity.

use crate::lock_free_hot_vector::LockFreeHotVectorIndex;
use crate::lock_free_hot_graph::LockFreeHotGraphIndex;
use common::DbResult;
use std::collections::HashMap;
use std::sync::Arc;

/// A batch operation for vector indexing.
#[derive(Debug, Clone)]
pub enum VectorBatchOperation {
    /// Add a vector to the index
    Add {
        /// The vector ID
        id: String,
        /// The vector data
        vector: Vec<f32>,
    },
    
    /// Remove a vector from the index
    Remove {
        /// The vector ID
        id: String,
    },
}

/// A batch operation for graph indexing.
#[derive(Debug, Clone)]
pub enum GraphBatchOperation {
    /// Add a node to the graph
    AddNode {
        /// The node ID
        id: String,
        /// Optional node metadata
        metadata: Option<String>,
    },
    
    /// Remove a node from the graph
    RemoveNode {
        /// The node ID
        id: String,
    },
    
    /// Add an edge to the graph
    AddEdge {
        /// The source node ID
        from: String,
        /// The target node ID
        to: String,
        /// Optional edge weight
        weight: Option<f32>,
    },
    
    /// Remove an edge from the graph
    RemoveEdge {
        /// The source node ID
        from: String,
        /// The target node ID
        to: String,
    },
}

/// A batch processor for vector operations.
pub struct VectorBatchProcessor {
    /// The underlying vector index
    index: Arc<LockFreeHotVectorIndex>,
}

impl VectorBatchProcessor {
    /// Creates a new vector batch processor.
    pub fn new(index: Arc<LockFreeHotVectorIndex>) -> Self {
        Self { index }
    }
    
    /// Processes a batch of vector operations.
    ///
    /// This method processes all operations in the batch and returns the number
    /// of successful operations.
    ///
    /// # Arguments
    ///
    /// * `operations` - A vector of batch operations to process
    ///
    /// # Returns
    ///
    /// The number of successful operations
    pub fn process_batch(&self, operations: Vec<VectorBatchOperation>) -> DbResult<usize> {
        let mut success_count = 0;
        
        for operation in operations {
            match operation {
                VectorBatchOperation::Add { id, vector } => {
                    if self.index.add_vector(&id, vector).is_ok() {
                        success_count += 1;
                    }
                }
                VectorBatchOperation::Remove { id } => {
                    if self.index.remove_vector(&id).is_ok() {
                        success_count += 1;
                    }
                }
            }
        }
        
        Ok(success_count)
    }
    
    /// Processes a batch of vector additions.
    ///
    /// This method is optimized for adding multiple vectors and can be more
    /// efficient than processing individual Add operations.
    ///
    /// # Arguments
    ///
    /// * `vectors` - A map of vector IDs to vector data
    ///
    /// # Returns
    ///
    /// The number of successful additions
    pub fn add_vectors(&self, vectors: HashMap<String, Vec<f32>>) -> DbResult<usize> {
        let mut success_count = 0;
        
        for (id, vector) in vectors {
            if self.index.add_vector(&id, vector).is_ok() {
                success_count += 1;
            }
        }
        
        Ok(success_count)
    }
    
    /// Processes a batch of vector removals.
    ///
    /// This method is optimized for removing multiple vectors and can be more
    /// efficient than processing individual Remove operations.
    ///
    /// # Arguments
    ///
    /// * `ids` - A vector of vector IDs to remove
    ///
    /// # Returns
    ///
    /// The number of successful removals
    pub fn remove_vectors(&self, ids: Vec<String>) -> DbResult<usize> {
        let mut success_count = 0;
        
        for id in ids {
            if self.index.remove_vector(&id).is_ok() {
                success_count += 1;
            }
        }
        
        Ok(success_count)
    }
}

/// A batch processor for graph operations.
pub struct GraphBatchProcessor {
    /// The underlying graph index
    index: Arc<LockFreeHotGraphIndex>,
}

impl GraphBatchProcessor {
    /// Creates a new graph batch processor.
    pub fn new(index: Arc<LockFreeHotGraphIndex>) -> Self {
        Self { index }
    }
    
    /// Processes a batch of graph operations.
    ///
    /// This method processes all operations in the batch and returns the number
    /// of successful operations.
    ///
    /// # Arguments
    ///
    /// * `operations` - A vector of batch operations to process
    ///
    /// # Returns
    ///
    /// The number of successful operations
    pub fn process_batch(&self, operations: Vec<GraphBatchOperation>) -> DbResult<usize> {
        let mut success_count = 0;
        
        for operation in operations {
            match operation {
                GraphBatchOperation::AddNode { id, metadata } => {
                    if self.index.add_node(&id, metadata.as_deref()).is_ok() {
                        success_count += 1;
                    }
                }
                GraphBatchOperation::RemoveNode { id } => {
                    if self.index.remove_node(&id).is_ok() {
                        success_count += 1;
                    }
                }
                GraphBatchOperation::AddEdge { from, to, weight } => {
                    let result = if let Some(w) = weight {
                        self.index.add_edge_with_weight(&from, &to, w)
                    } else {
                        self.index.add_edge(&from, &to)
                    };
                    
                    if result.is_ok() {
                        success_count += 1;
                    }
                }
                GraphBatchOperation::RemoveEdge { from, to } => {
                    if self.index.remove_edge(&from, &to).is_ok() {
                        success_count += 1;
                    }
                }
            }
        }
        
        Ok(success_count)
    }
    
    /// Processes a batch of node additions.
    ///
    /// This method is optimized for adding multiple nodes and can be more
    /// efficient than processing individual AddNode operations.
    ///
    /// # Arguments
    ///
    /// * `nodes` - A map of node IDs to optional metadata
    ///
    /// # Returns
    ///
    /// The number of successful additions
    pub fn add_nodes(&self, nodes: HashMap<String, Option<String>>) -> DbResult<usize> {
        let mut success_count = 0;
        
        for (id, metadata) in nodes {
            if self.index.add_node(&id, metadata.as_deref()).is_ok() {
                success_count += 1;
            }
        }
        
        Ok(success_count)
    }
    
    /// Processes a batch of node removals.
    ///
    /// This method is optimized for removing multiple nodes and can be more
    /// efficient than processing individual RemoveNode operations.
    ///
    /// # Arguments
    ///
    /// * `ids` - A vector of node IDs to remove
    ///
    /// # Returns
    ///
    /// The number of successful removals
    pub fn remove_nodes(&self, ids: Vec<String>) -> DbResult<usize> {
        let mut success_count = 0;
        
        for id in ids {
            if self.index.remove_node(&id).is_ok() {
                success_count += 1;
            }
        }
        
        Ok(success_count)
    }
    
    /// Processes a batch of edge additions.
    ///
    /// This method is optimized for adding multiple edges and can be more
    /// efficient than processing individual AddEdge operations.
    ///
    /// # Arguments
    ///
    /// * `edges` - A vector of (from, to, weight) tuples
    ///
    /// # Returns
    ///
    /// The number of successful additions
    pub fn add_edges(&self, edges: Vec<(String, String, Option<f32>)>) -> DbResult<usize> {
        let mut success_count = 0;
        
        for (from, to, weight) in edges {
            let result = if let Some(w) = weight {
                self.index.add_edge_with_weight(&from, &to, w)
            } else {
                self.index.add_edge(&from, &to)
            };
            
            if result.is_ok() {
                success_count += 1;
            }
        }
        
        Ok(success_count)
    }
    
    /// Processes a batch of edge removals.
    ///
    /// This method is optimized for removing multiple edges and can be more
    /// efficient than processing individual RemoveEdge operations.
    ///
    /// # Arguments
    ///
    /// * `edges` - A vector of (from, to) tuples
    ///
    /// # Returns
    ///
    /// The number of successful removals
    pub fn remove_edges(&self, edges: Vec<(String, String)>) -> DbResult<usize> {
        let mut success_count = 0;
        
        for (from, to) in edges {
            if self.index.remove_edge(&from, &to).is_ok() {
                success_count += 1;
            }
        }
        
        Ok(success_count)
    }
}

/// A combined batch processor for both vector and graph operations.
pub struct CombinedBatchProcessor {
    /// The vector batch processor
    vector_processor: VectorBatchProcessor,
    /// The graph batch processor
    graph_processor: GraphBatchProcessor,
}

impl CombinedBatchProcessor {
    /// Creates a new combined batch processor.
    pub fn new(
        vector_index: Arc<LockFreeHotVectorIndex>,
        graph_index: Arc<LockFreeHotGraphIndex>,
    ) -> Self {
        Self {
            vector_processor: VectorBatchProcessor::new(vector_index),
            graph_processor: GraphBatchProcessor::new(graph_index),
        }
    }
    
    /// Processes batches of vector and graph operations.
    ///
    /// This method processes both vector and graph operations and returns the
    /// number of successful operations for each type.
    ///
    /// # Arguments
    ///
    /// * `vector_operations` - A vector of vector batch operations
    /// * `graph_operations` - A vector of graph batch operations
    ///
    /// # Returns
    ///
    /// A tuple of (successful vector operations, successful graph operations)
    pub fn process_batches(
        &self,
        vector_operations: Vec<VectorBatchOperation>,
        graph_operations: Vec<GraphBatchOperation>,
    ) -> DbResult<(usize, usize)> {
        let vector_success = self.vector_processor.process_batch(vector_operations)?;
        let graph_success = self.graph_processor.process_batch(graph_operations)?;
        Ok((vector_success, graph_success))
    }
}
