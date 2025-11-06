//! 🌐 COMMUNITY DETECTION TESTS - Graph Clustering Algorithms

use crate::common::{setup_real_db, test_edge};
use std::collections::HashSet;

#[test]
fn test_louvain_community_detection() {
    println!("\n🌐 TEST: Louvain community detection algorithm");
    let (manager, _temp, _storage) = setup_real_db();
    let graph = manager.graph();
    
    // Create a simple graph with 2 communities:
    // Community 1: a <-> b <-> c
    // Community 2: x <-> y <-> z
    // Bridge: c -> x
    
    let edges = vec![
        // Community 1
        test_edge("ab", "a", "b"),
        test_edge("ba", "b", "a"),
        test_edge("bc", "b", "c"),
        test_edge("cb", "c", "b"),
        
        // Community 2
        test_edge("xy", "x", "y"),
        test_edge("yx", "y", "x"),
        test_edge("yz", "y", "z"),
        test_edge("zy", "z", "y"),
        
        // Bridge
        test_edge("cx", "c", "x"),
    ];
    
    println!("   📝 Creating graph with 2 communities + bridge...");
    for edge in &edges {
        graph.add_edge_with_struct(edge).unwrap();
    }
    
    println!("   🔍 Running Louvain algorithm...");
    use indexing::algorithms::louvain_zero_copy;
    
    let mut nodes = HashSet::new();
    nodes.insert("a".to_string());
    nodes.insert("b".to_string());
    nodes.insert("c".to_string());
    nodes.insert("x".to_string());
    nodes.insert("y".to_string());
    nodes.insert("z".to_string());
    
    let communities = louvain_zero_copy(&**graph, &nodes, 10).unwrap();
    
    assert!(!communities.is_empty());
    
    let assigned_nodes: usize = communities.values().filter(|&&c| c < 100).count();
    assert_eq!(assigned_nodes, nodes.len());
    println!("   ✅ PASS: Detected communities for {} nodes", assigned_nodes);
}

#[test]
fn test_community_detection_via_index_manager() {
    println!("\n🎯 TEST: Community detection via IndexManager");
    let (manager, _temp, _storage) = setup_real_db();
    
    println!("   📝 Creating graph...");
    let edges = vec![
        test_edge("ab", "a", "b"),
        test_edge("bc", "b", "c"),
    ];
    
    for edge in &edges {
        manager.index_edge(edge).unwrap();
    }
    
    println!("   🔍 Detecting communities...");
    let mut nodes = HashSet::new();
    nodes.insert("a".to_string());
    nodes.insert("b".to_string());
    nodes.insert("c".to_string());
    
    let communities = manager.detect_communities(&nodes, 10).unwrap();
    
    assert!(!communities.is_empty());
    println!("   ✅ PASS: Detected {} communities", communities.len());
}

