//! Common test utilities for ALL tests.
//!
//! Provides real MDBX database setup, not fake bullshit!

use indexing::IndexManager;
use tempfile::TempDir;
use common::{NodeId, EdgeId};
use common::models::Edge;

/// Creates a real IndexManager with MDBX database in temp directory (NEW ARCHITECTURE).
///
/// Returns (IndexManager, TempDir, StorageManager)
/// - TempDir auto-deletes when dropped
/// - StorageManager must be kept alive for IndexManager to work
pub fn setup_real_db() -> (IndexManager, TempDir, storage::StorageManager) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let storage = storage::StorageManager::new(temp_dir.path().to_str().unwrap())
        .expect("Failed to create StorageManager");
    
    // Create IndexManager using storage's pointers
    let env = unsafe { storage.get_raw_env() };
    let structural_dbi = storage.get_or_create_dbi("structural_index").unwrap();
    let outgoing_dbi = storage.get_or_create_dbi("graph_outgoing").unwrap();
    let incoming_dbi = storage.get_or_create_dbi("graph_incoming").unwrap();
    
    let manager = IndexManager::new_from_storage(env, structural_dbi, outgoing_dbi, incoming_dbi, false)
        .expect("Failed to create IndexManager with REAL MDBX database");
    
    (manager, temp_dir, storage)
}

/// Creates a real IndexManager with hybrid (hot/warm/cold) tiers enabled (NEW ARCHITECTURE).
pub fn setup_real_db_with_hybrid() -> (IndexManager, TempDir, storage::StorageManager) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let storage = storage::StorageManager::new(temp_dir.path().to_str().unwrap())
        .expect("Failed to create StorageManager");
    
    let env = unsafe { storage.get_raw_env() };
    let structural_dbi = storage.get_or_create_dbi("structural_index").unwrap();
    let outgoing_dbi = storage.get_or_create_dbi("graph_outgoing").unwrap();
    let incoming_dbi = storage.get_or_create_dbi("graph_incoming").unwrap();
    
    let manager = IndexManager::new_from_storage(env, structural_dbi, outgoing_dbi, incoming_dbi, true)
        .expect("Failed to create IndexManager with hybrid tiers");
    
    (manager, temp_dir, storage)
}

/// Helper to create test Edge struct using configuration constants.
pub fn test_edge(id: &str, from: &str, to: &str) -> Edge {
    use indexing::config::test_constants;
    
    Edge {
        id: EdgeId::from(id),
        from_node: NodeId::from(from),
        to_node: NodeId::from(to),
        edge_type: test_constants::DEFAULT_EDGE_TYPE.to_string(),
        created_at: test_constants::DEFAULT_TIMESTAMP,
        metadata: test_constants::DEFAULT_METADATA.to_string(),
    }
}

/// Helper to create multiple test edges at once.
pub fn test_edges(edges: &[(&str, &str, &str)]) -> Vec<Edge> {
    edges
        .iter()
        .map(|(id, from, to)| test_edge(id, from, to))
        .collect()
}

/// Asserts that IndexManager is using REAL MDBX (not fake in-memory).
pub fn assert_uses_real_mdbx(manager: &IndexManager) {
    // If we can add and retrieve data, MDBX is working
    let edge = test_edge("test_edge", "node_a", "node_b");
    manager.index_edge(&edge).expect("MDBX write failed");
    
    let guard = manager.graph().get_outgoing("node_a")
        .expect("MDBX read failed")
        .expect("Should have edge");
    
    assert!(guard.contains_edge("test_edge"), "MDBX didn't persist data!");
}

