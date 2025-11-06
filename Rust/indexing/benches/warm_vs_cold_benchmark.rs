//! BENCHMARK: Warm vs Cold Tier Performance
//!
//! Measures the performance difference between:
//! - Cold tier: Direct MDBX access (transaction pool)
//! - Warm tier: Cached access (first hit pays setup cost)

use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use indexing::core::graph::GraphIndex;
use storage::StorageManager;
use tempfile::TempDir;
use std::sync::Arc;

fn create_benchmark_graph() -> (Arc<GraphIndex>, TempDir, StorageManager) {
    let temp_dir = TempDir::new().unwrap();
    let storage = StorageManager::new(temp_dir.path().to_str().unwrap()).unwrap();
    
    // Get pointers from storage
    let env_ptr = unsafe { storage.get_raw_env() };
    let outgoing_dbi = storage.get_or_create_dbi("graph_outgoing").unwrap();
    let incoming_dbi = storage.get_or_create_dbi("graph_incoming").unwrap();
    
    let graph_index = GraphIndex::new(env_ptr, outgoing_dbi, incoming_dbi);
    
    // Add test edges
    for i in 0..10 {
        graph_index.add_edge("hot_node", &format!("node{}", i))
            .expect("Failed to add edge");
    }
    
    // CRITICAL: Clear the thread-local transaction pool!
    // The pooled read transaction was created BEFORE the writes,
    // so it has an old MVCC snapshot. We must clear it to force
    // a fresh transaction that can see the committed data.
    mdbx_base::txn_pool::cleanup_thread_txn();
    
    // Verify edges are now visible
    let count = graph_index.count_outgoing("hot_node")
        .expect("Failed to count edges after txn cleanup");
    assert_eq!(count, 10, "Expected 10 edges after txn pool cleanup");
    
    (Arc::new(graph_index), temp_dir, storage)
}

fn benchmark_cold_access(c: &mut Criterion) {
    let (graph_index, _temp, _storage) = create_benchmark_graph();
    
    c.bench_function("cold_tier_access", |b| {
        b.iter(|| {
            // Direct MDBX access (uses pooled transaction)
            let guard = graph_index.get_outgoing("hot_node").unwrap().unwrap();
            black_box(guard.len());
        });
    });
}

criterion_group!(benches, benchmark_cold_access);
criterion_main!(benches);

