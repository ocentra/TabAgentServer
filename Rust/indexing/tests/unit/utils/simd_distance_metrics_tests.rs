//! 🚀 SIMD DISTANCE METRIC TESTS - Vectorized Similarity Functions

use indexing::utils::simd_distance_metrics::{
    SimdCosineMetric, SimdEuclideanMetric, SimdManhattanMetric, SimdDotProductMetric,
    SimdDistanceMetric,
};

#[test]
fn test_simd_cosine_metric() {
    println!("\n🚀 TEST: SIMD cosine metric (vectorized)");
    let metric = SimdCosineMetric::new();
    
    println!("   🔍 Testing SIMD cosine (identical + orthogonal)...");
    let a = vec![1.0, 0.0, 0.0];
    let b = vec![1.0, 0.0, 0.0];
    let distance = metric.simd_distance(&a, &b).unwrap();
    let similarity = metric.simd_similarity(&a, &b).unwrap();
    assert!((distance - 0.0).abs() < f32::EPSILON);
    assert!((similarity - 1.0).abs() < f32::EPSILON);
    
    let a = vec![1.0, 0.0, 0.0];
    let b = vec![0.0, 1.0, 0.0];
    let distance = metric.simd_distance(&a, &b).unwrap();
    let similarity = metric.simd_similarity(&a, &b).unwrap();
    assert!((distance - 1.0).abs() < f32::EPSILON);
    assert!((similarity - 0.0).abs() < f32::EPSILON);
    println!("   ✅ PASS: SIMD cosine metric works");
}

#[test]
fn test_simd_euclidean_metric() {
    println!("\n🚀 TEST: SIMD euclidean metric (vectorized)");
    let metric = SimdEuclideanMetric::new();
    
    println!("   🔍 Testing SIMD euclidean...");
    let a = vec![1.0, 0.0, 0.0];
    let b = vec![1.0, 0.0, 0.0];
    let distance = metric.simd_distance(&a, &b).unwrap();
    assert!((distance - 0.0).abs() < f32::EPSILON);
    
    let a = vec![1.0, 0.0, 0.0];
    let b = vec![0.0, 0.0, 0.0];
    let distance = metric.simd_distance(&a, &b).unwrap();
    assert!((distance - 1.0).abs() < f32::EPSILON);
    println!("   ✅ PASS: SIMD euclidean metric works");
}

#[test]
fn test_simd_manhattan_metric() {
    println!("\n🚀 TEST: SIMD manhattan metric (vectorized)");
    let metric = SimdManhattanMetric::new();
    
    println!("   🔍 Testing SIMD manhattan...");
    let a = vec![1.0, 0.0, 0.0];
    let b = vec![1.0, 0.0, 0.0];
    let distance = metric.simd_distance(&a, &b).unwrap();
    assert!((distance - 0.0).abs() < f32::EPSILON);
    
    let a = vec![1.0, 0.0, 0.0];
    let b = vec![0.0, 0.0, 0.0];
    let distance = metric.simd_distance(&a, &b).unwrap();
    assert!((distance - 1.0).abs() < f32::EPSILON);
    println!("   ✅ PASS: SIMD manhattan metric works");
}

#[test]
fn test_simd_dot_product_metric() {
    println!("\n🚀 TEST: SIMD dot product metric (vectorized)");
    let metric = SimdDotProductMetric::new();
    
    println!("   🔍 Testing SIMD dot product...");
    let a = vec![1.0, 0.0, 0.0];
    let b = vec![1.0, 0.0, 0.0];
    let distance = metric.simd_distance(&a, &b).unwrap();
    let similarity = metric.simd_similarity(&a, &b).unwrap();
    assert!((distance - (-1.0)).abs() < f32::EPSILON);
    assert!((similarity - 1.0).abs() < f32::EPSILON);
    println!("   ✅ PASS: SIMD dot product metric works");
}

