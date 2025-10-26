//! Segment-based architecture for vector storage.
//!
//! This module implements a segment-based storage architecture inspired by Qdrant,
//! where vectors are organized into segments for efficient management, indexing,
//! and querying. Each segment can be independently optimized, searched, and managed.
//!
//! # Design Principles
//!
//! 1. **Isolation**: Segments are independent units that can be managed separately
//! 2. **Scalability**: New segments can be added as data grows
//! 3. **Optimization**: Each segment can be optimized independently
//! 4. **Concurrency**: Multiple segments can be searched in parallel
//! 5. **Persistence**: Segments can be persisted and loaded independently

use common::{DbError, DbResult, EmbeddingId};
use crate::distance_metrics::DistanceMetric;
use crate::payload_index::{Payload, PayloadIndex};
use crate::vector::{SearchResult, VectorIndex};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

/// A segment of the vector index.
///
/// Each segment contains a subset of vectors and their associated metadata.
/// Segments can be searched independently and combined for complete results.
pub struct Segment {
    /// Unique identifier for this segment
    id: String,
    
    /// Path where this segment is stored
    path: PathBuf,
    
    /// The underlying vector index for this segment
    vector_index: VectorIndex,
    
    /// Payload index for this segment
    payload_index: PayloadIndex,
    
    /// Number of vectors in this segment
    vector_count: usize,
    
    /// Maximum number of vectors this segment can hold
    max_vectors: usize,
    
    /// Whether this segment is appendable
    appendable: bool,
    
    /// Whether this segment is optimized
    optimized: bool,
}

impl Segment {
    /// Creates a new segment.
    pub fn new<P: AsRef<Path>>(
        id: String,
        path: P,
        max_vectors: usize,
        appendable: bool,
    ) -> DbResult<Self> {
        let path = path.as_ref().to_path_buf();
        
        // Create the segment directory if it doesn't exist
        std::fs::create_dir_all(&path)?;
        
        // Create the vector index for this segment
        let vector_index_path = path.join("vectors.hnsw");
        let vector_index = VectorIndex::new(vector_index_path)?;
        
        Ok(Self {
            id,
            path,
            vector_index,
            payload_index: PayloadIndex::new(),
            vector_count: 0,
            max_vectors,
            appendable,
            optimized: false,
        })
    }
    
    /// Gets the segment ID.
    pub fn id(&self) -> &str {
        &self.id
    }
    
    /// Gets the number of vectors in this segment.
    pub fn len(&self) -> usize {
        self.vector_count
    }
    
    /// Checks if this segment is empty.
    pub fn is_empty(&self) -> bool {
        self.vector_count == 0
    }
    
    /// Checks if this segment is appendable.
    pub fn is_appendable(&self) -> bool {
        self.appendable
    }
    
    /// Checks if this segment is optimized.
    pub fn is_optimized(&self) -> bool {
        self.optimized
    }
    
    /// Adds a vector to this segment.
    pub fn add_vector(
        &mut self,
        id: &str,
        vector: Vec<f32>,
        payload: Option<Payload>,
    ) -> DbResult<()> {
        if !self.appendable {
            return Err(DbError::InvalidOperation(
                "Segment is not appendable".to_string()
            ));
        }
        
        if self.vector_count >= self.max_vectors {
            return Err(DbError::InvalidOperation(
                "Segment is full".to_string()
            ));
        }
        
        // Add to vector index
        self.vector_index.add_vector(id, vector)?;
        
        // Add payload if provided
        if let Some(payload) = payload {
            self.payload_index.add_payload(EmbeddingId::from(id), payload)?;
        }
        
        self.vector_count += 1;
        Ok(())
    }
    
    /// Removes a vector from this segment.
    pub fn remove_vector(&mut self, id: &str) -> DbResult<bool> {
        let removed = self.vector_index.remove_vector(id)?;
        let payload_removed = self.payload_index.remove_payload(&EmbeddingId::from(id))?;
        
        if removed || payload_removed {
            self.vector_count = self.vector_count.saturating_sub(1);
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    /// Searches for vectors in this segment.
    pub fn search(
        &self,
        query: &[f32],
        k: usize,
        filter: Option<&crate::payload_index::PayloadFilter>,
    ) -> DbResult<Vec<SearchResult>> {
        self.vector_index.search(query, k)
    }
    
    /// Gets the payload for a vector.
    pub fn get_payload(&self, id: &EmbeddingId) -> Option<&Payload> {
        self.payload_index.get_payload(id)
    }
    
    /// Flushes this segment to disk.
    pub fn flush(&mut self) -> DbResult<()> {
        self.vector_index.save()?;
        Ok(())
    }
    
    /// Optimizes this segment.
    pub fn optimize(&mut self) -> DbResult<()> {
        // In a real implementation, this would rebuild the HNSW index
        // and compact the segment for better performance
        self.optimized = true;
        Ok(())
    }
    
    /// Marks this segment as appendable or not.
    pub fn set_appendable(&mut self, appendable: bool) {
        self.appendable = appendable;
    }
}

/// Configuration for segment management.
#[derive(Debug, Clone)]
pub struct SegmentConfig {
    /// Maximum number of vectors per segment
    pub max_vectors_per_segment: usize,
    
    /// Minimum number of vectors required to create a new segment
    pub min_vectors_for_new_segment: usize,
    
    /// Whether to optimize segments automatically
    pub auto_optimize: bool,
    
    /// Path where segments are stored
    pub segments_path: PathBuf,
}

impl Default for SegmentConfig {
    fn default() -> Self {
        Self {
            max_vectors_per_segment: 100_000,
            min_vectors_for_new_segment: 10_000,
            auto_optimize: true,
            segments_path: PathBuf::from("segments"),
        }
    }
}

/// A segment manager that coordinates multiple segments.
///
/// The segment manager handles the creation, deletion, and coordination
/// of multiple segments, providing a unified interface for vector operations.
pub struct SegmentManager {
    /// Configuration for segment management
    config: SegmentConfig,
    
    /// Active segments
    segments: HashMap<String, Arc<RwLock<Segment>>>,
    
    /// Appendable segment (where new vectors are added)
    appendable_segment: Option<String>,
    
    /// Next segment ID
    next_segment_id: usize,
}

impl SegmentManager {
    /// Creates a new segment manager.
    pub fn new(config: SegmentConfig) -> DbResult<Self> {
        // Create the segments directory if it doesn't exist
        std::fs::create_dir_all(&config.segments_path)?;
        
        let mut manager = Self {
            config,
            segments: HashMap::new(),
            appendable_segment: None,
            next_segment_id: 0,
        };
        
        // Create the first appendable segment
        manager.create_appendable_segment()?;
        
        Ok(manager)
    }
    
    /// Creates a new appendable segment.
    fn create_appendable_segment(&mut self) -> DbResult<()> {
        let segment_id = format!("segment_{}", self.next_segment_id);
        self.next_segment_id += 1;
        
        let segment_path = self.config.segments_path.join(&segment_id);
        let segment = Segment::new(
            segment_id.clone(),
            segment_path,
            self.config.max_vectors_per_segment,
            true,
        )?;
        
        // Add the segment to the manager
        self.segments.insert(segment_id.clone(), Arc::new(RwLock::new(segment)));
        self.appendable_segment = Some(segment_id);
        
        Ok(())
    }
    
    /// Gets the current appendable segment, creating a new one if needed.
    fn get_appendable_segment(&mut self) -> DbResult<Arc<RwLock<Segment>>> {
        // Check if we need a new segment
        let need_new_segment = if let Some(ref segment_id) = self.appendable_segment {
            if let Some(segment) = self.segments.get(segment_id) {
                let segment = segment.read().map_err(|_| DbError::Other("Lock poisoned".to_string()))?;
                segment.len() >= self.config.max_vectors_per_segment
            } else {
                true
            }
        } else {
            true
        };
        
        if need_new_segment {
            self.create_appendable_segment()?;
        }
        
        // Return the current appendable segment
        if let Some(ref segment_id) = self.appendable_segment {
            self.segments.get(segment_id)
                .cloned()
                .ok_or_else(|| DbError::Other("Appendable segment not found".to_string()))
        } else {
            Err(DbError::Other("No appendable segment available".to_string()))
        }
    }
    
    /// Adds a vector to the appropriate segment.
    pub fn add_vector(
        &mut self,
        id: &str,
        vector: Vec<f32>,
        payload: Option<Payload>,
    ) -> DbResult<()> {
        // Get the appendable segment
        let segment = self.get_appendable_segment()?;
        let mut segment = segment.write().map_err(|_| DbError::Other("Lock poisoned".to_string()))?;
        
        // Add the vector to the segment
        segment.add_vector(id, vector, payload)?;
        
        Ok(())
    }
    
    /// Removes a vector from any segment.
    pub fn remove_vector(&mut self, id: &str) -> DbResult<bool> {
        // Try to remove from each segment
        for segment in self.segments.values() {
            let mut segment = segment.write().map_err(|_| DbError::Other("Lock poisoned".to_string()))?;
            if segment.remove_vector(id)? {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    /// Searches across all segments.
    pub fn search(
        &self,
        query: &[f32],
        k: usize,
        filter: Option<&crate::payload_index::PayloadFilter>,
    ) -> DbResult<Vec<SearchResult>> {
        let mut all_results = Vec::new();
        
        // Search each segment
        for segment in self.segments.values() {
            let segment = segment.read().map_err(|_| DbError::Other("Lock poisoned".to_string()))?;
            let mut results = segment.search(query, k, filter)?;
            all_results.append(&mut results);
        }
        
        // Sort and limit results
        all_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        all_results.truncate(k);
        
        Ok(all_results)
    }
    
    /// Gets the payload for a vector.
    pub fn get_payload(&self, id: &EmbeddingId) -> DbResult<Option<Payload>> {
        // Try to get payload from each segment
        for segment in self.segments.values() {
            let segment = segment.read().map_err(|_| DbError::Other("Lock poisoned".to_string()))?;
            if let Some(payload) = segment.get_payload(id) {
                return Ok(Some(payload.clone()));
            }
        }
        
        Ok(None)
    }
    
    /// Flushes all segments to disk.
    pub fn flush(&mut self) -> DbResult<()> {
        for segment in self.segments.values() {
            let mut segment = segment.write().map_err(|_| DbError::Other("Lock poisoned".to_string()))?;
            segment.flush()?;
        }
        Ok(())
    }
    
    /// Optimizes all segments.
    pub fn optimize(&mut self) -> DbResult<()> {
        for segment in self.segments.values() {
            let mut segment = segment.write().map_err(|_| DbError::Other("Lock poisoned".to_string()))?;
            segment.optimize()?;
        }
        Ok(())
    }
    
    /// Gets statistics about all segments.
    pub fn get_statistics(&self) -> SegmentStatistics {
        let mut stats = SegmentStatistics::default();
        
        for segment in self.segments.values() {
            let segment = match segment.read() {
                Ok(s) => s,
                Err(_) => continue,
            };
            
            stats.total_vectors += segment.len();
            stats.segment_count += 1;
            
            if segment.is_appendable() {
                stats.appendable_segments += 1;
            }
            
            if segment.is_optimized() {
                stats.optimized_segments += 1;
            }
        }
        
        stats
    }
}

/// Statistics about segments.
#[derive(Debug, Clone, Default)]
pub struct SegmentStatistics {
    /// Total number of vectors across all segments
    pub total_vectors: usize,
    
    /// Number of segments
    pub segment_count: usize,
    
    /// Number of appendable segments
    pub appendable_segments: usize,
    
    /// Number of optimized segments
    pub optimized_segments: usize,
}

/// A segment-based vector index that combines multiple segments.
pub struct SegmentBasedVectorIndex {
    /// The segment manager
    segment_manager: SegmentManager,
    
    /// Default distance metric
    distance_metric: Box<dyn DistanceMetric>,
}

impl SegmentBasedVectorIndex {
    /// Creates a new segment-based vector index.
    pub fn new<P: AsRef<Path>>(
        segments_path: P,
        distance_metric: Box<dyn DistanceMetric>,
    ) -> DbResult<Self> {
        let config = SegmentConfig {
            segments_path: segments_path.as_ref().to_path_buf(),
            ..Default::default()
        };
        
        let segment_manager = SegmentManager::new(config)?;
        
        Ok(Self {
            segment_manager,
            distance_metric,
        })
    }
    
    /// Adds a vector to the index.
    pub fn add_vector(
        &mut self,
        id: &str,
        vector: Vec<f32>,
        payload: Option<Payload>,
    ) -> DbResult<()> {
        self.segment_manager.add_vector(id, vector, payload)
    }
    
    /// Removes a vector from the index.
    pub fn remove_vector(&mut self, id: &str) -> DbResult<bool> {
        self.segment_manager.remove_vector(id)
    }
    
    /// Searches for the k nearest neighbors of a query vector.
    pub fn search(
        &self,
        query: &[f32],
        k: usize,
        filter: Option<&crate::payload_index::PayloadFilter>,
    ) -> DbResult<Vec<SearchResult>> {
        self.segment_manager.search(query, k, filter)
    }
    
    /// Gets the payload for a vector.
    pub fn get_payload(&self, id: &EmbeddingId) -> DbResult<Option<Payload>> {
        self.segment_manager.get_payload(id)
    }
    
    /// Flushes the index to disk.
    pub fn flush(&mut self) -> DbResult<()> {
        self.segment_manager.flush()
    }
    
    /// Optimizes the index.
    pub fn optimize(&mut self) -> DbResult<()> {
        self.segment_manager.optimize()
    }
    
    /// Gets statistics about the index.
    pub fn get_statistics(&self) -> SegmentStatistics {
        self.segment_manager.get_statistics()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_segment_creation() {
        let temp_dir = TempDir::new().unwrap();
        let segment_path = temp_dir.path().join("test_segment");
        
        let segment = Segment::new(
            "test_segment".to_string(),
            &segment_path,
            1000,
            true,
        );
        
        assert!(segment.is_ok());
        let segment = segment.unwrap();
        assert_eq!(segment.id(), "test_segment");
        assert_eq!(segment.len(), 0);
        assert!(segment.is_appendable());
    }
    
    #[test]
    fn test_segment_add_remove_vector() {
        let temp_dir = TempDir::new().unwrap();
        let segment_path = temp_dir.path().join("test_segment");
        
        let mut segment = Segment::new(
            "test_segment".to_string(),
            &segment_path,
            1000,
            true,
        ).unwrap();
        
        // Add a vector
        assert!(segment.add_vector("vector1", vec![1.0, 0.0, 0.0], None).is_ok());
        assert_eq!(segment.len(), 1);
        
        // Remove a vector
        assert!(segment.remove_vector("vector1").unwrap());
        assert_eq!(segment.len(), 0);
    }
    
    #[test]
    fn test_segment_manager() {
        let temp_dir = TempDir::new().unwrap();
        let segments_path = temp_dir.path().join("segments");
        
        let config = SegmentConfig {
            segments_path,
            max_vectors_per_segment: 2,
            ..Default::default()
        };
        
        let mut manager = SegmentManager::new(config).unwrap();
        
        // Add vectors
        assert!(manager.add_vector("vector1", vec![1.0, 0.0, 0.0], None).is_ok());
        assert!(manager.add_vector("vector2", vec![0.0, 1.0, 0.0], None).is_ok());
        assert!(manager.add_vector("vector3", vec![0.0, 0.0, 1.0], None).is_ok());
        
        // Check that we have multiple segments
        let stats = manager.get_statistics();
        assert_eq!(stats.total_vectors, 3);
        assert!(stats.segment_count >= 2);
    }
    
    #[test]
    fn test_segment_based_vector_index() {
        let temp_dir = TempDir::new().unwrap();
        let segments_path = temp_dir.path().join("segments");
        
        let mut index = SegmentBasedVectorIndex::new(
            &segments_path,
            Box::new(crate::distance_metrics::CosineMetric::new()),
        ).unwrap();
        
        // Add vectors
        assert!(index.add_vector("vector1", vec![1.0, 0.0, 0.0], None).is_ok());
        assert!(index.add_vector("vector2", vec![0.9, 0.1, 0.0], None).is_ok());
        
        // Search
        let results = index.search(&[1.0, 0.0, 0.0], 2, None).unwrap();
        assert_eq!(results.len(), 2);
        
        // Check statistics
        let stats = index.get_statistics();
        assert_eq!(stats.total_vectors, 2);
    }
}