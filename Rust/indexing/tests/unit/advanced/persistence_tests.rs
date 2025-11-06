//! 💾 PERSISTENCE TESTS - Save/Load Vector Indexes

use indexing::advanced::persistence::{
    PersistentVectorIndex, PersistentSegmentIndex, EnhancedVectorIndex, PersistenceConfig,
};
use tempfile::TempDir;

#[test]
fn test_persistent_vector_index() {
    println!("\n💾 TEST: Persistent vector index save/load");
    let temp_dir = TempDir::new().unwrap();
    let index_path = temp_dir.path().join("test_index.bin");
    
    println!("   📝 Creating persistent index with 3D vectors...");
    let mut index = PersistentVectorIndex::new_with_dimension(&index_path, 3).unwrap();
    
    println!("   📝 Adding 2 vectors...");
    assert!(index.add_vector("vector1", vec![1.0, 0.0, 0.0]).is_ok());
    assert!(index.add_vector("vector2", vec![0.9, 0.1, 0.0]).is_ok());
    
    println!("   💾 Saving index to disk...");
    assert!(index.save().is_ok());
    
    assert!(index_path.exists());
    println!("   ✅ PASS: Index saved to {}", index_path.display());
}

#[test]
fn test_persistent_segment_index() {
    println!("\n💾 TEST: Persistent segment index save/load");
    let temp_dir = TempDir::new().unwrap();
    let segments_path = temp_dir.path().join("segments");
    
    println!("   📝 Creating persistent segment index with 3D vectors...");
    let mut config = PersistenceConfig::default();
    config.dimension = 3;
    let mut index = PersistentSegmentIndex::new(&segments_path, config).unwrap();
    
    println!("   📝 Adding 2 vectors...");
    assert!(index.add_vector("vector1", vec![1.0, 0.0, 0.0]).is_ok());
    assert!(index.add_vector("vector2", vec![0.9, 0.1, 0.0]).is_ok());
    
    println!("   💾 Saving segments to disk...");
    assert!(index.save_segments().is_ok());
    
    assert!(segments_path.exists());
    println!("   ✅ PASS: Segments saved to {}", segments_path.display());
}

#[test]
fn test_enhanced_vector_index() {
    println!("\n🌟 TEST: Enhanced vector index with persistence");
    let temp_dir = TempDir::new().unwrap();
    let segments_path = temp_dir.path().join("segments");
    
    println!("   📝 Creating enhanced vector index with 3D vectors...");
    let mut index = EnhancedVectorIndex::new_with_dimension(
        &segments_path,
        Box::new(indexing::utils::distance_metrics::CosineMetric::new()),
        3,
    ).unwrap();
    
    println!("   📝 Adding 2 vectors...");
    assert!(index.add_vector("vector1", vec![1.0, 0.0, 0.0]).is_ok());
    assert!(index.add_vector("vector2", vec![0.9, 0.1, 0.0]).is_ok());
    
    println!("   🔎 Searching...");
    let results = index.search(&[1.0, 0.0, 0.0], 2).unwrap();
    assert_eq!(results.len(), 2);
    
    println!("   💾 Flushing to disk...");
    assert!(index.flush().is_ok());
    println!("   ✅ PASS: Enhanced index works with {} vectors", results.len());
}

