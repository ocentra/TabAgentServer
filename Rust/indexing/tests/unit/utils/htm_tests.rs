//! ⚡ HTM (Hardware Transactional Memory) TESTS

use indexing::utils::htm::{HtmCounter, HtmHashMap};
use std::sync::Arc;
use std::thread;

#[test]
fn test_htm_counter_basic() {
    println!("\n⚡ TEST: HTM counter basic operations");
    let counter = HtmCounter::new(0);
    
    println!("   🔍 Initial value: {}", counter.get());
    assert_eq!(counter.get(), 0);
    
    counter.increment().unwrap();
    assert_eq!(counter.get(), 1);
    
    counter.increment().unwrap();
    assert_eq!(counter.get(), 2);
    println!("   ✅ PASS: HTM counter increments (0 -> 2)");
}

#[test]
fn test_htm_counter_concurrent() {
    println!("\n⚡ TEST: HTM counter concurrent increments (10 threads)");
    let counter = Arc::new(HtmCounter::new(0));
    let mut handles = vec![];
    
    println!("   🚀 Spawning 10 threads...");
    for _ in 0..10 {
        let counter_clone = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            counter_clone.increment().unwrap();
        });
        handles.push(handle);
    }
    
    println!("   ⏳ Waiting for threads to complete...");
    for handle in handles {
        handle.join().unwrap();
    }
    
    assert_eq!(counter.get(), 10);
    println!("   ✅ PASS: HTM counter correctly incremented by 10 threads (result=10)");
}

#[test]
fn test_htm_hash_map() {
    println!("\n⚡ TEST: HTM HashMap (not yet implemented)");
    let map: HtmHashMap<String, i32> = HtmHashMap::new();
    
    println!("   🔍 Testing operations return errors (not implemented)...");
    assert!(map.insert("key".to_string(), 42).is_err());
    assert!(map.get(&"key".to_string()).is_err());
    assert!(map.remove(&"key".to_string()).is_err());
    println!("   ✅ PASS: HTM HashMap correctly returns errors (unimplemented)");
}

