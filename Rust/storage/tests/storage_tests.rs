//! Comprehensive tests for storage layer
//! Following RAG Rule 17.6: Test real functionality with real data

use common::models::{Node, Message, Chat, Edge, Embedding};
use common::{NodeId, EdgeId, EmbeddingId};
use storage::{StorageManager, DatabaseCoordinator, DatabaseType, TemperatureTier};
use tempfile::TempDir;
use serde_json::json;

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

#[test]
fn test_storage_manager_creation() {
    println!("\n🧪 Testing StorageManager creation...");
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    
    let storage = StorageManager::new(db_path.to_str().unwrap()).expect("Failed to create storage");
    // Storage created via new() doesn't have a specific type or tier
    assert_eq!(storage.tier(), None);
    
    println!("✅ StorageManager created successfully");
}

#[test]
fn test_node_crud_operations() {
    println!("\n🧪 Testing Node CRUD operations...");
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let storage = StorageManager::new(temp_dir.path().join("test.db").to_str().unwrap())
        .expect("Failed to create storage");
    
    // CREATE: Insert a message node
    let msg_id = NodeId::new(format!("msg_{}", uuid::Uuid::new_v4()));
    let chat_id = NodeId::new("chat_123".to_string());
    let message = Message {
        id: msg_id.clone(),
        chat_id: chat_id.clone(),
        sender: "user".to_string(),
        timestamp: current_timestamp(),
        text_content: "Hello, world!".to_string(),
        attachment_ids: vec![],
        embedding_id: None,
        metadata: json!({"test": true}),
    };
    
    println!("➕ Creating message node: {}", msg_id.as_str());
    storage.insert_node(&Node::Message(message.clone())).expect("Failed to insert message");
    
    // READ: Retrieve the node
    println!("📖 Reading message node...");
    let retrieved = storage.get_node(msg_id.as_str()).expect("Failed to get node");
    assert!(retrieved.is_some(), "Node should exist");
    
    if let Some(Node::Message(retrieved_msg)) = retrieved {
        assert_eq!(retrieved_msg.id, msg_id);
        assert_eq!(retrieved_msg.text_content, "Hello, world!");
        println!("✅ Message retrieved: {}", retrieved_msg.text_content);
    } else {
        panic!("Expected Message variant");
    }
    
    // DELETE: Remove the node
    println!("🗑️ Deleting message node...");
    let deleted = storage.delete_node(msg_id.as_str()).expect("Failed to delete node");
    assert!(deleted.is_some(), "Deleted node should be returned");
    
    // Verify deletion
    let after_delete = storage.get_node(msg_id.as_str()).expect("Failed to get node");
    assert!(after_delete.is_none(), "Node should be deleted");
    println!("✅ Node deleted successfully");
}

#[test]
fn test_edge_crud_operations() {
    println!("\n🧪 Testing Edge CRUD operations...");
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let storage = StorageManager::new(temp_dir.path().join("test.db").to_str().unwrap())
        .expect("Failed to create storage");
    
    // Create two message nodes first
    let node1_id = NodeId::new("node1".to_string());
    let node2_id = NodeId::new("node2".to_string());
    let chat_id = NodeId::new("chat_456".to_string());
    
    let msg1 = Message {
        id: node1_id.clone(),
        chat_id: chat_id.clone(),
        sender: "user".to_string(),
        timestamp: current_timestamp(),
        text_content: "First message".to_string(),
        attachment_ids: vec![],
        embedding_id: None,
        metadata: json!({}),
    };
    
    let msg2 = Message {
        id: node2_id.clone(),
        chat_id: chat_id.clone(),
        sender: "assistant".to_string(),
        timestamp: current_timestamp() + 1000,
        text_content: "Second message".to_string(),
        attachment_ids: vec![],
        embedding_id: None,
        metadata: json!({}),
    };
    
    storage.insert_node(&Node::Message(msg1)).expect("Failed to insert node1");
    storage.insert_node(&Node::Message(msg2)).expect("Failed to insert node2");
    println!("✅ Created test nodes");
    
    // CREATE: Insert edge
    let edge_id = EdgeId::new(format!("edge_{}", uuid::Uuid::new_v4()));
    let edge = Edge {
        id: edge_id.clone(),
        from_node: node1_id.clone(),
        to_node: node2_id.clone(),
        edge_type: "REPLY".to_string(),
        created_at: current_timestamp(),
        metadata: json!({"weight": 1.0}),
    };
    
    println!("➕ Creating edge: {} -> {}", node1_id.as_str(), node2_id.as_str());
    storage.insert_edge(&edge).expect("Failed to insert edge");
    
    // READ: Retrieve the edge
    println!("📖 Reading edge...");
    let retrieved = storage.get_edge(edge_id.as_str()).expect("Failed to get edge");
    assert!(retrieved.is_some(), "Edge should exist");
    
    if let Some(retrieved_edge) = retrieved {
        assert_eq!(retrieved_edge.id, edge_id);
        assert_eq!(retrieved_edge.edge_type, "REPLY");
        println!("✅ Edge retrieved: {}", retrieved_edge.edge_type);
    }
    
    // DELETE: Remove the edge
    println!("🗑️ Deleting edge...");
    let deleted = storage.delete_edge(edge_id.as_str()).expect("Failed to delete edge");
    assert!(deleted.is_some(), "Deleted edge should be returned");
    
    // Verify deletion
    let after_delete = storage.get_edge(edge_id.as_str()).expect("Failed to get edge");
    assert!(after_delete.is_none(), "Edge should be deleted");
    println!("✅ Edge deleted successfully");
}

#[test]
fn test_embedding_operations() {
    println!("\n🧪 Testing Embedding operations...");
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let storage = StorageManager::new(temp_dir.path().join("test.db").to_str().unwrap())
        .expect("Failed to create storage");
    
    // CREATE: Insert embedding
    let embed_id = EmbeddingId::new(format!("embed_{}", uuid::Uuid::new_v4()));
    let embedding = Embedding {
        id: embed_id.clone(),
        vector: vec![0.1, 0.2, 0.3, 0.4, 0.5],
        model: "test-model".to_string(),
    };
    
    println!("➕ Creating embedding with {} dimensions", embedding.vector.len());
    storage.insert_embedding(&embedding).expect("Failed to insert embedding");
    
    // READ: Retrieve embedding
    println!("📖 Reading embedding...");
    let retrieved = storage.get_embedding(embed_id.as_str()).expect("Failed to get embedding");
    assert!(retrieved.is_some(), "Embedding should exist");
    
    if let Some(retrieved_embed) = retrieved {
        assert_eq!(retrieved_embed.id, embed_id);
        assert_eq!(retrieved_embed.vector.len(), 5);
        assert_eq!(retrieved_embed.model, "test-model");
        println!("✅ Embedding retrieved: {} dims", retrieved_embed.vector.len());
    }
    
    // DELETE: Remove embedding
    println!("🗑️ Deleting embedding...");
    let deleted = storage.delete_embedding(embed_id.as_str()).expect("Failed to delete embedding");
    assert!(deleted.is_some(), "Deleted embedding should be returned");
    
    // Verify deletion
    let after_delete = storage.get_embedding(embed_id.as_str()).expect("Failed to get embedding");
    assert!(after_delete.is_none(), "Embedding should be deleted");
    println!("✅ Embedding deleted successfully");
}

#[test]
fn test_storage_with_indexing() {
    println!("\n🧪 Testing StorageManager with indexing...");
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let storage = StorageManager::with_indexing(temp_dir.path().join("test.db").to_str().unwrap())
        .expect("Failed to create storage with indexing");
    
    assert!(storage.index_manager().is_some(), "Index manager should be present");
    
    // Create and insert a chat node
    let chat_id = NodeId::new("chat_789".to_string());
    let chat = Chat {
        id: chat_id.clone(),
        title: "Test Chat".to_string(),
        topic: "Testing".to_string(),
        created_at: current_timestamp(),
        updated_at: current_timestamp(),
        message_ids: vec![],
        summary_ids: vec![],
        embedding_id: None,
        metadata: json!({"indexed": true}),
    };
    
    println!("➕ Creating indexed chat node");
    storage.insert_node(&Node::Chat(chat)).expect("Failed to insert chat");
    
    let retrieved = storage.get_node(chat_id.as_str()).expect("Failed to get chat");
    assert!(retrieved.is_some(), "Indexed chat should exist");
    println!("✅ Indexed storage working correctly");
}

#[test]
fn test_database_coordinator() {
    println!("\n🧪 Testing DatabaseCoordinator...");
    
    // DatabaseCoordinator::new() uses platform-specific paths
    let coordinator = DatabaseCoordinator::new().expect("Failed to create coordinator");
    
    // Test conversations database (Active tier)
    println!("📝 Testing conversations database...");
    let conv_db = coordinator.conversations_active();
    assert_eq!(conv_db.db_type(), DatabaseType::Conversations);
    assert_eq!(conv_db.tier(), Some(TemperatureTier::Active));
    
    // Create a test chat in conversations
    let chat_id = NodeId::new(format!("chat_{}", uuid::Uuid::new_v4()));
    let chat = Chat {
        id: chat_id.clone(),
        title: "Coordinator Test".to_string(),
        topic: "Testing".to_string(),
        created_at: current_timestamp(),
        updated_at: current_timestamp(),
        message_ids: vec![],
        summary_ids: vec![],
        embedding_id: None,
        metadata: json!({}),
    };
    
    conv_db.insert_node(&Node::Chat(chat)).expect("Failed to insert into conversations");
    
    let retrieved = conv_db.get_node(chat_id.as_str()).expect("Failed to get chat");
    assert!(retrieved.is_some(), "Chat should exist in conversations DB");
    println!("✅ DatabaseCoordinator working correctly");
}

#[test]
fn test_database_type_paths() {
    println!("\n🧪 Testing DatabaseType path generation...");
    
    // Test conversations path (Active)
    let conv_path = DatabaseType::Conversations.get_path(Some(TemperatureTier::Active));
    assert!(conv_path.to_str().unwrap().contains("conversations"));
    assert!(conv_path.to_str().unwrap().contains("active"));
    println!("✅ Conversations Active path: {:?}", conv_path);
    
    // Test knowledge path (Recent)
    let knowledge_path = DatabaseType::Knowledge.get_path(Some(TemperatureTier::Recent));
    assert!(knowledge_path.to_str().unwrap().contains("knowledge"));
    assert!(knowledge_path.to_str().unwrap().contains("recent"));
    println!("✅ Knowledge Recent path: {:?}", knowledge_path);
    
    // Test embeddings path (Archive)
    let embed_path = DatabaseType::Embeddings.get_path(Some(TemperatureTier::Archive));
    assert!(embed_path.to_str().unwrap().contains("embeddings"));
    assert!(embed_path.to_str().unwrap().contains("archive"));
    println!("✅ Embeddings Archive path: {:?}", embed_path);
}

#[test]
fn test_concurrent_operations() {
    println!("\n🧪 Testing concurrent operations...");
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let storage = std::sync::Arc::new(
        StorageManager::new(temp_dir.path().join("test.db").to_str().unwrap())
            .expect("Failed to create storage")
    );
    
    // Spawn multiple threads inserting nodes
    let mut handles = vec![];
    
    for i in 0..5 {
        let storage_clone = storage.clone();
        let handle = std::thread::spawn(move || {
            let msg_id = NodeId::new(format!("msg_{}", i));
            let message = Message {
                id: msg_id.clone(),
                chat_id: NodeId::new("chat_concurrent".to_string()),
                sender: format!("user_{}", i),
                timestamp: current_timestamp(),
                text_content: format!("Message {}", i),
                attachment_ids: vec![],
                embedding_id: None,
                metadata: json!({"thread": i}),
            };
            
            storage_clone.insert_node(&Node::Message(message)).expect("Failed to insert");
            msg_id
        });
        handles.push(handle);
    }
    
    // Wait for all threads
    let ids: Vec<NodeId> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    
    // Verify all nodes exist
    for id in &ids {
        let node = storage.get_node(id.as_str()).expect("Failed to get node");
        assert!(node.is_some(), "Concurrent node should exist");
    }
    
    println!("✅ {} concurrent operations completed successfully", ids.len());
}

#[test]
fn test_error_handling() {
    println!("\n🧪 Testing error handling...");
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let storage = StorageManager::new(temp_dir.path().join("test.db").to_str().unwrap())
        .expect("Failed to create storage");
    
    // Test retrieval of non-existent node
    let result = storage.get_node("non_existent_id");
    assert!(result.is_ok(), "Getting non-existent node should return Ok(None)");
    assert!(result.unwrap().is_none(), "Non-existent node should be None");
    
    // Test deletion of non-existent node
    let delete_result = storage.delete_node("non_existent_id");
    assert!(delete_result.is_ok(), "Deleting non-existent node should return Ok(None)");
    assert!(delete_result.unwrap().is_none(), "Deleted non-existent should be None");
    
    println!("✅ Error handling working correctly");
}
