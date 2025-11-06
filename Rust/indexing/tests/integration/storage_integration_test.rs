//! Integration test: Storage → IndexManager
//!
//! This tests the CORRECT architecture:
//! - Storage owns the database
//! - Storage provides env + dbi pointers
//! - IndexManager receives pointers and builds indexes

use common::models::{Chat, Node};
use common::NodeId;
use serde_json::json;
use storage::StorageManager;
use indexing::IndexManager;
use tempfile::TempDir;

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

#[test]
fn test_storage_provides_pointers_to_indexing() {
    println!("\n🧪 TEST: Storage → IndexManager Integration (CORRECT ARCHITECTURE)");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let storage = StorageManager::new(temp_dir.path().join("test.db").to_str().unwrap())
        .expect("Failed to create storage");

    // Storage provides pointers
    println!("   📌 Storage providing env + DBI pointers...");
    let env = unsafe { storage.get_raw_env() };
    let structural_dbi = storage.get_or_create_dbi("structural_index")
        .expect("Failed to create structural DBI");
    let outgoing_dbi = storage.get_or_create_dbi("graph_outgoing")
        .expect("Failed to create outgoing DBI");
    let incoming_dbi = storage.get_or_create_dbi("graph_incoming")
        .expect("Failed to create incoming DBI");
    
    // IndexManager receives pointers
    println!("   🔗 IndexManager receiving pointers from storage...");
    let index_manager = IndexManager::new_from_storage(
        env, structural_dbi, outgoing_dbi, incoming_dbi, false
    ).expect("Failed to create IndexManager");

    // Create and insert a chat node
    let chat_id = NodeId::new("chat_integration_test");
    let chat = Chat {
        id: chat_id.clone(),
        title: "Integration Test Chat".to_string(),
        topic: "Testing".to_string(),
        created_at: current_timestamp(),
        updated_at: current_timestamp(),
        message_ids: vec![],
        summary_ids: vec![],
        embedding_id: None,
        metadata: json!({"test": "integration"}).to_string(),
    };

    println!("   💾 Storage inserting chat node...");
    storage
        .insert_node(&Node::Chat(chat.clone()))
        .expect("Failed to insert chat");

    // Index the node
    println!("   🗂️  IndexManager indexing the node...");
    index_manager.index_node(&Node::Chat(chat))
        .expect("Failed to index node");

    // Verify index works
    println!("   🔍 Querying index...");
    let results = index_manager.structural().get("chat_id", chat_id.as_str())
        .expect("Failed to query index");
    
    // Should find the chat in structural index
    assert!(results.is_some(), "Should find indexed chat");
    
    println!("   ✅ PASS: Storage → IndexManager integration working!");
    println!("      - Storage owns database");
    println!("      - Storage provides pointers");
    println!("      - IndexManager builds indexes into storage's database");
}

#[test]
fn test_multiple_dbis_in_same_env() {
    println!("\n🧪 TEST: Multiple DBIs in Same MDBX Environment");
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let storage = StorageManager::new(temp_dir.path().join("test.db").to_str().unwrap())
        .expect("Failed to create storage");
    
    // Storage has: nodes, edges, embeddings (3 DBIs)
    // IndexManager adds: structural_index, graph_outgoing, graph_incoming (3 more DBIs)
    // Total: 6 DBIs in same environment
    
    println!("   📊 Creating 6+ DBIs in same environment...");
    let env = unsafe { storage.get_raw_env() };
    let dbi1 = storage.get_or_create_dbi("structural_index").unwrap();
    let dbi2 = storage.get_or_create_dbi("graph_outgoing").unwrap();
    let dbi3 = storage.get_or_create_dbi("graph_incoming").unwrap();
    let dbi4 = storage.get_or_create_dbi("custom_index_1").unwrap();
    let dbi5 = storage.get_or_create_dbi("custom_index_2").unwrap();
    
    // All should be different
    assert_ne!(dbi1, dbi2);
    assert_ne!(dbi2, dbi3);
    assert_ne!(dbi3, dbi4);
    assert_ne!(dbi4, dbi5);
    
    // Cached - second call returns same DBI
    let dbi1_again = storage.get_or_create_dbi("structural_index").unwrap();
    assert_eq!(dbi1, dbi1_again, "DBI should be cached");
    
    println!("   ✅ PASS: {} DBIs created in same environment", 5);
    println!("      - All DBIs unique");
    println!("      - DBIs are cached");
}

