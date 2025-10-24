//! Tests for the common crate
//!
//! This file contains unit tests for the common crate functionality,
//! including newtype wrappers, error types, and utility functions.

use common::{NodeId, EdgeId, EmbeddingId, DbError};

#[test]
fn test_newtype_wrappers() {
    // Test creation
    let node_id = NodeId::new("node_123");
    let edge_id = EdgeId::new("edge_456");
    let embedding_id = EmbeddingId::new("embed_789");

    // Test as_str()
    assert_eq!(node_id.as_str(), "node_123");
    assert_eq!(edge_id.as_str(), "edge_456");
    assert_eq!(embedding_id.as_str(), "embed_789");
    
    // Test Display
    assert_eq!(node_id.to_string(), "node_123");
    assert_eq!(edge_id.to_string(), "edge_456");
    assert_eq!(embedding_id.to_string(), "embed_789");
    
    // Test From<String>
    let node_from_string: NodeId = "test".to_string().into();
    assert_eq!(node_from_string.as_str(), "test");
    
    // Test From<&str>
    let edge_from_str: EdgeId = "test2".into();
    assert_eq!(edge_from_str.as_str(), "test2");
}

#[test]
fn test_type_safety() {
    // This demonstrates type safety - these won't compile if uncommented:
    // let node_id = NodeId::new("node_123");
    // let edge_id: EdgeId = node_id; // ❌ Compile error!
    // let _mixed: bool = node_id == edge_id; // ❌ Compile error!
    
    // But same-type comparisons work:
    let node1 = NodeId::new("same");
    let node2 = NodeId::new("same");
    assert_eq!(node1, node2);
}

#[test]
fn test_error_display() {
    let err = DbError::NotFound("test_id".to_string());
    assert_eq!(err.to_string(), "Entity not found: test_id");

    let err = DbError::InvalidOperation("test operation".to_string());
    assert_eq!(err.to_string(), "Invalid operation: test operation");
}