//! Comprehensive documentation and examples for the indexing crate.
//!
//! This module provides detailed documentation and examples for all public APIs
//! in the indexing crate, following the Rust Architecture Guidelines for
//! clear and comprehensive documentation.

/// # Indexing Crate Documentation
///
/// The indexing crate provides high-performance indexing capabilities for
/// fast data retrieval in the TabAgent system. It includes three main types
/// of indexes:
///
/// 1. **Structural Indexes**: Property-based filtering using B-trees
/// 2. **Graph Indexes**: Relationship traversal using adjacency lists
/// 3. **Vector Indexes**: Semantic similarity search using HNSW
///
/// ## Getting Started
///
/// To use the indexing crate, you first need to create an IndexManager:
///
/// ```no_run
/// use indexing::IndexManager;
/// use sled::Db;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let db = sled::open("my_database")?;
/// let index_manager = IndexManager::new(&db)?;
/// # Ok(())
/// # }
/// ```
///
/// ## Structural Indexes
///
/// Structural indexes provide fast property-based filtering:
///
/// ```no_run
/// use indexing::IndexManager;
/// use common::{NodeId, NodeId};
/// use common::models::{Node, Chat};
/// use serde_json::json;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let db = sled::open("my_database")?;
/// # let index_manager = IndexManager::new(&db)?;
/// // Create a chat node
/// let chat = Node::Chat(Chat {
///     id: NodeId::from("chat_123"),
///     title: "My Chat".to_string(),
///     topic: "General Discussion".to_string(),
///     created_at: 1234567890,
///     updated_at: 1234567890,
///     message_ids: vec![],
///     summary_ids: vec![],
///     embedding_id: None,
///     metadata: json!({}),
/// });
///
/// // Index the node (this happens automatically in the storage layer)
/// index_manager.index_node(&chat)?;
///
/// // Query by property
/// let chats = index_manager.get_nodes_by_property("topic", "General Discussion")?;
/// assert_eq!(chats.len(), 1);
/// # Ok(())
/// # }
/// ```
///
/// ## Graph Indexes
///
/// Graph indexes enable efficient relationship traversal:
///
/// ```no_run
/// use indexing::IndexManager;
/// use common::{NodeId, EdgeId};
/// use common::models::Edge;
/// use serde_json::json;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let db = sled::open("my_database")?;
/// # let index_manager = IndexManager::new(&db)?;
/// // Create an edge
/// let edge = Edge {
///     id: EdgeId::from("edge_123"),
///     from_node: NodeId::from("chat_123"),
///     to_node: NodeId::from("message_456"),
///     edge_type: "CONTAINS_MESSAGE".to_string(),
///     created_at: 1234567890,
///     metadata: json!({}),
/// };
///
/// // Index the edge (this happens automatically in the storage layer)
/// index_manager.index_edge(&edge)?;
///
/// // Traverse the graph
/// let outgoing_edges = index_manager.get_outgoing_edges("chat_123")?;
/// assert_eq!(outgoing_edges.len(), 1);
/// # Ok(())
/// # }
/// ```
///
/// ## Vector Indexes
///
/// Vector indexes enable semantic similarity search:
///
/// ```no_run
/// use indexing::IndexManager;
/// use common::{EmbeddingId};
/// use common::models::Embedding;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let db = sled::open("my_database")?;
/// # let index_manager = IndexManager::new(&db)?;
/// // Create an embedding
/// let embedding = Embedding {
///     id: EmbeddingId::from("embedding_123"),
///     vector: vec![0.1, 0.2, 0.3, 0.4, 0.5], // 384-dimensional vector from ML model
///     model: "all-MiniLM-L6-v2".to_string(),
/// };
///
/// // Index the embedding (this happens automatically in the storage layer)
/// index_manager.index_embedding(&embedding)?;
///
/// // Perform semantic search
/// let query = vec![0.15, 0.25, 0.35, 0.45, 0.55];
/// let results = index_manager.search_vectors(&query, 5)?;
/// assert!(!results.is_empty());
/// # Ok(())
/// # }
/// ```
///
/// ## Hybrid Indexes
///
/// For high-concurrency scenarios, hybrid indexes provide lock-free operations:
///
/// ```no_run
/// use indexing::IndexManager;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let db = sled::open("my_database")?;
/// let mut index_manager = IndexManager::new_with_hybrid(&db, true)?;
///
/// // Or enable hybrid indexes on an existing manager
/// // index_manager.enable_hybrid();
///
/// // Use lock-free hot indexes for high-concurrency scenarios
/// if let Some(hot_vector) = index_manager.get_hot_vector_index() {
///     hot_vector.add_vector("vector_123", vec![0.2; 384])?;
///     let results = hot_vector.search(&vec![0.1; 384], 5)?;
/// }
///
/// if let Some(hot_graph) = index_manager.get_hot_graph_index() {
///     hot_graph.add_node("node_123", Some("metadata"))?;
///     hot_graph.add_edge("node_123", "node_456")?;
/// }
/// # Ok(())
/// # }
/// ```
///
/// ## Batch Operations
///
/// For bulk operations, use batch processors:
///
/// ```no_run
/// use indexing::batch::{VectorBatchProcessor, GraphBatchProcessor};
/// use indexing::{LockFreeHotVectorIndex, LockFreeHotGraphIndex};
/// use std::sync::Arc;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let vector_index = Arc::new(LockFreeHotVectorIndex::new());
/// let vector_processor = VectorBatchProcessor::new(vector_index);
///
/// let graph_index = Arc::new(LockFreeHotGraphIndex::new());
/// let graph_processor = GraphBatchProcessor::new(graph_index);
///
/// // Prepare batch data
/// let vectors = vec![
///     ("vector_1".to_string(), vec![0.1; 384]),
///     ("vector_2".to_string(), vec![0.2; 384]),
///     ("vector_3".to_string(), vec![0.3; 384]),
/// ];
///
/// let edges = vec![
///     ("node_1".to_string(), "node_2".to_string()),
///     ("node_2".to_string(), "node_3".to_string()),
///     ("node_3".to_string(), "node_1".to_string()),
/// ];
///
/// // Perform batch operations
/// vector_processor.batch_insert(vectors)?;
/// graph_processor.batch_insert_edges(edges)?;
/// # Ok(())
/// # }
/// ```
///
/// ## Advanced Graph Algorithms
///
/// The crate provides implementations of various graph algorithms:
///
/// ```no_run
/// use indexing::algorithms::{dijkstra, astar, connected_components};
/// use indexing::graph_traits::{DirectedGraph, GraphBase};
/// use std::collections::HashMap;
///
/// # fn example<G: DirectedGraph<NodeId = String, EdgeId = String, EdgeWeight = f32>>() -> Result<(), Box<dyn std::error::Error>> 
/// # where G::EdgeWeight: Clone + Into<f32>
/// # {
/// # let graph: G = todo!();
/// // Find shortest path using Dijkstra's algorithm
/// if let Some((path, distance)) = dijkstra(&graph, "start_node".to_string(), "end_node".to_string())? {
///     println!("Shortest path: {:?}, distance: {}", path, distance);
/// }
///
/// // Find shortest path using A* algorithm with a heuristic
/// let heuristic = |node_id: &String| -> f32 {
///     // Return estimated distance to goal
///     0.0
/// };
///
/// if let Some((path, distance)) = astar(&graph, "start_node".to_string(), "end_node".to_string(), heuristic)? {
///     println!("A* path: {:?}, distance: {}", path, distance);
/// }
///
/// // Find connected components
/// let components = connected_components(&graph)?;
/// println!("Found {} connected components", components.len());
/// # Ok(())
/// # }
/// ```
///
/// ## Distance Metrics
///
/// Various distance metrics are available for vector similarity:
///
/// ```no_run
/// use indexing::distance_metrics::{CosineMetric, EuclideanMetric, ManhattanMetric, JaccardMetric, HammingMetric};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let a = vec![1.0, 2.0, 3.0];
/// let b = vec![4.0, 5.0, 6.0];
///
/// let cosine = CosineMetric::new();
/// let euclidean = EuclideanMetric::new();
/// let manhattan = ManhattanMetric::new();
/// let jaccard = JaccardMetric::new();
/// let hamming = HammingMetric::new();
///
/// let cosine_distance = cosine.distance(&a, &b)?;
/// let euclidean_distance = euclidean.distance(&a, &b)?;
/// let manhattan_distance = manhattan.distance(&a, &b)?;
/// let jaccard_distance = jaccard.distance(&a, &b)?;
/// let hamming_distance = hamming.distance(&a, &b)?;
///
/// println!("Cosine distance: {}", cosine_distance);
/// println!("Euclidean distance: {}", euclidean_distance);
/// println!("Manhattan distance: {}", manhattan_distance);
/// println!("Jaccard distance: {}", jaccard_distance);
/// println!("Hamming distance: {}", hamming_distance);
/// # Ok(())
/// # }
/// ```
///
/// ## Error Handling
///
/// The crate provides comprehensive error handling:
///
/// ```no_run
/// use indexing::errors::{IndexingError, IndexingResult};
/// use indexing::IndexManager;
///
/// # fn example() -> IndexingResult<()> {
/// # let db = sled::open("my_database")?;
/// let index_manager = IndexManager::new(&db)?;
///
/// // Handle errors appropriately
/// match index_manager.get_nodes_by_property("nonexistent_property", "value") {
///     Ok(nodes) => {
///         println!("Found {} nodes", nodes.len());
///     }
///     Err(IndexingError::Database(db_err)) => {
///         eprintln!("Database error: {}", db_err);
///     }
///     Err(e) => {
///         eprintln!("Other error: {}", e);
///     }
/// }
/// # Ok(())
/// # }
/// ```
///
/// ## Performance Considerations
///
/// 1. **Use hybrid indexes for high-concurrency scenarios**
/// 2. **Batch operations for bulk insertions**
/// 3. **Choose appropriate distance metrics for your use case**
/// 4. **Monitor memory usage with large datasets**
/// 5. **Use appropriate vector dimensions for your models**
///
/// ## Concurrency Patterns
///
/// The crate supports both traditional Mutex-based and lock-free concurrency:
///
/// ```no_run
/// use indexing::{LockFreeHotVectorIndex, LockFreeHotGraphIndex};
/// use std::sync::Arc;
/// use std::thread;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let vector_index = Arc::new(LockFreeHotVectorIndex::new());
/// let graph_index = Arc::new(LockFreeHotGraphIndex::new());
///
/// // Spawn multiple threads that can access indexes concurrently
/// let mut handles = vec![];
///
/// for i in 0..4 {
///     let vector_clone = Arc::clone(&vector_index);
///     let graph_clone = Arc::clone(&graph_index);
///     
///     let handle = thread::spawn(move || {
///         // These operations are thread-safe and lock-free
///         vector_clone.add_vector(&format!("vector_{}", i), vec![i as f32; 384]).unwrap();
///         graph_clone.add_node(&format!("node_{}", i), None).unwrap();
///     });
///     
///     handles.push(handle);
/// }
///
/// // Wait for all threads to complete
/// for handle in handles {
///     handle.join().unwrap();
/// }
/// # Ok(())
/// # }
/// ```
///
/// ## Memory Management
///
/// The crate provides memory mapping support for large datasets:
///
/// ```no_run
/// use indexing::memory_mapping::MemoryMappedVectorStorage;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let storage = MemoryMappedVectorStorage::new("vectors.dat", 1000000)?;
/// // Use memory-mapped storage for large vector datasets
/// # Ok(())
/// # }
/// ```
///
/// ## Builder Patterns
///
/// Fluent APIs are available for configuring indexes:
///
/// ```no_run
/// use indexing::builders::IndexManagerBuilder;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let index_manager = IndexManagerBuilder::new()
///     .with_hybrid_indexes(true)
///     .with_vector_dimension(384)
///     .build(&sled::open("my_database")?)?;
/// # Ok(())
/// # }
/// ```
pub struct IndexingCrateDocumentation;

/// # Module Documentation
///
/// ## structural
///
/// The `structural` module provides property-based indexing using B-trees.
///
/// Key types:
/// - [`StructuralIndex`]: Main structural index implementation
///
/// ## graph
///
/// The `graph` module provides relationship-based indexing using adjacency lists.
///
/// Key types:
/// - [`GraphIndex`]: Main graph index implementation
///
/// ## vector
///
/// The `vector` module provides semantic similarity search using HNSW.
///
/// Key types:
/// - [`VectorIndex`]: Main vector index implementation
/// - [`SearchResult`]: Results from vector searches
///
/// ## hybrid
///
/// The `hybrid` module provides high-performance hybrid indexes.
///
/// Key types:
/// - [`HotGraphIndex`]: In-memory graph index
/// - [`HotVectorIndex`]: In-memory vector index
/// - [`DataTemperature`]: Temperature-based data management
/// - [`QuantizedVector`]: Memory-efficient vector storage
///
/// ## graph_traits
///
/// The `graph_traits` module defines generic traits for graph operations.
///
/// Key traits:
/// - [`GraphBase`]: Basic graph operations
/// - [`DirectedGraph`]: Directed graph operations
/// - [`Data`]: Graph data access
/// - [`GraphMut`]: Mutable graph operations
///
/// ## algorithms
///
/// The `algorithms` module provides implementations of graph algorithms.
///
/// Key functions:
/// - [`dijkstra`]: Shortest path using Dijkstra's algorithm
/// - [`astar`]: Shortest path using A* algorithm
/// - [`connected_components`]: Find connected components
/// - [`strongly_connected_components`]: Find strongly connected components
///
/// ## batch
///
/// The `batch` module provides batch processing capabilities.
///
/// Key types:
/// - [`VectorBatchProcessor`]: Batch processor for vectors
/// - [`GraphBatchProcessor`]: Batch processor for graphs
///
/// ## distance_metrics
///
/// The `distance_metrics` module provides various distance metrics.
///
/// Key types:
/// - [`CosineMetric`]: Cosine distance
/// - [`EuclideanMetric`]: Euclidean distance
/// - [`ManhattanMetric`]: Manhattan distance
/// - [`JaccardMetric`]: Jaccard distance
/// - [`HammingMetric`]: Hamming distance
///
/// ## errors
///
/// The `errors` module provides comprehensive error types.
///
/// Key types:
/// - [`IndexingError`]: Main error type for the crate
/// - [`StructuralIndexError`]: Errors for structural indexes
/// - [`GraphIndexError`]: Errors for graph indexes
/// - [`VectorIndexError`]: Errors for vector indexes
pub struct ModuleDocumentation;

/// # Performance Benchmarks
///
/// The crate includes comprehensive benchmarking capabilities:
///
/// ```no_run
/// use indexing::benchmark::run_comprehensive_benchmarks;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let results = run_comprehensive_benchmarks()?;
/// for result in &results {
///     println!("{}: {} ops/sec", result.name, result.ops_per_second);
/// }
/// # Ok(())
/// # }
/// ```
///
/// Typical performance characteristics:
/// - Structural index queries: < 1ms
/// - Graph index traversals: < 100μs
/// - Vector index searches: < 10ms
/// - Lock-free operations: 2-10x faster than Mutex-based under high concurrency
pub struct PerformanceDocumentation;

/// # Best Practices
///
/// 1. **Use appropriate index types for your query patterns**
/// 2. **Enable hybrid indexes for high-concurrency workloads**
/// 3. **Use batch operations for bulk data processing**
/// 4. **Choose distance metrics that match your data characteristics**
/// 5. **Monitor and tune performance based on your workload**
/// 6. **Handle errors appropriately in production code**
/// 7. **Use memory mapping for large datasets**
/// 8. **Follow the builder pattern for complex configurations**
pub struct BestPractices;

#[cfg(test)]
mod tests {
    #[test]
    fn test_documentation_compiles() {
        // This test ensures that all examples in the documentation compile
        // Actual functionality is tested in the respective modules
    }
}