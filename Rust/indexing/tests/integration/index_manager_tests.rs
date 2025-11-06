//! 🎯 INDEX MANAGER TESTS - Coordinating All Indexes

use crate::common::setup_real_db;
use common::{NodeId, EdgeId, EmbeddingId};
use common::models::{Chat, Message, Edge, Embedding, Node};

#[test]
fn test_index_chat_node_zero_copy() {
    println!("\n💬 TEST: Index Chat node across all indexes");
    let (manager, _temp, _storage) = setup_real_db();
    
    let chat = Node::Chat(Chat {
        id: NodeId::from("chat_001"),
        title: "Test Chat".to_string(),
        topic: "Testing".to_string(),
        created_at: 1697500000000,
        updated_at: 1697500000000,
        message_ids: vec![],
        summary_ids: vec![],
        embedding_id: None,
        metadata: "{}".to_string(),
    });
    
    println!("   📝 Indexing chat node...");
    manager.index_node(&chat).unwrap();
    
    println!("   📖 Querying by node_type (ZERO-COPY)...");
    let guard = manager.get_nodes_by_property("node_type", "Chat").unwrap().expect("Should have results");
    assert_eq!(guard.len(), 1);
    assert!(guard.contains_str("chat_001"));
    
    println!("   📖 Querying by topic (ZERO-COPY)...");
    let topic_guard = manager.get_nodes_by_property("topic", "Testing").unwrap().expect("Should have results");
    assert_eq!(topic_guard.len(), 1);
    println!("   ✅ PASS: Chat indexed across all properties");
}

#[test]
fn test_index_message_node_zero_copy() {
    println!("\n✉️  TEST: Index Message node across all indexes");
    let (manager, _temp, _storage) = setup_real_db();
    
    let message = Node::Message(Message {
        id: NodeId::from("msg_001"),
        chat_id: NodeId::from("chat_123"),
        sender: "user".to_string(),
        timestamp: 1697500000000,
        text_content: "Hello".to_string(),
        attachment_ids: vec![],
        embedding_id: None,
        metadata: "{}".to_string(),
    });
    
    println!("   📝 Indexing message node...");
    manager.index_node(&message).unwrap();
    
    println!("   📖 Querying by chat_id (ZERO-COPY)...");
    let guard = manager.get_nodes_by_property("chat_id", "chat_123").unwrap().expect("Should have results");
    assert_eq!(guard.len(), 1);
    
    let strs: Vec<&str> = guard.iter_strs().collect();
    assert_eq!(strs[0], "msg_001");
    println!("   ✅ PASS: Message indexed and retrieved");
}

#[test]
fn test_index_edge_zero_copy() {
    println!("\n🔗 TEST: Index edge in graph index");
    let (manager, _temp, _storage) = setup_real_db();
    
    let edge = Edge {
        id: EdgeId::from("edge_001"),
        from_node: NodeId::from("chat_123"),
        to_node: NodeId::from("msg_456"),
        edge_type: "CONTAINS".to_string(),
        created_at: 1697500000000,
        metadata: "{}".to_string(),
    };
    
    println!("   📝 Indexing edge...");
    manager.index_edge(&edge).unwrap();
    
    println!("   📖 Querying outgoing edges (ZERO-COPY)...");
    let guard = manager.get_outgoing_edges("chat_123").unwrap().expect("Should have results");
    assert_eq!(guard.len(), 1);
    assert!(guard.contains_edge("edge_001"));
    println!("   ✅ PASS: Edge indexed in graph");
}

#[test]
fn test_index_embedding() {
    println!("\n🎨 TEST: Index embedding in vector index");
    let (manager, _temp, _storage) = setup_real_db();
    
    // Use 384D (default dimension) to match VectorIndex configuration
    let test_vector: Vec<f32> = (0..384).map(|i| (i as f32) * 0.001).collect();
    
    let embedding = Embedding {
        id: EmbeddingId::from("embed_001"),
        vector: test_vector.clone(),
        model: "test-model".to_string(),
    };
    
    println!("   📝 Indexing 384D embedding...");
    manager.index_embedding(&embedding).unwrap();
    
    println!("   🔎 Searching for similar vectors...");
    let results = manager.search_vectors(&test_vector, 5).unwrap();
    assert!(!results.is_empty());
    println!("   ✅ PASS: Found {} similar vectors", results.len());
}
