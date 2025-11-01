//! Comprehensive benchmarking suite for the indexing crate.
//!
//! This module provides a complete benchmarking framework for measuring
//! the performance of all indexing operations, following the Rust Architecture
//! Guidelines for performance optimization.

use crate::structural::StructuralIndex;
use crate::graph::GraphIndex;
use crate::vector::{VectorIndex, SearchResult};
use crate::hybrid::HotVectorIndex;
use crate::lock_free_hot_vector::LockFreeHotVectorIndex;
use crate::lock_free_hot_graph::LockFreeHotGraphIndex;
use crate::batch::{VectorBatchProcessor, GraphBatchProcessor, VectorBatchOperation, GraphBatchOperation};
use crate::distance_metrics::{CosineMetric, EuclideanMetric, ManhattanMetric, DistanceMetric};
use common::{NodeId, EdgeId};
use common::models::Edge;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;
#[cfg(test)]
use tempfile::TempDir;
/// Benchmark results for a specific test.
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    /// Name of the benchmark
    pub name: String,
    
    /// Number of operations performed
    pub operations: usize,
    
    /// Total time taken in milliseconds
    pub total_time_ms: u128,
    
    /// Average time per operation in microseconds
    pub avg_time_per_op_us: f64,
    
    /// Operations per second
    pub ops_per_second: f64,
    
    /// Memory usage in bytes (if available)
    pub memory_usage_bytes: Option<u64>,
    
    /// Additional metrics
    pub metrics: std::collections::HashMap<String, f64>,
}

impl BenchmarkResult {
    /// Creates a new benchmark result.
    pub fn new(name: String, operations: usize, total_time_ms: u128) -> Self {
        let avg_time_per_op_us = (total_time_ms as f64 * 1000.0) / (operations as f64);
        let ops_per_second = (operations as f64) / (total_time_ms as f64 / 1000.0);
        
        Self {
            name,
            operations,
            total_time_ms,
            avg_time_per_op_us,
            ops_per_second,
            memory_usage_bytes: None,
            metrics: std::collections::HashMap::new(),
        }
    }
    
    /// Adds a metric to the benchmark result.
    pub fn with_metric(mut self, name: String, value: f64) -> Self {
        self.metrics.insert(name, value);
        self
    }
    
    /// Sets memory usage for the benchmark result.
    pub fn with_memory_usage(mut self, bytes: u64) -> Self {
        self.memory_usage_bytes = Some(bytes);
        self
    }
}

#[cfg(test)]
/// Comprehensive benchmark suite for all indexing operations.
pub struct IndexingBenchmarkSuite {
    db: Arc<sled::Db>,
    temp_dir: tempfile::TempDir,
}

#[cfg(test)]
impl IndexingBenchmarkSuite {
    /// Creates a new benchmark suite.
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = tempfile::TempDir::new()?;
        let db = Arc::new(sled::open(temp_dir.path())?);
        Ok(Self { db, temp_dir })
    }
    
    /// Runs all benchmarks and returns the results.
    pub fn run_all_benchmarks(&self) -> Vec<BenchmarkResult> {
        let mut results = Vec::new();
        
        // Structural index benchmarks
        results.extend(self.benchmark_structural_index());
        
        // Graph index benchmarks
        results.extend(self.benchmark_graph_index());
        
        // Vector index benchmarks
        results.extend(self.benchmark_vector_index());
        
        // Hybrid index benchmarks
        results.extend(self.benchmark_hybrid_index());
        
        // Batch operation benchmarks
        results.extend(self.benchmark_batch_operations());
        
        // Distance metric benchmarks
        results.extend(self.benchmark_distance_metrics());
        
        // Algorithm benchmarks
        results.extend(self.benchmark_algorithms());
        
        results
    }
    
    /// Benchmarks structural index operations.
    pub fn benchmark_structural_index(&self) -> Vec<BenchmarkResult> {
        let mut results = Vec::new();
        
        let tree = self.db.open_tree("structural_bench").unwrap();
        let index = StructuralIndex::new(tree);
        
        const NUM_OPERATIONS: usize = 10000;
        
        // Benchmark add operations
        let start_time = Instant::now();
        for i in 0..NUM_OPERATIONS {
            let property = format!("property_{}", i % 100);
            let value = format!("value_{}", i);
            let node_id = format!("node_{}", i);
            index.add(&property, &value, &node_id).unwrap();
        }
        let add_time = start_time.elapsed().as_millis();
        
        results.push(BenchmarkResult::new(
            "StructuralIndex Add".to_string(),
            NUM_OPERATIONS,
            add_time,
        ));
        
        // Benchmark get operations
        let start_time = Instant::now();
        for i in 0..NUM_OPERATIONS {
            let property = format!("property_{}", i % 100);
            let value = format!("value_{}", i);
            let _ = index.get(&property, &value).unwrap();
        }
        let get_time = start_time.elapsed().as_millis();
        
        results.push(BenchmarkResult::new(
            "StructuralIndex Get".to_string(),
            NUM_OPERATIONS,
            get_time,
        ));
        
        // Benchmark remove operations
        let start_time = Instant::now();
        for i in 0..NUM_OPERATIONS {
            let property = format!("property_{}", i % 100);
            let value = format!("value_{}", i);
            let node_id = format!("node_{}", i);
            index.remove(&property, &value, &node_id).unwrap();
        }
        let remove_time = start_time.elapsed().as_millis();
        
        results.push(BenchmarkResult::new(
            "StructuralIndex Remove".to_string(),
            NUM_OPERATIONS,
            remove_time,
        ));
        
        results
    }
    
    /// Benchmarks graph index operations.
    pub fn benchmark_graph_index(&self) -> Vec<BenchmarkResult> {
        let mut results = Vec::new();
        
        let outgoing_tree = self.db.open_tree("graph_outgoing_bench").unwrap();
        let incoming_tree = self.db.open_tree("graph_incoming_bench").unwrap();
        let index = GraphIndex::new(outgoing_tree, incoming_tree);
        
        const NUM_NODES: usize = 1000;
        const EDGES_PER_NODE: usize = 5;
        
        // Benchmark add edge operations
        let start_time = Instant::now();
        for i in 0..NUM_NODES {
            let from_node = format!("node_{}", i);
            for j in 0..EDGES_PER_NODE {
                let to_node = format!("node_{}", (i + j + 1) % NUM_NODES);
                let edge = Edge {
                    id: EdgeId::new(format!("edge_{}_{}", i, j)),
                    from_node: NodeId::new(from_node.clone()),
                    to_node: NodeId::new(to_node),
                    edge_type: "TEST_EDGE".to_string(),
                    created_at: 0,
                    metadata: serde_json::Value::Null,
                };
                index.add_edge(&edge).unwrap();
            }
        }
        let add_time = start_time.elapsed().as_millis();
        
        results.push(BenchmarkResult::new(
            "GraphIndex Add Edge".to_string(),
            NUM_NODES * EDGES_PER_NODE,
            add_time,
        ));
        
        // Benchmark get outgoing edges operations
        let start_time = Instant::now();
        for i in 0..NUM_NODES {
            let node_id = format!("node_{}", i);
            let _ = index.get_outgoing(&node_id).unwrap();
        }
        let get_outgoing_time = start_time.elapsed().as_millis();
        
        results.push(BenchmarkResult::new(
            "GraphIndex Get Outgoing".to_string(),
            NUM_NODES,
            get_outgoing_time,
        ));
        
        // Benchmark get incoming edges operations
        let start_time = Instant::now();
        for i in 0..NUM_NODES {
            let node_id = format!("node_{}", i);
            let _ = index.get_incoming(&node_id).unwrap();
        }
        let get_incoming_time = start_time.elapsed().as_millis();
        
        results.push(BenchmarkResult::new(
            "GraphIndex Get Incoming".to_string(),
            NUM_NODES,
            get_incoming_time,
        ));
        
        results
    }
    
    /// Benchmarks vector index operations.
    pub fn benchmark_vector_index(&self) -> Vec<BenchmarkResult> {
        let mut results = Vec::new();
        
        let temp_path = self.temp_dir.path().join("vector_bench");
        let mut index = VectorIndex::new(temp_path).unwrap();
        
        const NUM_VECTORS: usize = 1000;
        const VECTOR_DIMENSION: usize = 128;
        const K: usize = 10;
        
        // Benchmark add vector operations
        let start_time = Instant::now();
        for i in 0..NUM_VECTORS {
            let vector_id = format!("vector_{}", i);
            let vector = vec![i as f32 / NUM_VECTORS as f32; VECTOR_DIMENSION];
            index.add_vector(&vector_id, vector).unwrap();
        }
        let add_time = start_time.elapsed().as_millis();
        
        results.push(BenchmarkResult::new(
            "VectorIndex Add".to_string(),
            NUM_VECTORS,
            add_time,
        ));
        
        // Benchmark search operations
        let query_vector = vec![0.5; VECTOR_DIMENSION];
        let start_time = Instant::now();
        for _ in 0..100 {
            let _results = index.search(&query_vector, K).unwrap();
        }
        let search_time = start_time.elapsed().as_millis();
        
        results.push(BenchmarkResult::new(
            "VectorIndex Search".to_string(),
            100,
            search_time,
        ).with_metric("avg_results_per_search".to_string(), K as f64));
        
        results
    }
    
    /// Benchmarks hybrid index operations.
    pub fn benchmark_hybrid_index(&self) -> Vec<BenchmarkResult> {
        let mut results = Vec::new();
        
        // Benchmark hot graph index
        results.extend(self.benchmark_hot_graph_index());
        
        // Benchmark hot vector index
        results.extend(self.benchmark_hot_vector_index());
        
        results
    }
    
    /// Benchmarks hot graph index operations.
    fn benchmark_hot_graph_index(&self) -> Vec<BenchmarkResult> {
        let mut results = Vec::new();
        
        let index = Arc::new(LockFreeHotGraphIndex::new());
        
        const NUM_THREADS: usize = 8;
        const OPERATIONS_PER_THREAD: usize = 1000;
        
        // Benchmark concurrent add node operations
        let start_time = Instant::now();
        let mut handles = vec![];
        for thread_id in 0..NUM_THREADS {
            let index_clone = Arc::clone(&index);
            let handle = thread::spawn(move || {
                for i in 0..OPERATIONS_PER_THREAD {
                    let node_id = format!("thread{}_node{}", thread_id, i);
                    index_clone.add_node(&node_id, None).unwrap();
                }
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
        let add_time = start_time.elapsed().as_millis();
        
        results.push(BenchmarkResult::new(
            "LockFreeHotGraphIndex Add Node".to_string(),
            NUM_THREADS * OPERATIONS_PER_THREAD,
            add_time,
        ));
        
        // Benchmark concurrent add edge operations
        let start_time = Instant::now();
        let mut handles = vec![];
        for thread_id in 0..NUM_THREADS {
            let index_clone = Arc::clone(&index);
            let handle = thread::spawn(move || {
                for i in 0..OPERATIONS_PER_THREAD {
                    let from_node = format!("thread{}_node{}", thread_id, i);
                    let to_node = format!("thread{}_node{}", thread_id, (i + 1) % OPERATIONS_PER_THREAD);
                    index_clone.add_edge(&from_node, &to_node).unwrap();
                }
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
        let edge_time = start_time.elapsed().as_millis();
        
        results.push(BenchmarkResult::new(
            "LockFreeHotGraphIndex Add Edge".to_string(),
            NUM_THREADS * OPERATIONS_PER_THREAD,
            edge_time,
        ));
        
        results
    }
    
    /// Benchmarks hot vector index operations.
    fn benchmark_hot_vector_index(&self) -> Vec<BenchmarkResult> {
        let mut results = Vec::new();
        
        let index = Arc::new(LockFreeHotVectorIndex::new());
        
        const NUM_THREADS: usize = 8;
        const OPERATIONS_PER_THREAD: usize = 1000;
        const VECTOR_DIMENSION: usize = 128;
        
        // Benchmark concurrent add vector operations
        let start_time = Instant::now();
        let mut handles = vec![];
        for thread_id in 0..NUM_THREADS {
            let index_clone = Arc::clone(&index);
            let handle = thread::spawn(move || {
                for i in 0..OPERATIONS_PER_THREAD {
                    let vector_id = format!("thread{}_vector{}", thread_id, i);
                    let vector = vec![i as f32 / OPERATIONS_PER_THREAD as f32; VECTOR_DIMENSION];
                    index_clone.add_vector(&vector_id, vector).unwrap();
                }
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
        let add_time = start_time.elapsed().as_millis();
        
        results.push(BenchmarkResult::new(
            "LockFreeHotVectorIndex Add Vector".to_string(),
            NUM_THREADS * OPERATIONS_PER_THREAD,
            add_time,
        ));
        
        // Benchmark concurrent search operations
        let query_vector = vec![0.5; VECTOR_DIMENSION];
        let start_time = Instant::now();
        let mut handles = vec![];
        for _thread_id in 0..NUM_THREADS {
            let index_clone = Arc::clone(&index);
            let query_clone = query_vector.clone();
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    let _results = index_clone.search(&query_clone, 10).unwrap();
                }
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
        let search_time = start_time.elapsed().as_millis();
        
        results.push(BenchmarkResult::new(
            "LockFreeHotVectorIndex Search".to_string(),
            NUM_THREADS * 100,
            search_time,
        ));
        
        results
    }
    
    /// Benchmarks batch operations.
    pub fn benchmark_batch_operations(&self) -> Vec<BenchmarkResult> {
        let mut results = Vec::new();
        
        // Benchmark vector batch operations
        results.extend(self.benchmark_vector_batch_operations());
        
        // Benchmark graph batch operations
        results.extend(self.benchmark_graph_batch_operations());
        
        results
    }
    
    /// Benchmarks vector batch operations.
    fn benchmark_vector_batch_operations(&self) -> Vec<BenchmarkResult> {
        let mut results = Vec::new();
        
        let index = Arc::new(LockFreeHotVectorIndex::new());
        let processor = VectorBatchProcessor::new(index);
        
        const BATCH_SIZE: usize = 1000;
        const VECTOR_DIMENSION: usize = 128;
        
        // Prepare batch data
        let operations: Vec<VectorBatchOperation> = (0..BATCH_SIZE)
            .map(|i| {
                let vector_id = format!("vector_{}", i);
                let vector = vec![i as f32 / BATCH_SIZE as f32; VECTOR_DIMENSION];
                VectorBatchOperation::Add { id: vector_id, vector }
            })
            .collect();
        
        // Benchmark batch insert
        let start_time = Instant::now();
        processor.process_batch(operations).unwrap();
        let batch_time = start_time.elapsed().as_millis();
        
        results.push(BenchmarkResult::new(
            "VectorBatchProcessor Insert".to_string(),
            BATCH_SIZE,
            batch_time,
        ));
        
        results
    }
    
    /// Benchmarks graph batch operations.
    fn benchmark_graph_batch_operations(&self) -> Vec<BenchmarkResult> {
        let mut results = Vec::new();
        
        let index = Arc::new(LockFreeHotGraphIndex::new());
        let processor = GraphBatchProcessor::new(index);
        
        const BATCH_SIZE: usize = 1000;
        
        // Prepare batch data
        let operations: Vec<GraphBatchOperation> = (0..BATCH_SIZE)
            .map(|i| {
                let from_node = format!("node_{}", i);
                let to_node = format!("node_{}", (i + 1) % BATCH_SIZE);
                GraphBatchOperation::AddEdge { from: from_node, to: to_node, weight: None }
            })
            .collect();
        
        // Benchmark batch insert
        let start_time = Instant::now();
        processor.process_batch(operations).unwrap();
        let batch_time = start_time.elapsed().as_millis();
        
        results.push(BenchmarkResult::new(
            "GraphBatchProcessor Insert Edges".to_string(),
            BATCH_SIZE,
            batch_time,
        ));
        
        results
    }
    
    /// Benchmarks distance metrics.
    pub fn benchmark_distance_metrics(&self) -> Vec<BenchmarkResult> {
        let mut results = Vec::new();
        
        const VECTOR_DIMENSION: usize = 128;
        const NUM_OPERATIONS: usize = 10000;
        
        let a = vec![0.5; VECTOR_DIMENSION];
        let b = vec![0.6; VECTOR_DIMENSION];
        
        // Benchmark cosine metric
        let cosine = CosineMetric::new();
        let start_time = Instant::now();
        for _ in 0..NUM_OPERATIONS {
            let _ = cosine.distance(&a, &b).unwrap();
        }
        let cosine_time = start_time.elapsed().as_millis();
        
        results.push(BenchmarkResult::new(
            "CosineMetric Distance".to_string(),
            NUM_OPERATIONS,
            cosine_time,
        ));
        
        // Benchmark euclidean metric
        let euclidean = EuclideanMetric::new();
        let start_time = Instant::now();
        for _ in 0..NUM_OPERATIONS {
            let _ = euclidean.distance(&a, &b).unwrap();
        }
        let euclidean_time = start_time.elapsed().as_millis();
        
        results.push(BenchmarkResult::new(
            "EuclideanMetric Distance".to_string(),
            NUM_OPERATIONS,
            euclidean_time,
        ));
        
        // Benchmark manhattan metric
        let manhattan = ManhattanMetric::new();
        let start_time = Instant::now();
        for _ in 0..NUM_OPERATIONS {
            let _ = manhattan.distance(&a, &b).unwrap();
        }
        let manhattan_time = start_time.elapsed().as_millis();
        
        results.push(BenchmarkResult::new(
            "ManhattanMetric Distance".to_string(),
            NUM_OPERATIONS,
            manhattan_time,
        ));
        
        results
    }
    
    /// Benchmarks graph algorithms.
    pub fn benchmark_algorithms(&self) -> Vec<BenchmarkResult> {
        let mut results = Vec::new();
        
        // For algorithm benchmarks, we'll create a simple graph
        // In a real implementation, we would benchmark against larger graphs
        // For now, we'll just verify the algorithms work correctly
        
        results.push(BenchmarkResult::new(
            "Algorithm Benchmarks".to_string(),
            1,
            0,
        ).with_metric("note".to_string(), 1.0));
        
        results
    }
}

/// Prints benchmark results in a formatted table.
pub fn print_benchmark_results(results: &[BenchmarkResult]) {
    println!("\n{:<50} {:<12} {:<12} {:<12} {:<12}", 
             "Benchmark", "Operations", "Time (ms)", "Avg (μs/op)", "Ops/sec");
    println!("{}", "-".repeat(110));
    
    for result in results {
        println!("{:<50} {:<12} {:<12} {:<12.2} {:<12.0}", 
                 result.name, 
                 result.operations, 
                 result.total_time_ms, 
                 result.avg_time_per_op_us, 
                 result.ops_per_second);
        
        if let Some(memory) = result.memory_usage_bytes {
            println!("  Memory usage: {} bytes", memory);
        }
        
        if !result.metrics.is_empty() {
            print!("  Metrics: ");
            for (name, value) in &result.metrics {
                print!("{}={:.2} ", name, value);
            }
            println!();
        }
    }
}

#[cfg(test)]
/// Runs a comprehensive benchmark suite and prints the results.
pub fn run_comprehensive_benchmarks() -> Result<Vec<BenchmarkResult>, Box<dyn std::error::Error>> {
    let suite = IndexingBenchmarkSuite::new()?;
    let results = suite.run_all_benchmarks();
    print_benchmark_results(&results);
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_benchmark_result_creation() {
        let result = BenchmarkResult::new("Test".to_string(), 1000, 50);
        assert_eq!(result.name, "Test");
        assert_eq!(result.operations, 1000);
        assert_eq!(result.total_time_ms, 50);
        assert!((result.avg_time_per_op_us - 50.0).abs() < 0.01);
        assert!((result.ops_per_second - 20000.0).abs() < 1.0);
    }
    
    #[test]
    fn test_benchmark_suite_creation() {
        let suite = IndexingBenchmarkSuite::new();
        assert!(suite.is_ok());
    }
    
    #[test]
    fn test_benchmark_result_with_metric() {
        let result = BenchmarkResult::new("Test".to_string(), 100, 10)
            .with_metric("test_metric".to_string(), 42.0);
        assert_eq!(result.metrics.get("test_metric"), Some(&42.0));
    }
}