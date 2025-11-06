//! 📦 BATCH PROCESSING TESTS - Bulk Operations

use indexing::utils::batch::{
    VectorBatchProcessor, GraphBatchProcessor, CombinedBatchProcessor,
    VectorBatchOperation, GraphBatchOperation,
};
use indexing::{LockFreeHotVectorIndex, LockFreeHotGraphIndex};
use std::sync::Arc;
use std::collections::HashMap;

#[test]
fn test_vector_batch_processor() {
    println!("\n📦 TEST: Vector batch processor (bulk add/remove)");
    let index = Arc::new(LockFreeHotVectorIndex::new());
    let processor = VectorBatchProcessor::new(index.clone());
    
    println!("   📝 Batch adding 2 vectors...");
    let mut vectors = HashMap::new();
    vectors.insert("vec1".to_string(), vec![1.0, 0.0, 0.0]);
    vectors.insert("vec2".to_string(), vec![0.0, 1.0, 0.0]);
    
    let result = processor.add_vectors(vectors).unwrap();
    assert_eq!(result, 2);
    assert_eq!(index.len(), 2);
    
    println!("   🗑️  Batch removing 2 vectors...");
    let ids = vec!["vec1".to_string(), "vec2".to_string()];
    let result = processor.remove_vectors(ids).unwrap();
    assert_eq!(result, 2);
    assert_eq!(index.len(), 0);
    println!("   ✅ PASS: Batch add/remove works (2 vectors)");
}

#[test]
fn test_graph_batch_processor() {
    println!("\n📦 TEST: Graph batch processor (bulk nodes/edges)");
    let index = Arc::new(LockFreeHotGraphIndex::new());
    let processor = GraphBatchProcessor::new(index.clone());
    
    println!("   📝 Batch adding 2 nodes...");
    let mut nodes = HashMap::new();
    nodes.insert("node1".to_string(), Some("metadata1".to_string()));
    nodes.insert("node2".to_string(), None);
    
    let result = processor.add_nodes(nodes).unwrap();
    assert_eq!(result, 2);
    assert_eq!(index.node_count(), 2);
    
    println!("   📝 Batch adding 2 edges...");
    let edges = vec![
        ("node1".to_string(), "node2".to_string(), Some(1.5)),
        ("node2".to_string(), "node1".to_string(), None),
    ];
    
    let result = processor.add_edges(edges).unwrap();
    assert_eq!(result, 2);
    
    println!("   🗑️  Batch removing 2 edges...");
    let edges = vec![
        ("node1".to_string(), "node2".to_string()),
        ("node2".to_string(), "node1".to_string()),
    ];
    
    let result = processor.remove_edges(edges).unwrap();
    assert_eq!(result, 2);
    
    println!("   🗑️  Batch removing 2 nodes...");
    let ids = vec!["node1".to_string(), "node2".to_string()];
    let result = processor.remove_nodes(ids).unwrap();
    assert_eq!(result, 2);
    assert_eq!(index.node_count(), 0);
    println!("   ✅ PASS: Batch graph operations work (2 nodes, 2 edges)");
}

#[test]
fn test_combined_batch_processor() {
    println!("\n📦 TEST: Combined batch processor (vectors + graph)");
    let vector_index = Arc::new(LockFreeHotVectorIndex::new());
    let graph_index = Arc::new(LockFreeHotGraphIndex::new());
    let processor = CombinedBatchProcessor::new(vector_index, graph_index);
    
    println!("   📝 Batch processing vectors + graph ops...");
    let vector_ops = vec![
        VectorBatchOperation::Add {
            id: "vec1".to_string(),
            vector: vec![1.0, 0.0, 0.0],
        },
        VectorBatchOperation::Add {
            id: "vec2".to_string(),
            vector: vec![0.0, 1.0, 0.0],
        },
    ];
    
    let graph_ops = vec![
        GraphBatchOperation::AddNode {
            id: "node1".to_string(),
            metadata: Some("metadata1".to_string()),
        },
        GraphBatchOperation::AddNode {
            id: "node2".to_string(),
            metadata: None,
        },
    ];
    
    let (vector_success, graph_success) = processor.process_batches(vector_ops, graph_ops).unwrap();
    assert_eq!(vector_success, 2);
    assert_eq!(graph_success, 2);
    println!("   ✅ PASS: Combined batch works (2 vectors, 2 nodes)");
}

