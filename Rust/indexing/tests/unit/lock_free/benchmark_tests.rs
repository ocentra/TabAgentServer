//! ⚡ LOCK-FREE BENCHMARK TESTS - Performance Measurement

use indexing::lock_free::lock_free_benchmark::{
    BenchmarkResult, benchmark_hot_vector_index, benchmark_hot_vector_index_search,
    benchmark_mixed_workload,
};

#[test]
fn test_benchmark_result_creation() {
    println!("\n⚡ TEST: Benchmark result calculations");
    let result = BenchmarkResult::new("Test".to_string(), 1000, 50);
    
    assert_eq!(result.name, "Test");
    assert_eq!(result.operations, 1000);
    assert_eq!(result.total_time_ms, 50);
    assert!((result.avg_time_per_op_us - 50.0).abs() < 0.01);
    assert!((result.ops_per_second - 20000.0).abs() < 1.0);
    println!("   ✅ PASS: {} ops/sec calculated correctly", result.ops_per_second as u64);
}

#[test]
fn test_benchmark_hot_vector_index() {
    println!("\n⚡ TEST: Benchmark hot vector index performance");
    println!("   🔧 Running benchmark...");
    let results = benchmark_hot_vector_index();
    
    assert_eq!(results.len(), 2);
    assert!(results[0].operations > 0);
    assert!(results[1].operations > 0);
    println!("   ✅ PASS: {} benchmark results collected", results.len());
}

#[test]
#[ignore = "Long-running benchmark test - run with --ignored"]
fn test_benchmark_hot_vector_index_search() {
    println!("\n🔎 TEST: Benchmark hot vector index search");
    println!("   🔧 Running search benchmark...");
    let results = benchmark_hot_vector_index_search();
    
    assert_eq!(results.len(), 2);
    assert!(results[0].operations > 0);
    assert!(results[1].operations > 0);
    println!("   ✅ PASS: {} search benchmark results", results.len());
}

#[test]
#[ignore = "Long-running benchmark test - run with --ignored"]
fn test_benchmark_mixed_workload() {
    println!("\n🔀 TEST: Benchmark mixed read/write workload");
    println!("   🔧 Running mixed workload benchmark...");
    let results = benchmark_mixed_workload();
    
    assert_eq!(results.len(), 2);
    assert!(results[0].operations > 0);
    assert!(results[1].operations > 0);
    println!("   ✅ PASS: {} mixed workload results", results.len());
}
