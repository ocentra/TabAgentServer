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
    let storage: Arc<StorageManager<storage::engine::MdbxEngine>> = Arc::new(StorageManager::new(
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
                metadata: json!({}).to_string(),
    };

    // Insert the chat in the main thread
    storage.insert_node(&Node::Chat(chat))?;

    // Spawn multiple threads that will try to read the same chat
    let mut handles = vec![];

    for i in 0..5 {
        let storage_clone = Arc::clone(&storage);
        let handle = thread::spawn(move || -> DbResult<Option<Chat>> {
            use storage::engine::ReadGuard;
            // Each thread tries to read the chat - ZERO-COPY path
            if let Some(guard) = storage_clone.get_node_guard("concurrent_chat")? {
                let data_slice: &[u8] = ReadGuard::data(&guard);
                let data_vec: Vec<u8> = data_slice.to_vec();
                let node: Node = rkyv::from_bytes::<common::models::Node, rkyv::rancor::Error>(&data_vec)
                    .map_err(|e| common::DbError::Serialization(e.to_string()))?;
                match node {
                    Node::Chat(chat) => Ok(Some(chat)),
                    _ => Ok(None),
                }
            } else {
                Ok(None)
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
    let storage: Arc<StorageManager<storage::engine::MdbxEngine>> = Arc::new(StorageManager::new(
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
                metadata: json!({}).to_string(),
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

    // Verify all chats were inserted - ZERO-COPY path
    use storage::engine::ReadGuard;
    for i in 0..5 {
        let chat_id = format!("concurrent_chat_{}", i);
        if let Some(guard) = storage.get_node_guard(&chat_id)? {
            let data_slice: &[u8] = ReadGuard::data(&guard);
            let data_vec: Vec<u8> = data_slice.to_vec();
            let node: Node = rkyv::from_bytes::<common::models::Node, rkyv::rancor::Error>(&data_vec)
                .map_err(|e| common::DbError::Serialization(e.to_string()))?;
            if let Node::Chat(chat) = node {
                assert_eq!(chat.title, format!("Concurrent Chat {}", i));
            }
        } else {
            panic!("Chat should exist");
        }
    }

    Ok(())
}
