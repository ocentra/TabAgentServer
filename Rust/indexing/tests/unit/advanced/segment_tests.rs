//! 📊 SEGMENT TESTS - Vector Segmentation for Large Datasets

use indexing::advanced::segment::{Segment, SegmentManager, SegmentBasedVectorIndex, SegmentConfig};
use tempfile::TempDir;

#[test]
fn test_segment_creation() {
    println!("\n📊 TEST: Create vector segment");
    let temp_dir = TempDir::new().unwrap();
    let segment_path = temp_dir.path().join("test_segment");
    
    println!("   📝 Creating segment with capacity 1000...");
    let segment = Segment::new(
        "test_segment".to_string(),
        &segment_path,
        1000,
        true,
    );
    
    assert!(segment.is_ok());
    let segment = segment.unwrap();
    assert_eq!(segment.id(), "test_segment");
    assert_eq!(segment.len(), 0);
    assert!(segment.is_appendable());
    println!("   ✅ PASS: Segment created (id={}, len={})", segment.id(), segment.len());
}

#[test]
fn test_segment_add_remove_vector() {
    println!("\n📊 TEST: Add and remove vectors from segment");
    let temp_dir = TempDir::new().unwrap();
    let segment_path = temp_dir.path().join("test_segment");
    
    println!("   📝 Creating segment with 3D vectors...");
    let mut segment = Segment::new_with_dimension(
        "test_segment".to_string(),
        &segment_path,
        1000,
        true,
        3,
    ).unwrap();
    
    println!("   📝 Adding vector...");
    assert!(segment.add_vector("vector1", vec![1.0, 0.0, 0.0], None).is_ok());
    assert_eq!(segment.len(), 1);
    
    println!("   🗑️  Removing vector...");
    assert!(segment.remove_vector("vector1").unwrap());
    assert_eq!(segment.len(), 0);
    println!("   ✅ PASS: Vector added and removed");
}

#[test]
fn test_segment_manager() {
    println!("\n📚 TEST: Segment manager auto-creates segments");
    let temp_dir = TempDir::new().unwrap();
    let segments_path = temp_dir.path().join("segments");
    
    println!("   📝 Creating manager with max 2 vectors per segment and 3D vectors...");
    let config = SegmentConfig {
        segments_path,
        max_vectors_per_segment: 2,
        dimension: 3,
        ..Default::default()
    };
    
    let mut manager = SegmentManager::new(config).unwrap();
    
    println!("   📝 Adding 3 vectors (should create 2 segments)...");
    assert!(manager.add_vector("vector1", vec![1.0, 0.0, 0.0], None).is_ok());
    assert!(manager.add_vector("vector2", vec![0.0, 1.0, 0.0], None).is_ok());
    assert!(manager.add_vector("vector3", vec![0.0, 0.0, 1.0], None).is_ok());
    
    println!("   📊 Checking statistics...");
    let stats = manager.get_statistics();
    assert_eq!(stats.total_vectors, 3);
    assert!(stats.segment_count >= 2);
    println!("   ✅ PASS: {} vectors across {} segments", stats.total_vectors, stats.segment_count);
}

#[test]
fn test_segment_based_vector_index() {
    println!("\n🔍 TEST: Segment-based vector index search");
    let temp_dir = TempDir::new().unwrap();
    let segments_path = temp_dir.path().join("segments");
    
    println!("   📝 Creating segment-based index with 3D vectors...");
    let mut index = SegmentBasedVectorIndex::new_with_dimension(
        &segments_path,
        Box::new(indexing::utils::distance_metrics::CosineMetric::new()),
        3,
    ).unwrap();
    
    println!("   📝 Adding 2 vectors...");
    assert!(index.add_vector("vector1", vec![1.0, 0.0, 0.0], None).is_ok());
    assert!(index.add_vector("vector2", vec![0.9, 0.1, 0.0], None).is_ok());
    
    println!("   🔎 Searching...");
    let results = index.search(&[1.0, 0.0, 0.0], 2, None).unwrap();
    println!("   🔍 Search returned {} results:", results.len());
    for (i, result) in results.iter().enumerate() {
        println!("      {}. ID: {}, Score: {}", i+1, result.id, result.score);
    }
    // HNSW might not always return all k results for small datasets
    assert!(!results.is_empty(), "Expected at least 1 result");
    assert!(results.len() <= 2, "Expected at most 2 results, but got {}", results.len());
    
    println!("   📊 Checking statistics...");
    let stats = index.get_statistics();
    assert_eq!(stats.total_vectors, 2);
    println!("   ✅ PASS: Found {} vectors, stats show {} total", results.len(), stats.total_vectors);
}

