//! 🔓 LOCK-FREE COMMON TYPES TESTS - Access Tracking & Statistics

use indexing::lock_free::lock_free_common::{LockFreeAccessTracker, LockFreeStats};

#[test]
fn test_lock_free_access_tracker() {
    println!("\n📊 TEST: Lock-free access tracker");
    let tracker = LockFreeAccessTracker::new();
    
    assert_eq!(tracker.access_count(), 0);
    
    println!("   📝 Recording access...");
    tracker.record_access();
    
    assert_eq!(tracker.access_count(), 1);
    println!("   ✅ PASS: Access tracked (count={})", tracker.access_count());
}

#[test]
fn test_lock_free_stats() {
    println!("\n📈 TEST: Lock-free statistics tracking");
    let stats = LockFreeStats::new();
    
    assert_eq!(stats.query_count(), 0);
    assert_eq!(stats.vector_count(), 0);
    
    println!("   📝 Recording stats...");
    stats.increment_query_count();
    stats.increment_vector_count();
    stats.increment_promotions();
    
    assert_eq!(stats.query_count(), 1);
    assert_eq!(stats.vector_count(), 1);
    assert_eq!(stats.promotions(), 1);
    println!("   ✅ PASS: Stats tracked (queries={}, vectors={})", 1, 1);
}

