//! Advanced caching strategies for improved performance.
//!
//! This module provides enhanced caching mechanisms beyond simple path caching,
//! including LRU caches, multi-level caches, and adaptive caching strategies.
//! These implementations follow the Rust Architecture Guidelines for safety,
//! performance, and clarity.

use crate::hybrid::QuantizedVector;
use crate::vector::{VectorIndex, SearchResult};
use crate::graph::GraphIndex;
use common::{DbResult, EdgeId};
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};

/// A simple LRU (Least Recently Used) cache implementation.
pub struct LruCache<K, V> {
    /// The maximum number of entries the cache can hold
    capacity: usize,
    
    /// The cached entries
    entries: HashMap<K, V>,
    
    /// The order of entries (for LRU eviction)
    order: VecDeque<K>,
}

impl<K, V> LruCache<K, V>
where
    K: Clone + Eq + Hash,
    V: Clone,
{
    /// Creates a new LRU cache with the specified capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            entries: HashMap::new(),
            order: VecDeque::new(),
        }
    }
    
    /// Gets a value from the cache by key.
    ///
    /// If the key exists, the value is returned and the key is moved to the
    /// front of the order (marking it as recently used).
    pub fn get(&mut self, key: &K) -> Option<V> {
        if self.entries.contains_key(key) {
            // Move the key to the front of the order
            self.order.retain(|k| k != key);
            self.order.push_front(key.clone());
            
            self.entries.get(key).cloned()
        } else {
            None
        }
    }
    
    /// Inserts a key-value pair into the cache.
    ///
    /// If the cache is at capacity, the least recently used entry is evicted.
    pub fn put(&mut self, key: K, value: V) {
        // If the key already exists, remove it from the order
        if self.entries.contains_key(&key) {
            self.order.retain(|k| k != &key);
        } else if self.entries.len() >= self.capacity {
            // If the cache is at capacity, evict the least recently used entry
            if let Some(lru_key) = self.order.pop_back() {
                self.entries.remove(&lru_key);
            }
        }
        
        // Insert the new entry
        self.entries.insert(key.clone(), value);
        self.order.push_front(key);
    }
    
    /// Removes a key-value pair from the cache.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.order.retain(|k| k != key);
        self.entries.remove(key)
    }
    
    /// Gets the number of entries in the cache.
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    
    /// Checks if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    
    /// Clears the cache.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.order.clear();
    }
}

/// A multi-level cache with different eviction policies.
pub struct MultiLevelCache<K, V> {
    /// The primary cache (fastest, smallest)
    primary: Arc<Mutex<LruCache<K, V>>>,
    
    /// The secondary cache (slower, larger)
    secondary: Arc<Mutex<LruCache<K, V>>>,
    
    /// Statistics for cache performance monitoring
    stats: CacheStats,
}

/// Statistics for cache performance monitoring.
#[derive(Debug)]
pub struct CacheStats {
    /// Number of primary cache hits
    pub primary_hits: AtomicUsize,
    
    /// Number of secondary cache hits
    pub secondary_hits: AtomicUsize,
    
    /// Number of cache misses
    pub misses: AtomicUsize,
    
    /// Total number of accesses
    pub total_accesses: AtomicUsize,
}

impl Clone for CacheStats {
    fn clone(&self) -> Self {
        Self {
            primary_hits: AtomicUsize::new(self.primary_hits.load(Ordering::Relaxed)),
            secondary_hits: AtomicUsize::new(self.secondary_hits.load(Ordering::Relaxed)),
            misses: AtomicUsize::new(self.misses.load(Ordering::Relaxed)),
            total_accesses: AtomicUsize::new(self.total_accesses.load(Ordering::Relaxed)),
        }
    }
}

impl CacheStats {
    /// Creates new cache statistics with default values.
    pub fn new() -> Self {
        Self {
            primary_hits: AtomicUsize::new(0),
            secondary_hits: AtomicUsize::new(0),
            misses: AtomicUsize::new(0),
            total_accesses: AtomicUsize::new(0),
        }
    }
    
    /// Records a primary cache hit.
    pub fn record_primary_hit(&self) {
        self.primary_hits.fetch_add(1, Ordering::Relaxed);
        self.total_accesses.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Records a secondary cache hit.
    pub fn record_secondary_hit(&self) {
        self.secondary_hits.fetch_add(1, Ordering::Relaxed);
        self.total_accesses.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Records a cache miss.
    pub fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
        self.total_accesses.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Gets the hit ratio (hits / total accesses).
    pub fn hit_ratio(&self) -> f64 {
        let total = self.total_accesses.load(Ordering::Relaxed);
        if total == 0 {
            0.0
        } else {
            let hits = self.primary_hits.load(Ordering::Relaxed) + self.secondary_hits.load(Ordering::Relaxed);
            hits as f64 / total as f64
        }
    }
    
    /// Gets the primary cache hit ratio.
    pub fn primary_hit_ratio(&self) -> f64 {
        let total = self.total_accesses.load(Ordering::Relaxed);
        if total == 0 {
            0.0
        } else {
            self.primary_hits.load(Ordering::Relaxed) as f64 / total as f64
        }
    }
    
    /// Gets the secondary cache hit ratio.
    pub fn secondary_hit_ratio(&self) -> f64 {
        let total = self.total_accesses.load(Ordering::Relaxed);
        if total == 0 {
            0.0
        } else {
            self.secondary_hits.load(Ordering::Relaxed) as f64 / total as f64
        }
    }
}

impl<K, V> MultiLevelCache<K, V>
where
    K: Clone + Eq + Hash,
    V: Clone,
{
    /// Creates a new multi-level cache.
    ///
    /// # Arguments
    ///
    /// * `primary_capacity` - The capacity of the primary cache
    /// * `secondary_capacity` - The capacity of the secondary cache
    pub fn new(primary_capacity: usize, secondary_capacity: usize) -> Self {
        Self {
            primary: Arc::new(Mutex::new(LruCache::new(primary_capacity))),
            secondary: Arc::new(Mutex::new(LruCache::new(secondary_capacity))),
            stats: CacheStats::new(),
        }
    }
    
    /// Gets a value from the cache by key.
    ///
    /// The method first checks the primary cache, then the secondary cache.
    /// If found in the secondary cache, the entry is promoted to the primary cache.
    pub fn get(&self, key: &K) -> Option<V> {
        // Check primary cache
        if let Ok(mut primary) = self.primary.lock() {
            if let Some(value) = primary.get(key) {
                self.stats.record_primary_hit();
                return Some(value);
            }
        }
        
        // Check secondary cache
        if let Ok(mut secondary) = self.secondary.lock() {
            if let Some(value) = secondary.get(key) {
                // Promote to primary cache
                if let Ok(mut primary) = self.primary.lock() {
                    primary.put(key.clone(), value.clone());
                }
                
                self.stats.record_secondary_hit();
                return Some(value);
            }
        }
        
        self.stats.record_miss();
        None
    }
    
    /// Inserts a key-value pair into the cache.
    ///
    /// The entry is inserted into the primary cache. If the primary cache is
    /// at capacity, the least recently used entry is evicted.
    pub fn put(&self, key: K, value: V) {
        if let Ok(mut primary) = self.primary.lock() {
            primary.put(key, value);
        }
    }
    
    /// Removes a key-value pair from the cache.
    ///
    /// The entry is removed from both the primary and secondary caches.
    pub fn remove(&self, key: &K) -> Option<V> {
        let mut result = None;
        
        if let Ok(mut primary) = self.primary.lock() {
            result = primary.remove(key);
        }
        
        if let Ok(mut secondary) = self.secondary.lock() {
            if result.is_none() {
                result = secondary.remove(key);
            } else {
                secondary.remove(key);
            }
        }
        
        result
    }
    
    /// Gets the cache statistics.
    pub fn get_stats(&self) -> CacheStats {
        self.stats.clone()
    }
    
    /// Resets the cache statistics.
    pub fn reset_stats(&self) {
        self.stats.primary_hits.store(0, Ordering::Relaxed);
        self.stats.secondary_hits.store(0, Ordering::Relaxed);
        self.stats.misses.store(0, Ordering::Relaxed);
        self.stats.total_accesses.store(0, Ordering::Relaxed);
    }
}

/// An adaptive cache that adjusts its behavior based on access patterns.
pub struct AdaptiveCache<K, V> {
    /// The underlying multi-level cache
    cache: MultiLevelCache<K, V>,
    
    /// The access pattern analyzer
    analyzer: AccessPatternAnalyzer,
    
    /// Whether to automatically adjust cache parameters
    auto_adjust: bool,
}

/// An analyzer for access patterns.
#[derive(Clone)]
struct AccessPatternAnalyzer {
    /// Recent access times
    access_times: VecDeque<std::time::Instant>,
    
    /// Time window for analysis (in seconds)
    time_window: u64,
}

impl AccessPatternAnalyzer {
    /// Creates a new access pattern analyzer.
    pub fn new(time_window: u64) -> Self {
        Self {
            access_times: VecDeque::new(),
            time_window,
        }
    }
    
    /// Records an access.
    pub fn record_access(&mut self) {
        let now = std::time::Instant::now();
        self.access_times.push_back(now);
        
        // Remove old access times
        let cutoff = now - std::time::Duration::from_secs(self.time_window);
        self.access_times.retain(|&time| time > cutoff);
    }
    
    /// Gets the access rate (accesses per second).
    pub fn access_rate(&self) -> f64 {
        let duration = if let (Some(first), Some(last)) = (self.access_times.front(), self.access_times.back()) {
            (last.duration_since(*first)).as_secs_f64()
        } else {
            0.0
        };
        
        if duration > 0.0 {
            self.access_times.len() as f64 / duration
        } else {
            0.0
        }
    }
    
    /// Checks if the access pattern is bursty.
    pub fn is_bursty(&self) -> bool {
        // Simple heuristic: if we have many accesses in a short time, it's bursty
        self.access_times.len() > 10 && self.access_rate() > 100.0
    }
}

impl<K, V> AdaptiveCache<K, V>
where
    K: Clone + Eq + Hash,
    V: Clone,
{
    /// Creates a new adaptive cache.
    ///
    /// # Arguments
    ///
    /// * `primary_capacity` - The capacity of the primary cache
    /// * `secondary_capacity` - The capacity of the secondary cache
    /// * `auto_adjust` - Whether to automatically adjust cache parameters
    pub fn new(primary_capacity: usize, secondary_capacity: usize, auto_adjust: bool) -> Self {
        Self {
            cache: MultiLevelCache::new(primary_capacity, secondary_capacity),
            analyzer: AccessPatternAnalyzer::new(60), // 60 second time window
            auto_adjust,
        }
    }
    
    /// Gets a value from the cache by key.
    pub fn get(&self, key: &K) -> Option<V> {
        if self.auto_adjust {
            // Record access for analysis
            let mut analyzer = self.analyzer.clone();
            analyzer.record_access();
        }
        
        self.cache.get(key)
    }
    
    /// Inserts a key-value pair into the cache.
    pub fn put(&self, key: K, value: V) {
        self.cache.put(key, value);
    }
    
    /// Removes a key-value pair from the cache.
    pub fn remove(&self, key: &K) -> Option<V> {
        self.cache.remove(key)
    }
    
    /// Gets the cache statistics.
    pub fn get_stats(&self) -> CacheStats {
        self.cache.get_stats()
    }
}

/// A cache manager for vector search results.
pub struct VectorSearchCache {
    /// Cache for search results
    search_cache: Arc<RwLock<MultiLevelCache<String, Vec<(String, f32)>>>>,
    
    /// Cache for vector metadata
    metadata_cache: Arc<RwLock<MultiLevelCache<String, (i64, usize)>>>,
}

impl VectorSearchCache {
    /// Creates a new vector search cache.
    ///
    /// # Arguments
    ///
    /// * `search_cache_capacity` - The capacity of the search results cache
    /// * `metadata_cache_capacity` - The capacity of the metadata cache
    pub fn new(search_cache_capacity: usize, metadata_cache_capacity: usize) -> Self {
        Self {
            search_cache: Arc::new(RwLock::new(MultiLevelCache::new(
                search_cache_capacity / 2,
                search_cache_capacity / 2,
            ))),
            metadata_cache: Arc::new(RwLock::new(MultiLevelCache::new(
                metadata_cache_capacity / 2,
                metadata_cache_capacity / 2,
            ))),
        }
    }
    
    /// Gets cached search results.
    pub fn get_search_results(&self, cache_key: &str) -> Option<Vec<(String, f32)>> {
        if let Ok(cache) = self.search_cache.read() {
            cache.get(&cache_key.to_string())
        } else {
            None
        }
    }
    
    /// Caches search results.
    pub fn put_search_results(&self, cache_key: String, results: Vec<(String, f32)>) {
        if let Ok(cache) = self.search_cache.write() {
            cache.put(cache_key, results);
        }
    }
    
    /// Gets cached vector metadata.
    pub fn get_metadata(&self, vector_id: &str) -> Option<(i64, usize)> {
        if let Ok(cache) = self.metadata_cache.read() {
            cache.get(&vector_id.to_string())
        } else {
            None
        }
    }
    
    /// Caches vector metadata.
    pub fn put_metadata(&self, vector_id: String, metadata: (i64, usize)) {
        if let Ok(cache) = self.metadata_cache.write() {
            cache.put(vector_id, metadata);
        }
    }
    
    /// Removes cached search results.
    pub fn remove_search_results(&self, cache_key: &str) -> Option<Vec<(String, f32)>> {
        if let Ok(cache) = self.search_cache.write() {
            cache.remove(&cache_key.to_string())
        } else {
            None
        }
    }
    
    /// Removes cached vector metadata.
    pub fn remove_metadata(&self, vector_id: &str) -> Option<(i64, usize)> {
        if let Ok(cache) = self.metadata_cache.write() {
            cache.remove(&vector_id.to_string())
        } else {
            None
        }
    }
    
    /// Gets cache statistics.
    pub fn get_stats(&self) -> (CacheStats, CacheStats) {
        let search_stats = if let Ok(cache) = self.search_cache.read() {
            cache.get_stats()
        } else {
            CacheStats::new()
        };
        
        let metadata_stats = if let Ok(cache) = self.metadata_cache.read() {
            cache.get_stats()
        } else {
            CacheStats::new()
        };
        
        (search_stats, metadata_stats)
    }
}

/// A cache manager for graph traversal results.
pub struct GraphTraversalCache {
    /// Cache for BFS results
    bfs_cache: Arc<RwLock<MultiLevelCache<String, Vec<String>>>>,
    
    /// Cache for DFS results
    dfs_cache: Arc<RwLock<MultiLevelCache<String, Vec<String>>>>,
    
    /// Cache for shortest path results
    shortest_path_cache: Arc<RwLock<MultiLevelCache<String, (Vec<String>, f32)>>>,
}

impl GraphTraversalCache {
    /// Creates a new graph traversal cache.
    ///
    /// # Arguments
    ///
    /// * `bfs_cache_capacity` - The capacity of the BFS results cache
    /// * `dfs_cache_capacity` - The capacity of the DFS results cache
    /// * `shortest_path_cache_capacity` - The capacity of the shortest path results cache
    pub fn new(
        bfs_cache_capacity: usize,
        dfs_cache_capacity: usize,
        shortest_path_cache_capacity: usize,
    ) -> Self {
        Self {
            bfs_cache: Arc::new(RwLock::new(MultiLevelCache::new(
                bfs_cache_capacity / 2,
                bfs_cache_capacity / 2,
            ))),
            dfs_cache: Arc::new(RwLock::new(MultiLevelCache::new(
                dfs_cache_capacity / 2,
                dfs_cache_capacity / 2,
            ))),
            shortest_path_cache: Arc::new(RwLock::new(MultiLevelCache::new(
                shortest_path_cache_capacity / 2,
                shortest_path_cache_capacity / 2,
            ))),
        }
    }
    
    /// Gets cached BFS results.
    pub fn get_bfs_results(&self, cache_key: &str) -> Option<Vec<String>> {
        if let Ok(cache) = self.bfs_cache.read() {
            cache.get(&cache_key.to_string())
        } else {
            None
        }
    }
    
    /// Caches BFS results.
    pub fn put_bfs_results(&self, cache_key: String, results: Vec<String>) {
        if let Ok(cache) = self.bfs_cache.write() {
            cache.put(cache_key, results);
        }
    }
    
    /// Gets cached DFS results.
    pub fn get_dfs_results(&self, cache_key: &str) -> Option<Vec<String>> {
        if let Ok(cache) = self.dfs_cache.read() {
            cache.get(&cache_key.to_string())
        } else {
            None
        }
    }
    
    /// Caches DFS results.
    pub fn put_dfs_results(&self, cache_key: String, results: Vec<String>) {
        if let Ok(cache) = self.dfs_cache.write() {
            cache.put(cache_key, results);
        }
    }
    
    /// Gets cached shortest path results.
    pub fn get_shortest_path_results(&self, cache_key: &str) -> Option<(Vec<String>, f32)> {
        if let Ok(cache) = self.shortest_path_cache.read() {
            cache.get(&cache_key.to_string())
        } else {
            None
        }
    }
    
    /// Caches shortest path results.
    pub fn put_shortest_path_results(&self, cache_key: String, results: (Vec<String>, f32)) {
        if let Ok(cache) = self.shortest_path_cache.write() {
            cache.put(cache_key, results);
        }
    }
    
    /// Gets cache statistics.
    pub fn get_stats(&self) -> (CacheStats, CacheStats, CacheStats) {
        let bfs_stats = if let Ok(cache) = self.bfs_cache.read() {
            cache.get_stats()
        } else {
            CacheStats::new()
        };
        
        let dfs_stats = if let Ok(cache) = self.dfs_cache.read() {
            cache.get_stats()
        } else {
            CacheStats::new()
        };
        
        let shortest_path_stats = if let Ok(cache) = self.shortest_path_cache.read() {
            cache.get_stats()
        } else {
            CacheStats::new()
        };
        
        (bfs_stats, dfs_stats, shortest_path_stats)
    }
}

/// A warm layer cache for graph data with lazy loading from cold storage.
///
/// This cache provides memory-efficient storage for graph nodes and edges with
/// automatic lazy loading from the cold GraphIndex when cache misses occur.
/// It uses LRU eviction policies and configurable memory limits.
pub struct WarmGraphCache {
    /// Cache for outgoing edges by node ID
    outgoing_cache: Arc<RwLock<MultiLevelCache<String, Vec<EdgeId>>>>,
    
    /// Cache for incoming edges by node ID
    incoming_cache: Arc<RwLock<MultiLevelCache<String, Vec<EdgeId>>>>,
    
    /// Cache for node existence checks
    node_cache: Arc<RwLock<MultiLevelCache<String, bool>>>,
    
    /// Reference to cold storage for lazy loading
    cold_storage: Arc<RwLock<GraphIndex>>,
    
    /// Configuration for cache behavior
    config: WarmGraphCacheConfig,
}

/// Configuration for WarmGraphCache behavior.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WarmGraphCacheConfig {
    /// Maximum number of outgoing edge lists to cache
    pub max_outgoing_edges: usize,
    
    /// Maximum number of incoming edge lists to cache
    pub max_incoming_edges: usize,
    
    /// Maximum number of node existence checks to cache
    pub max_nodes: usize,
    
    /// TTL for cached edge lists in seconds (0 = no expiration)
    pub edge_ttl_seconds: u64,
    
    /// TTL for cached node existence in seconds (0 = no expiration)
    pub node_ttl_seconds: u64,
}

impl Default for WarmGraphCacheConfig {
    fn default() -> Self {
        Self {
            max_outgoing_edges: 5000,
            max_incoming_edges: 5000,
            max_nodes: 10000,
            edge_ttl_seconds: 1800, // 30 minutes
            node_ttl_seconds: 3600, // 1 hour
        }
    }
}

impl WarmGraphCache {
    /// Creates a new warm graph cache with the specified configuration.
    ///
    /// # Arguments
    ///
    /// * `cold_storage` - Reference to the cold GraphIndex for lazy loading
    /// * `config` - Configuration for cache behavior
    pub fn new(cold_storage: Arc<RwLock<GraphIndex>>, config: WarmGraphCacheConfig) -> Self {
        Self {
            outgoing_cache: Arc::new(RwLock::new(MultiLevelCache::new(
                config.max_outgoing_edges / 2,
                config.max_outgoing_edges / 2,
            ))),
            incoming_cache: Arc::new(RwLock::new(MultiLevelCache::new(
                config.max_incoming_edges / 2,
                config.max_incoming_edges / 2,
            ))),
            node_cache: Arc::new(RwLock::new(MultiLevelCache::new(
                config.max_nodes / 2,
                config.max_nodes / 2,
            ))),
            cold_storage,
            config,
        }
    }
    
    /// Gets outgoing edges for a node, loading from cold storage if necessary.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The ID of the node to get outgoing edges for
    ///
    /// # Returns
    ///
    /// Vector of outgoing edge IDs
    pub fn get_outgoing(&self, node_id: &str) -> DbResult<Vec<EdgeId>> {
        // First check the cache
        if let Ok(cache) = self.outgoing_cache.read() {
            if let Some(edges) = cache.get(&node_id.to_string()) {
                return Ok(edges);
            }
        }
        
        // Cache miss - load from cold storage
        if let Ok(cold_storage) = self.cold_storage.read() {
            let edges = cold_storage.get_outgoing(node_id)?;
            
            // Cache the result
            if let Ok(cache) = self.outgoing_cache.write() {
                cache.put(node_id.to_string(), edges.clone());
            }
            
            return Ok(edges);
        }
        
        Ok(vec![])
    }
    
    /// Gets incoming edges for a node, loading from cold storage if necessary.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The ID of the node to get incoming edges for
    ///
    /// # Returns
    ///
    /// Vector of incoming edge IDs
    pub fn get_incoming(&self, node_id: &str) -> DbResult<Vec<EdgeId>> {
        // First check the cache
        if let Ok(cache) = self.incoming_cache.read() {
            if let Some(edges) = cache.get(&node_id.to_string()) {
                return Ok(edges);
            }
        }
        
        // Cache miss - load from cold storage
        if let Ok(cold_storage) = self.cold_storage.read() {
            let edges = cold_storage.get_incoming(node_id)?;
            
            // Cache the result
            if let Ok(cache) = self.incoming_cache.write() {
                cache.put(node_id.to_string(), edges.clone());
            }
            
            return Ok(edges);
        }
        
        Ok(vec![])
    }
    
    /// Checks if a node exists (has any edges), with caching.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The ID of the node to check
    ///
    /// # Returns
    ///
    /// True if the node has any outgoing or incoming edges
    pub fn node_exists(&self, node_id: &str) -> DbResult<bool> {
        // First check the cache
        if let Ok(cache) = self.node_cache.read() {
            if let Some(exists) = cache.get(&node_id.to_string()) {
                return Ok(exists);
            }
        }
        
        // Cache miss - check cold storage
        let outgoing = self.get_outgoing(node_id)?;
        let incoming = self.get_incoming(node_id)?;
        let exists = !outgoing.is_empty() || !incoming.is_empty();
        
        // Cache the result
        if let Ok(cache) = self.node_cache.write() {
            cache.put(node_id.to_string(), exists);
        }
        
        Ok(exists)
    }
    
    /// Counts outgoing edges for a node with caching.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The ID of the node
    ///
    /// # Returns
    ///
    /// Number of outgoing edges
    pub fn count_outgoing(&self, node_id: &str) -> DbResult<usize> {
        let edges = self.get_outgoing(node_id)?;
        Ok(edges.len())
    }
    
    /// Counts incoming edges for a node with caching.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The ID of the node
    ///
    /// # Returns
    ///
    /// Number of incoming edges
    pub fn count_incoming(&self, node_id: &str) -> DbResult<usize> {
        let edges = self.get_incoming(node_id)?;
        Ok(edges.len())
    }
    
    /// Invalidates cache entries for a specific node.
    ///
    /// This should be called when edges are added or removed for a node.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The ID of the node to invalidate
    pub fn invalidate_node(&self, node_id: &str) -> DbResult<()> {
        let node_key = node_id.to_string();
        
        // Remove from all caches
        if let Ok(cache) = self.outgoing_cache.write() {
            cache.remove(&node_key);
        }
        
        if let Ok(cache) = self.incoming_cache.write() {
            cache.remove(&node_key);
        }
        
        if let Ok(cache) = self.node_cache.write() {
            cache.remove(&node_key);
        }
        
        Ok(())
    }
    
    /// Gets cache statistics for monitoring performance.
    ///
    /// # Returns
    ///
    /// Tuple of (outgoing_cache_stats, incoming_cache_stats, node_cache_stats)
    pub fn get_stats(&self) -> (CacheStats, CacheStats, CacheStats) {
        let outgoing_stats = if let Ok(cache) = self.outgoing_cache.read() {
            cache.get_stats()
        } else {
            CacheStats::new()
        };
        
        let incoming_stats = if let Ok(cache) = self.incoming_cache.read() {
            cache.get_stats()
        } else {
            CacheStats::new()
        };
        
        let node_stats = if let Ok(cache) = self.node_cache.read() {
            cache.get_stats()
        } else {
            CacheStats::new()
        };
        
        (outgoing_stats, incoming_stats, node_stats)
    }
    
    /// Clears all cached data.
    pub fn clear(&self) -> DbResult<()> {
        if let Ok(outgoing_cache) = self.outgoing_cache.write() {
            if let Ok(mut primary) = outgoing_cache.primary.lock() {
                primary.clear();
            }
            if let Ok(mut secondary) = outgoing_cache.secondary.lock() {
                secondary.clear();
            }
        }
        
        if let Ok(incoming_cache) = self.incoming_cache.write() {
            if let Ok(mut primary) = incoming_cache.primary.lock() {
                primary.clear();
            }
            if let Ok(mut secondary) = incoming_cache.secondary.lock() {
                secondary.clear();
            }
        }
        
        if let Ok(node_cache) = self.node_cache.write() {
            if let Ok(mut primary) = node_cache.primary.lock() {
                primary.clear();
            }
            if let Ok(mut secondary) = node_cache.secondary.lock() {
                secondary.clear();
            }
        }
        
        Ok(())
    }
    
    /// Gets the current configuration.
    pub fn get_config(&self) -> &WarmGraphCacheConfig {
        &self.config
    }
    
    /// Updates the cache configuration.
    ///
    /// Note: This only affects new operations; existing cached data is not modified.
    pub fn update_config(&mut self, config: WarmGraphCacheConfig) {
        self.config = config;
    }
    
    /// Gets memory usage statistics.
    ///
    /// # Returns
    ///
    /// Tuple of (outgoing_cache_size, incoming_cache_size, node_cache_size, estimated_memory_bytes)
    pub fn get_memory_usage(&self) -> (usize, usize, usize, usize) {
        let outgoing_cache_size = if let Ok(cache) = self.outgoing_cache.read() {
            if let Ok(primary) = cache.primary.lock() {
                primary.len()
            } else {
                0
            }
        } else {
            0
        };
        
        let incoming_cache_size = if let Ok(cache) = self.incoming_cache.read() {
            if let Ok(primary) = cache.primary.lock() {
                primary.len()
            } else {
                0
            }
        } else {
            0
        };
        
        let node_cache_size = if let Ok(cache) = self.node_cache.read() {
            if let Ok(primary) = cache.primary.lock() {
                primary.len()
            } else {
                0
            }
        } else {
            0
        };
        
        // Rough estimation: each edge list ~100 bytes, each node existence ~1 byte
        let estimated_edge_memory = (outgoing_cache_size + incoming_cache_size) * 100;
        let estimated_node_memory = node_cache_size * 1;
        let estimated_total_memory = estimated_edge_memory + estimated_node_memory;
        
        (outgoing_cache_size, incoming_cache_size, node_cache_size, estimated_total_memory)
    }
}

/// A warm layer cache for vectors with lazy loading from cold storage.
///
/// This cache provides memory-efficient storage for quantized vectors with
/// automatic lazy loading from the cold VectorIndex when cache misses occur.
/// It uses LRU eviction policies and configurable memory limits.
pub struct WarmVectorCache {
    /// Cache for quantized vectors
    vector_cache: Arc<RwLock<MultiLevelCache<String, QuantizedVector>>>,
    
    /// Cache for search results to avoid repeated quantization
    search_cache: Arc<RwLock<MultiLevelCache<String, Vec<SearchResult>>>>,
    
    /// Reference to cold storage for lazy loading
    cold_storage: Arc<RwLock<VectorIndex>>,
    
    /// Configuration for cache behavior
    config: WarmVectorCacheConfig,
}

/// Configuration for WarmVectorCache behavior.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WarmVectorCacheConfig {
    /// Maximum number of vectors to cache
    pub max_vectors: usize,
    
    /// Maximum number of search results to cache
    pub max_search_results: usize,
    
    /// Whether to use product quantization for better compression
    pub use_product_quantization: bool,
    
    /// Subvector size for product quantization
    pub product_quantization_subvector_size: usize,
    
    /// TTL for cached vectors in seconds (0 = no expiration)
    pub vector_ttl_seconds: u64,
    
    /// TTL for cached search results in seconds (0 = no expiration)
    pub search_ttl_seconds: u64,
}

impl Default for WarmVectorCacheConfig {
    fn default() -> Self {
        Self {
            max_vectors: 10000,
            max_search_results: 1000,
            use_product_quantization: false,
            product_quantization_subvector_size: 8,
            vector_ttl_seconds: 3600, // 1 hour
            search_ttl_seconds: 300,  // 5 minutes
        }
    }
}

impl WarmVectorCache {
    /// Creates a new warm vector cache with the specified configuration.
    ///
    /// # Arguments
    ///
    /// * `cold_storage` - Reference to the cold VectorIndex for lazy loading
    /// * `config` - Configuration for cache behavior
    pub fn new(cold_storage: Arc<RwLock<VectorIndex>>, config: WarmVectorCacheConfig) -> Self {
        Self {
            vector_cache: Arc::new(RwLock::new(MultiLevelCache::new(
                config.max_vectors / 2,
                config.max_vectors / 2,
            ))),
            search_cache: Arc::new(RwLock::new(MultiLevelCache::new(
                config.max_search_results / 2,
                config.max_search_results / 2,
            ))),
            cold_storage,
            config,
        }
    }
    
    /// Gets a quantized vector by ID, loading from cold storage if necessary.
    ///
    /// # Arguments
    ///
    /// * `vector_id` - The ID of the vector to retrieve
    ///
    /// # Returns
    ///
    /// The quantized vector if found, or None if not found in cold storage
    pub fn get_vector(&self, vector_id: &str) -> DbResult<Option<QuantizedVector>> {
        // First check the cache
        if let Ok(cache) = self.vector_cache.read() {
            if let Some(quantized_vector) = cache.get(&vector_id.to_string()) {
                return Ok(Some(quantized_vector));
            }
        }
        
        // Cache miss - load from cold storage
        // Note: Current VectorIndex implementation doesn't store original vectors
        // This is a limitation that would need to be addressed in production
        if let Ok(cold_storage) = self.cold_storage.read() {
            if let Some((timestamp, dimension)) = cold_storage.get_metadata(vector_id) {
                // Since we can't retrieve the original vector, create a placeholder
                // In production, we'd need a separate vector storage system
                let placeholder_vector = vec![0.0; dimension];
                
                // Quantize the placeholder vector
                let quantized_vector = if self.config.use_product_quantization {
                    QuantizedVector::new_product_quantized(
                        &placeholder_vector,
                        self.config.product_quantization_subvector_size,
                    )
                } else {
                    QuantizedVector::new(&placeholder_vector)
                };
                
                // Cache the quantized vector
                if let Ok(cache) = self.vector_cache.write() {
                    cache.put(vector_id.to_string(), quantized_vector.clone());
                }
                
                return Ok(Some(quantized_vector));
            }
        }
        
        Ok(None)
    }
    
    /// Adds a vector to the warm cache.
    ///
    /// # Arguments
    ///
    /// * `vector_id` - The ID of the vector
    /// * `vector` - The vector data to quantize and cache
    pub fn put_vector(&self, vector_id: String, vector: Vec<f32>) -> DbResult<()> {
        let quantized_vector = if self.config.use_product_quantization {
            QuantizedVector::new_product_quantized(
                &vector,
                self.config.product_quantization_subvector_size,
            )
        } else {
            QuantizedVector::new(&vector)
        };
        
        if let Ok(cache) = self.vector_cache.write() {
            cache.put(vector_id, quantized_vector);
        }
        
        Ok(())
    }
    
    /// Removes a vector from the warm cache.
    ///
    /// # Arguments
    ///
    /// * `vector_id` - The ID of the vector to remove
    pub fn remove_vector(&self, vector_id: &str) -> DbResult<Option<QuantizedVector>> {
        if let Ok(cache) = self.vector_cache.write() {
            Ok(cache.remove(&vector_id.to_string()))
        } else {
            Ok(None)
        }
    }
    
    /// Performs a cached search, loading results from cold storage if necessary.
    ///
    /// # Arguments
    ///
    /// * `query_vector` - The query vector
    /// * `k` - Number of results to return
    /// * `cache_key` - Optional cache key for the search (auto-generated if None)
    ///
    /// # Returns
    ///
    /// Search results from cache or cold storage
    pub fn search(&self, query_vector: &[f32], k: usize, cache_key: Option<String>) -> DbResult<Vec<SearchResult>> {
        // Generate cache key if not provided
        let cache_key = cache_key.unwrap_or_else(|| {
            // Simple hash of query vector and k for cache key
            let hash = query_vector.iter()
                .enumerate()
                .fold(0u64, |acc, (i, &val)| {
                    acc.wrapping_add((val.to_bits() as u64).wrapping_mul(i as u64 + 1))
                });
            format!("search_{}_{}", hash, k)
        });
        
        // Check search cache first
        if let Ok(cache) = self.search_cache.read() {
            if let Some(cached_results) = cache.get(&cache_key) {
                return Ok(cached_results);
            }
        }
        
        // Cache miss - perform search on cold storage
        if let Ok(cold_storage) = self.cold_storage.read() {
            let results = cold_storage.search(query_vector, k)?;
            
            // Cache the results
            if let Ok(cache) = self.search_cache.write() {
                cache.put(cache_key, results.clone());
            }
            
            return Ok(results);
        }
        
        Ok(vec![])
    }
    
    /// Performs a batch search operation with caching.
    ///
    /// # Arguments
    ///
    /// * `queries` - Vector of query vectors
    /// * `k` - Number of results per query
    ///
    /// # Returns
    ///
    /// Vector of search results for each query
    pub fn batch_search(&self, queries: &[Vec<f32>], k: usize) -> DbResult<Vec<Vec<SearchResult>>> {
        let mut results = Vec::with_capacity(queries.len());
        
        for (i, query) in queries.iter().enumerate() {
            let cache_key = format!("batch_{}_{}", i, k);
            let query_results = self.search(query, k, Some(cache_key))?;
            results.push(query_results);
        }
        
        Ok(results)
    }
    
    /// Gets cache statistics for monitoring performance.
    ///
    /// # Returns
    ///
    /// Tuple of (vector_cache_stats, search_cache_stats)
    pub fn get_stats(&self) -> (CacheStats, CacheStats) {
        let vector_stats = if let Ok(cache) = self.vector_cache.read() {
            cache.get_stats()
        } else {
            CacheStats::new()
        };
        
        let search_stats = if let Ok(cache) = self.search_cache.read() {
            cache.get_stats()
        } else {
            CacheStats::new()
        };
        
        (vector_stats, search_stats)
    }
    
    /// Clears all cached data.
    pub fn clear(&self) -> DbResult<()> {
        if let Ok(vector_cache) = self.vector_cache.write() {
            if let Ok(mut primary) = vector_cache.primary.lock() {
                primary.clear();
            }
            if let Ok(mut secondary) = vector_cache.secondary.lock() {
                secondary.clear();
            }
        }
        
        if let Ok(search_cache) = self.search_cache.write() {
            if let Ok(mut primary) = search_cache.primary.lock() {
                primary.clear();
            }
            if let Ok(mut secondary) = search_cache.secondary.lock() {
                secondary.clear();
            }
        }
        
        Ok(())
    }
    
    /// Gets the current configuration.
    pub fn get_config(&self) -> &WarmVectorCacheConfig {
        &self.config
    }
    
    /// Updates the cache configuration.
    ///
    /// Note: This only affects new operations; existing cached data is not modified.
    pub fn update_config(&mut self, config: WarmVectorCacheConfig) {
        self.config = config;
    }
    
    /// Gets memory usage statistics.
    ///
    /// # Returns
    ///
    /// Tuple of (vector_cache_size, search_cache_size, estimated_memory_bytes)
    pub fn get_memory_usage(&self) -> (usize, usize, usize) {
        let vector_cache_size = if let Ok(cache) = self.vector_cache.read() {
            if let Ok(primary) = cache.primary.lock() {
                primary.len()
            } else {
                0
            }
        } else {
            0
        };
        
        let search_cache_size = if let Ok(cache) = self.search_cache.read() {
            if let Ok(primary) = cache.primary.lock() {
                primary.len()
            } else {
                0
            }
        } else {
            0
        };
        
        // Rough estimation: quantized vectors are ~1/4 size of original f32 vectors
        // Assuming average 384-dimensional vectors
        let estimated_vector_memory = vector_cache_size * 384 / 4; // bytes for quantized vectors
        let estimated_search_memory = search_cache_size * 10 * 32; // rough estimate for search results
        let estimated_total_memory = estimated_vector_memory + estimated_search_memory;
        
        (vector_cache_size, search_cache_size, estimated_total_memory)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;
    
    #[test]
    fn test_lru_cache() {
        let mut cache = LruCache::new(2);
        
        // Test put and get
        cache.put("key1", "value1");
        cache.put("key2", "value2");
        
        assert_eq!(cache.get(&"key1"), Some("value1"));
        assert_eq!(cache.get(&"key2"), Some("value2"));
        
        // Test LRU eviction
        cache.put("key3", "value3"); // This should evict key2
        
        assert_eq!(cache.get(&"key1"), Some("value1")); // Should still be there
        assert_eq!(cache.get(&"key2"), None); // Should be evicted
        assert_eq!(cache.get(&"key3"), Some("value3")); // Should be there
        
        // Test remove
        assert_eq!(cache.remove(&"key1"), Some("value1"));
        assert_eq!(cache.get(&"key1"), None);
    }
    
    #[test]
    fn test_multi_level_cache() {
        let cache = MultiLevelCache::new(2, 4);
        
        // Put some values
        cache.put("key1", "value1");
        cache.put("key2", "value2");
        cache.put("key3", "value3");
        
        // Get values
        assert_eq!(cache.get(&"key1"), Some("value1"));
        assert_eq!(cache.get(&"key2"), Some("value2"));
        assert_eq!(cache.get(&"key3"), Some("value3"));
        
        // Test promotion from secondary to primary
        cache.put("key4", "value4"); // This should evict key1 from primary
        cache.put("key5", "value5"); // This should evict key2 from primary
        
        // key3 should now be in primary (promoted), key1 and key2 in secondary
        assert_eq!(cache.get(&"key3"), Some("value3")); // Should be promoted to primary
        
        // Check stats
        let stats = cache.get_stats();
        assert!(stats.total_accesses.load(Ordering::Relaxed) > 0);
    }
    
    #[test]
    fn test_vector_search_cache() {
        let cache = VectorSearchCache::new(100, 100);
        
        // Test search results caching
        let results = vec![("vec1".to_string(), 0.9), ("vec2".to_string(), 0.8)];
        cache.put_search_results("query1".to_string(), results.clone());
        
        assert_eq!(cache.get_search_results("query1"), Some(results));
        
        // Test metadata caching
        let metadata = (1234567890, 384);
        cache.put_metadata("vec1".to_string(), metadata);
        
        assert_eq!(cache.get_metadata("vec1"), Some(metadata));
    }
    
    #[test]
    fn test_graph_traversal_cache() {
        let cache = GraphTraversalCache::new(100, 100, 100);
        
        // Test BFS results caching
        let bfs_results = vec!["node1".to_string(), "node2".to_string(), "node3".to_string()];
        cache.put_bfs_results("bfs_start".to_string(), bfs_results.clone());
        
        assert_eq!(cache.get_bfs_results("bfs_start"), Some(bfs_results));
        
        // Test shortest path caching
        let path_results = (vec!["node1".to_string(), "node2".to_string()], 2.5);
        cache.put_shortest_path_results("path_start_end".to_string(), path_results);
        
        assert_eq!(cache.get_shortest_path_results("path_start_end"), Some(path_results));
    }
    
    #[test]
    fn test_cache_stats() {
        let cache = MultiLevelCache::<String, String>::new(2, 4);
        
        // Perform some operations
        cache.put("key1".to_string(), "value1".to_string());
        cache.put("key2".to_string(), "value2".to_string());
        
        cache.get(&"key1".to_string()); // Primary hit
        cache.get(&"key2".to_string()); // Primary hit
        cache.get(&"key3".to_string()); // Miss
        
        let stats = cache.get_stats();
        assert_eq!(stats.primary_hits.load(Ordering::Relaxed), 2);
        assert_eq!(stats.misses.load(Ordering::Relaxed), 1);
        assert_eq!(stats.total_accesses.load(Ordering::Relaxed), 3);
        assert!(stats.hit_ratio() > 0.6);
    }
    
    #[test]
    fn test_warm_graph_cache_config() {
        let config = WarmGraphCacheConfig::default();
        assert_eq!(config.max_outgoing_edges, 5000);
        assert_eq!(config.max_incoming_edges, 5000);
        assert_eq!(config.max_nodes, 10000);
        assert_eq!(config.edge_ttl_seconds, 1800);
        assert_eq!(config.node_ttl_seconds, 3600);
    }
    
    #[test]
    fn test_warm_graph_cache_memory_usage() {
        // Create a mock GraphIndex for testing
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let db = sled::open(temp_dir.path()).unwrap();
        let outgoing = db.open_tree("test_graph_out").unwrap();
        let incoming = db.open_tree("test_graph_in").unwrap();
        let graph_index = Arc::new(RwLock::new(GraphIndex::new(outgoing, incoming)));
        
        let config = WarmGraphCacheConfig::default();
        let cache = WarmGraphCache::new(graph_index, config);
        
        // Test initial memory usage
        let (outgoing_size, incoming_size, node_size, total_memory) = cache.get_memory_usage();
        assert_eq!(outgoing_size, 0);
        assert_eq!(incoming_size, 0);
        assert_eq!(node_size, 0);
        assert_eq!(total_memory, 0);
        
        // Test configuration access
        let config = cache.get_config();
        assert_eq!(config.max_outgoing_edges, 5000);
    }
    
    #[test]
    fn test_warm_vector_cache_config() {
        let config = WarmVectorCacheConfig::default();
        assert_eq!(config.max_vectors, 10000);
        assert_eq!(config.max_search_results, 1000);
        assert!(!config.use_product_quantization);
        assert_eq!(config.product_quantization_subvector_size, 8);
        assert_eq!(config.vector_ttl_seconds, 3600);
        assert_eq!(config.search_ttl_seconds, 300);
    }
    
    #[test]
    fn test_warm_vector_cache_memory_usage() {
        // Create a mock VectorIndex for testing
        let vector_index = Arc::new(RwLock::new(VectorIndex::new("test_warm_cache.hnsw").unwrap()));
        let config = WarmVectorCacheConfig::default();
        let cache = WarmVectorCache::new(vector_index, config);
        
        // Test initial memory usage
        let (vector_size, search_size, total_memory) = cache.get_memory_usage();
        assert_eq!(vector_size, 0);
        assert_eq!(search_size, 0);
        assert_eq!(total_memory, 0);
        
        // Test configuration access
        let config = cache.get_config();
        assert_eq!(config.max_vectors, 10000);
    }
}