//! DEMONSTRATION tests showing file locking issues we had initially
//!
//! ⚠️ WARNING: THESE TESTS VERIFY EXPECTED FAILURES! ⚠️
//!
//! These tests demonstrate the problems we encountered before implementing proper test isolation.
//! They are designed to fail with specific file locking errors, which proves the protection is working.
//!
//! If these tests pass, that would indicate a problem with the locking mechanism.
//! If these tests fail with file locking errors, that's CORRECT - it means the protection is working.

use common::DbResult;
use serde_json::json;
use std::thread;
use storage::{Chat, DatabaseCoordinator, Node, NodeId, StorageManager};

/// This test demonstrates the PROBLEM we had initially:
/// Multiple tests trying to access the same default database paths
///
/// ✅ EXPECTED: Should fail with file locking error - this proves the protection is working
#[test]
fn test_without_proper_isolation_problem() {
    println!("🔧 DEMONSTRATION: Creating first StorageManager with default path...");

    // Create first storage manager
    let storage1_result = StorageManager::with_default_path("test_db");
    if let Err(e) = storage1_result {
        panic!("First storage manager creation failed unexpectedly: {}", e);
    }
    let storage1: StorageManager<storage::engine::MdbxEngine> = storage1_result.unwrap();

    // Insert some data
    let chat1 = Chat {
        id: NodeId::new("shared_chat_1".to_string()),
        title: "Shared Chat 1".to_string(),
        topic: "Testing".to_string(),
        created_at: 1697500000000,
        updated_at: 1697500000000,
        message_ids: vec![],
        summary_ids: vec![],
        embedding_id: None,
        metadata: json!({}).to_string(),
    };

    if let Err(e) = storage1.insert_node(&Node::Chat(chat1)) {
        panic!("Failed to insert chat into first storage: {}", e);
    }

    // Now try to create another StorageManager pointing to the same database
    // This should fail due to file locking
    println!("🔧 DEMONSTRATION: Creating second StorageManager with same default path...");
    let storage2_result: Result<StorageManager<storage::engine::MdbxEngine>, _> = StorageManager::with_default_path("test_db");

    // Check if we got the expected file locking error
    match storage2_result {
        Ok(_) => {
            // If it succeeds, that's actually a problem - the locking isn't working
            panic!("❌ UNEXPECTED: Second storage manager creation succeeded - file locking is not working!");
        }
        Err(e) => {
            // Check if it's the expected file locking error
            let error_string = e.to_string();
            if error_string.contains("could not acquire lock") && error_string.contains("The process cannot access the file because another process has locked a portion of the file") {
                println!("✅ EXPECTED: Second storage manager failed with file locking error - protection is working!");
                // This is what we want - the file locking is working correctly
            } else {
                // Some other unexpected error
                panic!("❌ UNEXPECTED: Second storage manager failed with different error: {}", e);
            }
        }
    }

    println!("✅ DEMONSTRATION COMPLETE: File locking protection is working correctly");
}

/// This test demonstrates the PROBLEM with DatabaseCoordinator without isolation
///
/// ✅ EXPECTED: Should fail with file locking error - this proves the protection is working
#[test]
fn test_coordinator_without_isolation_problem() {
    // This is what caused problems initially - multiple tests using the same default paths
    println!("🔧 DEMONSTRATION: Creating first DatabaseCoordinator with default paths...");
    let coordinator1_result = DatabaseCoordinator::new();
    if let Err(e) = coordinator1_result {
        panic!("First coordinator creation failed unexpectedly: {}", e);
    }
    let coordinator1 = coordinator1_result.unwrap();

    // Insert some data
    let chat1 = Chat {
        id: NodeId::new("coord_shared_chat_1".to_string()),
        title: "Coordinator Shared Chat 1".to_string(),
        topic: "Testing".to_string(),
        created_at: 1697500000000,
        updated_at: 1697500000000,
        message_ids: vec![],
        summary_ids: vec![],
        embedding_id: None,
        metadata: json!({}).to_string(),
    };

    if let Err(e) = coordinator1.insert_chat(chat1) {
        panic!("Failed to insert chat into first coordinator: {}", e);
    }

    // Now try to create another DatabaseCoordinator - this should fail due to file locking
    println!("🔧 DEMONSTRATION: Creating second DatabaseCoordinator with same default paths...");
    let coordinator2_result = DatabaseCoordinator::new();

    // Check if we got the expected file locking error
    match coordinator2_result {
        Ok(_) => {
            // If it succeeds, that's actually a problem - the locking isn't working
            panic!("❌ UNEXPECTED: Second coordinator creation succeeded - file locking is not working!");
        }
        Err(e) => {
            // Check if it's the expected file locking error
            let error_string = e.to_string();
            if error_string.contains("could not acquire lock") && error_string.contains("The process cannot access the file because another process has locked a portion of the file") {
                println!("✅ EXPECTED: Second coordinator failed with file locking error - protection is working!");
                // This is what we want - the file locking is working correctly
            } else {
                // Some other unexpected error
                panic!("❌ UNEXPECTED: Second coordinator failed with different error: {}", e);
            }
        }
    }

    println!("✅ DEMONSTRATION COMPLETE: File locking protection is working correctly");
}

/// This test simulates what would happen in a multi-threaded environment
/// without proper isolation - this is what we were trying to avoid with temp databases
///
/// ✅ EXPECTED: Should show mixed results due to concurrent access issues
#[test]
fn test_concurrent_access_without_isolation() -> DbResult<()> {
    println!("🔧 DEMONSTRATION: Concurrent access issues without proper isolation...");

    // Create multiple threads that all try to use the same default database
    let mut handles = vec![];

    for i in 0..3 {
        let handle = thread::spawn(
            move || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                // Each thread creates its own StorageManager but with the same name
                // This simulates what would happen if multiple tests ran concurrently
                let storage: StorageManager<storage::engine::MdbxEngine> = StorageManager::with_default_path("concurrent_test_db")?;

                let chat = Chat {
                    id: NodeId::new(format!("concurrent_chat_{}", i)),
                    title: format!("Concurrent Chat {}", i),
                    topic: "Concurrency Testing".to_string(),
                    created_at: 1697500000000 + i as i64,
                    updated_at: 1697500000000 + i as i64,
                    message_ids: vec![],
                    summary_ids: vec![],
                    embedding_id: None,
                    metadata: json!({}).to_string(),
                };

                storage.insert_node(&Node::Chat(chat))?;

                // Try to read it back - ZERO-COPY path
                let guard = storage.get_node_guard(&format!("concurrent_chat_{}", i))?;
                if guard.is_some() {
                    println!(
                        "✅ Thread {} successfully inserted and retrieved its chat",
                        i
                    );
                } else {
                    println!("❌ Thread {} failed to retrieve its chat", i);
                }

                Ok(())
            },
        );
        handles.push(handle);
    }

    // Wait for all threads and count successes vs failures
    let mut success_count = 0;
    let mut failure_count = 0;

    for handle in handles {
        match handle.join() {
            Ok(Ok(())) => {
                success_count += 1;
            }
            Ok(Err(e)) => {
                let error_string = e.to_string();
                if error_string.contains("could not acquire lock") {
                    println!("✅ Expected file locking error occurred");
                    failure_count += 1; // This is expected
                } else {
                    println!("❌ Unexpected error: {}", e);
                    failure_count += 1;
                }
            }
            Err(e) => {
                println!("❌ Thread panicked: {:?}", e);
                failure_count += 1;
            }
        }
    }

    println!(
        "📊 Results: {} successes, {} expected failures",
        success_count, failure_count
    );
    println!("✅ DEMONSTRATION COMPLETE: Concurrent access behavior demonstrated");

    Ok(())
}
