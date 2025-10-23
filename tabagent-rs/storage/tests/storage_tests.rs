/// Storage layer tests
/// 
/// REAL TESTS - NO MOCKS:
/// - Uses real sled database
/// - Tests actual CRUD operations
/// - Tests real multi-database coordination
/// - Cleans up with tempfile

use storage::{StorageManager, DatabaseCoordinator, DatabaseType, TemperatureTier};
use common::models::{Node, Edge, NodeType, EdgeType};
use tempfile::TempDir;
use uuid::Uuid;

#[test]
fn test_real_node_crud() {
    println!("\n🧪 Testing REAL node CRUD...");
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let storage = StorageManager::new(temp_dir.path()).expect("Failed to create storage");
    
    // Create real node
    let node_id = Uuid::new_v4().to_string();
    let node = Node {
        id: node_id.clone(),
        node_type: NodeType::Message,
        content: Some("Test message content".to_string()),
        metadata: None,
        created_at: chrono::Utc::now().timestamp(),
        updated_at: chrono::Utc::now().timestamp(),
    };
    
    println!("➕ Creating node: {}", node_id);
    storage.insert_node(&node).expect("Failed to insert node");
    
    // Read it back
    println!("🔍 Reading node...");
    let retrieved = storage.get_node(&node_id).expect("Failed to get node");
    assert!(retrieved.is_some(), "Node not found after insert");
    
    let retrieved_node = retrieved.unwrap();
    assert_eq!(retrieved_node.id, node.id);
    assert_eq!(retrieved_node.content, node.content);
    println!("✅ Node retrieved correctly");
    
    // Update it
    let mut updated_node = retrieved_node.clone();
    updated_node.content = Some("Updated content".to_string());
    updated_node.updated_at = chrono::Utc::now().timestamp();
    
    println!("📝 Updating node...");
    storage.update_node(&updated_node).expect("Failed to update node");
    
    let after_update = storage.get_node(&node_id).expect("Failed to get updated node");
    assert_eq!(after_update.unwrap().content.unwrap(), "Updated content");
    println!("✅ Node updated correctly");
    
    // Delete it
    println!("🗑️  Deleting node...");
    storage.delete_node(&node_id).expect("Failed to delete node");
    
    let after_delete = storage.get_node(&node_id).expect("Failed to check deletion");
    assert!(after_delete.is_none(), "Node still exists after deletion");
    println!("✅ Node deleted correctly");
}

#[test]
fn test_real_edge_crud() {
    println!("\n🧪 Testing REAL edge CRUD...");
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let storage = StorageManager::new(temp_dir.path()).expect("Failed to create storage");
    
    // Create two nodes first
    let node1_id = Uuid::new_v4().to_string();
    let node2_id = Uuid::new_v4().to_string();
    
    let node1 = Node {
        id: node1_id.clone(),
        node_type: NodeType::Message,
        content: Some("Node 1".to_string()),
        metadata: None,
        created_at: chrono::Utc::now().timestamp(),
        updated_at: chrono::Utc::now().timestamp(),
    };
    
    let node2 = Node {
        id: node2_id.clone(),
        node_type: NodeType::Message,
        content: Some("Node 2".to_string()),
        metadata: None,
        created_at: chrono::Utc::now().timestamp(),
        updated_at: chrono::Utc::now().timestamp(),
    };
    
    storage.insert_node(&node1).expect("Failed to insert node1");
    storage.insert_node(&node2).expect("Failed to insert node2");
    println!("✅ Created test nodes");
    
    // Create edge
    let edge_id = Uuid::new_v4().to_string();
    let edge = Edge {
        id: edge_id.clone(),
        from_node: node1_id.clone(),
        to_node: node2_id.clone(),
        edge_type: EdgeType::Reply,
        weight: Some(1.0),
        metadata: None,
        created_at: chrono::Utc::now().timestamp(),
    };
    
    println!("➕ Creating edge: {} -> {}", node1_id, node2_id);
    storage.insert_edge(&edge).expect("Failed to insert edge");
    
    // Read it back
    let retrieved = storage.get_edge(&edge_id).expect("Failed to get edge");
    assert!(retrieved.is_some(), "Edge not found after insert");
    
    let retrieved_edge = retrieved.unwrap();
    assert_eq!(retrieved_edge.from_node, node1_id);
    assert_eq!(retrieved_edge.to_node, node2_id);
    println!("✅ Edge retrieved correctly");
    
    // Query edges from node1
    let outgoing = storage.get_outgoing_edges(&node1_id).expect("Failed to get outgoing edges");
    assert_eq!(outgoing.len(), 1);
    assert_eq!(outgoing[0].id, edge_id);
    println!("✅ Outgoing edges query works");
    
    // Query edges to node2
    let incoming = storage.get_incoming_edges(&node2_id).expect("Failed to get incoming edges");
    assert_eq!(incoming.len(), 1);
    assert_eq!(incoming[0].id, edge_id);
    println!("✅ Incoming edges query works");
    
    // Delete edge
    storage.delete_edge(&edge_id).expect("Failed to delete edge");
    let after_delete = storage.get_edge(&edge_id).expect("Failed to check deletion");
    assert!(after_delete.is_none(), "Edge still exists after deletion");
    println!("✅ Edge deleted correctly");
}

#[test]
fn test_real_database_coordinator() {
    println!("\n🧪 Testing REAL DatabaseCoordinator...");
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let coordinator = DatabaseCoordinator::new(temp_dir.path()).expect("Failed to create coordinator");
    
    // Test conversations database (HOT tier)
    println!("📝 Testing conversations database...");
    let conv_db = coordinator.conversations_active();
    
    let node = Node {
        id: Uuid::new_v4().to_string(),
        node_type: NodeType::Message,
        content: Some("Test conversation".to_string()),
        metadata: None,
        created_at: chrono::Utc::now().timestamp(),
        updated_at: chrono::Utc::now().timestamp(),
    };
    
    conv_db.insert_node(&node).expect("Failed to insert to conversations DB");
    let retrieved = conv_db.get_node(&node.id).expect("Failed to get from conversations DB");
    assert!(retrieved.is_some());
    println!("✅ Conversations DB works");
    
    // Test knowledge database (WARM tier)
    println!("📝 Testing knowledge database...");
    let knowledge_db = coordinator.knowledge_recent();
    
    let knowledge_node = Node {
        id: Uuid::new_v4().to_string(),
        node_type: NodeType::Document,
        content: Some("Test knowledge".to_string()),
        metadata: None,
        created_at: chrono::Utc::now().timestamp(),
        updated_at: chrono::Utc::now().timestamp(),
    };
    
    knowledge_db.insert_node(&knowledge_node).expect("Failed to insert to knowledge DB");
    let retrieved = knowledge_db.get_node(&knowledge_node.id).expect("Failed to get from knowledge DB");
    assert!(retrieved.is_some());
    println!("✅ Knowledge DB works");
    
    // Verify databases are separate
    let from_conv = conv_db.get_node(&knowledge_node.id).expect("Query should work");
    assert!(from_conv.is_none(), "Knowledge node should not be in conversations DB");
    println!("✅ Database isolation verified");
}

#[test]
fn test_database_paths() {
    println!("\n🧪 Testing database path generation...");
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    
    // Test conversations path (HOT)
    let conv_path = DatabaseType::Conversations.get_path(temp_dir.path(), Some(TemperatureTier::Hot));
    assert!(conv_path.to_str().unwrap().contains("conversations"));
    assert!(conv_path.to_str().unwrap().contains("hot"));
    println!("✅ Conversations HOT path: {:?}", conv_path);
    
    // Test knowledge path (WARM)
    let knowledge_path = DatabaseType::Knowledge.get_path(temp_dir.path(), Some(TemperatureTier::Warm));
    assert!(knowledge_path.to_str().unwrap().contains("knowledge"));
    assert!(knowledge_path.to_str().unwrap().contains("warm"));
    println!("✅ Knowledge WARM path: {:?}", knowledge_path);
    
    // Test embeddings path (COLD)
    let embed_path = DatabaseType::Embeddings.get_path(temp_dir.path(), Some(TemperatureTier::Cold));
    assert!(embed_path.to_str().unwrap().contains("embeddings"));
    assert!(embed_path.to_str().unwrap().contains("cold"));
    println!("✅ Embeddings COLD path: {:?}", embed_path);
}

#[test]
fn test_bulk_operations() {
    println!("\n🧪 Testing bulk operations...");
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let storage = StorageManager::new(temp_dir.path()).expect("Failed to create storage");
    
    // Insert 100 nodes
    println!("➕ Inserting 100 nodes...");
    let node_ids: Vec<String> = (0..100)
        .map(|i| {
            let id = Uuid::new_v4().to_string();
            let node = Node {
                id: id.clone(),
                node_type: NodeType::Message,
                content: Some(format!("Message {}", i)),
                metadata: None,
                created_at: chrono::Utc::now().timestamp(),
                updated_at: chrono::Utc::now().timestamp(),
            };
            storage.insert_node(&node).expect("Failed to insert node");
            id
        })
        .collect();
    
    println!("✅ Inserted 100 nodes");
    
    // Verify all exist
    println!("🔍 Verifying all nodes...");
    for id in &node_ids {
        let node = storage.get_node(id).expect("Failed to get node");
        assert!(node.is_some(), "Node {} not found", id);
    }
    println!("✅ All 100 nodes verified");
    
    // Delete all
    println!("🗑️  Deleting all nodes...");
    for id in &node_ids {
        storage.delete_node(id).expect("Failed to delete node");
    }
    
    // Verify all deleted
    for id in &node_ids {
        let node = storage.get_node(id).expect("Failed to check deletion");
        assert!(node.is_none(), "Node {} still exists", id);
    }
    println!("✅ All nodes deleted");
}

#[test]
fn test_concurrent_access() {
    println!("\n🧪 Testing concurrent access...");
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let storage = std::sync::Arc::new(
        StorageManager::new(temp_dir.path()).expect("Failed to create storage")
    );
    
    let mut handles = vec![];
    
    // Spawn 10 threads, each inserting 10 nodes
    for thread_id in 0..10 {
        let storage_clone = storage.clone();
        let handle = std::thread::spawn(move || {
            for i in 0..10 {
                let node = Node {
                    id: format!("thread_{}_node_{}", thread_id, i),
                    node_type: NodeType::Message,
                    content: Some(format!("Thread {} message {}", thread_id, i)),
                    metadata: None,
                    created_at: chrono::Utc::now().timestamp(),
                    updated_at: chrono::Utc::now().timestamp(),
                };
                storage_clone.insert_node(&node).expect("Failed to insert node");
            }
        });
        handles.push(handle);
    }
    
    // Wait for all threads
    for handle in handles {
        handle.join().expect("Thread panicked");
    }
    
    println!("✅ 100 nodes inserted concurrently");
    
    // Verify all nodes exist
    for thread_id in 0..10 {
        for i in 0..10 {
            let id = format!("thread_{}_node_{}", thread_id, i);
            let node = storage.get_node(&id).expect("Failed to get node");
            assert!(node.is_some(), "Node {} not found", id);
        }
    }
    
    println!("✅ All concurrent writes verified");
}

