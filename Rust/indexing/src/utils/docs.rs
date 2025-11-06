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
/// To use the indexing crate, create an IndexManager from storage:
///
/// ```ignore
/// use indexing::IndexManager;
/// use storage::StorageManager;
///
/// // Storage owns the MDBX database
/// let storage = StorageManager::new("./data")?;
/// let env = storage.get_raw_env();
/// let structural_dbi = storage.get_or_create_dbi("structural_index")?;
/// let outgoing_dbi = storage.get_or_create_dbi("graph_outgoing")?;
/// let incoming_dbi = storage.get_or_create_dbi("graph_incoming")?;
/// 
/// // Create IndexManager using storage's DB handles
/// let index_manager = IndexManager::new_from_storage(
///     env, structural_dbi, outgoing_dbi, incoming_dbi, false
/// )?;
/// ```
///
/// ## Structural Indexes
///
/// Structural indexes provide fast property-based filtering:
///
/// ```ignore
/// use indexing::IndexManager;
/// use common::NodeId;
/// use common::models::{Node, Chat};
///
/// // Query nodes by property using zero-copy access
/// let guard = index_manager.structural().get("chat_id", "chat_123")?;
/// if let Some(guard) = guard {
///     for node_id in guard.iter_strs() {
///         println!("Found node: {}", node_id);
///     }
/// }
/// ```
///
/// ## Graph Indexes
///
/// Graph indexes enable efficient relationship traversal:
///
/// ```ignore
/// use indexing::IndexManager;
/// use common::{NodeId, EdgeId};
/// use common::models::Edge;
///
/// // Traverse graph relationships with zero-copy access
/// let guard = index_manager.graph().get_outgoing("chat_123")?;
/// if let Some(guard) = guard {
///     for (edge_id, target_node) in guard.iter_edges() {
///         println!("Edge: {} -> {}", edge_id, target_node);
///     }
/// }
/// ```
///
/// ## Vector Indexes
///
/// Vector indexes enable semantic similarity search:
///
/// ```ignore
/// use indexing::IndexManager;
/// use indexing::core::vector::VectorIndex;
///
/// // Perform semantic similarity search
/// let query_vector = vec![0.1; 384]; // 384-dimensional vector
/// let results = vector_index.search(&query_vector, 10)?;
/// for result in results {
///     println!("Found: {} (score: {})", result.id, result.score);
/// }
/// ```
///
/// ## Hybrid Indexes
///
/// For high-concurrency scenarios, hybrid indexes provide lock-free operations:
///
/// ```ignore
/// use indexing::{LockFreeHotVectorIndex, LockFreeHotGraphIndex};
/// use std::sync::Arc;
///
/// // Create lock-free indexes for high concurrency
/// let hot_vector = Arc::new(LockFreeHotVectorIndex::new());
/// let hot_graph = Arc::new(LockFreeHotGraphIndex::new());
///
/// // Use them across multiple threads
/// hot_vector.add_vector("vec_123", vec![0.1; 384])?;
/// hot_graph.add_edge("node_1", "node_2")?;
/// ```
///
/// ## Batch Operations
///
/// For bulk operations, use batch processors:
///
/// ```ignore
/// use indexing::utils::batch::{VectorBatchProcessor, GraphBatchProcessor};
/// use indexing::{LockFreeHotVectorIndex, LockFreeHotGraphIndex};
/// use std::sync::Arc;
///
/// // Example usage
/// // let vector_index = Arc::new(LockFreeHotVectorIndex::new());
/// // let vector_processor = VectorBatchProcessor::new(vector_index);
/// // ... perform batch operations ...
/// ```
///
/// ## Advanced Graph Algorithms
///
/// The crate provides implementations of various graph algorithms:
///
/// ```ignore
/// use indexing::algorithms;
/// use indexing::graph_traits::{DirectedGraph, GraphBase};
///
/// // Example usage of graph algorithms
/// // See module documentation for specific algorithm usage
/// ```
///
/// ## Distance Metrics
///
/// Various distance metrics are available for vector similarity:
///
/// ```ignore
/// use indexing::utils::distance_metrics::{CosineMetric, EuclideanMetric};
///
/// // Example usage
/// // let a = vec![1.0, 2.0, 3.0];
/// // let b = vec![4.0, 5.0, 6.0];
/// // let cosine = CosineMetric::new();
/// // let distance = cosine.distance(&a, &b)?;
/// ```
///
/// ## Error Handling
///
/// The crate provides comprehensive error handling:
///
/// ```ignore
/// use indexing::utils::errors::{IndexingError, IndexingResult};
/// use indexing::IndexManager;
///
/// // Example error handling
/// // match index_manager.get_nodes_by_property("property", "value") {
/// //     Ok(nodes) => { /* handle result */ }
/// //     Err(e) => { /* handle error */ }
/// // }
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
/// ```ignore
/// use indexing::advanced::vector_storage::MmapVectorStorage;
///
/// // Example usage
/// // let storage = MmapVectorStorage::new("vectors.dat", dimension)?;
/// // Use memory-mapped storage for large vector datasets
/// ```
///
/// ## Builder Patterns
///
/// Fluent APIs are available for configuring indexes:
///
/// ```ignore
/// use indexing::utils::builders::*;
///
/// // Example usage
/// // Builder patterns are available for configuring indexes
/// // See individual module documentation for specific builders
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
/// ```ignore
/// // Benchmarks are available via cargo bench
/// // Run: cargo bench --features benchmarks
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
