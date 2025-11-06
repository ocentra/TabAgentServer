//! 🔥 LOCK-FREE HOT VECTOR INDEX TESTS - Concurrent Vector Operations

use indexing::lock_free::lock_free_hot_vector::LockFreeHotVectorIndex;
use std::sync::Arc;
use std::thread;

#[test]
fn test_lock_free_hot_vector_index_basic() {
    println!("\n🔥 TEST: Lock-free hot vector index basic operations");
    let index = LockFreeHotVectorIndex::new();
    
    println!("   📝 Adding vector...");
    let vector = vec![0.1, 0.2, 0.3, 0.4];
    assert!(index.add_vector("vec1", vector).is_ok());
    assert_eq!(index.len(), 1);
    
    println!("   🗑️  Removing vector...");
    assert!(index.remove_vector("vec1").unwrap());
    assert_eq!(index.len(), 0);
    println!("   ✅ PASS: Add/remove works");
}

#[test]
fn test_lock_free_hot_vector_index_concurrent() {
    println!("\n🔥 TEST: Lock-free hot vector index concurrent adds");
    let index = Arc::new(LockFreeHotVectorIndex::new());
    let mut handles = vec![];
    
    println!("   🧵 Spawning 10 threads...");
    for i in 0..10 {
        let index_clone = Arc::clone(&index);
        let handle = thread::spawn(move || {
            let vector = vec![i as f32 * 0.1; 4];
            index_clone.add_vector(&format!("vec{}", i), vector).unwrap();
        });
        handles.push(handle);
    }
    
    println!("   ⏳ Waiting for threads...");
    for handle in handles {
        handle.join().unwrap();
    }
    
    assert_eq!(index.len(), 10);
    println!("   ✅ PASS: {} concurrent vector adds successful", index.len());
}

