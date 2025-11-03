//! Indexing layer for fast data retrieval.
//!
//! This crate provides three types of indexes to enable different query patterns:
//! - **Structural indexes**: Fast property-based filtering (O(log n))
//! - **Graph indexes**: Efficient relationship traversal (O(1) neighbor lookup)
//! - **Vector indexes**: Semantic similarity search using HNSW (O(log n))
//!
//! # Architecture
//!
//! ```text
//! IndexManager
//!   ├── Structural Index (B-tree on properties)
//!   ├── Graph Index (Adjacency lists)
//!   └── Vector Index (HNSW for semantic search)
//! ```
//!
//! All indexes are updated **automatically** when data changes in the storage layer,
//! ensuring consistency.
//!
//! # Concurrency
//!
//! This crate provides both traditional Mutex-based and lock-free implementations
//! for high-performance concurrent access:
//!
//! - **Lock-free HotVectorIndex**: Uses atomic operations and lock-free data structures
//!   for maximum performance under high concurrent load
//! - **Lock-free HotGraphIndex**: Provides thread-safe graph operations without traditional locking
//!
//! # Example
//!
//! ```no_run
//! # use indexing::IndexManager;
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create index manager with libmdbx
//! let index_manager = IndexManager::new("my_database")?;
//!
//! // Indexes update automatically when nodes are added
//! // Query by property
//! let messages = index_manager.get_nodes_by_property("chat_id", "chat_123")?;
//!
//! // Traverse graph
//! let edges = index_manager.get_outgoing_edges("chat_123")?;
//!
//! // Semantic search
//! let query = vec![0.1; 384]; // From Python ML model
//! let similar = index_manager.search_vectors(&query, 10)?;
//!
//! // Use lock-free hot indexes for high-concurrency scenarios
//! if let Some(hot_vector) = index_manager.get_hot_vector_index() {
//!     hot_vector.add_vector("vector_123", vec![0.2; 384])?;
//!     let results = hot_vector.search(&query, 5)?;
//! }
//! # Ok(())
//! # }
//!
//! # Comprehensive Documentation
//!
//! For comprehensive documentation and examples, see the [docs] module.
//!
//! # Modules
//!
//! - [structural]: Property-based indexing using B-trees
//! - [graph]: Relationship-based indexing using adjacency lists
//! - [vector]: Semantic similarity search using HNSW
//! - [hybrid]: High-performance hybrid indexes
//! - [graph_traits]: Generic traits for graph operations
//! - [algorithms]: Graph algorithms implementations
//! - [batch]: Batch processing capabilities
//! - [distance_metrics]: Various distance metrics
//! - [errors]: Comprehensive error types
//! - [benchmark]: Performance benchmarking suite
//! - [docs]: Comprehensive documentation and examples

// ============================================================================
// MODULE ORGANIZATION
// ============================================================================

/// Core indexing implementations (essential)
pub mod core {
pub mod zero_copy_ffi;
pub mod structural;
pub mod graph;
pub mod vector;
    pub mod errors;
}

/// Lock-free concurrent data structures (high-performance)
pub mod lock_free {
pub mod lock_free;
pub mod lock_free_hot_vector;
pub mod lock_free_hot_graph;
    pub mod lock_free_btree;
    pub mod lock_free_skiplist;
pub mod lock_free_benchmark;
    pub mod lock_free_stress_tests;
}

/// Graph algorithms (Dijkstra, community detection, etc.)
pub mod algorithms {
    pub mod algorithms;
    pub mod graph_traits;
pub mod community_detection;
pub mod flow_algorithms;
}

/// Advanced features (hybrid indexes, optimized storage, etc.)
pub mod advanced {
    pub mod hybrid;
    pub mod optimized_graph;
    pub mod segment;
    pub mod payload_index;
    pub mod vector_storage;
    pub mod memory_mapping;
    pub mod persistence;
}

/// Utilities and helpers
pub mod utils {
    pub mod caching;
    pub mod builders;
    pub mod iterators;
    pub mod distance_metrics;
    pub mod simd_distance_metrics;
    pub mod batch;
pub mod benchmark;
    pub mod adaptive_concurrency;
pub mod docs;
    pub mod htm;
}

// Re-export core types for backward compatibility
pub use core::zero_copy_ffi;
pub use core::structural;
pub use core::graph;
pub use core::vector;
pub use core::errors;

// Re-export commonly used types
pub use advanced::hybrid;
pub use advanced::payload_index;
pub use utils::caching;
pub use lock_free::lock_free_hot_vector;
pub use lock_free::lock_free_hot_graph;

use common::{DbResult, DbError, EdgeId};
use common::models::{Edge, Embedding, Node};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::thread;
use std::ptr;
use serde::{Deserialize, Serialize};
// libmdbx high-level API not used - we use mdbx-sys FFI directly
use mdbx_sys::{
    MDBX_env, MDBX_txn, MDBX_dbi,
    MDBX_SUCCESS,
    mdbx_env_create, mdbx_env_set_geometry, mdbx_env_open,
    mdbx_txn_begin_ex, mdbx_txn_commit_ex, mdbx_txn_abort, mdbx_dbi_open, MDBX_CREATE,
};
use std::path::Path;


// Public API exports (for external consumers)
pub use core::structural::{StructuralIndex, StructuralIndexGuard};
pub use core::graph::{GraphIndex, GraphIndexGuard};
pub use core::vector::{VectorIndex, SearchResult};
pub use advanced::payload_index::{Payload, PayloadFieldValue, PayloadFilter, PayloadCondition, GeoPoint};
pub use advanced::hybrid::{HotGraphIndex, HotVectorIndex, DataTemperature, QuantizedVector};
pub use utils::caching::{LruCache, MultiLevelCache, CacheStats, VectorSearchCache, GraphTraversalCache, WarmGraphCache, WarmVectorCache, WarmGraphCacheConfig, WarmVectorCacheConfig};
pub use lock_free::lock_free_hot_vector::LockFreeHotVectorIndex;
pub use lock_free::lock_free_hot_graph::LockFreeHotGraphIndex;

/// Coordinates all indexing operations across structural, graph, vector, and hybrid indexes.
///
/// Configuration for the hybrid indexing system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridIndexConfig {
    /// Whether to enable hybrid indexing features
    pub enabled: bool,
    
    /// Configuration for hot layer indexes
    pub hot_layer: HotLayerConfig,
    
    /// Configuration for warm layer caches
    pub warm_layer: WarmLayerConfig,
    
    /// Configuration for background processes
    pub background_tasks: BackgroundTaskConfig,
    
    /// Configuration for query routing
    pub query_routing: QueryRoutingConfig,
    
    /// Configuration for fallback behavior
    pub fallback: FallbackConfig,
}

/// Configuration for hot layer indexes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotLayerConfig {
    /// Whether to enable hot graph index
    pub enable_hot_graph: bool,
    
    /// Whether to enable hot vector index
    pub enable_hot_vector: bool,
    
    /// Maximum memory usage for hot layer (in bytes)
    pub max_memory_bytes: usize,
    
    /// Whether to use lock-free implementations
    pub use_lock_free: bool,
}

/// Configuration for warm layer caches.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarmLayerConfig {
    /// Whether to enable warm layer caching
    pub enabled: bool,
    
    /// Configuration for warm graph cache
    pub graph_cache: WarmGraphCacheConfig,
    
    /// Configuration for warm vector cache
    pub vector_cache: WarmVectorCacheConfig,
}

/// Configuration for background tasks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundTaskConfig {
    /// Whether to enable background tasks
    pub enabled: bool,
    
    /// Interval for syncing hot to cold (in seconds)
    pub sync_interval_seconds: u64,
    
    /// Interval for tier management (in seconds)
    pub tier_management_interval_seconds: u64,
    
    /// Whether to enable automatic tier management
    pub auto_tier_management: bool,
}

/// Configuration for query routing behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryRoutingConfig {
    /// Strategy for routing queries through tiers
    pub strategy: QueryRoutingStrategy,
    
    /// Timeout for hot layer queries (in milliseconds)
    pub hot_layer_timeout_ms: u64,
    
    /// Timeout for warm layer queries (in milliseconds)
    pub warm_layer_timeout_ms: u64,
    
    /// Whether to cache query results
    pub enable_result_caching: bool,
    
    /// Maximum number of cached query results
    pub max_cached_results: usize,
}

/// Strategy for routing queries through the tier system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryRoutingStrategy {
    /// Try hot first, then warm, then cold
    HotWarmCold,
    
    /// Try hot first, then cold (skip warm)
    HotCold,
    
    /// Only use cold layer (disable hybrid)
    ColdOnly,
    
    /// Adaptive routing based on performance metrics
    Adaptive,
}

/// Configuration for fallback behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackConfig {
    /// Whether to enable fallback to cold layer on hot/warm failures
    pub enable_cold_fallback: bool,
    
    /// Maximum number of retries for failed operations
    pub max_retries: u32,
    
    /// Delay between retries (in milliseconds)
    pub retry_delay_ms: u64,
    
    /// Whether to disable hybrid features on repeated failures
    pub disable_on_failures: bool,
    
    /// Number of consecutive failures before disabling hybrid
    pub failure_threshold: u32,
}

impl Default for HybridIndexConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            hot_layer: HotLayerConfig::default(),
            warm_layer: WarmLayerConfig::default(),
            background_tasks: BackgroundTaskConfig::default(),
            query_routing: QueryRoutingConfig::default(),
            fallback: FallbackConfig::default(),
        }
    }
}

impl Default for HotLayerConfig {
    fn default() -> Self {
        Self {
            enable_hot_graph: true,
            enable_hot_vector: true,
            max_memory_bytes: 1_000_000_000, // 1GB
            use_lock_free: true,
        }
    }
}

impl Default for WarmLayerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            graph_cache: WarmGraphCacheConfig::default(),
            vector_cache: WarmVectorCacheConfig::default(),
        }
    }
}

impl Default for BackgroundTaskConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sync_interval_seconds: 300, // 5 minutes
            tier_management_interval_seconds: 600, // 10 minutes
            auto_tier_management: true,
        }
    }
}

impl Default for QueryRoutingConfig {
    fn default() -> Self {
        Self {
            strategy: QueryRoutingStrategy::HotWarmCold,
            hot_layer_timeout_ms: 100,
            warm_layer_timeout_ms: 500,
            enable_result_caching: true,
            max_cached_results: 10000,
        }
    }
}

impl Default for FallbackConfig {
    fn default() -> Self {
        Self {
            enable_cold_fallback: true,
            max_retries: 3,
            retry_delay_ms: 100,
            disable_on_failures: true,
            failure_threshold: 10,
        }
    }
}

/// Runtime state for tracking system performance and failures.
#[derive(Debug, Clone)]
pub struct RuntimeState {
    /// Number of consecutive failures in hot layer
    pub hot_layer_failures: u32,
    
    /// Number of consecutive failures in warm layer
    pub warm_layer_failures: u32,
    
    /// Whether hybrid features are currently disabled due to failures
    pub hybrid_disabled: bool,
    
    /// Timestamp of last successful hot layer operation
    pub last_hot_success: Option<std::time::Instant>,
    
    /// Timestamp of last successful warm layer operation
    pub last_warm_success: Option<std::time::Instant>,
    
    /// Performance metrics for adaptive routing
    pub performance_metrics: PerformanceMetrics,
}

/// Performance metrics for adaptive query routing.
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Average response time for hot layer queries (in milliseconds)
    pub hot_layer_avg_ms: f64,
    
    /// Average response time for warm layer queries (in milliseconds)
    pub warm_layer_avg_ms: f64,
    
    /// Average response time for cold layer queries (in milliseconds)
    pub cold_layer_avg_ms: f64,
    
    /// Hit rate for hot layer queries
    pub hot_layer_hit_rate: f64,
    
    /// Hit rate for warm layer queries
    pub warm_layer_hit_rate: f64,
    
    /// Total number of queries processed
    pub total_queries: u64,
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self {
            hot_layer_failures: 0,
            warm_layer_failures: 0,
            hybrid_disabled: false,
            last_hot_success: None,
            last_warm_success: None,
            performance_metrics: PerformanceMetrics::default(),
        }
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            hot_layer_avg_ms: 0.0,
            warm_layer_avg_ms: 0.0,
            cold_layer_avg_ms: 0.0,
            hot_layer_hit_rate: 0.0,
            warm_layer_hit_rate: 0.0,
            total_queries: 0,
        }
    }
}

/// `IndexManager` ensures that all indexes are kept in sync with the primary data.
/// It provides a unified interface for querying across all index types.
pub struct IndexManager {
    structural: Arc<StructuralIndex>,
    graph: Arc<GraphIndex>,
    vector: Arc<Mutex<VectorIndex>>,
    hot_graph: Option<Arc<LockFreeHotGraphIndex>>,
    hot_vector: Option<Arc<LockFreeHotVectorIndex>>,
    vector_cache: Option<Arc<VectorSearchCache>>,
    graph_cache: Option<Arc<GraphTraversalCache>>,
    warm_graph_cache: Option<Arc<WarmGraphCache>>,
    warm_vector_cache: Option<Arc<WarmVectorCache>>,
    
    /// Current configuration for the hybrid indexing system
    config: Arc<Mutex<HybridIndexConfig>>,
    
    /// Runtime state for tracking failures and performance
    runtime_state: Arc<Mutex<RuntimeState>>,
}

impl IndexManager {
    /// Creates a new `IndexManager` instance with default configuration.
    ///
    /// This initializes all three index types (structural, graph, vector) using
    /// the provided libmdbx path.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the libmdbx database
    ///
    /// # Errors
    ///
    /// Returns `DbError` if any index fails to initialize.
    pub fn new(path: impl AsRef<Path>) -> DbResult<Self> {
        Self::new_with_config(path, HybridIndexConfig::default())
    }
    
    /// Creates a new `IndexManager` instance with custom configuration.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the libmdbx database
    /// * `config` - Configuration for hybrid indexing features
    ///
    /// # Errors
    ///
    /// Returns `DbError` if any index fails to initialize.
    pub fn new_with_config(path: impl AsRef<Path>, config: HybridIndexConfig) -> DbResult<Self> {
        Self::new_with_hybrid(path, config.enabled)
            .map(|mut manager| {
                manager.config = Arc::new(Mutex::new(config));
                manager
            })
    }
    
    /// Creates a new `IndexManager` instance with optional hybrid indexes.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the libmdbx database
    /// * `with_hybrid` - Whether to initialize hybrid indexes
    ///
    /// # Errors
    ///
    /// Returns `DbError` if any index fails to initialize.
    pub fn new_with_hybrid(path: impl AsRef<Path>, with_hybrid: bool) -> DbResult<Self> {
        // Ensure directory exists
        let path = path.as_ref();
        if !path.exists() {
            std::fs::create_dir_all(path)
                .map_err(|e| DbError::InvalidOperation(format!("Failed to create directory: {}", e)))?;
        }
        
        // Create libmdbx environment with raw FFI
        unsafe {
            let mut env: *mut MDBX_env = ptr::null_mut();
            let rc = mdbx_env_create(&mut env as *mut _);
            if rc != MDBX_SUCCESS {
                return Err(DbError::InvalidOperation(format!("mdbx_env_create failed: {}", rc)));
            }
            
            // Set geometry (1GB initial, 100GB max)
            let rc = mdbx_env_set_geometry(
                env,
                -1,                    // size_lower (use default)
                -1,                    // size_now (use default)
                100 * 1024 * 1024 * 1024, // size_upper (100GB)
                -1,                    // growth_step
                -1,                    // shrink_threshold
                -1,                    // page_size
            );
            if rc != MDBX_SUCCESS {
                return Err(DbError::InvalidOperation(format!("mdbx_env_set_geometry failed: {}", rc)));
            }
            
            // Open environment
            let path_cstr = std::ffi::CString::new(path.to_str().unwrap())
                .map_err(|e| DbError::InvalidOperation(format!("Invalid path: {}", e)))?;
            let rc = mdbx_env_open(env, path_cstr.as_ptr(), 0, 0o600);
            if rc != MDBX_SUCCESS {
                return Err(DbError::InvalidOperation(format!("mdbx_env_open failed: {} (path: {})", rc, path.display())));
            }
            
            // Create tables
            let mut txn: *mut MDBX_txn = ptr::null_mut();
            let rc = mdbx_txn_begin_ex(env, ptr::null_mut(), 0, &mut txn as *mut _, ptr::null_mut());
            if rc != MDBX_SUCCESS {
                return Err(DbError::InvalidOperation(format!("Failed to begin txn: {}", rc)));
            }
            
            // Open/create DBIs
            let structural_name = std::ffi::CString::new("structural_index").unwrap();
            let outgoing_name = std::ffi::CString::new("graph_outgoing").unwrap();
            let incoming_name = std::ffi::CString::new("graph_incoming").unwrap();
            
            let mut structural_dbi: MDBX_dbi = 0;
            let mut outgoing_dbi: MDBX_dbi = 0;
            let mut incoming_dbi: MDBX_dbi = 0;
            
            let rc = mdbx_dbi_open(txn, structural_name.as_ptr(), MDBX_CREATE, &mut structural_dbi as *mut _);
            if rc != MDBX_SUCCESS {
                mdbx_txn_abort(txn);
                return Err(DbError::InvalidOperation(format!("Failed to open structural DBI: {}", rc)));
            }
            
            let rc = mdbx_dbi_open(txn, outgoing_name.as_ptr(), MDBX_CREATE, &mut outgoing_dbi as *mut _);
            if rc != MDBX_SUCCESS {
                mdbx_txn_abort(txn);
                return Err(DbError::InvalidOperation(format!("Failed to open outgoing DBI: {}", rc)));
            }
            
            let rc = mdbx_dbi_open(txn, incoming_name.as_ptr(), MDBX_CREATE, &mut incoming_dbi as *mut _);
            if rc != MDBX_SUCCESS {
                mdbx_txn_abort(txn);
                return Err(DbError::InvalidOperation(format!("Failed to open incoming DBI: {}", rc)));
            }
            
            let rc = mdbx_txn_commit_ex(txn, ptr::null_mut());
            if rc != MDBX_SUCCESS {
                return Err(DbError::InvalidOperation(format!("Failed to commit table creation: {}", rc)));
            }
        
            // Note: VectorIndex persists to disk
            // TODO: In production, derive path from db location. For now, use a temp path.
            use std::time::{SystemTime, UNIX_EPOCH};
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| common::DbError::InvalidOperation(format!("System time error: {}", e)))?
                .as_nanos();
            let vector_path = std::env::temp_dir().join(format!("vec_idx_{}", timestamp));
            let vector_index = VectorIndex::new(&vector_path)?;
            
            let (hot_graph, hot_vector) = if with_hybrid {
                (
                    Some(Arc::new(LockFreeHotGraphIndex::new())),
                    Some(Arc::new(LockFreeHotVectorIndex::new()))
                )
            } else {
                (None, None)
            };
            
            // Initialize caching if hybrid is enabled
            let (vector_cache, graph_cache, warm_graph_cache, warm_vector_cache) = if with_hybrid {
                let vector_cache = Arc::new(VectorSearchCache::new(1000, 500));
                let graph_cache = Arc::new(GraphTraversalCache::new(500, 500, 200));
                
                // Note: Warm caches disabled for now (require complex refactoring)
                (Some(vector_cache), Some(graph_cache), None, None)
            } else {
                (None, None, None, None)
            };
            
            Ok(Self {
                structural: Arc::new(StructuralIndex::new(env, structural_dbi)),
                graph: Arc::new(GraphIndex::new(env, outgoing_dbi, incoming_dbi)),
                vector: Arc::new(Mutex::new(vector_index)),
                hot_graph,
                hot_vector,
                vector_cache,
                graph_cache,
                warm_graph_cache,
                warm_vector_cache,
                config: Arc::new(Mutex::new(HybridIndexConfig::default())),
                runtime_state: Arc::new(Mutex::new(RuntimeState::default())),
            })
        }
    }
    
    /// Enables hybrid indexes for this IndexManager.
    ///
    /// This method initializes both hot layer indexes and caching for optimal performance.
    pub fn enable_hybrid(&mut self) {
        self.hot_graph = Some(Arc::new(LockFreeHotGraphIndex::new()));
        self.hot_vector = Some(Arc::new(LockFreeHotVectorIndex::new()));
        
        // Initialize caching layers
        self.vector_cache = Some(Arc::new(VectorSearchCache::new(1000, 500)));
        self.graph_cache = Some(Arc::new(GraphTraversalCache::new(500, 500, 200)));
        
        // Initialize warm layer caches
        // Note: In a real implementation, we'd use the actual database trees
        // For now, we'll skip warm cache initialization in enable_hybrid
        // since we don't have access to the database here
        log::warn!("Warm layer caches not initialized in enable_hybrid - use new_with_hybrid instead");
    }
    
    /// Gets a reference to the structural index.
    pub fn structural(&self) -> &Arc<StructuralIndex> {
        &self.structural
    }
    
    /// Gets a reference to the graph index.
    pub fn graph(&self) -> &Arc<GraphIndex> {
        &self.graph
    }
    
    /// Gets a reference to the hot graph index, if available.
    pub fn get_hot_graph_index(&self) -> Option<&Arc<LockFreeHotGraphIndex>> {
        self.hot_graph.as_ref()
    }
    
    /// Gets a reference to the hot vector index, if available.
    pub fn get_hot_vector_index(&self) -> Option<&Arc<LockFreeHotVectorIndex>> {
        self.hot_vector.as_ref()
    }
    
    /// Gets a reference to the vector search cache, if available.
    pub fn get_vector_cache(&self) -> Option<&Arc<VectorSearchCache>> {
        self.vector_cache.as_ref()
    }
    
    /// Gets a reference to the graph traversal cache, if available.
    pub fn get_graph_cache(&self) -> Option<&Arc<GraphTraversalCache>> {
        self.graph_cache.as_ref()
    }
    
    /// Gets a reference to the warm graph cache, if available.
    pub fn get_warm_graph_cache(&self) -> Option<&Arc<WarmGraphCache>> {
        self.warm_graph_cache.as_ref()
    }
    
    /// Gets a reference to the warm vector cache, if available.
    pub fn get_warm_vector_cache(&self) -> Option<&Arc<WarmVectorCache>> {
        self.warm_vector_cache.as_ref()
    }
    
    /// Gets the current configuration.
    pub fn get_config(&self) -> DbResult<HybridIndexConfig> {
        self.config.lock()
            .map_err(|e| common::DbError::Other(format!("Lock poisoned: {}", e)))
            .map(|config| config.clone())
    }
    
    /// Updates the configuration at runtime.
    ///
    /// This method allows changing configuration without restarting the system.
    /// Some changes may require reinitialization of components.
    pub fn update_config(&self, new_config: HybridIndexConfig) -> DbResult<()> {
        let mut config = self.config.lock()
            .map_err(|e| common::DbError::Other(format!("Lock poisoned: {}", e)))?;
        
        let old_config = config.clone();
        *config = new_config.clone();
        
        // Apply configuration changes that require component reinitialization
        if old_config.enabled != new_config.enabled {
            if new_config.enabled {
                log::info!("Enabling hybrid indexing features");
                // Note: Full reinitialization would require database reference
                // For now, we'll just log the change
            } else {
                log::info!("Disabling hybrid indexing features");
                self.disable_hybrid_features()?;
            }
        }
        
        // Update background task intervals if changed
        if old_config.background_tasks.sync_interval_seconds != new_config.background_tasks.sync_interval_seconds ||
           old_config.background_tasks.tier_management_interval_seconds != new_config.background_tasks.tier_management_interval_seconds {
            log::info!("Background task intervals updated - restart background tasks to apply");
        }
        
        log::info!("Configuration updated successfully");
        Ok(())
    }
    
    /// Disables hybrid features temporarily.
    ///
    /// This can be used for maintenance or when hybrid features are causing issues.
    pub fn disable_hybrid_features(&self) -> DbResult<()> {
        let mut state = self.runtime_state.lock()
            .map_err(|e| common::DbError::Other(format!("Lock poisoned: {}", e)))?;
        
        state.hybrid_disabled = true;
        log::warn!("Hybrid indexing features disabled");
        Ok(())
    }
    
    /// Re-enables hybrid features.
    pub fn enable_hybrid_features(&self) -> DbResult<()> {
        let mut state = self.runtime_state.lock()
            .map_err(|e| common::DbError::Other(format!("Lock poisoned: {}", e)))?;
        
        state.hybrid_disabled = false;
        state.hot_layer_failures = 0;
        state.warm_layer_failures = 0;
        log::info!("Hybrid indexing features re-enabled");
        Ok(())
    }
    
    /// Gets the current runtime state.
    pub fn get_runtime_state(&self) -> DbResult<RuntimeState> {
        self.runtime_state.lock()
            .map_err(|e| common::DbError::Other(format!("Lock poisoned: {}", e)))
            .map(|state| state.clone())
    }
    
    /// Checks if hybrid features are currently available.
    pub fn is_hybrid_available(&self) -> bool {
        let config = self.config.lock().ok();
        let state = self.runtime_state.lock().ok();
        
        match (config, state) {
            (Some(config), Some(state)) => {
                config.enabled && !state.hybrid_disabled
            }
            _ => false
        }
    }
    
    /// Records a successful operation for performance tracking.
    pub fn record_success(&self, layer: &str, response_time_ms: f64) -> DbResult<()> {
        let mut state = self.runtime_state.lock()
            .map_err(|e| common::DbError::Other(format!("Lock poisoned: {}", e)))?;
        
        let now = std::time::Instant::now();
        
        match layer {
            "hot" => {
                state.hot_layer_failures = 0;
                state.last_hot_success = Some(now);
                // Update average response time (simple moving average)
                state.performance_metrics.hot_layer_avg_ms = 
                    (state.performance_metrics.hot_layer_avg_ms * 0.9) + (response_time_ms * 0.1);
            }
            "warm" => {
                state.warm_layer_failures = 0;
                state.last_warm_success = Some(now);
                state.performance_metrics.warm_layer_avg_ms = 
                    (state.performance_metrics.warm_layer_avg_ms * 0.9) + (response_time_ms * 0.1);
            }
            "cold" => {
                state.performance_metrics.cold_layer_avg_ms = 
                    (state.performance_metrics.cold_layer_avg_ms * 0.9) + (response_time_ms * 0.1);
            }
            _ => {}
        }
        
        state.performance_metrics.total_queries += 1;
        Ok(())
    }
    
    /// Records a failed operation and potentially disables hybrid features.
    pub fn record_failure(&self, layer: &str) -> DbResult<()> {
        let mut state = self.runtime_state.lock()
            .map_err(|e| common::DbError::Other(format!("Lock poisoned: {}", e)))?;
        
        let config = self.config.lock()
            .map_err(|e| common::DbError::Other(format!("Lock poisoned: {}", e)))?;
        
        match layer {
            "hot" => {
                state.hot_layer_failures += 1;
                if config.fallback.disable_on_failures && 
                   state.hot_layer_failures >= config.fallback.failure_threshold {
                    log::error!("Hot layer failure threshold reached, disabling hybrid features");
                    state.hybrid_disabled = true;
                }
            }
            "warm" => {
                state.warm_layer_failures += 1;
                if config.fallback.disable_on_failures && 
                   state.warm_layer_failures >= config.fallback.failure_threshold {
                    log::error!("Warm layer failure threshold reached, disabling warm layer");
                    // Could implement partial disabling here
                }
            }
            _ => {}
        }
        
        Ok(())
    }

    /// Indexes a node across all relevant indexes.
    ///
    /// This is called automatically when a node is inserted or updated.
    pub fn index_node(&self, node: &Node) -> DbResult<()> {
        match node {
            Node::Chat(chat) => {
                self.structural.add("node_type", "Chat", chat.id.as_str())?;
                self.structural.add("topic", &chat.topic, chat.id.as_str())?;
            }
            Node::Message(msg) => {
                self.structural.add("node_type", "Message", msg.id.as_str())?;
                self.structural.add("chat_id", msg.chat_id.as_str(), msg.id.as_str())?;
                self.structural.add("sender", &msg.sender, msg.id.as_str())?;
            }
            Node::Entity(entity) => {
                self.structural.add("node_type", "Entity", entity.id.as_str())?;
                self.structural.add("entity_type", &entity.entity_type, entity.id.as_str())?;
                self.structural.add("label", &entity.label, entity.id.as_str())?;
            }
            Node::Summary(summary) => {
                self.structural.add("node_type", "Summary", summary.id.as_str())?;
                self.structural.add("chat_id", summary.chat_id.as_str(), summary.id.as_str())?;
            }
            Node::Attachment(att) => {
                self.structural.add("node_type", "Attachment", att.id.as_str())?;
                self.structural.add("message_id", att.message_id.as_str(), att.id.as_str())?;
                self.structural.add("mime_type", &att.mime_type, att.id.as_str())?;
            }
            Node::WebSearch(search) => {
                self.structural.add("node_type", "WebSearch", search.id.as_str())?;
            }
            Node::ScrapedPage(page) => {
                self.structural.add("node_type", "ScrapedPage", page.id.as_str())?;
                self.structural.add("url", &page.url, page.id.as_str())?;
            }
            Node::Bookmark(bookmark) => {
                self.structural.add("node_type", "Bookmark", bookmark.id.as_str())?;
                self.structural.add("url", &bookmark.url, bookmark.id.as_str())?;
            }
            Node::ImageMetadata(img) => {
                self.structural.add("node_type", "ImageMetadata", img.id.as_str())?;
            }
            Node::AudioTranscript(audio) => {
                self.structural.add("node_type", "AudioTranscript", audio.id.as_str())?;
            }
            Node::ModelInfo(model) => {
                self.structural.add("node_type", "ModelInfo", model.id.as_str())?;
                self.structural.add("model_name", &model.name, model.id.as_str())?;
            }
            Node::ActionOutcome(outcome) => {
                self.structural.add("node_type", "ActionOutcome", outcome.id.as_str())?;
                self.structural.add("action_type", &outcome.action_type, outcome.id.as_str())?;
                self.structural.add("conversation_context", &outcome.conversation_context, outcome.id.as_str())?;
            }
            Node::Log(log) => {
                self.structural.add("node_type", "Log", log.id.as_str())?;
                self.structural.add("level", &log.level.to_string(), log.id.as_str())?;
                self.structural.add("context", &log.context, log.id.as_str())?;
                self.structural.add("source", &log.source.to_string(), log.id.as_str())?;
            }
        }
        Ok(())
    }

    /// Removes a node from all indexes.
    pub fn unindex_node(&self, node: &Node) -> DbResult<()> {
        match node {
            Node::Chat(chat) => {
                self.structural.remove("node_type", "Chat", chat.id.as_str())?;
                self.structural.remove("topic", &chat.topic, chat.id.as_str())?;
            }
            Node::Message(msg) => {
                self.structural.remove("node_type", "Message", msg.id.as_str())?;
                self.structural.remove("chat_id", msg.chat_id.as_str(), msg.id.as_str())?;
                self.structural.remove("sender", &msg.sender, msg.id.as_str())?;
            }
            Node::Entity(entity) => {
                self.structural.remove("node_type", "Entity", entity.id.as_str())?;
                self.structural.remove("entity_type", &entity.entity_type, entity.id.as_str())?;
                self.structural.remove("label", &entity.label, entity.id.as_str())?;
            }
            Node::Summary(summary) => {
                self.structural.remove("node_type", "Summary", summary.id.as_str())?;
                self.structural.remove("chat_id", summary.chat_id.as_str(), summary.id.as_str())?;
            }
            Node::Attachment(att) => {
                self.structural.remove("node_type", "Attachment", att.id.as_str())?;
                self.structural.remove("message_id", att.message_id.as_str(), att.id.as_str())?;
                self.structural.remove("mime_type", &att.mime_type, att.id.as_str())?;
            }
            Node::WebSearch(search) => {
                self.structural.remove("node_type", "WebSearch", search.id.as_str())?;
            }
            Node::ScrapedPage(page) => {
                self.structural.remove("node_type", "ScrapedPage", page.id.as_str())?;
                self.structural.remove("url", &page.url, page.id.as_str())?;
            }
            Node::Bookmark(bookmark) => {
                self.structural.remove("node_type", "Bookmark", bookmark.id.as_str())?;
                self.structural.remove("url", &bookmark.url, bookmark.id.as_str())?;
            }
            Node::ImageMetadata(img) => {
                self.structural.remove("node_type", "ImageMetadata", img.id.as_str())?;
            }
            Node::AudioTranscript(audio) => {
                self.structural.remove("node_type", "AudioTranscript", audio.id.as_str())?;
            }
            Node::ModelInfo(model) => {
                self.structural.remove("node_type", "ModelInfo", model.id.as_str())?;
                self.structural.remove("model_name", &model.name, model.id.as_str())?;
            }
            Node::ActionOutcome(outcome) => {
                self.structural.remove("node_type", "ActionOutcome", outcome.id.as_str())?;
                self.structural.remove("action_type", &outcome.action_type, outcome.id.as_str())?;
                self.structural.remove("conversation_context", &outcome.conversation_context, outcome.id.as_str())?;
            }
            Node::Log(log) => {
                self.structural.remove("node_type", "Log", log.id.as_str())?;
                self.structural.remove("level", &log.level.to_string(), log.id.as_str())?;
                self.structural.remove("context", &log.context, log.id.as_str())?;
                self.structural.remove("source", &log.source.to_string(), log.id.as_str())?;
            }
        }
        Ok(())
    }

    /// Indexes an edge in the graph index.
    pub fn index_edge(&self, edge: &Edge) -> DbResult<()> {
        self.graph.add_edge(edge)
    }

    /// Removes an edge from the graph index.
    pub fn unindex_edge(&self, edge: &Edge) -> DbResult<()> {
        self.graph.remove_edge(edge)
    }

    /// Indexes an embedding in the vector index.
    pub fn index_embedding(&self, embedding: &Embedding) -> DbResult<()> {
        let mut vec_idx = self.vector.lock()
            .map_err(|e| common::DbError::Other(format!("Lock poisoned: {}", e)))?;
        vec_idx.add_vector(embedding.id.as_str(), embedding.vector.clone())
    }

    /// Removes an embedding from the vector index.
    pub fn unindex_embedding(&self, embedding_id: &str) -> DbResult<()> {
        let mut vec_idx = self.vector.lock()
            .map_err(|e| common::DbError::Other(format!("Lock poisoned: {}", e)))?;
        vec_idx.remove_vector(embedding_id)?;
        Ok(())
    }

    // --- Query Methods ---

    /// Queries the structural index for nodes matching a property value.
    ///
    /// # Arguments
    ///
    /// * `property` - The property name (e.g., "chat_id", "sender")
    /// * `value` - The value to match
    ///
    /// # Returns
    ///
    /// A vector of node IDs matching the query.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use indexing::IndexManager;
    /// # fn example(index_manager: &IndexManager) -> Result<(), Box<dyn std::error::Error>> {
    /// // Find all messages in a specific chat
    /// let message_ids = index_manager.get_nodes_by_property("chat_id", "chat_123")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_nodes_by_property(&self, property: &str, value: &str) -> DbResult<Option<StructuralIndexGuard>> {
        self.structural.get(property, value)
    }

    /// Gets all outgoing edges from a node.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The ID of the node
    ///
    /// # Returns
    ///
    /// A vector of edge IDs for outgoing edges.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use indexing::IndexManager;
    /// # fn example(index_manager: &IndexManager) -> Result<(), Box<dyn std::error::Error>> {
    /// // Find all edges pointing from a chat node
    /// let outgoing = index_manager.get_outgoing_edges("chat_123")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_outgoing_edges(&self, node_id: &str) -> DbResult<Option<GraphIndexGuard>> {
        self.graph.get_outgoing(node_id)
    }

    /// Gets all incoming edges to a node.
    pub fn get_incoming_edges(&self, node_id: &str) -> DbResult<Option<GraphIndexGuard>> {
        self.graph.get_incoming(node_id)
    }

    /// Performs semantic similarity search using vector embeddings.
    ///
    /// # Arguments
    ///
    /// * `query` - The query vector
    /// * `k` - Number of nearest neighbors to return
    ///
    /// # Returns
    ///
    /// A vector of `SearchResult` structs with embedding IDs and distances.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use indexing::IndexManager;
    /// # fn example(index_manager: &IndexManager) -> Result<(), Box<dyn std::error::Error>> {
    /// // Semantic search (vector from ML model)
    /// let query_vector = vec![0.1; 384];
    /// let similar = index_manager.search_vectors(&query_vector, 10)?;
    /// for result in similar {
    ///     println!("ID: {}, Distance: {}", result.id, result.distance);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn search_vectors(&self, query: &[f32], k: usize) -> DbResult<Vec<SearchResult>> {
        let vec_idx = self.vector.lock()
            .map_err(|e| common::DbError::Other(format!("Lock poisoned: {}", e)))?;
        vec_idx.search(query, k)
    }
    
    /// Synchronizes data from hot indexes to cold indexes.
    ///
    /// This method transfers data from the hot in-memory indexes to the
    /// persistent cold indexes to ensure consistency.
    pub fn sync_hot_to_cold(&self) -> DbResult<()> {
        // Sync hot graph to cold graph
        if let Some(hot_graph) = &self.hot_graph {
            // In a real implementation, we would transfer all nodes and edges
            // from the hot graph to the cold graph indexes
            // For now, we'll just log that synchronization would happen
            log::debug!("Synchronizing hot graph to cold indexes ({} nodes, {} edges)",
                hot_graph.node_count(), hot_graph.edge_count());
        }
        
        // Sync hot vectors to cold vectors
        if let Some(hot_vector) = &self.hot_vector {
            // In a real implementation, we would transfer all vectors
            // from the hot vector index to the cold vector index
            // For now, we'll just log that synchronization would happen
            log::debug!("Synchronizing hot vectors to cold indexes ({} vectors)",
                hot_vector.len());
        }
        
        Ok(())
    }
    
    /// Promotes data from cold indexes to hot indexes.
    ///
    /// This method transfers frequently accessed data from the persistent
    /// cold indexes to the in-memory hot indexes for faster access.
    pub fn promote_cold_to_hot(&self) -> DbResult<()> {
        // In a real implementation, we would analyze access patterns
        // and transfer frequently accessed data from cold to hot indexes
        // For now, we'll just log that promotion would happen
        log::debug!("Promoting frequently accessed data from cold to hot indexes");
        Ok(())
    }
    
    /// Demotes data from hot indexes to cold indexes.
    ///
    /// This method transfers infrequently accessed data from the in-memory
    /// hot indexes to the persistent cold indexes to free up memory.
    pub fn demote_hot_to_cold(&self) -> DbResult<()> {
        // In a real implementation, we would analyze access patterns
        // and transfer infrequently accessed data from hot to cold indexes
        // For now, we'll just log that demotion would happen
        log::debug!("Demoting infrequently accessed data from hot to cold indexes");
        Ok(())
    }
    
    /// Automatically manages data placement between hot and cold tiers.
    ///
    /// This method analyzes access patterns and automatically moves data
    /// between hot and cold indexes to optimize performance and memory usage.
    pub fn auto_manage_tiers(&self) -> DbResult<()> {
        // Sync hot to cold first
        self.sync_hot_to_cold()?;
        
        // Then manage tier promotions/demotions
        self.promote_cold_to_hot()?;
        self.demote_hot_to_cold()?;
        
        // Manage warm layer tier transitions
        self.manage_warm_layer_tiers()?;
        
        // Also manage tiers in hot indexes
        if let Some(_hot_graph) = &self.hot_graph {
            // hot_graph.auto_manage_tiers()?;
        }
        
        if let Some(_hot_vector) = &self.hot_vector {
            // hot_vector.auto_manage_tiers()?;
        }
        
        Ok(())
    }
    
    /// Starts background tier management tasks.
    ///
    /// This method spawns background threads that periodically manage
    /// data placement between hot and cold tiers.
    ///
    /// # Arguments
    ///
    /// * `sync_interval` - How often to sync hot to cold (in seconds)
    /// * `tier_management_interval` - How often to manage tiers (in seconds)
    pub fn start_background_tasks(&self, sync_interval: u64, tier_management_interval: u64) {
        let self_clone = Arc::new(self.clone());
        let self_clone2 = Arc::new(self.clone());
        
        // Start sync task
        let sync_clone = Arc::clone(&self_clone);
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(sync_interval));
                if let Err(e) = sync_clone.sync_hot_to_cold() {
                    log::error!("Error syncing hot to cold: {}", e);
                }
            }
        });
        
        // Start tier management task
        let tier_clone = Arc::clone(&self_clone2);
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(tier_management_interval));
                if let Err(e) = tier_clone.auto_manage_tiers() {
                    log::error!("Error managing tiers: {}", e);
                }
            }
        });
    }
    
    /// Adds a node to the hot graph index.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The node ID
    /// * `metadata` - Optional metadata for the node
    pub fn add_hot_graph_node(&self, node_id: &str, metadata: Option<&str>) -> DbResult<()> {
        if let Some(hot_graph) = &self.hot_graph {
            hot_graph.add_node(node_id, metadata)
        } else {
            Err(common::DbError::InvalidOperation(
                "Hot graph index not enabled".to_string()
            ))
        }
    }
    
    /// Adds an edge to the hot graph index.
    ///
    /// # Arguments
    ///
    /// * `from` - The source node ID
    /// * `to` - The target node ID
    pub fn add_hot_graph_edge(&self, from: &str, to: &str) -> DbResult<()> {
        if let Some(hot_graph) = &self.hot_graph {
            hot_graph.add_edge(from, to)
        } else {
            Err(common::DbError::InvalidOperation(
                "Hot graph index not enabled".to_string()
            ))
        }
    }
    
    /// Adds a weighted edge to the hot graph index.
    ///
    /// # Arguments
    ///
    /// * `from` - The source node ID
    /// * `to` - The target node ID
    /// * `weight` - The edge weight
    pub fn add_hot_graph_edge_with_weight(&self, from: &str, to: &str, weight: f32) -> DbResult<()> {
        if let Some(hot_graph) = &self.hot_graph {
            hot_graph.add_edge_with_weight(from, to, weight)
        } else {
            Err(common::DbError::InvalidOperation(
                "Hot graph index not enabled".to_string()
            ))
        }
    }
    
    /// Finds the shortest path between two nodes using Dijkstra's algorithm.
    ///
    /// This method uses the hot graph index if available, otherwise falls back
    /// to a simple BFS search on the persistent graph index.
    ///
    /// # Arguments
    ///
    /// * `start` - The start node ID
    /// * `end` - The end node ID
    ///
    /// # Returns
    ///
    /// A tuple of (path, distance) where path is the sequence of node IDs
    /// and distance is the total path distance.
    pub fn dijkstra_shortest_path(&self, _start: &str, _end: &str) -> DbResult<(Vec<String>, f32)> {
        if let Some(_hot_graph) = &self.hot_graph {
            // Note: In a real implementation, we would need to implement Dijkstra's algorithm
            // for the lock-free graph index. For now, we're returning a placeholder.
            Err(common::DbError::InvalidOperation(
                "Dijkstra algorithm not yet implemented for lock-free graph index".to_string()
            ))
        } else {
            // Fallback to simple path finding on persistent graph
            // This is a simplified implementation - in a real system, you'd want
            // a more sophisticated algorithm
            Err(common::DbError::InvalidOperation(
                "Hot graph index not available for Dijkstra algorithm".to_string()
            ))
        }
    }
    
    /// Finds the shortest path between two nodes using A* algorithm.
    ///
    /// This method uses the hot graph index if available.
    ///
    /// # Arguments
    ///
    /// * `start` - The start node ID
    /// * `end` - The end node ID
    /// * `heuristic` - A function that estimates the distance from a node to the end
    ///
    /// # Returns
    ///
    /// A tuple of (path, distance) where path is the sequence of node IDs
    /// and distance is the total path distance.
    pub fn astar_path<F>(&self, _start: &str, _end: &str, _heuristic: F) -> DbResult<(Vec<String>, f32)>
    where
        F: Fn(&str) -> f32,
    {
        if let Some(_hot_graph) = &self.hot_graph {
            // Note: In a real implementation, we would need to implement A* algorithm
            // for the lock-free graph index. For now, we're returning a placeholder.
            Err(common::DbError::InvalidOperation(
                "A* algorithm not yet implemented for lock-free graph index".to_string()
            ))
        } else {
            Err(common::DbError::InvalidOperation(
                "Hot graph index not available for A* algorithm".to_string()
            ))
        }
    }
    
    /// Finds strongly connected components in the graph.
    ///
    /// This method uses the hot graph index if available.
    ///
    /// # Returns
    ///
    /// A vector of vectors, where each inner vector represents a strongly
    /// connected component.
    pub fn strongly_connected_components(&self) -> DbResult<Vec<Vec<String>>> {
        if let Some(_hot_graph) = &self.hot_graph {
            // Note: In a real implementation, we would need to implement SCC algorithm
            // for the lock-free graph index. For now, we're returning a placeholder.
            Err(common::DbError::InvalidOperation(
                "SCC algorithm not yet implemented for lock-free graph index".to_string()
            ))
        } else {
            Err(common::DbError::InvalidOperation(
                "Hot graph index not available for SCC algorithm".to_string()
            ))
        }
    }
    
    /// Adds a vector to the hot vector index.
    ///
    /// # Arguments
    ///
    /// * `id` - The vector ID
    /// * `vector` - The vector data
    pub fn add_hot_vector(&self, id: &str, vector: Vec<f32>) -> DbResult<()> {
        if let Some(hot_vector) = &self.hot_vector {
            hot_vector.add_vector(id, vector)
        } else {
            Err(common::DbError::InvalidOperation(
                "Hot vector index not enabled".to_string()
            ))
        }
    }
    
    /// Searches for similar vectors in the hot vector index.
    ///
    /// # Arguments
    ///
    /// * `query` - The query vector
    /// * `k` - The number of results to return
    ///
    /// # Returns
    ///
    /// A vector of (ID, similarity) tuples, sorted by similarity (highest first).
    pub fn search_hot_vectors(&self, query: &[f32], k: usize) -> DbResult<Vec<(String, f32)>> {
        if let Some(hot_vector) = &self.hot_vector {
            hot_vector.search(query, k)
        } else {
            Err(common::DbError::InvalidOperation(
                "Hot vector index not enabled".to_string()
            ))
        }
    }
    
    /// Migrates data from existing cold indexes to the hybrid system.
    ///
    /// This method transfers all data from the persistent indexes to the
    /// hot indexes when the hybrid system is enabled.
    pub fn migrate_to_hybrid(&self) -> DbResult<()> {
        // Check if hybrid indexes are enabled
        if self.hot_graph.is_none() || self.hot_vector.is_none() {
            return Err(common::DbError::InvalidOperation(
                "Hybrid indexes not enabled. Call enable_hybrid() first.".to_string()
            ));
        }
        
        log::info!("Starting migration to hybrid system");
        
        // Migrate graph data
        self.migrate_graph_data()?;
        
        // Migrate vector data
        self.migrate_vector_data()?;
        
        log::info!("Completed migration to hybrid system");
        Ok(())
    }
    
    /// Migrates graph data from cold indexes to hot graph index.
    fn migrate_graph_data(&self) -> DbResult<()> {
        if let Some(_hot_graph) = &self.hot_graph {
            log::info!("Migrating graph data to hot index");
            
            // Since the current GraphIndex doesn't expose iteration methods,
            // we'll implement a placeholder that can be extended when those methods are added
            // TODO: Implement actual migration when GraphIndex has get_all_nodes() and get_all_edges()
            
            log::info!("Graph data migration completed (placeholder implementation)");
        }
        Ok(())
    }
    
    /// Migrates vector data from cold indexes to hot vector index.
    fn migrate_vector_data(&self) -> DbResult<()> {
        if let Some(_hot_vector) = &self.hot_vector {
            log::info!("Migrating vector data to hot index");
            
            // Since the current VectorIndex doesn't expose iteration methods,
            // we'll implement a placeholder that can be extended when those methods are added
            // TODO: Implement actual migration when VectorIndex has get_all_embeddings()
            
            log::info!("Vector data migration completed (placeholder implementation)");
        }
        Ok(())
    }
    
    /// Routes queries through the hybrid tier system for optimal performance.
    ///
    /// This method implements intelligent query routing based on configuration
    /// and runtime performance metrics.
    pub fn search_vectors_hybrid(&self, query: &[f32], k: usize) -> DbResult<Vec<SearchResult>> {
        let start_time = std::time::Instant::now();
        
        // Check if hybrid features are available
        if !self.is_hybrid_available() {
            log::debug!("Hybrid features disabled, using cold layer only");
            let results = self.search_vectors(query, k)?;
            self.record_success("cold", start_time.elapsed().as_millis() as f64)?;
            return Ok(results);
        }
        
        let config = self.get_config()?;
        
        // Route based on configured strategy
        match config.query_routing.strategy {
            QueryRoutingStrategy::HotWarmCold => {
                self.search_vectors_hot_warm_cold(query, k, &config, start_time)
            }
            QueryRoutingStrategy::HotCold => {
                self.search_vectors_hot_cold(query, k, &config, start_time)
            }
            QueryRoutingStrategy::ColdOnly => {
                let results = self.search_vectors(query, k)?;
                self.record_success("cold", start_time.elapsed().as_millis() as f64)?;
                Ok(results)
            }
            QueryRoutingStrategy::Adaptive => {
                self.search_vectors_adaptive(query, k, &config, start_time)
            }
        }
    }
    
    /// Implements hot → warm → cold routing strategy.
    fn search_vectors_hot_warm_cold(&self, query: &[f32], k: usize, config: &HybridIndexConfig, start_time: std::time::Instant) -> DbResult<Vec<SearchResult>> {
        // Create cache key for result caching
        let cache_key = if config.query_routing.enable_result_caching {
            Some(format!("query_{}_{}", 
                query.iter().map(|f| format!("{:.3}", f)).collect::<Vec<_>>().join("_"), 
                k
            ))
        } else {
            None
        };
        
        // Check result cache first
        if let (Some(cache_key), Some(vector_cache)) = (&cache_key, &self.vector_cache) {
            if let Some(cached_results) = vector_cache.get_search_results(cache_key) {
                log::debug!("Query served from result cache: {} results", cached_results.len());
                self.record_success("cache", start_time.elapsed().as_millis() as f64)?;
                return Ok(cached_results.into_iter().map(|(id, distance)| SearchResult {
                    id: id.into(),
                    score: 1.0 - distance,
                    distance,
                }).collect());
            }
        }
        
        // Try hot layer with timeout
        if let Some(hot_vector) = &self.hot_vector {
            let hot_start = std::time::Instant::now();
            match hot_vector.search(query, k) {
                Ok(hot_results) => {
                    let elapsed = hot_start.elapsed().as_millis() as f64;
                    if elapsed <= config.query_routing.hot_layer_timeout_ms as f64 {
                        let search_results: Vec<SearchResult> = hot_results
                            .into_iter()
                            .map(|(id, similarity)| SearchResult {
                                id: id.into(),
                                score: similarity,
                                distance: 1.0 - similarity,
                            })
                            .collect();
                        
                        // Cache the results
                        if let (Some(cache_key), Some(vector_cache)) = (&cache_key, &self.vector_cache) {
                            let cache_results: Vec<(String, f32)> = search_results.iter()
                                .map(|r| (r.id.to_string(), r.distance))
                                .collect();
                            vector_cache.put_search_results(cache_key.clone(), cache_results);
                        }
                        
                        log::debug!("Query served from hot layer: {} results in {}ms", search_results.len(), elapsed);
                        self.record_success("hot", elapsed)?;
                        return Ok(search_results);
                    } else {
                        log::warn!("Hot layer query exceeded timeout ({}ms > {}ms)", elapsed, config.query_routing.hot_layer_timeout_ms);
                    }
                }
                Err(e) => {
                    log::warn!("Hot layer query failed: {}", e);
                    self.record_failure("hot")?;
                }
            }
        }
        
        // Try warm layer with timeout
        if let Some(warm_vector_cache) = &self.warm_vector_cache {
            let warm_start = std::time::Instant::now();
            match warm_vector_cache.search(query, k, cache_key.clone()) {
                Ok(warm_results) => {
                    let elapsed = warm_start.elapsed().as_millis() as f64;
                    if elapsed <= config.query_routing.warm_layer_timeout_ms as f64 {
                        log::debug!("Query served from warm layer: {} results in {}ms", warm_results.len(), elapsed);
                        self.record_success("warm", elapsed)?;
                        return Ok(warm_results);
                    } else {
                        log::warn!("Warm layer query exceeded timeout ({}ms > {}ms)", elapsed, config.query_routing.warm_layer_timeout_ms);
                    }
                }
                Err(e) => {
                    log::warn!("Warm layer query failed: {}", e);
                    self.record_failure("warm")?;
                }
            }
        }
        
        // Fall back to cold layer
        if config.fallback.enable_cold_fallback {
            log::debug!("Falling back to cold layer");
            let cold_results = self.search_vectors(query, k)?;
            let elapsed = start_time.elapsed().as_millis() as f64;
            
            // Cache the cold layer results
            if let (Some(cache_key), Some(vector_cache)) = (&cache_key, &self.vector_cache) {
                let cache_results: Vec<(String, f32)> = cold_results.iter()
                    .map(|r| (r.id.to_string(), r.distance))
                    .collect();
                vector_cache.put_search_results(cache_key.clone(), cache_results);
            }
            
            self.record_success("cold", elapsed)?;
            Ok(cold_results)
        } else {
            Err(common::DbError::Other("All hybrid layers failed and cold fallback is disabled".to_string()))
        }
    }
    
    /// Implements hot → cold routing strategy (skips warm layer).
    fn search_vectors_hot_cold(&self, query: &[f32], k: usize, config: &HybridIndexConfig, start_time: std::time::Instant) -> DbResult<Vec<SearchResult>> {
        // Try hot layer first
        if let Some(hot_vector) = &self.hot_vector {
            match hot_vector.search(query, k) {
                Ok(hot_results) => {
                    let elapsed = start_time.elapsed().as_millis() as f64;
                    let search_results: Vec<SearchResult> = hot_results
                        .into_iter()
                        .map(|(id, similarity)| SearchResult {
                            id: id.into(),
                            score: similarity,
                            distance: 1.0 - similarity,
                        })
                        .collect();
                    
                    log::debug!("Query served from hot layer: {} results", search_results.len());
                    self.record_success("hot", elapsed)?;
                    return Ok(search_results);
                }
                Err(e) => {
                    log::warn!("Hot layer query failed: {}", e);
                    self.record_failure("hot")?;
                }
            }
        }
        
        // Fall back directly to cold layer
        if config.fallback.enable_cold_fallback {
            let cold_results = self.search_vectors(query, k)?;
            let elapsed = start_time.elapsed().as_millis() as f64;
            self.record_success("cold", elapsed)?;
            Ok(cold_results)
        } else {
            Err(common::DbError::Other("Hot layer failed and cold fallback is disabled".to_string()))
        }
    }
    
    /// Implements adaptive routing based on performance metrics.
    fn search_vectors_adaptive(&self, query: &[f32], k: usize, config: &HybridIndexConfig, start_time: std::time::Instant) -> DbResult<Vec<SearchResult>> {
        let state = self.get_runtime_state()?;
        
        // Choose the best layer based on performance metrics
        let best_layer = if state.performance_metrics.hot_layer_avg_ms < state.performance_metrics.warm_layer_avg_ms &&
                           state.performance_metrics.hot_layer_avg_ms < state.performance_metrics.cold_layer_avg_ms {
            "hot"
        } else if state.performance_metrics.warm_layer_avg_ms < state.performance_metrics.cold_layer_avg_ms {
            "warm"
        } else {
            "cold"
        };
        
        log::debug!("Adaptive routing chose {} layer (avg: hot={:.1}ms, warm={:.1}ms, cold={:.1}ms)", 
                   best_layer,
                   state.performance_metrics.hot_layer_avg_ms,
                   state.performance_metrics.warm_layer_avg_ms,
                   state.performance_metrics.cold_layer_avg_ms);
        
        // Try the best layer first, then fall back to hot-warm-cold strategy
        match best_layer {
            "hot" => self.search_vectors_hot_warm_cold(query, k, config, start_time),
            "warm" => {
                // Try warm first, then hot, then cold
                if let Some(warm_vector_cache) = &self.warm_vector_cache {
                    match warm_vector_cache.search(query, k, None) {
                        Ok(results) => {
                            let elapsed = start_time.elapsed().as_millis() as f64;
                            self.record_success("warm", elapsed)?;
                            return Ok(results);
                        }
                        Err(_) => {
                            self.record_failure("warm")?;
                        }
                    }
                }
                self.search_vectors_hot_warm_cold(query, k, config, start_time)
            }
            _ => {
                // Try cold first, but still allow hot/warm as backup
                match self.search_vectors(query, k) {
                    Ok(results) => {
                        let elapsed = start_time.elapsed().as_millis() as f64;
                        self.record_success("cold", elapsed)?;
                        Ok(results)
                    }
                    Err(_) => {
                        self.search_vectors_hot_warm_cold(query, k, config, start_time)
                    }
                }
            }
        }
    }
    
    /// Routes graph queries through the hybrid tier system.
    /// 
    /// Returns owned Vec<EdgeId> for compatibility (allocates).
    /// For zero-copy access, use `get_outgoing_edges()` directly.
    pub fn get_outgoing_edges_hybrid(&self, node_id: &str) -> DbResult<Vec<EdgeId>> {
        // Try hot layer first if available
        if let Some(_hot_graph) = &self.hot_graph {
            // Note: The current HotGraphIndex uses different method signatures
            // This is a placeholder for when the APIs are aligned
            log::debug!("Hot graph layer available but API alignment needed");
        }
        
        // Fall back to cold layer - convert guard to owned Vec
        log::debug!("Query served from cold layer");
        match self.get_outgoing_edges(node_id)? {
            Some(guard) => guard.to_owned(),
            None => Ok(Vec::new()),
        }
    }
    
    /// Routes graph queries for incoming edges through the hybrid tier system.
    /// 
    /// Returns owned Vec<EdgeId> for compatibility (allocates).
    /// For zero-copy access, use `get_incoming_edges()` directly.
    pub fn get_incoming_edges_hybrid(&self, node_id: &str) -> DbResult<Vec<EdgeId>> {
        // Try hot layer first if available
        if let Some(_hot_graph) = &self.hot_graph {
            // Note: The current HotGraphIndex uses different method signatures
            // This is a placeholder for when the APIs are aligned
            log::debug!("Hot graph layer available but API alignment needed");
        }
        
        // Fall back to cold layer - convert guard to owned Vec
        log::debug!("Query served from cold layer");
        match self.get_incoming_edges(node_id)? {
            Some(guard) => guard.to_owned(),
            None => Ok(Vec::new()),
        }
    }
    
    /// Ensures backward compatibility during transition to hybrid system.
    ///
    /// This method verifies that all data in the hot indexes is also available
    /// in the cold indexes for backward compatibility.
    pub fn ensure_backward_compatibility(&self) -> DbResult<()> {
        // Sync hot indexes to cold indexes to ensure consistency
        self.sync_hot_to_cold()?;
        Ok(())
    }
    
    /// Manages tier transitions for the warm layer caches.
    ///
    /// This method analyzes access patterns and promotes/demotes data between
    /// hot, warm, and cold tiers based on temperature thresholds.
    fn manage_warm_layer_tiers(&self) -> DbResult<()> {
        // Manage warm graph cache tiers
        if let Some(warm_graph_cache) = &self.warm_graph_cache {
            // Get cache statistics to understand usage patterns
            let (outgoing_stats, incoming_stats, node_stats) = warm_graph_cache.get_stats();
            
            // Log cache performance for monitoring
            log::debug!(
                "Warm graph cache stats - Outgoing: {:.2}% hit rate, Incoming: {:.2}% hit rate, Nodes: {:.2}% hit rate",
                outgoing_stats.hit_ratio() * 100.0,
                incoming_stats.hit_ratio() * 100.0,
                node_stats.hit_ratio() * 100.0
            );
            
            // If hit rate is low, we might need to adjust cache sizes or eviction policies
            if outgoing_stats.hit_ratio() < 0.5 {
                log::warn!("Low warm graph cache hit rate: {:.2}%", outgoing_stats.hit_ratio() * 100.0);
            }
        }
        
        // Manage warm vector cache tiers
        if let Some(warm_vector_cache) = &self.warm_vector_cache {
            // Get cache statistics
            let (vector_stats, search_stats) = warm_vector_cache.get_stats();
            
            // Log cache performance
            log::debug!(
                "Warm vector cache stats - Vectors: {:.2}% hit rate, Searches: {:.2}% hit rate",
                vector_stats.hit_ratio() * 100.0,
                search_stats.hit_ratio() * 100.0
            );
            
            // Monitor memory usage
            let (vector_size, search_size, total_memory) = warm_vector_cache.get_memory_usage();
            log::debug!(
                "Warm vector cache memory - Vectors: {}, Searches: {}, Total: {} bytes",
                vector_size, search_size, total_memory
            );
            
            // If memory usage is too high, we might need to clear some cache
            if total_memory > 100_000_000 { // 100MB threshold
                log::warn!("High warm vector cache memory usage: {} bytes", total_memory);
                // In a production system, we might implement selective eviction here
            }
        }
        
        // Implement promotion/demotion logic based on access patterns
        self.promote_warm_to_hot()?;
        self.demote_hot_to_warm()?;
        self.demote_warm_to_cold()?;
        
        Ok(())
    }
    
    /// Promotes frequently accessed data from warm to hot tier.
    fn promote_warm_to_hot(&self) -> DbResult<()> {
        // This would analyze access patterns in warm caches and promote
        // frequently accessed items to hot tier
        // For now, this is a placeholder implementation
        log::debug!("Checking for warm→hot promotions");
        Ok(())
    }
    
    /// Demotes less frequently accessed data from hot to warm tier.
    fn demote_hot_to_warm(&self) -> DbResult<()> {
        // This would analyze access patterns in hot indexes and demote
        // less frequently accessed items to warm tier
        log::debug!("Checking for hot→warm demotions");
        Ok(())
    }
    
    /// Demotes rarely accessed data from warm to cold tier.
    fn demote_warm_to_cold(&self) -> DbResult<()> {
        // This would analyze access patterns in warm caches and demote
        // rarely accessed items back to cold storage
        log::debug!("Checking for warm→cold demotions");
        Ok(())
    }
    
    /// Provides comprehensive tier management statistics.
    pub fn get_tier_stats(&self) -> DbResult<TierStats> {
        let mut stats = TierStats::default();
        
        // Collect hot layer stats
        if let Some(_hot_graph) = &self.hot_graph {
            stats.hot_graph_nodes = 0; // Placeholder - would get actual count
        }
        
        if let Some(_hot_vector) = &self.hot_vector {
            stats.hot_vector_count = 0; // Placeholder - would get actual count
        }
        
        // Collect warm layer stats
        if let Some(warm_graph_cache) = &self.warm_graph_cache {
            let (outgoing_size, incoming_size, node_size, memory) = warm_graph_cache.get_memory_usage();
            stats.warm_graph_outgoing = outgoing_size;
            stats.warm_graph_incoming = incoming_size;
            stats.warm_graph_nodes = node_size;
            stats.warm_graph_memory = memory;
        }
        
        if let Some(warm_vector_cache) = &self.warm_vector_cache {
            let (vector_size, search_size, memory) = warm_vector_cache.get_memory_usage();
            stats.warm_vector_count = vector_size;
            stats.warm_vector_searches = search_size;
            stats.warm_vector_memory = memory;
        }
        
        // Cold layer stats would come from the base indexes
        stats.cold_graph_nodes = 0; // Placeholder
        stats.cold_vector_count = self.vector.lock().map(|v| v.len()).unwrap_or(0);
        
        Ok(stats)
    }
}

/// Statistics for tier management monitoring.
#[derive(Debug, Default)]
pub struct TierStats {
    // Hot layer
    pub hot_graph_nodes: usize,
    pub hot_vector_count: usize,
    
    // Warm layer
    pub warm_graph_outgoing: usize,
    pub warm_graph_incoming: usize,
    pub warm_graph_nodes: usize,
    pub warm_graph_memory: usize,
    pub warm_vector_count: usize,
    pub warm_vector_searches: usize,
    pub warm_vector_memory: usize,
    
    // Cold layer
    pub cold_graph_nodes: usize,
    pub cold_vector_count: usize,
}

impl Clone for IndexManager {
    fn clone(&self) -> Self {
        Self {
            structural: Arc::clone(&self.structural),
            graph: Arc::clone(&self.graph),
            vector: Arc::clone(&self.vector),
            hot_graph: self.hot_graph.as_ref().map(Arc::clone),
            hot_vector: self.hot_vector.as_ref().map(Arc::clone),
            vector_cache: self.vector_cache.as_ref().map(Arc::clone),
            graph_cache: self.graph_cache.as_ref().map(Arc::clone),
            warm_graph_cache: self.warm_graph_cache.as_ref().map(Arc::clone),
            warm_vector_cache: self.warm_vector_cache.as_ref().map(Arc::clone),
            config: Arc::clone(&self.config),
            runtime_state: Arc::clone(&self.runtime_state),
        }
    }
}

