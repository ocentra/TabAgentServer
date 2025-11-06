//! Lock-free implementation of HotVectorIndex for concurrent access.
//!
//! This module provides a lock-free version of the HotVectorIndex that uses
//! atomic operations and lock-free data structures for improved performance
//! in highly concurrent scenarios.
//!
//! # Concurrency Model
//!
//! The lock-free implementation uses the following techniques:
//!
//! - **Atomic counters**: For statistics tracking without locks
//! - **Lock-free hash maps**: For concurrent vector storage using compare-and-swap operations
//! - **Memory pools**: For efficient allocation without contention
//!
//! # Performance Characteristics
//!
//! - **Insertions**: O(1) average case, lock-free
//! - **Searches**: O(n) where n is the number of vectors (can be optimized with indexing)
//! - **Memory usage**: Higher than mutex-based due to lock-free data structures
//!
//! # Example
//!
//! ```no_run
//! # use indexing::lock_free_hot_vector::LockFreeHotVectorIndex;
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let index = LockFreeHotVectorIndex::new();
//!
//! // Add vectors from any source
//! let vector1 = vec![0.1, 0.2, 0.3, 0.4];
//! index.add_vector("vec1", vector1)?;
//!
//! let vector2 = vec![0.2, 0.3, 0.4, 0.5];
//! index.add_vector("vec2", vector2)?;
//!
//! // Search for similar vectors
//! let query = vec![0.15, 0.25, 0.35, 0.45];
//! let results = index.search(&query, 10)?;
//!
//! // Get statistics
//! let stats = index.get_stats();
//! println!("Vector count: {}", stats.vector_count);
//! println!("Query count: {}", stats.query_count);
//! # Ok(())
//! # }
//! ```

use crate::hybrid::{QuantizedVector, QuantizationType};
use crate::lock_free::lock_free_common::{LockFreeAccessTracker, LockFreeStats};
use common::DbResult;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::Instant;

/// Lock-free implementation of HotVectorIndex for concurrent access.
///
/// This implementation uses lock-free data structures and atomic operations
/// to provide high-performance concurrent access without traditional locking.
pub struct LockFreeHotVectorIndex {
    /// Quantized vectors mapped by ID using DashMap (concurrent hash map)
    vectors: Arc<DashMap<String, QuantizedVector>>,
    
    /// Access tracking for temperature management using lock-free access trackers
    access_trackers: Arc<DashMap<String, LockFreeAccessTracker>>,
    
    /// Default quantization type for new vectors
    default_quantization: QuantizationType,
    
    /// Performance monitoring statistics using lock-free counters
    stats: Arc<LockFreeStats>,
}

impl LockFreeHotVectorIndex {
    /// Creates a new LockFreeHotVectorIndex with scalar quantization by default.
    pub fn new() -> Self {
        Self {
            vectors: Arc::new(DashMap::new()),
            access_trackers: Arc::new(DashMap::new()),
            default_quantization: QuantizationType::Scalar,
            stats: Arc::new(LockFreeStats::new()),
        }
    }
    
    /// Creates a new LockFreeHotVectorIndex with a specific quantization type.
    pub fn with_quantization(quantization_type: QuantizationType) -> Self {
        Self {
            vectors: Arc::new(DashMap::new()),
            access_trackers: Arc::new(DashMap::new()),
            default_quantization: quantization_type,
            stats: Arc::new(LockFreeStats::new()),
        }
    }
    
    /// Adds a vector to the index using the default quantization method.
    pub fn add_vector(&self, id: &str, vector: Vec<f32>) -> DbResult<()> {
        let quantized = match self.default_quantization {
            QuantizationType::Scalar => QuantizedVector::new(&vector),
            QuantizationType::Product { subvector_size } => {
                QuantizedVector::new_product_quantized(&vector, subvector_size)
            }
        };
        
        self.vectors.insert(id.to_string(), quantized);
        self.access_trackers.insert(id.to_string(), LockFreeAccessTracker::new());
        self.stats.increment_vector_count();
        Ok(())
    }
    
    /// Adds a vector to the index with scalar quantization.
    pub fn add_vector_scalar(&self, id: &str, vector: Vec<f32>) -> DbResult<()> {
        let quantized = QuantizedVector::new(&vector);
        self.vectors.insert(id.to_string(), quantized);
        self.access_trackers.insert(id.to_string(), LockFreeAccessTracker::new());
        self.stats.increment_vector_count();
        Ok(())
    }
    
    /// Adds a vector to the index with product quantization.
    pub fn add_vector_product(&self, id: &str, vector: Vec<f32>, subvector_size: usize) -> DbResult<()> {
        let quantized = QuantizedVector::new_product_quantized(&vector, subvector_size);
        self.vectors.insert(id.to_string(), quantized);
        self.access_trackers.insert(id.to_string(), LockFreeAccessTracker::new());
        self.stats.increment_vector_count();
        Ok(())
    }
    
    /// Removes a vector from the index.
    pub fn remove_vector(&self, id: &str) -> DbResult<bool> {
        let existed = self.vectors.remove(&id.to_string()).is_some();
        self.access_trackers.remove(&id.to_string());
        
        if existed {
            self.stats.decrement_vector_count();
        }
        
        Ok(existed)
    }
    
    /// Searches for the k nearest neighbors of a query vector.
    pub fn search(&self, query: &[f32], k: usize) -> DbResult<Vec<(String, f32)>> {
        let start_time = Instant::now();
        
        let query_quantized = match self.default_quantization {
            QuantizationType::Scalar => QuantizedVector::new(query),
            QuantizationType::Product { subvector_size } => {
                QuantizedVector::new_product_quantized(query, subvector_size)
            }
        };
        
        // Collect all vector IDs and their quantized vectors
        let mut similarities: Vec<(String, f32)> = Vec::new();
        
        // Iterate through all vectors in the index using DashMap's iterator
        for entry in self.vectors.iter() {
            let id = entry.key().clone();
            let vector = entry.value();
            let similarity = query_quantized.cosine_similarity(vector);
            similarities.push((id, similarity));
            self.stats.increment_similarity_computations();
        }
        
        // Sort by similarity (highest first) and take top k
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        similarities.truncate(k);
        
        self.stats.increment_query_count();
        
        let query_time = start_time.elapsed().as_micros() as u64;
        self.stats.add_query_time(query_time);
        
        Ok(similarities)
    }
    
    /// Gets the number of vectors in the index.
    pub fn len(&self) -> usize {
        self.stats.vector_count.load(std::sync::atomic::Ordering::Relaxed)
    }
    
    /// Checks if the index is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    
    /// Gets statistics about the index.
    pub fn get_stats(&self) -> LockFreeHotVectorStats {
        LockFreeHotVectorStats {
            vector_count: self.stats.vector_count.load(std::sync::atomic::Ordering::Relaxed),
            query_count: self.stats.query_count.load(std::sync::atomic::Ordering::Relaxed),
            total_query_time_micros: self.stats.total_query_time_micros.load(std::sync::atomic::Ordering::Relaxed) as u64,
            promotions: self.stats.promotions.load(std::sync::atomic::Ordering::Relaxed),
            demotions: self.stats.demotions.load(std::sync::atomic::Ordering::Relaxed),
            similarity_computations: self.stats.similarity_computations.load(std::sync::atomic::Ordering::Relaxed),
        }
    }
}

impl Default for LockFreeHotVectorIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for the lock-free HotVectorIndex.
#[derive(Debug, Clone)]
pub struct LockFreeHotVectorStats {
    /// Number of vectors in the index
    pub vector_count: usize,
    
    /// Total number of queries performed
    pub query_count: usize,
    
    /// Total time spent on queries (in microseconds)
    pub total_query_time_micros: u64,
    
    /// Number of tier promotions
    pub promotions: usize,
    
    /// Number of tier demotions
    pub demotions: usize,
    
    /// Total number of similarity computations
    pub similarity_computations: usize,
}
