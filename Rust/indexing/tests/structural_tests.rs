//! Tests for the StructuralIndex implementation

use indexing::structural::StructuralIndex;
use tempfile::TempDir;
use common::NodeId;

fn create_test_index() -> (StructuralIndex, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let db = sled::open(temp_dir.path()).unwrap();
    let tree = db.open_tree("test_structural").unwrap();
    (StructuralIndex::new(tree), temp_dir)
}

#[test]
fn test_add_and_get() {
    let (index, _temp) = create_test_index();
    
    index.add("chat_id", "chat_123", "msg_1").unwrap();
    index.add("chat_id", "chat_123", "msg_2").unwrap();
    index.add("chat_id", "chat_456", "msg_3").unwrap();
    
    let results = index.get("chat_id", "chat_123").unwrap();
    assert_eq!(results.len(), 2);
    assert!(results.contains(&NodeId::from("msg_1")));
    assert!(results.contains(&NodeId::from("msg_2")));
    
    let results = index.get("chat_id", "chat_456").unwrap();
    assert_eq!(results.len(), 1);
    assert!(results.contains(&NodeId::from("msg_3")));
}

#[test]
fn test_remove() {
    let (index, _temp) = create_test_index();
    
    index.add("chat_id", "chat_123", "msg_1").unwrap();
    index.add("chat_id", "chat_123", "msg_2").unwrap();
    
    index.remove("chat_id", "chat_123", "msg_1").unwrap();
    
    let results = index.get("chat_id", "chat_123").unwrap();
    assert_eq!(results.len(), 1);
    assert!(results.contains(&NodeId::from("msg_2")));
}

#[test]
fn test_count() {
    let (index, _temp) = create_test_index();
    
    index.add("node_type", "Message", "msg_1").unwrap();
    index.add("node_type", "Message", "msg_2").unwrap();
    index.add("node_type", "Message", "msg_3").unwrap();
    
    let count = index.count("node_type", "Message").unwrap();
    assert_eq!(count, 3);
}

#[test]
fn test_nonexistent_property() {
    let (index, _temp) = create_test_index();
    
    let results = index.get("nonexistent", "value").unwrap();
    assert_eq!(results.len(), 0);
}

#[test]
fn test_duplicate_add() {
    let (index, _temp) = create_test_index();
    
    index.add("chat_id", "chat_123", "msg_1").unwrap();
    index.add("chat_id", "chat_123", "msg_1").unwrap(); // Duplicate
    
    let results = index.get("chat_id", "chat_123").unwrap();
    assert_eq!(results.len(), 1); // Set deduplicates
}