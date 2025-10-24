//! Integration tests for the IndexManager

use indexing::IndexManager;
use tempfile::TempDir;
use serde_json::json;
use common::{NodeId, EdgeId, EmbeddingId};
use common::models::{Chat, Message, Edge, Embedding, Node};

fn create_test_manager() -> (IndexManager, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let db = sled::open(temp_dir.path()).unwrap();
    let manager = IndexManager::new(&db).unwrap();
    (manager, temp_dir)
}

#[test]
fn test_index_chat_node() {
    let (manager, _temp) = create_test_manager();
    
    let chat = Node::Chat(Chat {
        id: NodeId::from("chat_001"),
        title: "Test Chat".to_string(),
        topic: "Testing".to_string(),
        created_at: 1697500000000,
        updated_at: 1697500000000,
        message_ids: vec![],
        summary_ids: vec![],
        embedding_id: None,
        metadata: json!({}),
    });
    
    manager.index_node(&chat).unwrap();
    
    let results = manager.get_nodes_by_property("node_type", "Chat").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], NodeId::from("chat_001"));
    
    let topic_results = manager.get_nodes_by_property("topic", "Testing").unwrap();
    assert_eq!(topic_results.len(), 1);
}

#[test]
fn test_index_message_node() {
    let (manager, _temp) = create_test_manager();
    
    let message = Node::Message(Message {
        id: NodeId::from("msg_001"),
        chat_id: NodeId::from("chat_123"),
        sender: "user".to_string(),
        timestamp: 1697500000000,
        text_content: "Hello".to_string(),
        attachment_ids: vec![],
        embedding_id: None,
        metadata: json!({}),
    });
    
    manager.index_node(&message).unwrap();
    
    let results = manager.get_nodes_by_property("chat_id", "chat_123").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], NodeId::from("msg_001"));
}

#[test]
fn test_index_edge() {
    let (manager, _temp) = create_test_manager();
    
    let edge = Edge {
        id: EdgeId::from("edge_001"),
        from_node: NodeId::from("chat_123"),
        to_node: NodeId::from("msg_456"),
        edge_type: "CONTAINS".to_string(),
        created_at: 1697500000000,
        metadata: json!({}),
    };
    
    manager.index_edge(&edge).unwrap();
    
    let outgoing = manager.get_outgoing_edges("chat_123").unwrap();
    assert_eq!(outgoing.len(), 1);
    assert_eq!(outgoing[0], EdgeId::from("edge_001"));
}

#[test]
fn test_index_embedding() {
    let (manager, _temp) = create_test_manager();
    
    let embedding = Embedding {
        id: EmbeddingId::from("embed_001"),
        vector: vec![0.1, 0.2, 0.3],
        model: "test-model".to_string(),
    };
    
    manager.index_embedding(&embedding).unwrap();
    
    let query = vec![0.1, 0.2, 0.3];
    let results = manager.search_vectors(&query, 5).unwrap();
    assert!(!results.is_empty());
}