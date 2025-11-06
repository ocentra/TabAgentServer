//! 🌊 FLOW ALGORITHM TESTS - Network Flow Algorithms

use crate::common::{setup_real_db, test_edge};

#[test]
fn test_max_flow_zero_copy() {
    println!("\n🌊 TEST: Max flow algorithm (Ford-Fulkerson)");
    let (manager, _temp, _storage) = setup_real_db();
    let graph = manager.graph();
    
    // Create a simple flow network:
    //   s -> a -> t
    //   |    ^    ^
    //   v    |    |
    //   b --------+
    
    let edges = vec![
        test_edge("s_a", "s", "a"),
        test_edge("s_b", "s", "b"),
        test_edge("b_a", "b", "a"),
        test_edge("a_t", "a", "t"),
        test_edge("b_t", "b", "t"),
    ];
    
    println!("   📝 Creating flow network (s -> a,b -> t)...");
    for edge in &edges {
        graph.add_edge_with_struct(edge).unwrap();
    }
    
    println!("   🔍 Computing max flow...");
    use indexing::algorithms::flow_algorithms::max_flow_zero_copy;
    
    let capacity = |_from: &str, _to: &str| 1.0;
    let result = max_flow_zero_copy(&**graph, "s", "t", capacity);
    
    assert!(result.is_ok());
    println!("   ✅ PASS: Max flow computed successfully");
}

#[test]
fn test_min_cost_max_flow_zero_copy() {
    println!("\n💰 TEST: Min cost max flow algorithm");
    let (manager, _temp, _storage) = setup_real_db();
    let graph = manager.graph();
    
    println!("   📝 Creating network...");
    let edges = vec![
        test_edge("s_a", "s", "a"),
        test_edge("a_t", "a", "t"),
    ];
    
    for edge in &edges {
        graph.add_edge_with_struct(edge).unwrap();
    }
    
    println!("   🔍 Computing min cost max flow...");
    use indexing::algorithms::flow_algorithms::min_cost_max_flow_zero_copy;
    
    let capacity = |_from: &str, _to: &str| 1.0;
    let cost = |_from: &str, _to: &str| 1.0;
    let result = min_cost_max_flow_zero_copy(&**graph, "s", "t", capacity, cost);
    
    assert!(result.is_ok());
    println!("   ✅ PASS: Min cost max flow computed");
}

