//! ⚡ ADAPTIVE CONCURRENCY TESTS - Dynamic Mode Switching

use indexing::utils::adaptive_concurrency::{
    AdaptiveConcurrencyController, AdaptiveVectorIndex, AdaptiveGraphIndex,
    ConcurrencyMode, ConcurrencyMetrics,
};

#[test]
fn test_adaptive_concurrency_controller() {
    println!("\n⚡ TEST: Adaptive concurrency controller (mode switching)");
    let mut controller = AdaptiveConcurrencyController::with_thresholds(
        10000, // lock_free_threshold
        1000,  // traditional_threshold
        0,     // min_switch_interval (0 seconds for testing)
    );
    
    println!("   🔍 Initial mode: {:?}", controller.get_mode());
    assert_eq!(controller.get_mode(), ConcurrencyMode::Traditional);
    
    println!("   📊 Simulating high load (15k ops)...");
    let metrics = ConcurrencyMetrics {
        operation_count: 15000,
        avg_latency_micros: 100,
        contention_level: 50,
        memory_usage: 1024,
    };
    
    controller.update_metrics(metrics);
    let switch_result = controller.check_mode_switch();
    
    println!("   🔄 Mode after switch: {:?}", controller.get_mode());
    assert_eq!(switch_result, Some(ConcurrencyMode::LockFree));
    assert_eq!(controller.get_mode(), ConcurrencyMode::LockFree);
    println!("   ✅ PASS: Switched from Traditional -> LockFree under high load");
}

#[test]
fn test_adaptive_vector_index() {
    println!("\n🔍 TEST: Adaptive vector index (dynamic mode)");
    let index = AdaptiveVectorIndex::new();
    
    println!("   🔍 Initial mode: {:?}", index.get_mode());
    assert_eq!(index.get_mode(), ConcurrencyMode::Traditional);
    
    println!("   📝 Adding vector...");
    let vector = vec![0.1, 0.2, 0.3, 0.4];
    index.add_vector("vec1", vector).unwrap();
    
    println!("   🔍 Searching...");
    let query = vec![0.15, 0.25, 0.35, 0.45];
    let _results = index.search(&query, 10).unwrap();
    
    assert_eq!(index.len(), 1);
    assert!(!index.is_empty());
    println!("   ✅ PASS: Adaptive vector index works (len=1)");
}

#[test]
fn test_adaptive_graph_index() {
    println!("\n🕸️  TEST: Adaptive graph index (dynamic mode)");
    let index = AdaptiveGraphIndex::new();
    
    println!("   🔍 Initial mode: {:?}", index.get_mode());
    assert_eq!(index.get_mode(), ConcurrencyMode::Traditional);
    
    println!("   📝 Adding node + edge...");
    index.add_node("node1", Some("metadata")).unwrap();
    index.add_edge("node1", "node2").unwrap();
    println!("   ✅ PASS: Adaptive graph index works");
}

