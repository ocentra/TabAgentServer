//! Builder patterns for fluent APIs in the indexing system.
//!
//! This module provides builder patterns for creating complex objects
//! with fluent APIs, following the Rust Architecture Guidelines for
//! safety, performance, and clarity.

use common::{DbError, DbResult};
use crate::distance_metrics::{DistanceMetric, CosineMetric};
use crate::payload_index::{Payload, PayloadFieldValue};
use crate::vector::VectorIndex;
use crate::segment::{SegmentConfig, SegmentBasedVectorIndex};
use crate::persistence::EnhancedVectorIndex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use libmdbx::{Database, NoWriteMap};

/// Builder for creating vector indexes with custom configurations.
pub struct VectorIndexBuilder {
    /// Path for persistence
    persist_path: Option<PathBuf>,
    
    /// Maximum number of connections per layer (M parameter)
    max_connections: usize,
    
    /// Size of dynamic candidate list during construction (ef_c parameter)
    ef_construction: usize,
    
    /// Number of layers in the HNSW graph
    num_layers: usize,
    
    /// Initial capacity of the index
    initial_capacity: usize,
    
    /// Distance metric to use
    distance_metric: Box<dyn DistanceMetric>,
}

impl VectorIndexBuilder {
    /// Creates a new vector index builder with default values.
    pub fn new() -> Self {
        Self {
            persist_path: None,
            max_connections: 16,
            ef_construction: 200,
            num_layers: 16,
            initial_capacity: 1000,
            distance_metric: Box::new(CosineMetric::new()),
        }
    }
    
    /// Sets the persistence path for the vector index.
    pub fn persist_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.persist_path = Some(path.as_ref().to_path_buf());
        self
    }
    
    /// Sets the maximum number of connections per layer (M parameter).
    pub fn max_connections(mut self, max_connections: usize) -> Self {
        self.max_connections = max_connections;
        self
    }
    
    /// Sets the size of dynamic candidate list during construction (ef_c parameter).
    pub fn ef_construction(mut self, ef_construction: usize) -> Self {
        self.ef_construction = ef_construction;
        self
    }
    
    /// Sets the number of layers in the HNSW graph.
    pub fn num_layers(mut self, num_layers: usize) -> Self {
        self.num_layers = num_layers;
        self
    }
    
    /// Sets the initial capacity of the index.
    pub fn initial_capacity(mut self, initial_capacity: usize) -> Self {
        self.initial_capacity = initial_capacity;
        self
    }
    
    /// Sets the distance metric to use.
    pub fn distance_metric(mut self, distance_metric: Box<dyn DistanceMetric>) -> Self {
        self.distance_metric = distance_metric;
        self
    }
    
    /// Builds the vector index.
    pub fn build(self) -> DbResult<VectorIndex> {
        let persist_path = self.persist_path
            .ok_or_else(|| DbError::InvalidOperation("Persistence path is required".to_string()))?;
        
        // Create the HNSW index with the specified parameters
        // Note: This is a simplified implementation since we can't directly configure
        // the HNSW parameters in the current hnsw_rs crate
        VectorIndex::new(persist_path)
    }
}

impl Default for VectorIndexBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating segment-based vector indexes.
pub struct SegmentIndexBuilder {
    /// Path where segments are stored
    segments_path: Option<PathBuf>,
    
    /// Segment configuration
    segment_config: SegmentConfig,
    
    /// Distance metric to use
    distance_metric: Box<dyn DistanceMetric>,
}

impl SegmentIndexBuilder {
    /// Creates a new segment index builder with default values.
    pub fn new() -> Self {
        Self {
            segments_path: None,
            segment_config: SegmentConfig::default(),
            distance_metric: Box::new(CosineMetric::new()),
        }
    }
    
    /// Sets the path where segments are stored.
    pub fn segments_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.segments_path = Some(path.as_ref().to_path_buf());
        self
    }
    
    /// Sets the maximum number of vectors per segment.
    pub fn max_vectors_per_segment(mut self, max_vectors: usize) -> Self {
        self.segment_config.max_vectors_per_segment = max_vectors;
        self
    }
    
    /// Sets the minimum number of vectors required to create a new segment.
    pub fn min_vectors_for_new_segment(mut self, min_vectors: usize) -> Self {
        self.segment_config.min_vectors_for_new_segment = min_vectors;
        self
    }
    
    /// Sets whether to optimize segments automatically.
    pub fn auto_optimize(mut self, auto_optimize: bool) -> Self {
        self.segment_config.auto_optimize = auto_optimize;
        self
    }
    
    /// Sets the distance metric to use.
    pub fn distance_metric(mut self, distance_metric: Box<dyn DistanceMetric>) -> Self {
        self.distance_metric = distance_metric;
        self
    }
    
    /// Builds the segment-based vector index.
    pub fn build(self) -> DbResult<SegmentBasedVectorIndex> {
        let segments_path = self.segments_path
            .ok_or_else(|| DbError::InvalidOperation("Segments path is required".to_string()))?;
        
        SegmentBasedVectorIndex::new(segments_path, self.distance_metric)
    }
}

impl Default for SegmentIndexBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating enhanced vector indexes with persistence.
pub struct EnhancedIndexBuilder {
    /// Path where segments are stored
    segments_path: Option<PathBuf>,
    
    /// Segment configuration
    segment_config: SegmentConfig,
    
    /// Distance metric to use
    distance_metric: Box<dyn DistanceMetric>,
}

impl EnhancedIndexBuilder {
    /// Creates a new enhanced index builder with default values.
    pub fn new() -> Self {
        Self {
            segments_path: None,
            segment_config: SegmentConfig::default(),
            distance_metric: Box::new(CosineMetric::new()),
        }
    }
    
    /// Sets the path where segments are stored.
    pub fn segments_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.segments_path = Some(path.as_ref().to_path_buf());
        self
    }
    
    /// Sets the maximum number of vectors per segment.
    pub fn max_vectors_per_segment(mut self, max_vectors: usize) -> Self {
        self.segment_config.max_vectors_per_segment = max_vectors;
        self
    }
    
    /// Sets the minimum number of vectors required to create a new segment.
    pub fn min_vectors_for_new_segment(mut self, min_vectors: usize) -> Self {
        self.segment_config.min_vectors_for_new_segment = min_vectors;
        self
    }
    
    /// Sets whether to optimize segments automatically.
    pub fn auto_optimize(mut self, auto_optimize: bool) -> Self {
        self.segment_config.auto_optimize = auto_optimize;
        self
    }
    
    /// Sets the distance metric to use.
    pub fn distance_metric(mut self, distance_metric: Box<dyn DistanceMetric>) -> Self {
        self.distance_metric = distance_metric;
        self
    }
    
    /// Builds the enhanced vector index.
    pub fn build(self) -> DbResult<EnhancedVectorIndex> {
        let segments_path = self.segments_path
            .ok_or_else(|| DbError::InvalidOperation("Segments path is required".to_string()))?;
        
        EnhancedVectorIndex::new(segments_path, self.distance_metric)
    }
}

impl Default for EnhancedIndexBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating payload filters.
pub struct PayloadFilterBuilder {
    /// Conditions that must all be satisfied (AND)
    must: Vec<crate::payload_index::PayloadCondition>,
    
    /// Conditions where at least one must be satisfied (OR)
    should: Vec<crate::payload_index::PayloadCondition>,
    
    /// Conditions that must not be satisfied (NOT)
    must_not: Vec<crate::payload_index::PayloadCondition>,
}

impl PayloadFilterBuilder {
    /// Creates a new payload filter builder.
    pub fn new() -> Self {
        Self {
            must: Vec::new(),
            should: Vec::new(),
            must_not: Vec::new(),
        }
    }
    
    /// Adds a condition that must be satisfied.
    pub fn must_match_string(mut self, field: &str, value: &str) -> Self {
        self.must.push(crate::payload_index::PayloadCondition::Match {
            value: PayloadFieldValue::String(value.to_string()),
        });
        self
    }
    
    /// Adds a condition that must be satisfied.
    pub fn must_match_integer(mut self, field: &str, value: i64) -> Self {
        self.must.push(crate::payload_index::PayloadCondition::Match {
            value: PayloadFieldValue::Integer(value),
        });
        self
    }
    
    /// Adds a condition that must be satisfied.
    pub fn must_match_boolean(mut self, field: &str, value: bool) -> Self {
        self.must.push(crate::payload_index::PayloadCondition::Match {
            value: PayloadFieldValue::Boolean(value),
        });
        self
    }
    
    /// Adds a range condition that must be satisfied.
    pub fn must_range(mut self, field: &str, from: Option<f64>, to: Option<f64>) -> Self {
        let from = from.map(|f| ordered_float::OrderedFloat(f));
        let to = to.map(|t| ordered_float::OrderedFloat(t));
        self.must.push(crate::payload_index::PayloadCondition::Range { from, to });
        self
    }
    
    /// Adds a condition where at least one must be satisfied.
    pub fn should_match_string(mut self, field: &str, value: &str) -> Self {
        self.should.push(crate::payload_index::PayloadCondition::Match {
            value: PayloadFieldValue::String(value.to_string()),
        });
        self
    }
    
    /// Adds a condition that must not be satisfied.
    pub fn must_not_match_string(mut self, field: &str, value: &str) -> Self {
        self.must_not.push(crate::payload_index::PayloadCondition::Match {
            value: PayloadFieldValue::String(value.to_string()),
        });
        self
    }
    
    /// Builds the payload filter.
    pub fn build(self) -> crate::payload_index::PayloadFilter {
        crate::payload_index::PayloadFilter {
            must: self.must,
            should: self.should,
            must_not: self.must_not,
        }
    }
}

impl Default for PayloadFilterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating payloads.
pub struct PayloadBuilder {
    /// Fields in the payload
    fields: HashMap<String, PayloadFieldValue>,
}

impl PayloadBuilder {
    /// Creates a new payload builder.
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }
    
    /// Adds a string field to the payload.
    pub fn add_string_field(mut self, name: &str, value: &str) -> Self {
        self.fields.insert(name.to_string(), PayloadFieldValue::String(value.to_string()));
        self
    }
    
    /// Adds an integer field to the payload.
    pub fn add_integer_field(mut self, name: &str, value: i64) -> Self {
        self.fields.insert(name.to_string(), PayloadFieldValue::Integer(value));
        self
    }
    
    /// Adds a float field to the payload.
    pub fn add_float_field(mut self, name: &str, value: f64) -> Self {
        self.fields.insert(name.to_string(), PayloadFieldValue::Float(ordered_float::OrderedFloat(value)));
        self
    }
    
    /// Adds a boolean field to the payload.
    pub fn add_boolean_field(mut self, name: &str, value: bool) -> Self {
        self.fields.insert(name.to_string(), PayloadFieldValue::Boolean(value));
        self
    }
    
    /// Adds a list field to the payload.
    pub fn add_list_field(mut self, name: &str, values: Vec<PayloadFieldValue>) -> Self {
        self.fields.insert(name.to_string(), PayloadFieldValue::List(values));
        self
    }
    
    /// Builds the payload.
    pub fn build(self) -> Payload {
        let mut payload = Payload::new();
        for (name, value) in self.fields {
            payload.add_field(name, value);
        }
        payload
    }
}

impl Default for PayloadBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating search queries.
pub struct SearchQueryBuilder {
    /// Query vector
    query_vector: Option<Vec<f32>>,
    
    /// Number of results to return
    limit: usize,
    
    /// Payload filter
    filter: Option<crate::payload_index::PayloadFilter>,
    
    /// Whether to include payloads in results
    include_payload: bool,
    
    /// Whether to include vectors in results
    include_vectors: bool,
}

impl SearchQueryBuilder {
    /// Creates a new search query builder.
    pub fn new() -> Self {
        Self {
            query_vector: None,
            limit: 10,
            filter: None,
            include_payload: false,
            include_vectors: false,
        }
    }
    
    /// Sets the query vector.
    pub fn query_vector(mut self, vector: Vec<f32>) -> Self {
        self.query_vector = Some(vector);
        self
    }
    
    /// Sets the number of results to return.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }
    
    /// Sets the payload filter.
    pub fn filter(mut self, filter: crate::payload_index::PayloadFilter) -> Self {
        self.filter = Some(filter);
        self
    }
    
    /// Sets whether to include payloads in results.
    pub fn include_payload(mut self, include: bool) -> Self {
        self.include_payload = include;
        self
    }
    
    /// Sets whether to include vectors in results.
    pub fn include_vectors(mut self, include: bool) -> Self {
        self.include_vectors = include;
        self
    }
    
    /// Builds the search query.
    /// 
    /// Returns an error if the query vector is not set.
    pub fn build(self) -> DbResult<SearchQuery> {
        let query_vector = self.query_vector
            .ok_or_else(|| DbError::InvalidOperation("Query vector is required".to_string()))?;
        
        Ok(SearchQuery {
            query_vector,
            limit: self.limit,
            filter: self.filter,
            include_payload: self.include_payload,
            include_vectors: self.include_vectors,
        })
    }
}

impl Default for SearchQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// A search query that can be executed against a vector index.
pub struct SearchQuery {
    /// Query vector
    pub query_vector: Vec<f32>,
    
    /// Number of results to return
    pub limit: usize,
    
    /// Payload filter
    pub filter: Option<crate::payload_index::PayloadFilter>,
    
    /// Whether to include payloads in results
    pub include_payload: bool,
    
    /// Whether to include vectors in results
    pub include_vectors: bool,
}

/// Builder for creating graph indexes.
pub struct GraphIndexBuilder {
    /// Path for outgoing edges tree
    outgoing_tree_path: Option<String>,
    
    /// Path for incoming edges tree
    incoming_tree_path: Option<String>,
    
    /// Whether to enable hybrid indexes
    with_hybrid: bool,
}

impl GraphIndexBuilder {
    /// Creates a new graph index builder.
    pub fn new() -> Self {
        Self {
            outgoing_tree_path: None,
            incoming_tree_path: None,
            with_hybrid: false,
        }
    }
    
    /// Sets the path for the outgoing edges tree.
    pub fn outgoing_tree_path(mut self, path: String) -> Self {
        self.outgoing_tree_path = Some(path);
        self
    }
    
    /// Sets the path for the incoming edges tree.
    pub fn incoming_tree_path(mut self, path: String) -> Self {
        self.incoming_tree_path = Some(path);
        self
    }
    
    /// Sets whether to enable hybrid indexes.
    pub fn with_hybrid(mut self, enable: bool) -> Self {
        self.with_hybrid = enable;
        self
    }
    
    /// Builds the graph index.
    pub fn build(self, db: Arc<Database<NoWriteMap>>) -> DbResult<crate::graph::GraphIndex> {
        Ok(crate::graph::GraphIndex::new(
            db,
            "graph_outgoing".to_string(),
            "graph_incoming".to_string()
        ))
    }
}

impl Default for GraphIndexBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_vector_index_builder() {
        let temp_dir = TempDir::new().unwrap();
        let persist_path = temp_dir.path().join("test_index.hnsw");
        
        let builder = VectorIndexBuilder::new()
            .persist_path(&persist_path)
            .max_connections(32)
            .ef_construction(100)
            .num_layers(8)
            .initial_capacity(500);
        
        let index = builder.build();
        assert!(index.is_ok());
    }
    
    #[test]
    fn test_segment_index_builder() {
        let temp_dir = TempDir::new().unwrap();
        let segments_path = temp_dir.path().join("segments");
        
        let builder = SegmentIndexBuilder::new()
            .segments_path(&segments_path)
            .max_vectors_per_segment(5000)
            .min_vectors_for_new_segment(1000)
            .auto_optimize(true);
        
        let index = builder.build();
        assert!(index.is_ok());
    }
    
    #[test]
    fn test_enhanced_index_builder() {
        let temp_dir = TempDir::new().unwrap();
        let segments_path = temp_dir.path().join("segments");
        
        let builder = EnhancedIndexBuilder::new()
            .segments_path(&segments_path)
            .max_vectors_per_segment(5000)
            .min_vectors_for_new_segment(1000)
            .auto_optimize(true);
        
        let index = builder.build();
        assert!(index.is_ok());
    }
    
    #[test]
    fn test_payload_filter_builder() {
        let filter = PayloadFilterBuilder::new()
            .must_match_string("category", "test")
            .must_range("score", Some(0.5), Some(1.0))
            .should_match_string("tag", "important")
            .must_not_match_string("status", "deleted")
            .build();
        
        assert_eq!(filter.must.len(), 2);
        assert_eq!(filter.should.len(), 1);
        assert_eq!(filter.must_not.len(), 1);
    }
    
    #[test]
    fn test_payload_builder() {
        let payload = PayloadBuilder::new()
            .add_string_field("name", "test")
            .add_integer_field("age", 25)
            .add_boolean_field("active", true)
            .build();
        
        assert_eq!(payload.fields.len(), 3);
        assert!(payload.fields.contains_key("name"));
        assert!(payload.fields.contains_key("age"));
        assert!(payload.fields.contains_key("active"));
    }
    
    #[test]
    fn test_search_query_builder() {
        let query = SearchQueryBuilder::new()
            .query_vector(vec![1.0, 0.0, 0.0])
            .limit(5)
            .include_payload(true)
            .include_vectors(false)
            .build();
        
        assert!(query.is_ok());
        let query = query.unwrap();
        assert_eq!(query.query_vector, vec![1.0, 0.0, 0.0]);
        assert_eq!(query.limit, 5);
        assert_eq!(query.include_payload, true);
        assert_eq!(query.include_vectors, false);
    }
    
    #[test]
    fn test_graph_index_builder() {
        // This test would require a sled database instance
        // For now, we'll just test that the builder can be created
        let builder = GraphIndexBuilder::new()
            .outgoing_tree_path("outgoing".to_string())
            .incoming_tree_path("incoming".to_string())
            .with_hybrid(true);
        
        assert_eq!(builder.outgoing_tree_path, Some("outgoing".to_string()));
        assert_eq!(builder.incoming_tree_path, Some("incoming".to_string()));
        assert_eq!(builder.with_hybrid, true);
    }
}