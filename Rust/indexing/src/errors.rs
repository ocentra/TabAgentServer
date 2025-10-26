//! Enhanced error types for the indexing crate.
//!
//! This module provides more granular error types for indexing operations
//! following the Rust Architecture Guidelines and thiserror patterns.

use thiserror::Error;
use common::errors::{DatabaseError, GraphError, VectorError, IndexError};

/// Enhanced error types for indexing operations.
#[derive(Debug, Error)]
pub enum IndexingError {
    /// Database error.
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
    
    /// Graph error.
    #[error("Graph error: {0}")]
    Graph(#[from] GraphError),
    
    /// Vector error.
    #[error("Vector error: {0}")]
    Vector(#[from] VectorError),
    
    /// Index error.
    #[error("Index error: {0}")]
    Index(#[from] IndexError),
    
    /// Structural index error.
    #[error("Structural index error: {0}")]
    Structural(String),
    
    /// Graph index error.
    #[error("Graph index error: {0}")]
    GraphIndex(String),
    
    /// Vector index error.
    #[error("Vector index error: {0}")]
    VectorIndex(String),
    
    /// Hybrid index error.
    #[error("Hybrid index error: {0}")]
    Hybrid(String),
    
    /// Batch operation error.
    #[error("Batch operation error: {0}")]
    Batch(String),
    
    /// Persistence error.
    #[error("Persistence error: {0}")]
    Persistence(String),
    
    /// Lock poisoning error.
    #[error("Lock poisoning error: {0}")]
    LockPoisoning(String),
    
    /// Concurrency error.
    #[error("Concurrency error: {0}")]
    Concurrency(String),
    
    /// Invalid operation.
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    
    /// Resource exhaustion.
    #[error("Resource exhaustion: {0}")]
    ResourceExhaustion(String),
    
    /// Configuration error.
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    /// Other indexing error.
    #[error("Other indexing error: {0}")]
    Other(String),
}

impl Clone for IndexingError {
    fn clone(&self) -> Self {
        match self {
            IndexingError::Database(e) => IndexingError::Database(e.clone()),
            IndexingError::Graph(e) => IndexingError::Graph(e.clone()),
            IndexingError::Vector(e) => IndexingError::Vector(e.clone()),
            IndexingError::Index(e) => IndexingError::Index(e.clone()),
            IndexingError::Structural(s) => IndexingError::Structural(s.clone()),
            IndexingError::GraphIndex(s) => IndexingError::GraphIndex(s.clone()),
            IndexingError::VectorIndex(s) => IndexingError::VectorIndex(s.clone()),
            IndexingError::Hybrid(s) => IndexingError::Hybrid(s.clone()),
            IndexingError::Batch(s) => IndexingError::Batch(s.clone()),
            IndexingError::Persistence(s) => IndexingError::Persistence(s.clone()),
            IndexingError::LockPoisoning(s) => IndexingError::LockPoisoning(s.clone()),
            IndexingError::Concurrency(s) => IndexingError::Concurrency(s.clone()),
            IndexingError::InvalidOperation(s) => IndexingError::InvalidOperation(s.clone()),
            IndexingError::ResourceExhaustion(s) => IndexingError::ResourceExhaustion(s.clone()),
            IndexingError::Configuration(s) => IndexingError::Configuration(s.clone()),
            IndexingError::Other(s) => IndexingError::Other(s.clone()),
        }
    }
}

/// Result type for indexing operations.
pub type IndexingResult<T> = std::result::Result<T, IndexingError>;

/// Enhanced error types for structural index operations.
#[derive(Debug, Error, Clone)]
pub enum StructuralIndexError {
    /// Invalid property name.
    #[error("Invalid property name: {0}")]
    InvalidProperty(String),
    
    /// Invalid value.
    #[error("Invalid value: {0}")]
    InvalidValue(String),
    
    /// Property not found.
    #[error("Property not found: {0}")]
    PropertyNotFound(String),
    
    /// Value not found.
    #[error("Value not found: {0}")]
    ValueNotFound(String),
    
    /// Database error.
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
    
    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    /// Other structural index error.
    #[error("Other structural index error: {0}")]
    Other(String),
}

/// Enhanced error types for graph index operations.
#[derive(Debug, Error, Clone)]
pub enum GraphIndexError {
    /// Node not found.
    #[error("Node not found: {0}")]
    NodeNotFound(String),
    
    /// Edge not found.
    #[error("Edge not found: {0}")]
    EdgeNotFound(String),
    
    /// Invalid node ID.
    #[error("Invalid node ID: {0}")]
    InvalidNodeId(String),
    
    /// Invalid edge ID.
    #[error("Invalid edge ID: {0}")]
    InvalidEdgeId(String),
    
    /// Database error.
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
    
    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    /// Other graph index error.
    #[error("Other graph index error: {0}")]
    Other(String),
}

/// Enhanced error types for vector index operations.
#[derive(Debug, Error, Clone)]
pub enum VectorIndexError {
    /// Invalid vector dimension.
    #[error("Invalid vector dimension: expected {expected}, got {actual}")]
    InvalidDimension { expected: usize, actual: usize },
    
    /// Vector not found.
    #[error("Vector not found: {0}")]
    VectorNotFound(String),
    
    /// Invalid distance metric.
    #[error("Invalid distance metric: {0}")]
    InvalidDistanceMetric(String),
    
    /// Index error.
    #[error("Index error: {0}")]
    Index(String),
    
    /// Search error.
    #[error("Search error: {0}")]
    Search(String),
    
    /// Invalid query vector.
    #[error("Invalid query vector: {0}")]
    InvalidQuery(String),
    
    /// Database error.
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
    
    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    /// Resource exhaustion.
    #[error("Resource exhaustion: {0}")]
    ResourceExhaustion(String),
    
    /// Other vector index error.
    #[error("Other vector index error: {0}")]
    Other(String),
}

/// Enhanced error types for hybrid index operations.
#[derive(Debug, Error, Clone)]
pub enum HybridIndexError {
    /// Hot graph error.
    #[error("Hot graph error: {0}")]
    HotGraph(#[from] GraphError),
    
    /// Hot vector error.
    #[error("Hot vector error: {0}")]
    HotVector(#[from] VectorError),
    
    /// Index error.
    #[error("Index error: {0}")]
    Index(#[from] IndexError),
    
    /// Database error.
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
    
    /// Invalid operation.
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    
    /// Resource exhaustion.
    #[error("Resource exhaustion: {0}")]
    ResourceExhaustion(String),
    
    /// Other hybrid index error.
    #[error("Other hybrid index error: {0}")]
    Other(String),
}

/// Enhanced error types for batch operations.
#[derive(Debug, Error, Clone)]
pub enum BatchError {
    /// Invalid batch size.
    #[error("Invalid batch size: {0}")]
    InvalidBatchSize(usize),
    
    /// Batch operation failed.
    #[error("Batch operation failed: {0}")]
    BatchOperationFailed(String),
    
    /// Partial batch failure.
    #[error("Partial batch failure: {0}")]
    PartialBatchFailure(String),
    
    /// Database error.
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
    
    /// Graph error.
    #[error("Graph error: {0}")]
    Graph(#[from] GraphError),
    
    /// Vector error.
    #[error("Vector error: {0}")]
    Vector(#[from] VectorError),
    
    /// Index error.
    #[error("Index error: {0}")]
    Index(#[from] IndexError),
    
    /// Other batch error.
    #[error("Other batch error: {0}")]
    Other(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_indexing_error() {
        let error = IndexingError::InvalidOperation("test".to_string());
        assert_eq!(format!("{}", error), "Invalid operation: test");
    }
    
    #[test]
    fn test_structural_index_error() {
        let error = StructuralIndexError::InvalidProperty("test".to_string());
        assert_eq!(format!("{}", error), "Invalid property name: test");
    }
    
    #[test]
    fn test_graph_index_error() {
        let error = GraphIndexError::NodeNotFound("test".to_string());
        assert_eq!(format!("{}", error), "Node not found: test");
    }
    
    #[test]
    fn test_vector_index_error() {
        let error = VectorIndexError::InvalidDimension { expected: 3, actual: 2 };
        assert_eq!(format!("{}", error), "Invalid vector dimension: expected 3, got 2");
    }
    
    #[test]
    fn test_hybrid_index_error() {
        let error = HybridIndexError::InvalidOperation("test".to_string());
        assert_eq!(format!("{}", error), "Invalid operation: test");
    }
    
    #[test]
    fn test_batch_error() {
        let error = BatchError::InvalidBatchSize(0);
        assert_eq!(format!("{}", error), "Invalid batch size: 0");
    }
}