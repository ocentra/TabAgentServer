//! Concurrency tests for the storage layer

use common::DbResult;
use serde_json::json;
use std::sync::Arc;
use std::thread;
use storage::{Chat, Node, NodeId, StorageManager};

#[allow(unused_variables)]
#[test]
fn test_storage_manager_concurrent_access() -> DbResult<()> {
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
    let storage = Arc::new(StorageManager::new(
        temp_dir.path().join("test.db").to_str().unwrap(),
    )?);

    // Create a test chat
    let chat = Chat {
        id: NodeId::new("concurrent_chat".to_string()),
        title: "Concurrent Test Chat".to_string(),
        topic: "Concurrency".to_string(),
        created_at: 1697500000000,
        updated_at: 1697500000000,
        message_ids: vec![],
        summary_ids: vec![],
        embedding_id: None,
        metadata: json!({}),
    };

    // Insert the chat in the main thread
    storage.insert_node(&Node::Chat(chat))?;

    // Spawn multiple threads that will try to read the same chat
    let mut handles = vec![];

    for i in 0..5 {
        let storage_clone = Arc::clone(&storage);
        let handle = thread::spawn(move || -> DbResult<Option<Chat>> {
            // Each thread tries to read the chat
            match storage_clone.get_node("concurrent_chat")? {
                Some(Node::Chat(chat)) => Ok(Some(chat)),
                _ => Ok(None),
            }
        });
        handles.push(handle);
    }

    // Wait for all threads and collect results
    let mut results = vec![];
    for handle in handles {
        let result = handle.join().unwrap()?;
        results.push(result);
    }

    // All threads should have successfully read the chat
    assert_eq!(results.len(), 5);
    for result in results {
        assert!(result.is_some());
        assert_eq!(result.unwrap().title, "Concurrent Test Chat");
    }

    Ok(())
}

#[allow(unused_variables)]
#[test]
fn test_concurrent_writes() -> DbResult<()> {
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
    let storage = Arc::new(StorageManager::new(
        temp_dir.path().join("test.db").to_str().unwrap(),
    )?);

    // Spawn multiple threads that will try to write different chats
    let mut handles = vec![];

    for i in 0..5 {
        let storage_clone = Arc::clone(&storage);
        let handle = thread::spawn(move || -> DbResult<()> {
            let chat = Chat {
                id: NodeId::new(format!("concurrent_chat_{}", i)),
                title: format!("Concurrent Chat {}", i),
                topic: "Concurrency".to_string(),
                created_at: 1697500000000 + i as i64,
                updated_at: 1697500000000 + i as i64,
                message_ids: vec![],
                summary_ids: vec![],
                embedding_id: None,
                metadata: json!({}),
            };

            storage_clone.insert_node(&Node::Chat(chat))
        });
        handles.push(handle);
    }

    // Wait for all threads and collect results
    for handle in handles {
        let result = handle.join().unwrap();
        assert!(result.is_ok());
    }

    // Verify all chats were inserted
    for i in 0..5 {
        let chat_id = format!("concurrent_chat_{}", i);
        let chat = storage.get_node(&chat_id)?;
        assert!(chat.is_some());
        if let Some(Node::Chat(chat)) = chat {
            assert_eq!(chat.title, format!("Concurrent Chat {}", i));
        }
    }

    Ok(())
}
