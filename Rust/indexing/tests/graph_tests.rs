//! Tests for the GraphIndex implementation

use indexing::graph::GraphIndex;
use tempfile::TempDir;
use serde_json::json;
use common::{NodeId, EdgeId};
use common::models::Edge;

fn create_test_index() -> (GraphIndex, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let db = sled::open(temp_dir.path()).unwrap();
    let outgoing = db.open_tree("test_graph_out").unwrap();
    let incoming = db.open_tree("test_graph_in").unwrap();
    (GraphIndex::new(outgoing, incoming), temp_dir)
}

fn create_test_edge(id: &str, from: &str, to: &str) -> Edge {
    Edge {
        id: EdgeId::from(id),
        from_node: NodeId::from(from),
        to_node: NodeId::from(to),
        edge_type: "TEST".to_string(),
        created_at: 1697500000000,
        metadata: json!({}),
    }
}

#[test]
fn test_add_and_get_outgoing() {
    let (index, _temp) = create_test_index();
    
    let edge1 = create_test_edge("e1", "chat_1", "msg_1");
    let edge2 = create_test_edge("e2", "chat_1", "msg_2");
    
    index.add_edge(&edge1).unwrap();
    index.add_edge(&edge2).unwrap();
    
    let outgoing = index.get_outgoing("chat_1").unwrap();
    assert_eq!(outgoing.len(), 2);
    assert!(outgoing.contains(&EdgeId::from("e1")));
    assert!(outgoing.contains(&EdgeId::from("e2")));
}

#[test]
fn test_add_and_get_incoming() {
    let (index, _temp) = create_test_index();
    
    let edge1 = create_test_edge("e1", "chat_1", "msg_1");
    let edge2 = create_test_edge("e2", "chat_2", "msg_1");
    
    index.add_edge(&edge1).unwrap();
    index.add_edge(&edge2).unwrap();
    
    let incoming = index.get_incoming("msg_1").unwrap();
    assert_eq!(incoming.len(), 2);
    assert!(incoming.contains(&EdgeId::from("e1")));
    assert!(incoming.contains(&EdgeId::from("e2")));
}

#[test]
fn test_bidirectional_index() {
    let (index, _temp) = create_test_index();
    
    let edge = create_test_edge("e1", "chat_1", "msg_1");
    index.add_edge(&edge).unwrap();
    
    // Check outgoing from chat
    let outgoing = index.get_outgoing("chat_1").unwrap();
    assert_eq!(outgoing.len(), 1);
    
    // Check incoming to message
    let incoming = index.get_incoming("msg_1").unwrap();
    assert_eq!(incoming.len(), 1);
}

#[test]
fn test_remove_edge() {
    let (index, _temp) = create_test_index();
    
    let edge = create_test_edge("e1", "chat_1", "msg_1");
    index.add_edge(&edge).unwrap();
    index.remove_edge(&edge).unwrap();
    
    let outgoing = index.get_outgoing("chat_1").unwrap();
    assert_eq!(outgoing.len(), 0);
    
    let incoming = index.get_incoming("msg_1").unwrap();
    assert_eq!(incoming.len(), 0);
}

#[test]
fn test_count() {
    let (index, _temp) = create_test_index();
    
    let edge1 = create_test_edge("e1", "chat_1", "msg_1");
    let edge2 = create_test_edge("e2", "chat_1", "msg_2");
    
    index.add_edge(&edge1).unwrap();
    index.add_edge(&edge2).unwrap();
    
    assert_eq!(index.count_outgoing("chat_1").unwrap(), 2);
    assert_eq!(index.count_incoming("msg_1").unwrap(), 1);
}