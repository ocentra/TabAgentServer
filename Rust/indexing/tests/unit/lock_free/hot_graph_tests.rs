//! 🕸️  LOCK-FREE HOT GRAPH INDEX TESTS - Concurrent Graph Operations

use indexing::lock_free::lock_free_hot_graph::LockFreeHotGraphIndex;
use std::sync::Arc;
use std::thread;

#[test]
fn test_lock_free_hot_graph_index_basic() {
    println!("\n🕸️  TEST: Lock-free hot graph index basic operations");
    let graph = LockFreeHotGraphIndex::new();
    
    println!("   📝 Adding node...");
    assert!(graph.add_node("node1", None).is_ok());
    assert_eq!(graph.node_count(), 1);
    
    println!("   📝 Adding edge...");
    assert!(graph.add_edge("node1", "node2").is_ok());
    
    println!("   📖 Getting neighbors...");
    let neighbors = graph.get_outgoing_neighbors("node1").unwrap();
    assert!(!neighbors.is_empty());
    println!("   ✅ PASS: Graph has {} nodes", graph.node_count());
}

#[test]
fn test_lock_free_hot_graph_index_concurrent() {
    println!("\n🕸️  TEST: Lock-free hot graph index concurrent adds");
    let graph = Arc::new(LockFreeHotGraphIndex::new());
    let mut handles = vec![];
    
    println!("   🧵 Spawning 10 threads...");
    for i in 0..10 {
        let graph_clone = Arc::clone(&graph);
        let handle = thread::spawn(move || {
            graph_clone.add_node(&format!("node{}", i), None).unwrap();
        });
        handles.push(handle);
    }
    
    println!("   ⏳ Waiting for threads...");
    for handle in handles {
        handle.join().unwrap();
    }
    
    assert_eq!(graph.node_count(), 10);
    println!("   ✅ PASS: {} concurrent node adds successful", graph.node_count());
}

