//! Comprehensive tests for storage layer
//! Following RAG Rule 17.6: Test real functionality with real data

use common::models::{Chat, Edge, Embedding, Message, Node};
use common::{EdgeId, EmbeddingId, NodeId};
use serde_json::json;
use storage::{DatabaseType, StorageManager, TemperatureTier};
use tempfile::TempDir;

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

    let storage: StorageManager<storage::engine::MdbxEngine> = StorageManager::new(db_path.to_str().unwrap()).expect("Failed to create storage");
    // Storage created via new() doesn't have a specific type or tier
    assert_eq!(storage.tier(), None);

    println!("✅ StorageManager created successfully");
}

#[test]
fn test_node_crud_operations() {
    println!("\n🧪 Testing Node CRUD operations...");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let storage: StorageManager<storage::engine::MdbxEngine> = StorageManager::new(temp_dir.path().join("test.db").to_str().unwrap())
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
        metadata: json!({"test": true}).to_string(),
    };

    println!("➕ Creating message node: {}", msg_id.as_str());
    storage
        .insert_node(&Node::Message(message.clone()))
        .expect("Failed to insert message");

    // READ: Retrieve the node - ZERO-COPY path
    println!("📖 Reading message node...");
    let retrieved = if let Some(guard) = storage.get_node_guard(msg_id.as_str()).expect("Failed to get node guard") {
        use storage::engine::ReadGuard;
        Some(rkyv::from_bytes::<common::models::Node, rkyv::rancor::Error>(guard.data())
            .expect("Failed to deserialize"))
    } else {
        None
    };
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
    let deleted = storage
        .delete_node(msg_id.as_str())
        .expect("Failed to delete node");
    assert!(deleted.is_some(), "Deleted node should be returned");

    // Verify deletion - ZERO-COPY path
    let after_delete = storage.get_node_guard(msg_id.as_str())
        .expect("Failed to get node guard");
    assert!(after_delete.is_none(), "Node should be deleted");
    println!("✅ Node deleted successfully");
}

#[test]
fn test_edge_crud_operations() {
    println!("\n🧪 Testing Edge CRUD operations...");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let storage: StorageManager<storage::engine::MdbxEngine> = StorageManager::new(temp_dir.path().join("test.db").to_str().unwrap())
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
        metadata: json!({}).to_string(),
    };

    let msg2 = Message {
        id: node2_id.clone(),
        chat_id: chat_id.clone(),
        sender: "assistant".to_string(),
        timestamp: current_timestamp() + 1000,
        text_content: "Second message".to_string(),
        attachment_ids: vec![],
        embedding_id: None,
        metadata: json!({}).to_string(),
    };

    storage
        .insert_node(&Node::Message(msg1))
        .expect("Failed to insert node1");
    storage
        .insert_node(&Node::Message(msg2))
        .expect("Failed to insert node2");
    println!("✅ Created test nodes");

    // CREATE: Insert edge
    let edge_id = EdgeId::new(format!("edge_{}", uuid::Uuid::new_v4()));
    let edge = Edge {
        id: edge_id.clone(),
        from_node: node1_id.clone(),
        to_node: node2_id.clone(),
        edge_type: "REPLY".to_string(),
        created_at: current_timestamp(),
        metadata: json!({"weight": 1.0}).to_string(),
    };

    println!(
        "➕ Creating edge: {} -> {}",
        node1_id.as_str(),
        node2_id.as_str()
    );
    storage.insert_edge(&edge).expect("Failed to insert edge");

    // READ: Retrieve the edge - ZERO-COPY path
    println!("📖 Reading edge...");
    let retrieved = if let Some(guard) = storage.get_edge_guard(edge_id.as_str()).expect("Failed to get edge guard") {
        use storage::engine::ReadGuard;
        Some(rkyv::from_bytes::<common::models::Edge, rkyv::rancor::Error>(guard.data())
            .expect("Failed to deserialize"))
    } else {
        None
    };
    assert!(retrieved.is_some(), "Edge should exist");

    if let Some(retrieved_edge) = retrieved {
        assert_eq!(retrieved_edge.id, edge_id);
        assert_eq!(retrieved_edge.edge_type, "REPLY");
        println!("✅ Edge retrieved: {}", retrieved_edge.edge_type);
    }

    // DELETE: Remove the edge
    println!("🗑️ Deleting edge...");
    let deleted = storage
        .delete_edge(edge_id.as_str())
        .expect("Failed to delete edge");
    assert!(deleted.is_some(), "Deleted edge should be returned");

    // Verify deletion - ZERO-COPY path
    let after_delete = storage.get_edge_guard(edge_id.as_str())
        .expect("Failed to get edge guard");
    assert!(after_delete.is_none(), "Edge should be deleted");
    println!("✅ Edge deleted successfully");
}

#[test]
fn test_embedding_operations() {
    println!("\n🧪 Testing Embedding operations...");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let storage: StorageManager<storage::engine::MdbxEngine> = StorageManager::new(temp_dir.path().join("test.db").to_str().unwrap())
        .expect("Failed to create storage");

    // CREATE: Insert embedding
    let embed_id = EmbeddingId::new(format!("embed_{}", uuid::Uuid::new_v4()));
    let embedding = Embedding {
        id: embed_id.clone(),
        vector: vec![0.1, 0.2, 0.3, 0.4, 0.5],
        model: "test-model".to_string(),
    };

    println!(
        "➕ Creating embedding with {} dimensions",
        embedding.vector.len()
    );
    storage
        .insert_embedding(&embedding)
        .expect("Failed to insert embedding");

    // READ: Retrieve embedding - ZERO-COPY path
    println!("📖 Reading embedding...");
    let retrieved = if let Some(guard) = storage.get_embedding_guard(embed_id.as_str()).expect("Failed to get embedding guard") {
        use storage::engine::ReadGuard;
        Some(rkyv::from_bytes::<common::models::Embedding, rkyv::rancor::Error>(guard.data())
            .expect("Failed to deserialize"))
    } else {
        None
    };
    assert!(retrieved.is_some(), "Embedding should exist");

    if let Some(retrieved_embed) = retrieved {
        assert_eq!(retrieved_embed.id, embed_id);
        assert_eq!(retrieved_embed.vector.len(), 5);
        assert_eq!(retrieved_embed.model, "test-model");
        println!(
            "✅ Embedding retrieved: {} dims",
            retrieved_embed.vector.len()
        );
    }

    // DELETE: Remove embedding
    println!("🗑️ Deleting embedding...");
    let deleted = storage
        .delete_embedding(embed_id.as_str())
        .expect("Failed to delete embedding");
    assert!(deleted.is_some(), "Deleted embedding should be returned");

    // Verify deletion - ZERO-COPY path
    let after_delete = storage.get_embedding_guard(embed_id.as_str())
        .expect("Failed to get embedding guard");
    assert!(after_delete.is_none(), "Embedding should be deleted");
    println!("✅ Embedding deleted successfully");
}

#[test]
fn test_storage_with_indexing() {
    println!("\n🧪 Testing StorageManager with indexing...");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let storage: StorageManager<storage::engine::MdbxEngine> = StorageManager::with_indexing(temp_dir.path().join("test.db").to_str().unwrap())
        .expect("Failed to create storage with indexing");

    assert!(
        storage.index_manager().is_some(),
        "Index manager should be present"
    );

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
        metadata: json!({"indexed": true}).to_string(),
    };

    println!("➕ Creating indexed chat node");
    storage
        .insert_node(&Node::Chat(chat))
        .expect("Failed to insert chat");

    // ZERO-COPY path
    use storage::engine::{ReadGuard, MdbxEngine};
    let retrieved: Option<Node> = if let Some(guard) = storage.get_node_guard(chat_id.as_str()).expect("Failed to get node guard") {
        let data_slice: &[u8] = ReadGuard::data(&guard);
        let data_vec: Vec<u8> = data_slice.to_vec();
        let node: Node = rkyv::from_bytes::<common::models::Node, rkyv::rancor::Error>(&data_vec)
            .expect("Failed to deserialize");
        Some(node)
    } else {
        None
    };
    assert!(retrieved.is_some(), "Indexed chat should exist");
    println!("✅ Indexed storage working correctly");
}

#[test]
fn test_database_type_paths() {
    println!("\n🧪 Testing DatabaseType path generation...");

    // Test conversations path (Active)
    let conv_path = DatabaseType::Conversations.get_path(Some(TemperatureTier::Active));
    assert!(conv_path.to_str().unwrap().contains("conversations"));
    assert!(conv_path.to_str().unwrap().contains("active"));
    println!("✅ Conversations Active path: {:?}", conv_path);

    // Test knowledge path (Stable)
    let knowledge_path = DatabaseType::Knowledge.get_path(Some(TemperatureTier::Stable));
    assert!(knowledge_path.to_str().unwrap().contains("knowledge"));
    assert!(knowledge_path.to_str().unwrap().contains("stable"));
    println!("✅ Knowledge Stable path: {:?}", knowledge_path);

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
    let storage: std::sync::Arc<StorageManager<storage::engine::MdbxEngine>> = std::sync::Arc::new(
        StorageManager::new(temp_dir.path().join("test.db").to_str().unwrap())
            .expect("Failed to create storage"),
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
                metadata: json!({"thread": i}).to_string(),
            };

            storage_clone
                .insert_node(&Node::Message(message))
                .expect("Failed to insert");
            msg_id
        });
        handles.push(handle);
    }

    // Wait for all threads
    let ids: Vec<NodeId> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    // Verify all nodes exist - ZERO-COPY path
    for id in &ids {
        let guard = storage.get_node_guard(id.as_str()).expect("Failed to get node guard");
        assert!(guard.is_some(), "Concurrent node should exist");
    }

    println!(
        "✅ {} concurrent operations completed successfully",
        ids.len()
    );
}

#[test]
fn test_error_handling() {
    println!("\n🧪 Testing error handling...");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let storage: StorageManager<storage::engine::MdbxEngine> = StorageManager::new(temp_dir.path().join("test.db").to_str().unwrap())
        .expect("Failed to create storage");

    // Test retrieval of non-existent node
    let result = storage.get_node_guard("non_existent_id");
    assert!(
        result.is_ok(),
        "Getting non-existent node should return Ok(None)"
    );
    assert!(
        result.unwrap().is_none(),
        "Non-existent node should be None"
    );

    // Test deletion of non-existent node
    let delete_result = storage.delete_node("non_existent_id");
    assert!(
        delete_result.is_ok(),
        "Deleting non-existent node should return Ok(None)"
    );
    assert!(
        delete_result.unwrap().is_none(),
        "Deleted non-existent should be None"
    );

    println!("✅ Error handling working correctly");
}

/// Test true zero-copy access patterns using archived types
/// Following RAG Rule 17.6: Test real functionality with real data
#[test]
fn test_zero_copy_archived_access() {
    println!("\n🧪 Testing ZERO-COPY archived access patterns...");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let storage: StorageManager<storage::engine::MdbxEngine> = StorageManager::new(temp_dir.path().join("test.db").to_str().unwrap())
        .expect("Failed to create storage");

    // CREATE: Insert a message with metadata
    let msg_id = NodeId::new("msg_zero_copy");
    let chat_id = NodeId::new("chat_zero_copy");
    let message = Message {
        id: msg_id.clone(),
        chat_id: chat_id.clone(),
        sender: "test_user".to_string(),
        timestamp: current_timestamp(),
        text_content: "Testing zero-copy access!".to_string(),
        attachment_ids: vec![],
        embedding_id: None,
        metadata: json!({"test": "zero_copy", "performance": "critical"}).to_string(),
    };

    storage
        .insert_node(&Node::Message(message.clone()))
        .expect("Failed to insert message");

    // TEST: Access archived fields without full deserialization
    println!("🔍 Testing archived access...");
    
    if let Some(node_ref) = storage.get_node_ref(msg_id.as_str())
        .expect("Failed to get node ref") 
    {
        // Access fields without deserializing entire struct
        if let Some(text) = node_ref.message_text() {
            assert!(!text.is_empty());
            println!("✅ Successfully accessed archived Message fields");
        } else {
            panic!("Expected Message variant");
        }
        
        assert!(node_ref.is_message(), "Expected Message variant");
    }

    // TEST: Compare with deserialization path (allocates)
    println!("📊 Comparing with deserialization path...");
    if let Some(Node::Message(retrieved)) = storage.get_node(msg_id.as_str())
        .expect("Failed to get node via deserialization")
    {
        assert_eq!(retrieved.id, msg_id);
        assert_eq!(retrieved.text_content, "Testing zero-copy access!");
        println!("✅ Deserialization path also works");
    } else {
        panic!("Should have retrieved message");
    }

    println!("✅ ZERO-COPY archived access working correctly");
}

/// Test zero-copy access for embeddings (critical for performance)
#[test]
fn test_zero_copy_embedding_access() {
    println!("\n🧪 Testing ZERO-COPY embedding access...");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let storage: StorageManager<storage::engine::MdbxEngine> = StorageManager::new(temp_dir.path().join("test.db").to_str().unwrap())
        .expect("Failed to create storage");

    // Create a test embedding with a large vector
    let embed_id = EmbeddingId::new("emb_zero_copy");
    let large_vector: Vec<f32> = (0..1536).map(|i| (i as f32) * 0.001).collect(); // 1536-dim embedding
    let embedding = Embedding {
        id: embed_id.clone(),
        vector: large_vector.clone(),
        model: "test-model".to_string(),
    };

    storage.insert_embedding(&embedding).expect("Failed to insert embedding");

    // TEST: Access archived embedding
    println!("🔍 Testing archived embedding access...");
    
    if let Some(emb_ref) = storage.get_embedding_ref(embed_id.as_str())
        .expect("Failed to get embedding ref")
    {
        let vector = emb_ref.vector();
        assert_eq!(vector.len(), 384);
        println!("✅ Successfully accessed archived Embedding");
    } else {
        panic!("Should have found embedding");
    }

    // Verify deserialization path also works
    if let Some(retrieved) = storage.get_embedding(embed_id.as_str())
        .expect("Failed to get embedding")
    {
        assert_eq!(retrieved.vector.len(), 1536);
        assert_eq!(retrieved.vector[0], large_vector[0]);
        println!("✅ Deserialization path works for large embeddings");
    }

    println!("✅ ZERO-COPY embedding access working correctly");
}

/// Test zero-copy access for edges
#[test]
fn test_zero_copy_edge_access() {
    println!("\n🧪 Testing ZERO-COPY edge access...");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let storage: StorageManager<storage::engine::MdbxEngine> = StorageManager::new(temp_dir.path().join("test.db").to_str().unwrap())
        .expect("Failed to create storage");

    let node1_id = NodeId::new("node1");
    let node2_id = NodeId::new("node2");
    let edge_id = EdgeId::new("edge_zero_copy");
    let edge = Edge {
        id: edge_id.clone(),
        from_node: node1_id.clone(),
        to_node: node2_id.clone(),
        edge_type: "RELATES_TO".to_string(),
        created_at: current_timestamp(),
        metadata: json!({"weight": 1.0, "confidence": 0.95}).to_string(),
    };

    storage.insert_edge(&edge).expect("Failed to insert edge");

    // TEST: Access archived edge
    if let Some(edge_ref) = storage.get_edge_ref(edge_id.as_str())
        .expect("Failed to get edge ref")
    {
        let from = edge_ref.from_node();
        let to = edge_ref.to_node();
        assert_eq!(from, node1_id.as_str());
        assert_eq!(to, node2_id.as_str());
        println!("✅ Successfully accessed archived Edge");
    } else {
        panic!("Should have found edge");
    }

    println!("✅ Archived edge access working correctly");
}
