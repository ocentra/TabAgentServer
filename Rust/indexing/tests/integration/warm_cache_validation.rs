//! VALIDATION TESTS: Prove warm cache trade-offs are worth it
//!
//! # What These Tests Prove
//!
//! These are NOT just "does it run" tests. Each test VALIDATES a specific claim:
//!
//! 1. **Test: `test_warm_cache_saves_cold_transactions`**
//!    - **Claim:** "1 allocation saves 100s of MDBX transactions"
//!    - **Proof:** Count actual cold_hits vs warm_hits
//!    - **Success:** 1 cold hit, 99 warm hits = 99 transactions saved
//!
//! 2. **Test: `test_ttl_expiration_refetches`**
//!    - **Claim:** "TTL prevents stale data"
//!    - **Proof:** Expired entries trigger refetch from COLD
//!    - **Success:** ttl_expirations > 0, data is fresh
//!
//! 3. **Test: `test_hot_node_cache_hit_ratio`**
//!    - **Claim:** "Cache hit ratio >80% for hot nodes"
//!    - **Proof:** Simulate realistic access pattern, measure hit_ratio()
//!    - **Success:** hit_ratio > 0.80

use indexing::{IndexManager, WarmGraphCache, WarmGraphCacheConfig};
use common::models::Edge;
use common::{EdgeId, NodeId};
use tempfile::TempDir;
use std::sync::Arc;
use std::sync::atomic::Ordering;

#[test]
fn test_warm_cache_saves_cold_transactions() {
    // CLAIM: "1 allocation saves 100s of MDBX transactions"
    // PROOF: Count cold_hits vs warm_hits with metrics
    
    let temp_dir = TempDir::new().unwrap();
    #[allow(deprecated)]
    let manager = IndexManager::new(temp_dir.path()).unwrap();
    
    // Add test edges to COLD tier
    let edge1 = Edge {
        id: EdgeId::from("e1"),
        from_node: NodeId::from("node1"),
        to_node: NodeId::from("node2"),
        edge_type: "TEST".to_string(),
        created_at: 1697500000000,
        metadata: "{}".to_string(),
    };
    
    let edge2 = Edge {
        id: EdgeId::from("e2"),
        from_node: NodeId::from("node1"),
        to_node: NodeId::from("node3"),
        edge_type: "TEST".to_string(),
        created_at: 1697500000000,
        metadata: "{}".to_string(),
    };
    
    manager.index_edge(&edge1).unwrap();
    manager.index_edge(&edge2).unwrap();
    
    // Create warm cache wrapping the COLD graph index
    let graph_arc = Arc::clone(manager.graph());
    let config = WarmGraphCacheConfig::default();
    let cache = WarmGraphCache::new(graph_arc, config);
    
    // Access the same node 100 times
    for _ in 0..100 {
        let _ = cache.get_outgoing("node1").unwrap();
    }
    
    // Get metrics
    let metrics = cache.get_metrics();
    let cold_hits = metrics.cold_hits.load(Ordering::Relaxed);
    let warm_hits = metrics.warm_hits.load(Ordering::Relaxed);
    
    // PROOF OF CLAIM:
    assert_eq!(cold_hits, 1, "Should have exactly 1 COLD hit (first access)");
    assert_eq!(warm_hits, 99, "Should have 99 WARM hits (cached accesses)");
    
    // VALIDATE: 1 allocation saved 99 MDBX transactions
    let savings = metrics.cold_savings_count();
    assert_eq!(savings, 99, "Cache saved 99 MDBX transactions!");
    
    println!("✅ PROOF: 1 allocation saved {} MDBX transactions", savings);
    println!("   Cold hits (MDBX): {}", cold_hits);
    println!("   Warm hits (cache): {}", warm_hits);
}

#[test]
fn test_ttl_expiration_refetches() {
    // CLAIM: "TTL prevents stale data - expired entries refetch from COLD"
    // PROOF: Configure short TTL, wait, verify refetch happens
    
    let temp_dir = TempDir::new().unwrap();
    #[allow(deprecated)]
    let manager = IndexManager::new(temp_dir.path()).unwrap();
    
    // Add test edge
    let edge = Edge {
        id: EdgeId::from("e1"),
        from_node: NodeId::from("node1"),
        to_node: NodeId::from("node2"),
        edge_type: "TEST".to_string(),
        created_at: 1697500000000,
        metadata: "{}".to_string(),
    };
    manager.index_edge(&edge).unwrap();
    
    // Create cache with SHORT TTL for testing
    let graph_arc = Arc::clone(manager.graph());
    let config = WarmGraphCacheConfig {
        max_outgoing_edges: 1000,
        max_incoming_edges: 1000,
        max_nodes: 1000,
        edge_ttl_seconds: 1, // SHORT TTL (1 second)
        node_ttl_seconds: 1,
    };
    let cache = WarmGraphCache::new(graph_arc, config);
    
    // First access: Cache miss
    let _ = cache.get_outgoing("node1").unwrap();
    
    // Second access: Cache hit (fresh)
    let _ = cache.get_outgoing("node1").unwrap();
    
    let metrics = cache.get_metrics();
    assert_eq!(metrics.cold_hits.load(Ordering::Relaxed), 1);
    assert_eq!(metrics.warm_hits.load(Ordering::Relaxed), 1);
    
    // Wait for TTL expiration
    std::thread::sleep(std::time::Duration::from_secs(2));
    
    // Third access: Cache hit but STALE → refetch from COLD
    let _ = cache.get_outgoing("node1").unwrap();
    
    // PROOF OF CLAIM:
    let ttl_expirations = metrics.ttl_expirations.load(Ordering::Relaxed);
    let cold_hits_after = metrics.cold_hits.load(Ordering::Relaxed);
    
    assert_eq!(ttl_expirations, 1, "Should have 1 TTL expiration");
    assert_eq!(cold_hits_after, 2, "Should refetch from COLD after expiration");
    
    println!("✅ PROOF: TTL expiration triggered refetch from COLD");
    println!("   TTL expirations: {}", ttl_expirations);
    println!("   Total COLD hits: {}", cold_hits_after);
}

#[test]
fn test_hot_node_cache_hit_ratio() {
    // CLAIM: "Cache hit ratio >80% for hot nodes"
    // PROOF: Simulate realistic access pattern (hot + cold nodes), measure hit_ratio
    
    let temp_dir = TempDir::new().unwrap();
    #[allow(deprecated)]
    let manager = IndexManager::new(temp_dir.path()).unwrap();
    
    // Add edges for one hot node
    let edge1 = Edge {
        id: EdgeId::from("e1"),
        from_node: NodeId::from("node1"),
        to_node: NodeId::from("node2"),
        edge_type: "TEST".to_string(),
        created_at: 1697500000000,
        metadata: "{}".to_string(),
    };
    manager.index_edge(&edge1).unwrap();
    
    // Create warm cache
    let graph_arc = Arc::clone(manager.graph());
    let config = WarmGraphCacheConfig::default();
    let cache = WarmGraphCache::new(graph_arc, config);
    
    // Simulate realistic access pattern:
    // - 1 hot node: accessed 90 times
    // - 10 cold nodes: accessed 1 time each (will miss)
    
    for _ in 0..90 {
        let _ = cache.get_outgoing("node1").unwrap(); // Hot node
    }
    
    for i in 0..10 {
        let _ = cache.get_outgoing(&format!("cold_node_{}", i)).unwrap(); // Cold nodes
    }
    
    // Get metrics
    let metrics = cache.get_metrics();
    let hit_ratio = metrics.hit_ratio();
    let total = metrics.total_accesses();
    
    // PROOF OF CLAIM:
    assert_eq!(total, 100, "Total: 90 hot + 10 cold = 100 accesses");
    assert!(hit_ratio > 0.80, "Hit ratio should be >80% for hot-heavy workload, got {:.2}%", hit_ratio * 100.0);
    
    println!("✅ PROOF: Cache hit ratio exceeds 80% target");
    println!("   Hit ratio: {:.2}%", hit_ratio * 100.0);
    println!("   Warm hits: {}", metrics.warm_hits.load(Ordering::Relaxed));
    println!("   Cold hits: {}", metrics.cold_hits.load(Ordering::Relaxed));
}
