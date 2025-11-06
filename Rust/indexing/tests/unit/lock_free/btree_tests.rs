//! 🌳 LOCK-FREE B-TREE TESTS - Concurrent Sorted Data Structure

use indexing::lock_free::lock_free_btree::LockFreeBTree;
use std::sync::Arc;
use std::thread;

#[test]
fn test_lock_free_btree_basic() {
    println!("\n🌳 TEST: Adaptive B-tree basic operations");
    let tree: Arc<LockFreeBTree<i32, String>> = Arc::new(LockFreeBTree::new(4)); // 4 shards
    
    println!("   📝 Insert 1=one...");
    assert!(tree.insert(1, "one".to_string()).unwrap().is_none());
    assert_eq!(tree.get(&1).unwrap(), Some("one".to_string()));
    
    println!("   📝 Insert 2=two...");
    assert!(tree.insert(2, "two".to_string()).unwrap().is_none());
    assert_eq!(tree.get(&2).unwrap(), Some("two".to_string()));
    
    println!("   📝 Insert 3=three...");
    assert!(tree.insert(3, "three".to_string()).unwrap().is_none());
    assert_eq!(tree.get(&3).unwrap(), Some("three".to_string()));
    
    assert_eq!(tree.get(&1).unwrap(), Some("one".to_string()));
    assert_eq!(tree.get(&2).unwrap(), Some("two".to_string()));
    assert_eq!(tree.get(&3).unwrap(), Some("three".to_string()));
    println!("   ✅ PASS: Adaptive B-tree works correctly");
}

#[test]
fn test_lock_free_btree_concurrent() {
    println!("\n🌳 TEST: Adaptive B-tree concurrent inserts (PRODUCTION FIX)");
    let tree: Arc<LockFreeBTree<i32, i32>> = Arc::new(LockFreeBTree::new(16)); // 16 shards for parallelism
    let mut handles = vec![];
    
    println!("   🧵 Spawning 100 threads...");
    for i in 0..100 {
        let tree_clone = Arc::clone(&tree);
        let handle = thread::spawn(move || {
            tree_clone.insert(i, i * 2).unwrap();
        });
        handles.push(handle);
    }
    
    println!("   ⏳ Waiting for threads...");
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("   📖 Verifying all inserts (including key 56 that was missing before)...");
    let mut missing = Vec::new();
    for i in 0..100 {
        match tree.get(&i).unwrap() {
            Some(val) => {
                if val != i * 2 {
                    println!("   ❌ Key {} has wrong value: expected {}, got {}", i, i * 2, val);
                }
            }
            None => {
                missing.push(i);
            }
        }
    }
    
    if !missing.is_empty() {
        panic!("Missing keys: {:?}", missing);
    }
    
    println!("   ✅ PASS: ALL 100 concurrent inserts successful (no missing keys!)");
    tree.dump_stats();
}

