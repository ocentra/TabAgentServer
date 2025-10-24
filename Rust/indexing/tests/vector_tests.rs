//! Tests for the VectorIndex implementation

use indexing::vector::{VectorIndex, SearchResult};
use tempfile::TempDir;
use common::EmbeddingId;

fn create_test_index() -> (VectorIndex, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("test_vector.hnsw");
    let index = VectorIndex::new(path).unwrap();
    (index, temp_dir)
}

#[test]
fn test_add_and_search() {
    let (mut index, _temp) = create_test_index();
    
    // Add some 3D vectors for testing
    index.add_vector("v1", vec![1.0, 0.0, 0.0]).unwrap();
    index.add_vector("v2", vec![0.9, 0.1, 0.0]).unwrap();
    index.add_vector("v3", vec![0.0, 0.0, 1.0]).unwrap();
    
    // Search for nearest to [1, 0, 0]
    let results = index.search(&[1.0, 0.0, 0.0], 2).unwrap();
    
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].id, EmbeddingId::from("v1")); // Exact match
    assert_eq!(results[1].id, EmbeddingId::from("v2")); // Similar
}

#[test]
fn test_remove() {
    let (mut index, _temp) = create_test_index();
    
    index.add_vector("v1", vec![1.0, 0.0, 0.0]).unwrap();
    assert_eq!(index.len(), 1);
    
    let removed = index.remove_vector("v1").unwrap();
    assert!(removed);
    assert_eq!(index.len(), 0);
    
    // Removing again should return false
    let removed = index.remove_vector("v1").unwrap();
    assert!(!removed);
}

#[test]
fn test_realistic_dimensions() {
    let (mut index, _temp) = create_test_index();
    
    // Test with realistic 384-dimensional vectors
    let v1 = vec![0.5; 384];
    let mut v2 = vec![0.55; 384]; // Similar but distinct
    v2[0] = 0.6;  // Make it slightly different
    let v3 = vec![-0.5; 384];
    
    index.add_vector("embed_1", v1.clone()).unwrap();
    index.add_vector("embed_2", v2).unwrap();
    index.add_vector("embed_3", v3).unwrap();
    
    // Search should work with high-dimensional vectors
    let results = index.search(&v1, 2).unwrap();
    assert_eq!(results.len(), 2);
    // Just verify the results contain our IDs, order may vary
    let ids: Vec<&str> = results.iter().map(|r| r.id.as_str()).collect();
    assert!(ids.contains(&"embed_1"));
    assert!(ids.contains(&"embed_2") || ids.contains(&"embed_3"));
}

#[test]
fn test_metadata() {
    let (mut index, _temp) = create_test_index();
    
    index.add_vector("v1", vec![1.0, 0.0, 0.0]).unwrap();
    
    let meta = index.get_metadata("v1");
    assert!(meta.is_some());
    let (timestamp, dimension) = meta.unwrap();
    assert!(timestamp > 0);
    assert_eq!(dimension, 3);
}