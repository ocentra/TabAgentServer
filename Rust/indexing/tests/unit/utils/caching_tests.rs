//! 💾 CACHING TESTS - LRU, Multi-Level, Vector, Graph Caches

use indexing::{
    LruCache, MultiLevelCache, VectorSearchCache, GraphTraversalCache,
    WarmVectorCacheConfig, WarmGraphCacheConfig,
};
use std::sync::atomic::Ordering;

#[test]
fn test_lru_cache() {
    println!("\n💾 TEST: LRU cache eviction policy");
    let mut cache = LruCache::new(2);
    
    println!("   📝 Adding key1, key2 (capacity=2)...");
    cache.put("key1", "value1");
    cache.put("key2", "value2");
    
    assert_eq!(cache.get(&"key1"), Some("value1"));
    assert_eq!(cache.get(&"key2"), Some("value2"));
    
    println!("   📝 Adding key3 (should evict LRU)...");
    cache.put("key3", "value3");
    
    assert_eq!(cache.get(&"key1"), None);
    assert_eq!(cache.get(&"key2"), Some("value2"));
    assert_eq!(cache.get(&"key3"), Some("value3"));
    
    println!("   🗑️  Removing key2...");
    assert_eq!(cache.remove(&"key2"), Some("value2"));
    assert_eq!(cache.get(&"key2"), None);
    println!("   ✅ PASS: LRU eviction works correctly");
}

#[test]
fn test_multi_level_cache() {
    println!("\n💾 TEST: Multi-level cache (primary + secondary)");
    let cache = MultiLevelCache::new(2, 4);
    
    println!("   📝 Adding 3 values...");
    cache.put("key1", "value1");
    cache.put("key2", "value2");
    cache.put("key3", "value3");
    
    assert_eq!(cache.get(&"key1"), Some("value1"));
    assert_eq!(cache.get(&"key2"), Some("value2"));
    assert_eq!(cache.get(&"key3"), Some("value3"));
    
    println!("   📝 Adding key4, key5 (triggers eviction)...");
    cache.put("key4", "value4");
    cache.put("key5", "value5");
    
    assert_eq!(cache.get(&"key3"), Some("value3"));
    
    println!("   📊 Checking stats...");
    let stats = cache.get_stats();
    assert!(stats.total_accesses.load(Ordering::Relaxed) > 0);
    println!("   ✅ PASS: Multi-level cache works ({} accesses)", stats.total_accesses.load(Ordering::Relaxed));
}

#[test]
fn test_vector_search_cache() {
    println!("\n🔍 TEST: Vector search result caching");
    let cache = VectorSearchCache::new(100, 100);
    
    println!("   📝 Caching search results...");
    let results = vec![("vec1".to_string(), 0.9), ("vec2".to_string(), 0.8)];
    cache.put_search_results("query1".to_string(), results.clone());
    
    assert_eq!(cache.get_search_results("query1"), Some(results));
    
    println!("   📝 Caching metadata...");
    let metadata = (1234567890, 384);
    cache.put_metadata("vec1".to_string(), metadata);
    
    assert_eq!(cache.get_metadata("vec1"), Some(metadata));
    println!("   ✅ PASS: Vector search cache stores results + metadata");
}

#[test]
fn test_graph_traversal_cache() {
    println!("\n🕸️  TEST: Graph traversal result caching");
    let cache = GraphTraversalCache::new(100, 100, 100);
    
    println!("   📝 Caching BFS results...");
    let bfs_results = vec!["node1".to_string(), "node2".to_string(), "node3".to_string()];
    cache.put_bfs_results("bfs_start".to_string(), bfs_results.clone());
    
    assert_eq!(cache.get_bfs_results("bfs_start"), Some(bfs_results));
    
    println!("   📝 Caching shortest path results...");
    let path_results = (vec!["node1".to_string(), "node2".to_string()], 2.5);
    cache.put_shortest_path_results("path_start_end".to_string(), path_results.clone());
    
    assert_eq!(cache.get_shortest_path_results("path_start_end"), Some(path_results));
    println!("   ✅ PASS: Graph traversal cache stores BFS + paths");
}

#[test]
fn test_cache_stats() {
    println!("\n📊 TEST: Cache statistics tracking");
    let cache = MultiLevelCache::<String, String>::new(2, 4);
    
    println!("   📝 Performing cache operations...");
    cache.put("key1".to_string(), "value1".to_string());
    cache.put("key2".to_string(), "value2".to_string());
    
    cache.get(&"key1".to_string());
    cache.get(&"key2".to_string());
    cache.get(&"key3".to_string());
    
    println!("   📊 Checking statistics...");
    let stats = cache.get_stats();
    assert_eq!(stats.primary_hits.load(Ordering::Relaxed), 2);
    assert_eq!(stats.misses.load(Ordering::Relaxed), 1);
    assert_eq!(stats.total_accesses.load(Ordering::Relaxed), 3);
    assert!(stats.hit_ratio() > 0.6);
    println!("   ✅ PASS: Stats tracked (hits={}, misses={}, ratio={:.0}%)", 
        stats.primary_hits.load(Ordering::Relaxed),
        stats.misses.load(Ordering::Relaxed),
        stats.hit_ratio() * 100.0);
}

// Warm cache config validation tests - actual behavior tests are in integration/caching_tests.rs

#[test]
fn test_warm_graph_cache_config() {
    println!("\n⚙️  TEST: Warm graph cache configuration defaults");
    let config = WarmGraphCacheConfig::default();
    
    assert_eq!(config.max_outgoing_edges, 5000);
    assert_eq!(config.max_incoming_edges, 5000);
    assert_eq!(config.max_nodes, 10000);
    assert_eq!(config.edge_ttl_seconds, 1800);
    assert_eq!(config.node_ttl_seconds, 3600);
    println!("   ✅ PASS: Default config (edges={}, nodes={}, TTL={}s)", 
        config.max_outgoing_edges, config.max_nodes, config.edge_ttl_seconds);
}

#[test]
fn test_warm_vector_cache_config() {
    println!("\n⚙️  TEST: Warm vector cache configuration defaults");
    let config = WarmVectorCacheConfig::default();
    
    assert_eq!(config.max_vectors, 10000);
    assert_eq!(config.max_search_results, 1000);
    assert!(!config.use_product_quantization);
    assert_eq!(config.product_quantization_subvector_size, 8);
    assert_eq!(config.vector_ttl_seconds, 3600);
    assert_eq!(config.search_ttl_seconds, 300);
    println!("   ✅ PASS: Default config (vectors={}, searches={}, TTL={}s)", 
        config.max_vectors, config.max_search_results, config.vector_ttl_seconds);
}

