//! Tests for the StructuralIndex implementation - TRUE ZERO-COPY!

use indexing::IndexManager;
use tempfile::TempDir;

fn create_test_manager() -> (IndexManager, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let manager = IndexManager::new(temp_dir.path()).unwrap();
    (manager, temp_dir)
}

#[test]
fn test_add_and_get_zero_copy() {
    let (manager, _temp) = create_test_manager();
    let index = manager.structural();
    
    index.add("chat_id", "chat_123", "msg_1").unwrap();
    index.add("chat_id", "chat_123", "msg_2").unwrap();
    index.add("chat_id", "chat_456", "msg_3").unwrap();
    
    // TRUE ZERO-COPY READ!
    let guard = index.get("chat_id", "chat_123").unwrap().expect("Should have results");
    assert_eq!(guard.len(), 2);
    
    // Zero-copy string iteration!
    let strs: Vec<&str> = guard.iter_strs().collect();
    assert!(strs.contains(&"msg_1"));
    assert!(strs.contains(&"msg_2"));
    
    let guard2 = index.get("chat_id", "chat_456").unwrap().expect("Should have results");
    assert_eq!(guard2.len(), 1);
    assert!(guard2.contains_str("msg_3"));
}

#[test]
fn test_remove() {
    let (manager, _temp) = create_test_manager();
    let index = manager.structural();
    
    index.add("chat_id", "chat_123", "msg_1").unwrap();
    index.add("chat_id", "chat_123", "msg_2").unwrap();
    
    index.remove("chat_id", "chat_123", "msg_1").unwrap();
    
    let guard = index.get("chat_id", "chat_123").unwrap().expect("Should have results");
    assert_eq!(guard.len(), 1);
    assert!(guard.contains_str("msg_2"));
}

#[test]
fn test_count_zero_copy() {
    let (manager, _temp) = create_test_manager();
    let index = manager.structural();
    
    index.add("node_type", "Message", "msg_1").unwrap();
    index.add("node_type", "Message", "msg_2").unwrap();
    index.add("node_type", "Message", "msg_3").unwrap();
    
    // TRUE ZERO-COPY COUNT - O(1)!
    let count = index.count("node_type", "Message").unwrap();
    assert_eq!(count, 3);
}

#[test]
fn test_nonexistent_property() {
    let (manager, _temp) = create_test_manager();
    let index = manager.structural();
    
    let result = index.get("nonexistent", "value").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_duplicate_add() {
    let (manager, _temp) = create_test_manager();
    let index = manager.structural();
    
    index.add("chat_id", "chat_123", "msg_1").unwrap();
    index.add("chat_id", "chat_123", "msg_1").unwrap(); // Duplicate
    
    let guard = index.get("chat_id", "chat_123").unwrap().expect("Should have results");
    assert_eq!(guard.len(), 1); // Binary search deduplicates
}

#[test]
fn test_to_owned_when_needed() {
    let (manager, _temp) = create_test_manager();
    let index = manager.structural();
    
    index.add("chat_id", "chat_123", "msg_1").unwrap();
    index.add("chat_id", "chat_123", "msg_2").unwrap();
    
    let guard = index.get("chat_id", "chat_123").unwrap().expect("Should have results");
    
    // Can convert to owned Vec<NodeId> if needed (allocates)
    let owned = guard.to_owned().unwrap();
    assert_eq!(owned.len(), 2);
}
