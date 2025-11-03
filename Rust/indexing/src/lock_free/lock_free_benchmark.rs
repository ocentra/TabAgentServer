//! Benchmark comparing lock-free and Mutex-based implementations.
//!
//! This module contains benchmarks to compare the performance of the lock-free
//! implementations against the traditional Mutex-based implementations under
//! various workloads.

use crate::hybrid::HotVectorIndex;
use crate::lock_free_hot_vector::LockFreeHotVectorIndex;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

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
        }
    }
}

/// Runs a benchmark comparing lock-free and Mutex-based HotVectorIndex implementations.
pub fn benchmark_hot_vector_index() -> Vec<BenchmarkResult> {
    const NUM_THREADS: usize = 8;
    const OPERATIONS_PER_THREAD: usize = 10000;
    const VECTOR_DIMENSION: usize = 128;
    
    let mut results = Vec::new();
    
    // Benchmark Mutex-based implementation
    {
        let index = Arc::new(Mutex::new(HotVectorIndex::new()));
        let start_time = Instant::now();
        
        let mut handles = vec![];
        for thread_id in 0..NUM_THREADS {
            let index_clone = Arc::clone(&index);
            let handle = thread::spawn(move || {
                for i in 0..OPERATIONS_PER_THREAD {
                    let vector_id = format!("mutex_thread{}_op{}", thread_id, i);
                    let vector = vec![(thread_id * OPERATIONS_PER_THREAD + i) as f32 / 1000000.0; VECTOR_DIMENSION];
                    index_clone.lock().unwrap().add_vector(&vector_id, vector).unwrap();
                }
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        let total_time = start_time.elapsed().as_millis();
        let result = BenchmarkResult::new(
            "Mutex-based HotVectorIndex".to_string(),
            NUM_THREADS * OPERATIONS_PER_THREAD,
            total_time,
        );
        results.push(result);
    }
    
    // Benchmark lock-free implementation
    {
        let index = Arc::new(LockFreeHotVectorIndex::new());
        let start_time = Instant::now();
        
        let mut handles = vec![];
        for thread_id in 0..NUM_THREADS {
            let index_clone = Arc::clone(&index);
            let handle = thread::spawn(move || {
                for i in 0..OPERATIONS_PER_THREAD {
                    let vector_id = format!("lockfree_thread{}_op{}", thread_id, i);
                    let vector = vec![(thread_id * OPERATIONS_PER_THREAD + i) as f32 / 1000000.0; VECTOR_DIMENSION];
                    index_clone.add_vector(&vector_id, vector).unwrap();
                }
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        let total_time = start_time.elapsed().as_millis();
        let result = BenchmarkResult::new(
            "Lock-free HotVectorIndex".to_string(),
            NUM_THREADS * OPERATIONS_PER_THREAD,
            total_time,
        );
        results.push(result);
    }
    
    results
}

/// Runs a benchmark comparing lock-free and Mutex-based HotVectorIndex search performance.
pub fn benchmark_hot_vector_index_search() -> Vec<BenchmarkResult> {
    const NUM_THREADS: usize = 6;
    const SEARCHES_PER_THREAD: usize = 5000;
    const VECTOR_COUNT: usize = 1000;
    const VECTOR_DIMENSION: usize = 64;
    const QUERY_DIMENSION: usize = 64;
    const K: usize = 10;
    
    let mut results = Vec::new();
    
    // Prepare test data
    let test_vectors: Vec<(String, Vec<f32>)> = (0..VECTOR_COUNT)
        .map(|i| {
            let vector_id = format!("vector_{}", i);
            let vector = vec![i as f32 / VECTOR_COUNT as f32; VECTOR_DIMENSION];
            (vector_id, vector)
        })
        .collect();
    
    let query_vector = vec![0.5; QUERY_DIMENSION];
    
    // Benchmark Mutex-based implementation
    {
        let index = Arc::new(Mutex::new(HotVectorIndex::new()));
        
        // Add test vectors
        for (vector_id, vector) in &test_vectors {
            index.lock().unwrap().add_vector(vector_id, vector.clone()).unwrap();
        }
        
        let start_time = Instant::now();
        
        let mut handles = vec![];
        for thread_id in 0..NUM_THREADS {
            let index_clone = Arc::clone(&index);
            let query_clone = query_vector.clone();
            let handle = thread::spawn(move || {
                for _ in 0..SEARCHES_PER_THREAD {
                    let _results = index_clone.lock().unwrap().search(&query_clone, K).unwrap();
                }
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        let total_time = start_time.elapsed().as_millis();
        let result = BenchmarkResult::new(
            "Mutex-based HotVectorIndex Search".to_string(),
            NUM_THREADS * SEARCHES_PER_THREAD,
            total_time,
        );
        results.push(result);
    }
    
    // Benchmark lock-free implementation
    {
        let index = Arc::new(LockFreeHotVectorIndex::new());
        
        // Add test vectors
        for (vector_id, vector) in &test_vectors {
            index.add_vector(vector_id, vector.clone()).unwrap();
        }
        
        let start_time = Instant::now();
        
        let mut handles = vec![];
        for thread_id in 0..NUM_THREADS {
            let index_clone = Arc::clone(&index);
            let query_clone = query_vector.clone();
            let handle = thread::spawn(move || {
                for _ in 0..SEARCHES_PER_THREAD {
                    let _results = index_clone.search(&query_clone, K).unwrap();
                }
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        let total_time = start_time.elapsed().as_millis();
        let result = BenchmarkResult::new(
            "Lock-free HotVectorIndex Search".to_string(),
            NUM_THREADS * SEARCHES_PER_THREAD,
            total_time,
        );
        results.push(result);
    }
    
    results
}

/// Runs a mixed workload benchmark comparing lock-free and Mutex-based implementations.
pub fn benchmark_mixed_workload() -> Vec<BenchmarkResult> {
    const NUM_THREADS: usize = 8;
    const OPERATIONS_PER_THREAD: usize = 5000;
    const VECTOR_DIMENSION: usize = 64;
    const K: usize = 5;
    
    let mut results = Vec::new();
    
    // Prepare query vector
    let query_vector = vec![0.3; VECTOR_DIMENSION];
    
    // Benchmark Mutex-based implementation
    {
        let index = Arc::new(Mutex::new(HotVectorIndex::new()));
        let start_time = Instant::now();
        
        let mut handles = vec![];
        for thread_id in 0..NUM_THREADS {
            let index_clone = Arc::clone(&index);
            let query_clone = query_vector.clone();
            let handle = thread::spawn(move || {
                for i in 0..OPERATIONS_PER_THREAD {
                    if i % 3 == 0 {
                        // Add operation
                        let vector_id = format!("mutex_mixed_thread{}_op{}", thread_id, i);
                        let vector = vec![(thread_id * OPERATIONS_PER_THREAD + i) as f32 / 1000000.0; VECTOR_DIMENSION];
                        index_clone.lock().unwrap().add_vector(&vector_id, vector).unwrap();
                    } else {
                        // Search operation
                        let _results = index_clone.lock().unwrap().search(&query_clone, K).unwrap();
                    }
                }
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        let total_time = start_time.elapsed().as_millis();
        let result = BenchmarkResult::new(
            "Mutex-based HotVectorIndex Mixed Workload".to_string(),
            NUM_THREADS * OPERATIONS_PER_THREAD,
            total_time,
        );
        results.push(result);
    }
    
    // Benchmark lock-free implementation
    {
        let index = Arc::new(LockFreeHotVectorIndex::new());
        let start_time = Instant::now();
        
        let mut handles = vec![];
        for thread_id in 0..NUM_THREADS {
            let index_clone = Arc::clone(&index);
            let query_clone = query_vector.clone();
            let handle = thread::spawn(move || {
                for i in 0..OPERATIONS_PER_THREAD {
                    if i % 3 == 0 {
                        // Add operation
                        let vector_id = format!("lockfree_mixed_thread{}_op{}", thread_id, i);
                        let vector = vec![(thread_id * OPERATIONS_PER_THREAD + i) as f32 / 1000000.0; VECTOR_DIMENSION];
                        index_clone.add_vector(&vector_id, vector).unwrap();
                    } else {
                        // Search operation
                        let _results = index_clone.search(&query_clone, K).unwrap();
                    }
                }
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        let total_time = start_time.elapsed().as_millis();
        let result = BenchmarkResult::new(
            "Lock-free HotVectorIndex Mixed Workload".to_string(),
            NUM_THREADS * OPERATIONS_PER_THREAD,
            total_time,
        );
        results.push(result);
    }
    
    results
}

/// Prints benchmark results in a formatted table.
pub fn print_benchmark_results(results: &[BenchmarkResult]) {
    println!("\n{:<40} {:<12} {:<12} {:<12} {:<12}", 
             "Benchmark", "Operations", "Time (ms)", "Avg (μs/op)", "Ops/sec");
    println!("{}", "-".repeat(100));
    
    for result in results {
        println!("{:<40} {:<12} {:<12} {:<12.2} {:<12.0}", 
                 result.name, 
                 result.operations, 
                 result.total_time_ms, 
                 result.avg_time_per_op_us, 
                 result.ops_per_second);
    }
    
    // Compare performance
    if results.len() >= 2 {
        let mutex_result = &results[0];
        let lockfree_result = &results[1];
        
        let speedup = lockfree_result.ops_per_second / mutex_result.ops_per_second;
        println!("\nPerformance Comparison:");
        if speedup > 1.0 {
            println!("Lock-free implementation is {:.2}x faster", speedup);
        } else {
            println!("Mutex-based implementation is {:.2}x faster", 1.0 / speedup);
        }
    }
}
