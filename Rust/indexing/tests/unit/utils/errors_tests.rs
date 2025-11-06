//! ❌ ERROR TYPE TESTS - Error Formatting & Display

use indexing::errors::{
    IndexingError, StructuralIndexError, GraphIndexError, VectorIndexError,
    HybridIndexError, BatchError,
};

#[test]
fn test_indexing_error() {
    println!("\n❌ TEST: IndexingError display formatting");
    let error = IndexingError::InvalidOperation("test".to_string());
    assert_eq!(format!("{}", error), "Invalid operation: test");
    println!("   ✅ PASS: IndexingError formats correctly");
}

#[test]
fn test_structural_index_error() {
    println!("\n❌ TEST: StructuralIndexError display formatting");
    let error = StructuralIndexError::InvalidProperty("test".to_string());
    assert_eq!(format!("{}", error), "Invalid property name: test");
    println!("   ✅ PASS: StructuralIndexError formats correctly");
}

#[test]
fn test_graph_index_error() {
    println!("\n❌ TEST: GraphIndexError display formatting");
    let error = GraphIndexError::NodeNotFound("test".to_string());
    assert_eq!(format!("{}", error), "Node not found: test");
    println!("   ✅ PASS: GraphIndexError formats correctly");
}

#[test]
fn test_vector_index_error() {
    println!("\n❌ TEST: VectorIndexError display formatting");
    let error = VectorIndexError::InvalidDimension { expected: 3, actual: 2 };
    assert_eq!(format!("{}", error), "Invalid vector dimension: expected 3, got 2");
    println!("   ✅ PASS: VectorIndexError formats correctly");
}

#[test]
fn test_hybrid_index_error() {
    println!("\n❌ TEST: HybridIndexError display formatting");
    let error = HybridIndexError::InvalidOperation("test".to_string());
    assert_eq!(format!("{}", error), "Invalid operation: test");
    println!("   ✅ PASS: HybridIndexError formats correctly");
}

#[test]
fn test_batch_error() {
    println!("\n❌ TEST: BatchError display formatting");
    let error = BatchError::InvalidBatchSize(0);
    assert_eq!(format!("{}", error), "Invalid batch size: 0");
    println!("   ✅ PASS: BatchError formats correctly");
}

