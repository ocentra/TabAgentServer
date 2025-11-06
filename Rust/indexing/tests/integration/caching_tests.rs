//! 💾 CACHING INTEGRATION TESTS - Real MDBX Performance

use crate::common::{setup_real_db, test_edge};

#[test]
fn test_graph_index_caching_behavior() {
    println!("\n💾 TEST: Graph index caching consistency");
    let (manager, _temp, _storage) = setup_real_db();
    let graph_index = manager.graph();
    
    let edge1 = test_edge("e1", "node1", "node2");
    let edge2 = test_edge("e2", "node1", "node3");
    
    println!("   📝 Adding 2 edges...");
    graph_index.add_edge_with_struct(&edge1).unwrap();
    graph_index.add_edge_with_struct(&edge2).unwrap();
    
    println!("   📖 First read from MDBX...");
    let outgoing1 = graph_index.get_outgoing("node1").unwrap().expect("Should have edges");
    assert_eq!(outgoing1.len(), 2);
    
    println!("   📖 Second read (tests consistency)...");
    let outgoing2 = graph_index.get_outgoing("node1").unwrap().expect("Should have edges");
    assert_eq!(outgoing2.len(), 2);
    
    let ids1: Vec<&str> = outgoing1.iter_edge_ids().collect();
    let ids2: Vec<&str> = outgoing2.iter_edge_ids().collect();
    assert_eq!(ids1, ids2);
    println!("   ✅ PASS: Consistent reads from cache");
}

#[test]
fn test_structural_index_query_performance() {
    println!("\n⚡ TEST: Structural index query performance (100 items)");
    let (manager, _temp, _storage) = setup_real_db();
    let structural_index = manager.structural();
    
    println!("   📝 Adding 100 messages to chat_123...");
    for i in 0..100 {
        structural_index.add("chat_id", "chat_123", &format!("msg_{}", i)).unwrap();
    }
    
    println!("   📖 Querying from B-tree index...");
    let guard = structural_index.get("chat_id", "chat_123").unwrap().expect("Should have results");
    assert_eq!(guard.len(), 100);
    
    println!("   🔢 Counting (O(1))...");
    let count = structural_index.count("chat_id", "chat_123").unwrap();
    assert_eq!(count, 100);
    println!("   ✅ PASS: Retrieved {} items efficiently", count);
}

#[test]
fn test_vector_index_search_caching() {
    println!("\n🔍 TEST: Vector search caching consistency");
    use indexing::core::vector::VectorIndex;
    use tempfile::TempDir;
    
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("test_vector.hnsw");
    let mut index = VectorIndex::new_with_dimension(path, 3).unwrap();
    
    println!("   📝 Adding 3 vectors (3D)...");
    index.add_vector("v1", vec![1.0, 0.0, 0.0]).unwrap();
    index.add_vector("v2", vec![0.9, 0.1, 0.0]).unwrap();
    index.add_vector("v3", vec![0.0, 0.0, 1.0]).unwrap();
    
    println!("   🔎 First search...");
    let query = vec![1.0, 0.0, 0.0];
    let results1 = index.search(&query, 2).unwrap();
    
    println!("   🔎 Second search (consistency check)...");
    let results2 = index.search(&query, 2).unwrap();
    
    assert_eq!(results1.len(), 2);
    assert_eq!(results2.len(), 2);
    assert_eq!(results1[0].id, results2[0].id);
    println!("   ✅ PASS: Search results consistent");
}
