//! 🗄️  VECTOR STORAGE TESTS - Different Storage Backends

use indexing::advanced::vector_storage::{InMemoryVectorStorage, MmapVectorStorage, ChunkedVectorStorage, VectorStorage};
use indexing::EmbeddingId;
use tempfile::TempDir;

#[test]
fn test_in_memory_storage() {
    println!("\n🗄️  TEST: In-memory vector storage");
    let mut storage = InMemoryVectorStorage::new();
    
    let id = EmbeddingId::from("test_vector");
    let vector = vec![1.0, 2.0, 3.0];
    
    println!("   📝 Adding vector...");
    assert!(storage.add_vector(id.clone(), vector.clone()).is_ok());
    assert_eq!(storage.len(), 1);
    
    println!("   📖 Retrieving vector...");
    let retrieved = storage.get_vector(&id).unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap(), &vector);
    
    println!("   🗑️  Removing vector...");
    assert!(storage.remove_vector(&id).unwrap());
    assert_eq!(storage.len(), 0);
    assert!(storage.get_vector(&id).unwrap().is_none());
    println!("   ✅ PASS: In-memory storage works");
}

#[test]
fn test_mmap_storage() {
    println!("\n🗺️  TEST: Memory-mapped vector storage");
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test_vectors.bin");
    
    println!("   📝 Creating mmap storage...");
    let mut storage = MmapVectorStorage::new(&file_path, 3).unwrap();
    
    let id = EmbeddingId::from("test_vector");
    let vector = vec![1.0, 2.0, 3.0];
    
    println!("   📝 Adding vector...");
    assert!(storage.add_vector(id.clone(), vector.clone()).is_ok());
    assert_eq!(storage.len(), 1);
    
    println!("   🗑️  Removing vector...");
    assert!(storage.remove_vector(&id).unwrap());
    assert_eq!(storage.len(), 0);
    println!("   ✅ PASS: Mmap storage works");
}

#[test]
fn test_chunked_storage() {
    println!("\n📦 TEST: Chunked vector storage");
    let temp_dir = TempDir::new().unwrap();
    
    println!("   📝 Creating chunked storage (10 vectors/chunk)...");
    let mut storage = ChunkedVectorStorage::new(&temp_dir, 10, 3).unwrap();
    
    let id = EmbeddingId::from("test_vector");
    let vector = vec![1.0, 2.0, 3.0];
    
    println!("   📝 Adding vector...");
    assert!(storage.add_vector(id.clone(), vector.clone()).is_ok());
    assert_eq!(storage.len(), 1);
    
    println!("   🗑️  Removing vector...");
    assert!(storage.remove_vector(&id).unwrap());
    assert_eq!(storage.len(), 0);
    println!("   ✅ PASS: Chunked storage works");
}

