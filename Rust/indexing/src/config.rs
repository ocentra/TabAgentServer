//! Configuration constants and structures for the indexing crate.
//!
//! Following Rust Architecture Guidelines Rule 13.5:
//! - No magic strings or hard-coded values
//! - Domain-specific values use constants/enums
//! - All configuration is centralized and documented

// ============================================================================
// MDBX Database Configuration
// ============================================================================

/// Maximum number of named databases (DBIs) we plan to use in MDBX environment.
/// 
/// Note: MDBX doesn't require pre-setting this limit (unlike LMDB).
/// This constant is for documentation and capacity planning purposes.
/// MDBX allows opening named databases on-demand.
pub const PLANNED_MAX_DBS: u32 = 10;

/// Database table names
pub mod db_tables {
    /// Structural index table name (property->value->node_ids mapping)
    pub const STRUCTURAL_INDEX: &str = "structural_index";
    
    /// Graph outgoing edges table name (from_node->[edge_ids, to_nodes])
    pub const GRAPH_OUTGOING: &str = "graph_outgoing";
    
    /// Graph incoming edges table name (to_node->[edge_ids, from_nodes])
    pub const GRAPH_INCOMING: &str = "graph_incoming";
}

/// MDBX geometry configuration
pub mod mdbx_geometry {
    /// Default size lower bound (-1 = use MDBX default)
    pub const SIZE_LOWER: isize = -1;
    
    /// Default current size (-1 = use MDBX default)
    pub const SIZE_NOW: isize = -1;
    
    /// Default maximum size (100GB)
    pub const SIZE_UPPER: isize = 100 * 1024 * 1024 * 1024;
    
    /// Default growth step (-1 = use MDBX default)
    pub const GROWTH_STEP: isize = -1;
    
    /// Default shrink threshold (-1 = use MDBX default)
    pub const SHRINK_THRESHOLD: isize = -1;
    
    /// Default page size (-1 = use MDBX default)
    pub const PAGE_SIZE: isize = -1;
}

// ============================================================================
// Vector Index Configuration
// ============================================================================

/// Default vector dimension for embeddings (common for many ML models)
pub const DEFAULT_VECTOR_DIMENSION: usize = 384;

/// Common vector dimensions for different embedding models
pub mod vector_dimensions {
    /// Small embeddings (e.g., sentence-transformers mini)
    pub const SMALL: usize = 384;
    
    /// Medium embeddings (e.g., sentence-transformers base)
    pub const MEDIUM: usize = 768;
    
    /// Large embeddings (e.g., OpenAI ada-002)
    pub const LARGE: usize = 1536;
}

/// HNSW index configuration
#[derive(Debug, Clone, Copy)]
pub struct HnswConfig {
    /// Maximum number of bi-directional links per node (M parameter)
    pub max_connections: usize,
    
    /// Size of the dynamic candidate list during construction (ef_c parameter)
    pub ef_construction: usize,
    
    /// Number of layers in the graph
    pub num_layers: usize,
    
    /// Initial capacity hint for the index
    pub initial_capacity: usize,
}

impl Default for HnswConfig {
    fn default() -> Self {
        Self {
            max_connections: 16,
            ef_construction: 200,
            num_layers: 16,
            initial_capacity: 1000,
        }
    }
}

impl HnswConfig {
    /// Configuration optimized for high-dimensional embeddings (384-768D)
    pub fn for_embeddings() -> Self {
        Self::default()
    }
    
    /// Configuration optimized for low-dimensional vectors (< 100D)
    pub fn for_low_dimension() -> Self {
        Self {
            max_connections: 8,
            ef_construction: 100,
            num_layers: 8,
            initial_capacity: 1000,
        }
    }
    
    /// Configuration optimized for very high-dimensional vectors (> 1000D)
    pub fn for_high_dimension() -> Self {
        Self {
            max_connections: 32,
            ef_construction: 400,
            num_layers: 20,
            initial_capacity: 1000,
        }
    }
}

/// Vector storage file extensions
pub mod vector_file_extensions {
    /// Extension for memory-mapped vector storage
    pub const MMAP_VECTORS: &str = "vectors.mmap";
    
    /// Extension for HNSW index persistence
    pub const HNSW_INDEX: &str = "hnsw.bin";
}

// ============================================================================
// Test Configuration
// ============================================================================

/// Test-specific constants (only used in test code)
pub mod test_constants {
    /// Default edge type for test edges
    pub const DEFAULT_EDGE_TYPE: &str = "TEST_EDGE";
    
    /// Default metadata for test edges
    pub const DEFAULT_METADATA: &str = "{}";
    
    /// Default timestamp for test data (Oct 17, 2023 00:00:00 UTC)
    pub const DEFAULT_TIMESTAMP: i64 = 1697500000000;
}

// ============================================================================
// Tier Threshold Configuration
// ============================================================================

/// Default thresholds for hybrid tier management
pub mod tier_thresholds {
    /// Default access count threshold for promoting to hot tier
    pub const HOT_PROMOTION_THRESHOLD: usize = 10;
    
    /// Default idle time threshold for demoting to cold tier (seconds)
    pub const COLD_DEMOTION_THRESHOLD: u64 = 3600; // 1 hour
    
    /// Default hot cache size (number of entries)
    pub const DEFAULT_HOT_CACHE_SIZE: usize = 10_000;
    
    /// Default warm cache size (number of entries)
    pub const DEFAULT_WARM_CACHE_SIZE: usize = 100_000;
}

