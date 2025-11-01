//! Example demonstrating archived data access.

use storage::{StorageManager, DefaultStorageManager};
use common::models::{Node, Message, NodeId};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create temporary database
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("zero_copy_demo");
    let storage = DefaultStorageManager::new(db_path.to_str().unwrap())?;

    // Insert a test message
    let message = Message {
        id: NodeId::new("msg_001"),
        chat_id: NodeId::new("chat_001"),
        sender: "user".to_string(),
        timestamp: 1234567890,
        text_content: "This is a test message with some content".to_string(),
        attachment_ids: vec![],
        embedding_id: None,
        metadata: "{}".to_string(),
    };
    storage.insert_node(&Node::Message(message))?;

    println!("=== Archived Access Example ===\n");

    // Method 1: get_node() - returns owned
    if let Some(node) = storage.get_node("msg_001")? {
        if let Node::Message(msg) = node {
            println!("get_node(): {}", msg.text_content);
        }
    }

    // Method 2: get_node_ref() - access archived fields
    if let Some(node_ref) = storage.get_node_ref("msg_001")? {
        if let Some(text) = node_ref.message_text() {
            println!("get_node_ref(): {}", text);
        }
    }
    
    println!("\n=== Filtering Example ===\n");
    
    // Insert more test data
    for i in 0..1000 {
        let msg = Message {
            id: NodeId::new(&format!("msg_{:04}", i)),
            chat_id: NodeId::new("chat_001"),
            sender: "user".to_string(),
            timestamp: 1234567890 + i,
            text_content: format!("Message {} - {}", i, 
                if i % 10 == 0 { "IMPORTANT" } else { "normal" }),
            attachment_ids: vec![],
            embedding_id: None,
            metadata: "{}".to_string(),
        };
        storage.insert_node(&Node::Message(msg))?;
    }

    // Scan and filter using archived field access
    let mut found_count = 0;
    for i in 0..1000 {
        let id = format!("msg_{:04}", i);
        if let Some(node_ref) = storage.get_node_ref(&id)? {
            if let Some(text) = node_ref.message_text() {
                if text.contains("IMPORTANT") {
                    found_count += 1;
                    let _owned = node_ref.deserialize()?;
                }
            }
        }
    }
    
    println!("Scanned 1000 messages");
    println!("Found {} matches", found_count);
    println!("Deserialized only matches");

    Ok(())
}

