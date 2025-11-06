//! 🔥 HYBRID INDEX TESTS - Hot/Warm/Cold Tiers

use indexing::advanced::hybrid::{QuantizedVector, HotGraphIndex, HotVectorIndex, DataTemperature};

#[test]
fn test_quantized_vector_basic() {
    println!("\n🔢 TEST: Quantized vector compression");
    let original = vec![0.1, 0.5, 0.9];
    
    println!("   📝 Quantizing vector...");
    let quantized = QuantizedVector::new(&original);
    
    assert_eq!(quantized.dimension, 3);
    assert!(!quantized.quantized_values.is_empty());
    
    println!("   📖 Reconstructing vector...");
    let reconstructed = quantized.reconstruct();
    assert_eq!(reconstructed.len(), 3);
    
    for (orig, recon) in original.iter().zip(reconstructed.iter()) {
        assert!((orig - recon).abs() < 0.01);
    }
    println!("   ✅ PASS: Quantization loss < 1%");
}

#[test]
fn test_quantized_vector_cosine_similarity() {
    println!("\n📐 TEST: Quantized vector cosine similarity");
    let vec1 = QuantizedVector::new(&[1.0, 0.0, 0.0]);
    let vec2 = QuantizedVector::new(&[0.9, 0.1, 0.0]);
    let vec3 = QuantizedVector::new(&[0.0, 0.0, 1.0]);
    
    println!("   🔎 Computing similarities...");
    let similarity_1_2 = vec1.cosine_similarity(&vec2);
    let similarity_1_3 = vec1.cosine_similarity(&vec3);
    
    assert!(similarity_1_2 > 0.9);
    assert!(similarity_1_3 < 0.1);
    println!("   ✅ PASS: Similar={:.2}, Orthogonal={:.2}", similarity_1_2, similarity_1_3);
}

#[test]
fn test_hot_graph_index_basic_operations() {
    println!("\n🔥 TEST: Hot graph index basic operations");
    let mut graph = HotGraphIndex::new();
    
    println!("   📝 Adding 2 nodes...");
    graph.add_node("node1", Some("metadata1")).unwrap();
    graph.add_node("node2", Some("metadata2")).unwrap();
    
    println!("   📝 Adding edge...");
    graph.add_edge("node1", "node2").unwrap();
    
    println!("   📖 Checking neighbors...");
    let outgoing = graph.get_outgoing_neighbors("node1").unwrap();
    assert_eq!(outgoing, vec!["node2"]);
    
    let incoming = graph.get_incoming_neighbors("node2").unwrap();
    assert_eq!(incoming, vec!["node1"]);
    
    println!("   🗑️  Removing edge...");
    graph.remove_edge("node1", "node2").unwrap();
    
    let outgoing = graph.get_outgoing_neighbors("node1").unwrap();
    assert!(outgoing.is_empty());
    println!("   ✅ PASS: Hot graph add/remove works");
}

#[test]
fn test_hot_vector_index_basic_operations() {
    println!("\n🔥 TEST: Hot vector index basic operations");
    let mut index = HotVectorIndex::new();
    
    println!("   📝 Adding 3 vectors to hot tier...");
    index.add_vector("vec1", vec![1.0, 0.0, 0.0]).unwrap();
    index.add_vector("vec2", vec![0.9, 0.1, 0.0]).unwrap();
    index.add_vector("vec3", vec![0.0, 0.0, 1.0]).unwrap();
    
    println!("   🔎 Searching hot tier...");
    let results = index.search(&[1.0, 0.0, 0.0], 2).unwrap();
    assert_eq!(results.len(), 2);
    
    assert_eq!(results[0].0, "vec1");
    assert!(results[0].1 > 0.99);
    println!("   ✅ PASS: Hot vector search works (similarity={:.2})", results[0].1);
}

#[test]
fn test_access_tracking() {
    println!("\n📊 TEST: Access pattern tracking for tier management");
    let mut graph = HotGraphIndex::new();
    
    println!("   📝 Adding node...");
    graph.add_node("test_node", None).unwrap();
    
    println!("   📖 Checking initial temperature...");
    let temp = graph.get_node_temperature("test_node").unwrap();
    assert_eq!(temp, DataTemperature::Hot);
    
    println!("   📝 Adding vector to hot tier...");
    let mut vector_index = HotVectorIndex::new();
    vector_index.add_vector("test_vec", vec![1.0, 0.0, 0.0]).unwrap();
    
    let temp = vector_index.get_vector_temperature("test_vec").unwrap();
    assert_eq!(temp, DataTemperature::Hot);
    println!("   ✅ PASS: Both graph and vector track temperature correctly");
}

