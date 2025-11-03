//! Tests for the GraphIndex implementation - TRUE ZERO-COPY!

use indexing::IndexManager;
use tempfile::TempDir;
use common::{NodeId, EdgeId};
use common::models::Edge;

fn create_test_manager() -> (IndexManager, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let manager = IndexManager::new(temp_dir.path()).unwrap();
    (manager, temp_dir)
}

fn create_test_edge(id: &str, from: &str, to: &str) -> Edge {
    Edge {
        id: EdgeId::from(id),
        from_node: NodeId::from(from),
        to_node: NodeId::from(to),
        edge_type: "TEST".to_string(),
        created_at: 1697500000000,
        metadata: "{}".to_string(),
    }
}

#[test]
fn test_add_and_get_outgoing_zero_copy() {
    let (manager, _temp) = create_test_manager();
    let index = manager.graph();
    
    let edge1 = create_test_edge("e1", "chat_1", "msg_1");
    let edge2 = create_test_edge("e2", "chat_1", "msg_2");
    
    index.add_edge(&edge1).unwrap();
    index.add_edge(&edge2).unwrap();
    
    // TRUE ZERO-COPY READ!
    let guard = index.get_outgoing("chat_1").unwrap().expect("Should have results");
    assert_eq!(guard.len(), 2);
    
    // Zero-copy string iteration!
    let strs: Vec<&str> = guard.iter_strs().collect();
    assert!(strs.contains(&"e1"));
    assert!(strs.contains(&"e2"));
}

#[test]
fn test_add_and_get_incoming_zero_copy() {
    let (manager, _temp) = create_test_manager();
    let index = manager.graph();
    
    let edge1 = create_test_edge("e1", "chat_1", "msg_1");
    let edge2 = create_test_edge("e2", "chat_2", "msg_1");
    
    index.add_edge(&edge1).unwrap();
    index.add_edge(&edge2).unwrap();
    
    let guard = index.get_incoming("msg_1").unwrap().expect("Should have results");
    assert_eq!(guard.len(), 2);
    assert!(guard.contains_str("e1"));
    assert!(guard.contains_str("e2"));
}

#[test]
fn test_bidirectional_index() {
    let (manager, _temp) = create_test_manager();
    let index = manager.graph();
    
    let edge = create_test_edge("e1", "chat_1", "msg_1");
    index.add_edge(&edge).unwrap();
    
    // Check outgoing from chat
    let outgoing = index.get_outgoing("chat_1").unwrap().expect("Should have results");
    assert_eq!(outgoing.len(), 1);
    
    // Check incoming to message
    let incoming = index.get_incoming("msg_1").unwrap().expect("Should have results");
    assert_eq!(incoming.len(), 1);
}

#[test]
fn test_remove_edge() {
    let (manager, _temp) = create_test_manager();
    let index = manager.graph();
    
    let edge = create_test_edge("e1", "chat_1", "msg_1");
    index.add_edge(&edge).unwrap();
    index.remove_edge(&edge).unwrap();
    
    let outgoing = index.get_outgoing("chat_1").unwrap();
    assert!(outgoing.is_none());
    
    let incoming = index.get_incoming("msg_1").unwrap();
    assert!(incoming.is_none());
}

#[test]
fn test_count_zero_copy() {
    let (manager, _temp) = create_test_manager();
    let index = manager.graph();
    
    let edge1 = create_test_edge("e1", "chat_1", "msg_1");
    let edge2 = create_test_edge("e2", "chat_1", "msg_2");
    
    index.add_edge(&edge1).unwrap();
    index.add_edge(&edge2).unwrap();
    
    // TRUE ZERO-COPY COUNT - O(1)!
    assert_eq!(index.count_outgoing("chat_1").unwrap(), 2);
    assert_eq!(index.count_incoming("msg_1").unwrap(), 1);
}
