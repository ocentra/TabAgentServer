//! 📐 DISTANCE METRIC TESTS - Vector Similarity Functions

use indexing::utils::distance_metrics::{
    CosineMetric, EuclideanMetric, ManhattanMetric, DotProductMetric,
    JaccardMetric, HammingMetric, DynamicDistanceMetric, DistanceMetric,
};

#[test]
fn test_cosine_metric() {
    println!("\n📐 TEST: Cosine similarity metric");
    let metric = CosineMetric::new();
    
    println!("   🔍 Testing identical vectors...");
    let a = vec![1.0, 0.0, 0.0];
    let b = vec![1.0, 0.0, 0.0];
    let distance = metric.distance(&a, &b).unwrap();
    let similarity = metric.similarity(&a, &b).unwrap();
    assert!((distance - 0.0).abs() < f32::EPSILON);
    assert!((similarity - 1.0).abs() < f32::EPSILON);
    
    println!("   🔍 Testing orthogonal vectors...");
    let a = vec![1.0, 0.0, 0.0];
    let b = vec![0.0, 1.0, 0.0];
    let distance = metric.distance(&a, &b).unwrap();
    let similarity = metric.similarity(&a, &b).unwrap();
    assert!((distance - 1.0).abs() < f32::EPSILON);
    assert!((similarity - 0.0).abs() < f32::EPSILON);
    
    println!("   🔍 Testing opposite vectors...");
    let a = vec![1.0, 0.0, 0.0];
    let b = vec![-1.0, 0.0, 0.0];
    let distance = metric.distance(&a, &b).unwrap();
    let similarity = metric.similarity(&a, &b).unwrap();
    assert!((distance - 2.0).abs() < f32::EPSILON);
    assert!((similarity - (-1.0)).abs() < f32::EPSILON);
    println!("   ✅ PASS: Cosine metric works (identical=1.0, orthogonal=0.0, opposite=-1.0)");
}

#[test]
fn test_euclidean_metric() {
    println!("\n📏 TEST: Euclidean distance metric");
    let metric = EuclideanMetric::new();
    
    println!("   🔍 Testing identical vectors (distance=0)...");
    let a = vec![1.0, 0.0, 0.0];
    let b = vec![1.0, 0.0, 0.0];
    let distance = metric.distance(&a, &b).unwrap();
    assert!((distance - 0.0).abs() < f32::EPSILON);
    
    println!("   🔍 Testing unit distance...");
    let a = vec![1.0, 0.0, 0.0];
    let b = vec![0.0, 0.0, 0.0];
    let distance = metric.distance(&a, &b).unwrap();
    assert!((distance - 1.0).abs() < f32::EPSILON);
    
    println!("   🔍 Testing diagonal (√2)...");
    let a = vec![0.0, 0.0];
    let b = vec![1.0, 1.0];
    let distance = metric.distance(&a, &b).unwrap();
    assert!((distance - 2.0f32.sqrt()).abs() < f32::EPSILON);
    println!("   ✅ PASS: Euclidean metric works");
}

#[test]
fn test_manhattan_metric() {
    println!("\n📍 TEST: Manhattan (L1) distance metric");
    let metric = ManhattanMetric::new();
    
    println!("   🔍 Testing Manhattan distance...");
    let a = vec![1.0, 0.0, 0.0];
    let b = vec![1.0, 0.0, 0.0];
    let distance = metric.distance(&a, &b).unwrap();
    assert!((distance - 0.0).abs() < f32::EPSILON);
    
    let a = vec![1.0, 0.0, 0.0];
    let b = vec![0.0, 0.0, 0.0];
    let distance = metric.distance(&a, &b).unwrap();
    assert!((distance - 1.0).abs() < f32::EPSILON);
    
    let a = vec![0.0, 0.0];
    let b = vec![1.0, 1.0];
    let distance = metric.distance(&a, &b).unwrap();
    assert!((distance - 2.0).abs() < f32::EPSILON);
    println!("   ✅ PASS: Manhattan metric (L1 norm) works");
}

#[test]
fn test_dot_product_metric() {
    println!("\n⚡ TEST: Dot product similarity metric");
    let metric = DotProductMetric::new();
    
    println!("   🔍 Testing identical vectors...");
    let a = vec![1.0, 0.0, 0.0];
    let b = vec![1.0, 0.0, 0.0];
    let distance = metric.distance(&a, &b).unwrap();
    let similarity = metric.similarity(&a, &b).unwrap();
    assert!((distance - (-1.0)).abs() < f32::EPSILON);
    assert!((similarity - 1.0).abs() < f32::EPSILON);
    
    println!("   🔍 Testing orthogonal vectors...");
    let a = vec![1.0, 0.0, 0.0];
    let b = vec![0.0, 1.0, 0.0];
    let distance = metric.distance(&a, &b).unwrap();
    let similarity = metric.similarity(&a, &b).unwrap();
    assert!((distance - 0.0).abs() < f32::EPSILON);
    assert!((similarity - 0.0).abs() < f32::EPSILON);
    println!("   ✅ PASS: Dot product metric works");
}

#[test]
fn test_jaccard_metric() {
    println!("\n🎯 TEST: Jaccard similarity (set overlap)");
    let metric = JaccardMetric::new();
    
    println!("   🔍 Testing identical binary vectors...");
    let a = vec![1.0, 0.0, 1.0, 0.0];
    let b = vec![1.0, 0.0, 1.0, 0.0];
    let distance = metric.distance(&a, &b).unwrap();
    let similarity = metric.similarity(&a, &b).unwrap();
    assert!((distance - 0.0).abs() < f32::EPSILON);
    assert!((similarity - 1.0).abs() < f32::EPSILON);
    
    println!("   🔍 Testing completely different vectors...");
    let a = vec![1.0, 1.0, 1.0, 1.0];
    let b = vec![0.0, 0.0, 0.0, 0.0];
    let distance = metric.distance(&a, &b).unwrap();
    let similarity = metric.similarity(&a, &b).unwrap();
    assert!((distance - 1.0).abs() < f32::EPSILON);
    assert!((similarity - 0.0).abs() < f32::EPSILON);
    
    println!("   🔍 Testing with custom threshold...");
    let metric = JaccardMetric::with_threshold(0.7);
    let a = vec![0.8, 0.6, 0.9, 0.4];
    let b = vec![0.7, 0.5, 0.8, 0.3];
    let distance = metric.distance(&a, &b).unwrap();
    let similarity = metric.similarity(&a, &b).unwrap();
    assert!((distance - 0.0).abs() < f32::EPSILON);
    assert!((similarity - 1.0).abs() < f32::EPSILON);
    println!("   ✅ PASS: Jaccard metric works");
}

#[test]
fn test_hamming_metric() {
    println!("\n🔢 TEST: Hamming distance (bit differences)");
    let metric = HammingMetric::new();
    
    println!("   🔍 Testing identical binary vectors...");
    let a = vec![1.0, 0.0, 1.0, 0.0];
    let b = vec![1.0, 0.0, 1.0, 0.0];
    let distance = metric.distance(&a, &b).unwrap();
    let similarity = metric.similarity(&a, &b).unwrap();
    assert!((distance - 0.0).abs() < f32::EPSILON);
    assert!((similarity - 1.0).abs() < f32::EPSILON);
    
    println!("   🔍 Testing completely different vectors...");
    let a = vec![1.0, 1.0, 1.0, 1.0];
    let b = vec![0.0, 0.0, 0.0, 0.0];
    let distance = metric.distance(&a, &b).unwrap();
    let similarity = metric.similarity(&a, &b).unwrap();
    assert!((distance - 4.0).abs() < f32::EPSILON);
    assert!((similarity - 0.0).abs() < f32::EPSILON);
    
    println!("   🔍 Testing with custom threshold...");
    let metric = HammingMetric::with_threshold(0.7);
    let a = vec![0.8, 0.6, 0.9, 0.4];
    let b = vec![0.7, 0.5, 0.8, 0.3];
    let distance = metric.distance(&a, &b).unwrap();
    let similarity = metric.similarity(&a, &b).unwrap();
    assert!((distance - 0.0).abs() < f32::EPSILON);
    assert!((similarity - 1.0).abs() < f32::EPSILON);
    println!("   ✅ PASS: Hamming metric works");
}

#[test]
fn test_dynamic_metric() {
    println!("\n🔄 TEST: Dynamic metric dispatch (runtime selection)");
    
    println!("   🔍 Testing cosine (dynamic)...");
    let metric = DynamicDistanceMetric::from_name("cosine").unwrap();
    let a = vec![1.0, 0.0, 0.0];
    let b = vec![1.0, 0.0, 0.0];
    let distance = metric.distance(&a, &b).unwrap();
    assert!((distance - 0.0).abs() < f32::EPSILON);
    
    println!("   🔍 Testing euclidean (dynamic)...");
    let metric = DynamicDistanceMetric::from_name("euclidean").unwrap();
    let distance = metric.distance(&a, &b).unwrap();
    assert!((distance - 0.0).abs() < f32::EPSILON);
    
    println!("   🔍 Testing jaccard with params (dynamic)...");
    let metric = DynamicDistanceMetric::from_name_with_params("jaccard", Some(0.7)).unwrap();
    let a = vec![0.8, 0.6, 0.9, 0.4];
    let b = vec![0.7, 0.5, 0.8, 0.3];
    let distance = metric.distance(&a, &b).unwrap();
    assert!((distance - 0.0).abs() < f32::EPSILON);
    println!("   ✅ PASS: Dynamic dispatch works for all metrics");
}

#[test]
fn test_binary_helpers() {
    println!("\n🔁 TEST: Binary conversion helpers");
    let jaccard = JaccardMetric::with_threshold(0.5);
    let hamming = HammingMetric::with_threshold(0.5);
    
    println!("   🔍 Testing threshold binarization...");
    let vec = vec![0.2, 0.6, 0.8, 0.4];
    let expected = vec![false, true, true, false];
    
    assert_eq!(jaccard.to_binary(&vec), expected);
    assert_eq!(hamming.to_binary(&vec), expected);
    println!("   ✅ PASS: Binary conversion works (threshold=0.5)");
}

