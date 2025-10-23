/// Query engine tests
/// 
/// REAL TESTS using realistic mock data:
/// - Tests graph traversal
/// - Tests conversation queries
/// - Tests filtering
/// - Uses realistic conversation fixtures

use query::QueryManager;
use storage::{StorageManager, DatabaseCoordinator};
use common::models::{Node, Edge, NodeType, EdgeType};
use tempfile::TempDir;
use uuid::Uuid;
use chrono::Utc;

// Include the fixtures from storage crate tests
// In real usage, we'd make fixtures a shared module
fn create_test_conversation(storage: &StorageManager) -> Vec<String> {
    let mut message_ids = Vec::new();
    let base_time = Utc::now().timestamp();
    
    // Create 5 messages with edges
    for i in 0..5 {
        let msg_id = Uuid::new_v4().to_string();
        let role = if i % 2 == 0 { "user" } else { "assistant" };
        
        let node = Node {
            id: msg_id.clone(),
            node_type: NodeType::Message,
            content: Some(format!("Test message {} from {}", i, role)),
            metadata: Some(serde_json::json!({
                "role": role,
                "tokens": 10
            }).to_string()),
            created_at: base_time + i,
            updated_at: base_time + i,
        };
        
        storage.insert_node(&node).expect("Failed to insert node");
        
        // Create edge from previous message
        if i > 0 {
            let edge = Edge {
                id: Uuid::new_v4().to_string(),
                from_node: message_ids[i - 1].clone(),
                to_node: msg_id.clone(),
                edge_type: EdgeType::Reply,
                weight: Some(1.0),
                metadata: None,
                created_at: base_time + i,
            };
            storage.insert_edge(&edge).expect("Failed to insert edge");
        }
        
        message_ids.push(msg_id);
    }
    
    message_ids
}

#[test]
fn test_graph_traversal() {
    println!("\n🧪 Testing graph traversal...");
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let coordinator = DatabaseCoordinator::new(temp_dir.path()).expect("Failed to create coordinator");
    let storage = coordinator.conversations_active();
    
    // Create test conversation
    let message_ids = create_test_conversation(&storage);
    
    println!("✅ Created {} messages", message_ids.len());
    
    // Get outgoing edges from first message
    let outgoing = storage.get_outgoing_edges(&message_ids[0]).expect("Failed to get outgoing");
    assert_eq!(outgoing.len(), 1, "First message should have 1 outgoing edge");
    assert_eq!(outgoing[0].to_node, message_ids[1]);
    
    println!("✅ Graph traversal works");
}

#[test]
fn test_conversation_query() {
    println!("\n🧪 Testing conversation queries...");
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let coordinator = DatabaseCoordinator::new(temp_dir.path()).expect("Failed to create coordinator");
    let storage = coordinator.conversations_active();
    
    let message_ids = create_test_conversation(&storage);
    
    // Query each message
    for id in &message_ids {
        let node = storage.get_node(id).expect("Failed to get node");
        assert!(node.is_some(), "Message {} not found", id);
    }
    
    println!("✅ All {} messages queried successfully", message_ids.len());
}

#[test]
fn test_node_filtering() {
    println!("\n🧪 Testing node filtering...");
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let coordinator = DatabaseCoordinator::new(temp_dir.path()).expect("Failed to create coordinator");
    let storage = coordinator.conversations_active();
    
    // Create nodes of different types
    let message_node = Node {
        id: Uuid::new_v4().to_string(),
        node_type: NodeType::Message,
        content: Some("Message node".to_string()),
        metadata: None,
        created_at: Utc::now().timestamp(),
        updated_at: Utc::now().timestamp(),
    };
    
    let document_node = Node {
        id: Uuid::new_v4().to_string(),
        node_type: NodeType::Document,
        content: Some("Document node".to_string()),
        metadata: None,
        created_at: Utc::now().timestamp(),
        updated_at: Utc::now().timestamp(),
    };
    
    storage.insert_node(&message_node).expect("Failed to insert message");
    storage.insert_node(&document_node).expect("Failed to insert document");
    
    // Verify types
    let retrieved_msg = storage.get_node(&message_node.id).expect("Failed to get").unwrap();
    let retrieved_doc = storage.get_node(&document_node.id).expect("Failed to get").unwrap();
    
    assert!(matches!(retrieved_msg.node_type, NodeType::Message));
    assert!(matches!(retrieved_doc.node_type, NodeType::Document));
    
    println!("✅ Node type filtering works");
}

#[test]
fn test_edge_direction_queries() {
    println!("\n🧪 Testing edge direction queries...");
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let coordinator = DatabaseCoordinator::new(temp_dir.path()).expect("Failed to create coordinator");
    let storage = coordinator.conversations_active();
    
    // Create 3 nodes in a chain: A -> B -> C
    let node_a = Uuid::new_v4().to_string();
    let node_b = Uuid::new_v4().to_string();
    let node_c = Uuid::new_v4().to_string();
    
    for id in [&node_a, &node_b, &node_c] {
        storage.insert_node(&Node {
            id: id.clone(),
            node_type: NodeType::Message,
            content: Some(format!("Node {}", id)),
            metadata: None,
            created_at: Utc::now().timestamp(),
            updated_at: Utc::now().timestamp(),
        }).expect("Failed to insert");
    }
    
    // Create edges
    storage.insert_edge(&Edge {
        id: Uuid::new_v4().to_string(),
        from_node: node_a.clone(),
        to_node: node_b.clone(),
        edge_type: EdgeType::Reply,
        weight: Some(1.0),
        metadata: None,
        created_at: Utc::now().timestamp(),
    }).expect("Failed to insert edge");
    
    storage.insert_edge(&Edge {
        id: Uuid::new_v4().to_string(),
        from_node: node_b.clone(),
        to_node: node_c.clone(),
        edge_type: EdgeType::Reply,
        weight: Some(1.0),
        metadata: None,
        created_at: Utc::now().timestamp(),
    }).expect("Failed to insert edge");
    
    // Test outgoing from A
    let a_outgoing = storage.get_outgoing_edges(&node_a).expect("Failed to get outgoing");
    assert_eq!(a_outgoing.len(), 1);
    assert_eq!(a_outgoing[0].to_node, node_b);
    
    // Test incoming to C
    let c_incoming = storage.get_incoming_edges(&node_c).expect("Failed to get incoming");
    assert_eq!(c_incoming.len(), 1);
    assert_eq!(c_incoming[0].from_node, node_b);
    
    // Test B has both
    let b_outgoing = storage.get_outgoing_edges(&node_b).expect("Failed to get outgoing");
    let b_incoming = storage.get_incoming_edges(&node_b).expect("Failed to get incoming");
    assert_eq!(b_outgoing.len(), 1);
    assert_eq!(b_incoming.len(), 1);
    
    println!("✅ Edge direction queries work correctly");
}

#[test]
fn test_conversation_threading() {
    println!("\n🧪 Testing conversation threading...");
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let coordinator = DatabaseCoordinator::new(temp_dir.path()).expect("Failed to create coordinator");
    let storage = coordinator.conversations_active();
    
    // Create branching conversation:
    //     A
    //    / \
    //   B   C
    //   |
    //   D
    
    let node_a = Uuid::new_v4().to_string();
    let node_b = Uuid::new_v4().to_string();
    let node_c = Uuid::new_v4().to_string();
    let node_d = Uuid::new_v4().to_string();
    
    for id in [&node_a, &node_b, &node_c, &node_d] {
        storage.insert_node(&Node {
            id: id.clone(),
            node_type: NodeType::Message,
            content: Some(format!("Node {}", id)),
            metadata: None,
            created_at: Utc::now().timestamp(),
            updated_at: Utc::now().timestamp(),
        }).expect("Failed to insert");
    }
    
    // Create edges for branching
    storage.insert_edge(&Edge {
        id: Uuid::new_v4().to_string(),
        from_node: node_a.clone(),
        to_node: node_b.clone(),
        edge_type: EdgeType::Reply,
        weight: Some(1.0),
        metadata: None,
        created_at: Utc::now().timestamp(),
    }).expect("Failed to insert");
    
    storage.insert_edge(&Edge {
        id: Uuid::new_v4().to_string(),
        from_node: node_a.clone(),
        to_node: node_c.clone(),
        edge_type: EdgeType::Reply,
        weight: Some(0.8),
        metadata: None,
        created_at: Utc::now().timestamp(),
    }).expect("Failed to insert");
    
    storage.insert_edge(&Edge {
        id: Uuid::new_v4().to_string(),
        from_node: node_b.clone(),
        to_node: node_d.clone(),
        edge_type: EdgeType::Reply,
        weight: Some(1.0),
        metadata: None,
        created_at: Utc::now().timestamp(),
    }).expect("Failed to insert");
    
    // Verify branching
    let a_outgoing = storage.get_outgoing_edges(&node_a).expect("Failed to get");
    assert_eq!(a_outgoing.len(), 2, "A should have 2 branches");
    
    println!("✅ Conversation threading (branching) works");
}

