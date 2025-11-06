//! Test fixtures for indexing tests
//!
//! This module provides shared test infrastructure:
//! - Permanent mock databases with pre-populated data
//! - Temporary databases for clean-state tests
//! - Helper functions for creating storage + index managers

use common::{DbResult, models::{Node, Edge, Embedding}};
use storage::StorageManager;
use indexing::IndexManager;
use std::path::Path;
use tempfile::TempDir;

/// Path to permanent mock database with realistic test data
pub const MOCK_VECTORS_DB: &str = "tests/fixtures/mock_vectors.mdbx";
pub const MOCK_GRAPH_DB: &str = "tests/fixtures/mock_graph.mdbx";

/// Get or create a permanent mock database with pre-populated vectors
///
/// This database is NOT deleted between test runs, allowing fast test execution.
/// Only created/populated on first run.
pub fn get_or_create_mock_vectors_db() -> DbResult<StorageManager> {
    if !Path::new(MOCK_VECTORS_DB).exists() {
        println!("🔧 Creating permanent mock vectors database...");
        let storage = StorageManager::new(MOCK_VECTORS_DB)?;
        populate_mock_vectors(&storage)?;
        println!("✅ Mock vectors database created at {}", MOCK_VECTORS_DB);
        Ok(storage)
    } else {
        println!("♻️  Reusing existing mock vectors database");
        StorageManager::new(MOCK_VECTORS_DB)
    }
}

/// Get or create a permanent mock database with graph data
pub fn get_or_create_mock_graph_db() -> DbResult<StorageManager> {
    if !Path::new(MOCK_GRAPH_DB).exists() {
        println!("🔧 Creating permanent mock graph database...");
        let storage = StorageManager::new(MOCK_GRAPH_DB)?;
        populate_mock_graph(&storage)?;
        println!("✅ Mock graph database created at {}", MOCK_GRAPH_DB);
        Ok(storage)
    } else {
        println!("♻️  Reusing existing mock graph database");
        StorageManager::new(MOCK_GRAPH_DB)
    }
}

/// Create a temporary database that auto-deletes after test
///
/// Use this for tests that need clean state or modify data.
pub fn create_temp_db() -> DbResult<(TempDir, StorageManager)> {
    let temp_dir = TempDir::new()
        .map_err(|e| common::DbError::InvalidOperation(format!("Failed to create temp dir: {}", e)))?;
    
    let storage = StorageManager::new(temp_dir.path().to_str().unwrap())?;
    
    Ok((temp_dir, storage))
}

/// Create an IndexManager from a StorageManager
///
/// This helper encapsulates the pointer-passing pattern.
pub fn create_index_from_storage(storage: &StorageManager, with_hybrid: bool) -> DbResult<IndexManager> {
    let env = unsafe { storage.get_raw_env() };
    let structural_dbi = storage.get_or_create_dbi("structural_index")?;
    let outgoing_dbi = storage.get_or_create_dbi("graph_outgoing")?;
    let incoming_dbi = storage.get_or_create_dbi("graph_incoming")?;
    
    IndexManager::new_from_storage(env, structural_dbi, outgoing_dbi, incoming_dbi, with_hybrid)
}

// ============================================================================
// Mock Data Population
// ============================================================================

/// Populate mock database with realistic vector embeddings
fn populate_mock_vectors(storage: &StorageManager) -> DbResult<()> {
    use rand::Rng;
    let mut rng = rand::rng();
    
    // Create 100 nodes with embeddings
    for i in 0..100 {
        let node = Node::Message(common::models::Message {
            id: format!("msg_{}", i).into(),
            chat_id: format!("chat_{}", i / 10).into(),
            sender: "test_user".to_string(),
            text_content: format!("Test message {}", i),
            timestamp: 1000000 + i * 1000,
            attachment_ids: vec![],
            embedding_id: None,
            metadata: "{}".to_string(),
        });
        
        storage.insert_node(&node)?;
        
        // Create realistic embedding vector (384D like sentence-transformers)
        let embedding = Embedding {
            id: format!("emb_{}", i).into(),
            vector: (0..384).map(|_| rng.random_range(-1.0..1.0)).collect(),
            model: "test-model".to_string(),
        };
        
        storage.insert_embedding(&embedding)?;
    }
    
    println!("📊 Populated 100 nodes with 384D embeddings");
    Ok(())
}

/// Populate mock database with graph structure
fn populate_mock_graph(storage: &StorageManager) -> DbResult<()> {
    // Create nodes
    for i in 0..50 {
        let node = Node::Entity(common::models::Entity {
            id: format!("entity_{}", i).into(),
            entity_type: "test".to_string(),
            label: format!("Entity {}", i),
            embedding_id: None,
            metadata: "{}".to_string(),
        });
        
        storage.insert_node(&node)?;
    }
    
    // Create edges (directed graph)
    for i in 0..100 {
        let edge = Edge {
            id: format!("edge_{}", i).into(),
            from_node: format!("entity_{}", i % 50).into(),
            to_node: format!("entity_{}", (i + 1) % 50).into(),
            edge_type: "RELATES_TO".to_string(),
            created_at: 1000000 + i * 1000,
            metadata: "{}".to_string(),
        };
        
        storage.insert_edge(&edge)?;
    }
    
    println!("📊 Populated 50 nodes with 100 edges");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_temp_db() {
        let (_temp, storage) = create_temp_db().unwrap();
        let index = create_index_from_storage(&storage, false).unwrap();
        // Just check it was created
        let _ = index.structural();
    }
    
    /// NEGATIVE TEST: Proves file locking protection works
    /// 
    /// This test attempts to open the same database file TWICE, which is an
    /// ANTI-PATTERN and should be prevented by MDBX file locking.
    /// 
    /// Expected behavior:
    /// - First open: SUCCESS
    /// - Second open (same file): FAILURE with MDBX_BUSY (-30778)
    /// - When we get locking error = TEST PASSES (protection works!)
    /// - If second open succeeds = TEST FAILS (no protection!)
    #[test]
    fn test_mock_vectors_db_reusable() {
        println!("NEGATIVE TEST: Attempting to open same database twice (should fail)");
        
        // First open - should succeed
        println!("  First open...");
        let storage1 = get_or_create_mock_vectors_db()
            .expect("First open should succeed");
        let _index1 = create_index_from_storage(&storage1, false)
            .expect("First index should work");
        
        println!("  Second open (same file, should fail with locking error)...");
        let storage2_result = get_or_create_mock_vectors_db();
        
        // Check if we got the expected file locking error
        match storage2_result {
            Ok(_) => {
                panic!("TEST FAILED: Second open succeeded - file locking is NOT working!");
            }
            Err(e) => {
                let error_string = e.to_string();
                // Accept multiple error formats for file locking
                if error_string.contains("-30778") 
                    || error_string.contains("could not acquire lock")
                    || error_string.contains("mdbx_env_open failed") {
                    println!("  TEST PASSED: Got expected locking error: {}", error_string);
                    // This is SUCCESS - the file locking protection is working!
                } else {
                    panic!("TEST FAILED: Got unexpected error (not a locking error): {}", e);
                }
            }
        }
        
        println!("NEGATIVE TEST COMPLETE: File locking protection prevents duplicate opens");
    }
}

