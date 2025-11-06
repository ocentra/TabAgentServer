//! 🗺️  MEMORY MAPPING TESTS - Zero-Copy Vector Storage

use indexing::advanced::vector_storage::MmapVectorStorage;
use tempfile::TempDir;

#[test]
fn test_mmap_vector_storage_write_and_read() {
    println!("\n🗺️  TEST: Memory-mapped vector write/read");
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("vectors.mmap");
    
    println!("   📝 Creating mmap storage (3D)...");
    let mut storage = MmapVectorStorage::new(&file_path, 3).unwrap();
    
    println!("   ✍️  Writing vector to mmap...");
    let vector = vec![1.0, 2.0, 3.0];
    let offset = storage.write_vector(&vector).unwrap();
    
    println!("   📖 Reading vector from mmap (ZERO-COPY)...");
    let read_vector = storage.read_vector(offset).unwrap();
    
    assert_eq!(read_vector, vector);
    println!("   ✅ PASS: Vector written and read via mmap");
}

#[test]
fn test_mmap_multiple_vectors() {
    println!("\n🗺️  TEST: Memory-map multiple vectors");
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("vectors.mmap");
    
    println!("   📝 Creating mmap storage (4D)...");
    let mut storage = MmapVectorStorage::new(&file_path, 4).unwrap();
    
    println!("   ✍️  Writing 3 vectors...");
    let v1 = vec![1.0, 0.0, 0.0, 0.0];
    let v2 = vec![0.0, 1.0, 0.0, 0.0];
    let v3 = vec![0.0, 0.0, 1.0, 0.0];
    
    let offset1 = storage.write_vector(&v1).unwrap();
    let offset2 = storage.write_vector(&v2).unwrap();
    let offset3 = storage.write_vector(&v3).unwrap();
    
    assert_ne!(offset1, offset2);
    assert_ne!(offset2, offset3);
    
    println!("   📖 Reading all 3 vectors (ZERO-COPY)...");
    assert_eq!(storage.read_vector(offset1).unwrap(), v1);
    assert_eq!(storage.read_vector(offset2).unwrap(), v2);
    assert_eq!(storage.read_vector(offset3).unwrap(), v3);
    println!("   ✅ PASS: {} vectors stored at different offsets", 3);
}

#[test]
fn test_mmap_persistence() {
    println!("\n💾 TEST: Memory-map persistence across instances");
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("vectors.mmap");
    
    let vector = vec![4.0, 5.0, 6.0];
    let offset;
    
    println!("   ✍️  Writing vector (first mmap instance)...");
    {
        let mut storage = MmapVectorStorage::new(&file_path, 3).unwrap();
        offset = storage.write_vector(&vector).unwrap();
        println!("   💾 Dropping storage (should flush to disk)...");
    }
    
    println!("   📖 Reading vector (new mmap instance)...");
    {
        let storage = MmapVectorStorage::new(&file_path, 3).unwrap();
        let read_vector = storage.read_vector(offset).unwrap();
        assert_eq!(read_vector, vector);
    }
    println!("   ✅ PASS: Data persists across mmap instances");
}

