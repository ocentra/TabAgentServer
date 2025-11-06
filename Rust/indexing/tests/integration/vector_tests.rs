//! 🔍 VECTOR INDEX TESTS - HNSW Semantic Search

use indexing::vector::VectorIndex;
use tempfile::TempDir;
use common::EmbeddingId;

fn create_test_index() -> (VectorIndex, TempDir) {
    create_test_index_with_dimension(3) // Default to 3D for simple tests
}

fn create_test_index_with_dimension(dimension: usize) -> (VectorIndex, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("test_vector.hnsw");
    let index = VectorIndex::new_with_dimension(path, dimension).unwrap();
    (index, temp_dir)
}

#[test]
fn test_add_and_search() {
    println!("\n🔍 TEST: Add 3D vectors and search with HNSW");
    let (mut index, _temp) = create_test_index();
    
    println!("   📝 Adding 3 vectors (3D)...");
    index.add_vector("v1", vec![1.0, 0.0, 0.0]).unwrap();
    index.add_vector("v2", vec![0.9, 0.1, 0.0]).unwrap();
    index.add_vector("v3", vec![0.0, 0.0, 1.0]).unwrap();
    
    println!("   🔎 Searching for nearest neighbors...");
    let results = index.search(&[1.0, 0.0, 0.0], 2).unwrap();
    
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].id, EmbeddingId::from("v1"));
    assert_eq!(results[1].id, EmbeddingId::from("v2"));
    println!("   ✅ PASS: Found {} nearest neighbors", results.len());
}

#[test]
fn test_remove() {
    println!("\n🗑️  TEST: Remove vector from HNSW index");
    let (mut index, _temp) = create_test_index();
    
    println!("   📝 Adding vector v1...");
    index.add_vector("v1", vec![1.0, 0.0, 0.0]).unwrap();
    assert_eq!(index.len(), 1);
    
    println!("   🗑️  Removing v1...");
    let removed = index.remove_vector("v1").unwrap();
    assert!(removed);
    assert_eq!(index.len(), 0);
    
    println!("   🗑️  Removing again (should fail)...");
    let removed = index.remove_vector("v1").unwrap();
    assert!(!removed);
    println!("   ✅ PASS: Vector removed successfully");
}

#[test]
fn test_realistic_dimensions() {
    println!("\n🌐 TEST: High-dimensional vectors (384D like real embeddings)");
    let (mut index, _temp) = create_test_index_with_dimension(384);
    
    println!("   📝 Creating 384-dimensional vectors...");
    let v1 = vec![0.5; 384];
    let mut v2 = vec![0.55; 384];
    v2[0] = 0.6;
    let v3 = vec![-0.5; 384];
    
    println!("   📝 Adding 3 high-dimensional vectors...");
    index.add_vector("embed_1", v1.clone()).unwrap();
    index.add_vector("embed_2", v2).unwrap();
    index.add_vector("embed_3", v3).unwrap();
    
    println!("   🔎 Searching in 384D space...");
    let results = index.search(&v1, 2).unwrap();
    assert_eq!(results.len(), 2);
    let ids: Vec<&str> = results.iter().map(|r| r.id.as_str()).collect();
    assert!(ids.contains(&"embed_1"));
    assert!(ids.contains(&"embed_2") || ids.contains(&"embed_3"));
    println!("   ✅ PASS: HNSW works with 384-dimensional vectors");
}

#[test]
fn test_metadata() {
    println!("\n📊 TEST: Vector metadata (timestamp, dimension)");
    let (mut index, _temp) = create_test_index();
    
    println!("   📝 Adding vector with metadata...");
    index.add_vector("v1", vec![1.0, 0.0, 0.0]).unwrap();
    
    println!("   📖 Reading metadata...");
    let meta = index.get_metadata("v1");
    assert!(meta.is_some());
    let (timestamp, dimension) = meta.unwrap();
    assert!(timestamp > 0);
    assert_eq!(dimension, 3);
    println!("   ✅ PASS: Metadata stored (timestamp={}, dim={})", timestamp, dimension);
}