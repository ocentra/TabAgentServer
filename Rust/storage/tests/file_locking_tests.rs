//! File locking tests to demonstrate the concurrency issue with multiple processes accessing the same database

use common::DbResult;
use serde_json::json;
use std::sync::Arc;
use std::thread;
use storage::{Chat, DatabaseCoordinator, Node, NodeId, StorageManager};

/// This test demonstrates what happens when multiple threads try to access the same database
/// from separate StorageManager instances pointing to the same file
#[test]
fn test_multiple_processes_same_database() -> DbResult<()> {
    // Create a temporary directory for our test database
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("shared_test.db");

    // Create the first StorageManager instance
    let storage1: StorageManager<storage::engine::MdbxEngine> = StorageManager::new(db_path.to_str().unwrap())?;

    // Try to create a second StorageManager instance pointing to the same database
    // This should demonstrate the file locking behavior
    let storage2_result: Result<StorageManager<storage::engine::MdbxEngine>, _> = StorageManager::new(db_path.to_str().unwrap());

    // The second attempt might fail due to file locking, or it might succeed but cause issues
    match storage2_result {
        Ok(storage2) => {
            println!(
                "Both StorageManager instances created successfully - testing concurrent access"
            );

            // Insert a chat using the first instance
            let chat1 = Chat {
                id: NodeId::new("chat_1".to_string()),
                title: "First Chat".to_string(),
                topic: "Testing".to_string(),
                created_at: 1697500000000,
                updated_at: 1697500000000,
                message_ids: vec![],
                summary_ids: vec![],
                embedding_id: None,
                metadata: json!({}).to_string(),
            };

            storage1.insert_node(&Node::Chat(chat1))?;

            // Try to read it using the second instance - ZERO-COPY path
            let guard = storage2.get_node_guard("chat_1")?;
            assert!(guard.is_some());
            println!("Successfully accessed the same database from two instances");
        }
        Err(e) => {
            println!("Failed to create second StorageManager instance: {}", e);
            // This demonstrates the file locking issue
            return Ok(());
        }
    }

    // Now test with multiple threads trying to access the same database
    let storage1_arc = Arc::new(storage1);
    let storage2: StorageManager<storage::engine::MdbxEngine> = StorageManager::new(db_path.to_str().unwrap())?;
    let storage2_arc = Arc::new(storage2);

    // Spawn multiple threads that will try to write to the database
    let mut handles = vec![];

    for i in 0..5 {
        let storage1_clone = Arc::clone(&storage1_arc);
        let storage2_clone = Arc::clone(&storage2_arc);
        let handle = thread::spawn(move || -> DbResult<()> {
            // Each thread inserts a different chat using different storage instances
            let chat = Chat {
                id: NodeId::new(format!("thread_chat_{}", i)),
                title: format!("Thread {} Chat", i),
                topic: "Concurrency".to_string(),
                created_at: 1697500000000 + i as i64,
                updated_at: 1697500000000 + i as i64,
                message_ids: vec![],
                summary_ids: vec![],
                embedding_id: None,
                metadata: json!({}).to_string(),
            };

            // Alternate between storage instances
            if i % 2 == 0 {
                storage1_clone.insert_node(&Node::Chat(chat))
            } else {
                storage2_clone.insert_node(&Node::Chat(chat))
            }
        });
        handles.push(handle);
    }

    // Wait for all threads and collect results
    for handle in handles {
        let result = handle.join().unwrap();
        // Print any errors but don't fail the test
        if let Err(e) = result {
            println!("Thread error: {}", e);
        }
    }

    Ok(())
}

/// This test demonstrates what happens when multiple coordinators try to access databases
/// from the same directory paths
#[test]
fn test_coordinator_with_shared_paths() -> DbResult<()> {
    // Create a temporary directory for our test
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");

    // Create the first coordinator with a specific path
    let coordinator1 = DatabaseCoordinator::with_base_path(Some(temp_dir.path().to_path_buf()))?;

    // Try to create a second coordinator with the same path
    // This should demonstrate the file locking behavior at the coordinator level
    let coordinator2_result =
        DatabaseCoordinator::with_base_path(Some(temp_dir.path().to_path_buf()));

    match coordinator2_result {
        Ok(coordinator2) => {
            println!("Both coordinators created successfully - testing concurrent access");

            // Insert a chat using the first coordinator
            let chat1 = Chat {
                id: NodeId::new("coord_chat_1".to_string()),
                title: "Coordinator Chat".to_string(),
                topic: "Testing".to_string(),
                created_at: 1697500000000,
                updated_at: 1697500000000,
                message_ids: vec![],
                summary_ids: vec![],
                embedding_id: None,
                metadata: json!({}).to_string(),
            };

            coordinator1.insert_chat(chat1)?;

            // Try to read it using the second coordinator
            let retrieved = coordinator2.get_chat("coord_chat_1")?;
            assert!(retrieved.is_some());
            println!("Successfully accessed the same database from two coordinators");

            // Now test with multiple threads
            let coord1_arc = Arc::new(coordinator1);
            let coord2_arc = Arc::new(coordinator2);

            // Spawn multiple threads that will try to write to the database
            let mut handles = vec![];

            for i in 0..5 {
                let coord1_clone = Arc::clone(&coord1_arc);
                let coord2_clone = Arc::clone(&coord2_arc);
                let handle = thread::spawn(move || -> DbResult<()> {
                    // Each thread inserts a different chat using different coordinators
                    let chat = Chat {
                        id: NodeId::new(format!("coord_thread_chat_{}", i)),
                        title: format!("Coord Thread {} Chat", i),
                        topic: "Concurrency".to_string(),
                        created_at: 1697500000000 + i as i64,
                        updated_at: 1697500000000 + i as i64,
                        message_ids: vec![],
                        summary_ids: vec![],
                        embedding_id: None,
                        metadata: json!({}).to_string(),
                    };

                    // Alternate between coordinators
                    if i % 2 == 0 {
                        coord1_clone.insert_chat(chat)
                    } else {
                        coord2_clone.insert_chat(chat)
                    }
                });
                handles.push(handle);
            }

            // Wait for all threads and collect results
            for handle in handles {
                let result = handle.join().unwrap();
                // Print any errors but don't fail the test
                if let Err(e) = result {
                    println!("Coordinator thread error: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Failed to create second coordinator: {}", e);
            // This demonstrates the file locking issue at the coordinator level
        }
    }

    Ok(())
}

/// This test simulates the real-world scenario where multiple processes might try
/// to access the same database files simultaneously
#[test]
fn test_real_world_concurrent_access() -> DbResult<()> {
    // Create a temporary directory for our test database
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");

    // Create a coordinator with a specific path
    let coordinator = DatabaseCoordinator::with_base_path(Some(temp_dir.path().to_path_buf()))?;
    let coordinator_arc = Arc::new(coordinator);

    // Spawn multiple threads that will try to access the same coordinator
    let mut handles = vec![];

    for i in 0..10 {
        let coordinator_clone = Arc::clone(&coordinator_arc);
        let handle = thread::spawn(move || -> DbResult<()> {
            // Each thread performs a series of operations
            let chat_id = format!("real_world_chat_{}", i);

            // Insert a chat
            let chat = Chat {
                id: NodeId::new(chat_id.clone()),
                title: format!("Real World Chat {}", i),
                topic: "Concurrency Testing".to_string(),
                created_at: 1697500000000 + i as i64,
                updated_at: 1697500000000 + i as i64,
                message_ids: vec![],
                summary_ids: vec![],
                embedding_id: None,
                metadata: json!({}).to_string(),
            };

            coordinator_clone.insert_chat(chat)?;

            // Try to retrieve the chat
            let retrieved = coordinator_clone.get_chat(&chat_id)?;
            assert!(retrieved.is_some());

            // Insert a message
            let message = storage::Message {
                id: NodeId::new(format!("msg_{}", i)),
                chat_id: NodeId::new(chat_id),
                sender: "test_user".to_string(),
                timestamp: 1697500000000 + i as i64,
                text_content: format!("Test message {}", i),
                attachment_ids: vec![],
                embedding_id: None,
                metadata: json!({}).to_string(),
            };

            coordinator_clone.insert_message(message)?;

            Ok(())
        });
        handles.push(handle);
    }

    // Wait for all threads and collect results
    let mut success_count = 0;
    let mut error_count = 0;

    for handle in handles {
        let result = handle.join().unwrap();
        match result {
            Ok(()) => success_count += 1,
            Err(e) => {
                error_count += 1;
                println!("Thread error: {}", e);
            }
        }
    }

    println!(
        "Successful threads: {}, Failed threads: {}",
        success_count, error_count
    );

    Ok(())
}
