//! 🕸️  GRAPH INDEX TESTS - TRUE ZERO-COPY MDBX!

use crate::common::{setup_real_db, test_edge};

#[test]
fn test_add_and_get_outgoing_zero_copy() {
    println!("\n🕸️  TEST: Add edges and retrieve outgoing edges with zero-copy");
    let (manager, _temp, _storage) = setup_real_db();
    let index = manager.graph();
    
    let edge1 = test_edge("e1", "chat_1", "msg_1");
    let edge2 = test_edge("e2", "chat_1", "msg_2");
    
    println!("   📝 Adding 2 edges from chat_1...");
    index.add_edge_with_struct(&edge1).unwrap();
    index.add_edge_with_struct(&edge2).unwrap();
    
    println!("   📖 Reading outgoing edges (ZERO-COPY)...");
    let guard = index.get_outgoing("chat_1").unwrap().expect("Should have results");
    assert_eq!(guard.len(), 2);
    
    let edge_ids: Vec<&str> = guard.iter_edge_ids().collect();
    assert!(edge_ids.contains(&"e1"));
    assert!(edge_ids.contains(&"e2"));
    println!("   ✅ PASS: Retrieved {} edges with zero-copy", guard.len());
}

#[test]
fn test_add_and_get_incoming_zero_copy() {
    println!("\n🕸️  TEST: Add edges and retrieve incoming edges with zero-copy");
    let (manager, _temp, _storage) = setup_real_db();
    let index = manager.graph();
    
    let edge1 = test_edge("e1", "chat_1", "msg_1");
    let edge2 = test_edge("e2", "chat_2", "msg_1");
    
    println!("   📝 Adding 2 edges to msg_1...");
    index.add_edge_with_struct(&edge1).unwrap();
    index.add_edge_with_struct(&edge2).unwrap();
    
    println!("   📖 Reading incoming edges (ZERO-COPY)...");
    let guard = index.get_incoming("msg_1").unwrap().expect("Should have results");
    assert_eq!(guard.len(), 2);
    assert!(guard.contains_edge("e1"));
    assert!(guard.contains_edge("e2"));
    println!("   ✅ PASS: Retrieved {} incoming edges", guard.len());
}

#[test]
fn test_bidirectional_index() {
    println!("\n🔄 TEST: Verify both outgoing AND incoming indexes updated atomically");
    let (manager, _temp, _storage) = setup_real_db();
    let index = manager.graph();
    
    let edge = test_edge("e1", "chat_1", "msg_1");
    println!("   📝 Adding edge: chat_1 -> msg_1");
    index.add_edge_with_struct(&edge).unwrap();
    
    println!("   📖 Checking outgoing from chat_1...");
    let outgoing = index.get_outgoing("chat_1").unwrap().expect("Should have results");
    assert_eq!(outgoing.len(), 1);
    
    println!("   📖 Checking incoming to msg_1...");
    let incoming = index.get_incoming("msg_1").unwrap().expect("Should have results");
    assert_eq!(incoming.len(), 1);
    println!("   ✅ PASS: Bidirectional index works atomically");
}

#[test]
fn test_remove_edge() {
    println!("\n🗑️  TEST: Remove edge and verify it's gone from both directions");
    let (manager, _temp, _storage) = setup_real_db();
    let index = manager.graph();
    
    let edge = test_edge("e1", "chat_1", "msg_1");
    println!("   📝 Adding edge...");
    index.add_edge_with_struct(&edge).unwrap();
    
    println!("   🗑️  Removing edge...");
    index.remove_edge(&edge).unwrap();
    
    let outgoing = index.get_outgoing("chat_1").unwrap();
    assert!(outgoing.is_none());
    
    let incoming = index.get_incoming("msg_1").unwrap();
    assert!(incoming.is_none());
    println!("   ✅ PASS: Edge removed from both directions");
}

#[test]
fn test_count_zero_copy() {
    println!("\n🔢 TEST: Zero-copy edge counting (O(1) performance)");
    let (manager, _temp, _storage) = setup_real_db();
    let index = manager.graph();
    
    let edge1 = test_edge("e1", "chat_1", "msg_1");
    let edge2 = test_edge("e2", "chat_1", "msg_2");
    
    println!("   📝 Adding 2 edges...");
    index.add_edge_with_struct(&edge1).unwrap();
    index.add_edge_with_struct(&edge2).unwrap();
    
    println!("   🔢 Counting edges (ZERO-COPY O(1))...");
    let outgoing = index.count_outgoing("chat_1").unwrap();
    let incoming = index.count_incoming("msg_1").unwrap();
    assert_eq!(outgoing, 2);
    assert_eq!(incoming, 1);
    println!("   ✅ PASS: Counted {} outgoing, {} incoming", outgoing, incoming);
}
