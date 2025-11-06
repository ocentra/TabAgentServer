//! 📦 PAYLOAD INDEX TESTS - Metadata Filtering

use indexing::advanced::payload_index::{
    Payload, PayloadFieldValue, PayloadIndex, HybridVectorIndex,
    PayloadFilter, PayloadCondition,
};
use indexing::EmbeddingId;
use ordered_float::OrderedFloat;
use tempfile::TempDir;

#[test]
fn test_payload_creation() {
    println!("\n📦 TEST: Create payload with mixed field types");
    let mut payload = Payload::new();
    
    println!("   📝 Adding string, integer, boolean fields...");
    payload.add_field("name".to_string(), PayloadFieldValue::String("test".to_string()));
    payload.add_field("age".to_string(), PayloadFieldValue::Integer(25));
    payload.add_field("active".to_string(), PayloadFieldValue::Boolean(true));
    
    assert_eq!(payload.get_field("name"), Some(&PayloadFieldValue::String("test".to_string())));
    assert_eq!(payload.get_field("age"), Some(&PayloadFieldValue::Integer(25)));
    assert_eq!(payload.get_field("active"), Some(&PayloadFieldValue::Boolean(true)));
    println!("   ✅ PASS: Payload created with 3 fields");
}

#[test]
fn test_payload_index() {
    println!("\n🔍 TEST: Filter vectors by payload metadata");
    let mut index = PayloadIndex::new();
    
    let id1 = EmbeddingId::from("vector1");
    let mut payload1 = Payload::new();
    payload1.add_field("category".to_string(), PayloadFieldValue::String("A".to_string()));
    payload1.add_field("score".to_string(), PayloadFieldValue::Float(OrderedFloat(0.8)));
    
    let id2 = EmbeddingId::from("vector2");
    let mut payload2 = Payload::new();
    payload2.add_field("category".to_string(), PayloadFieldValue::String("B".to_string()));
    payload2.add_field("score".to_string(), PayloadFieldValue::Float(OrderedFloat(0.9)));
    
    println!("   📝 Adding 2 vectors with payloads...");
    index.add_payload(id1.clone(), payload1).unwrap();
    index.add_payload(id2.clone(), payload2).unwrap();
    
    println!("   🔍 Filtering by category=A...");
    let mut filter = PayloadFilter::new();
    filter = filter.must(PayloadCondition::Match {
        value: PayloadFieldValue::String("A".to_string()),
    });
    
    let results = index.filter(&filter);
    assert!(results.contains(&id1));
    assert!(!results.contains(&id2));
    println!("   ✅ PASS: Filtered {} vectors by payload", results.len());
}

#[test]
fn test_hybrid_vector_index() {
    println!("\n🔀 TEST: Hybrid vector index with payload filtering");
    let temp_dir = TempDir::new().unwrap();
    let index_path = temp_dir.path().join("test_index");
    
    println!("   📝 Creating hybrid index...");
    let mut index = HybridVectorIndex::new(&index_path).unwrap();
    
    // Use 384D (default dimension) to match VectorIndex configuration
    let mut test_vector = vec![0.0f32; 384];
    test_vector[0] = 1.0;  // Make it non-zero for search
    
    let mut payload = Payload::new();
    payload.add_field("category".to_string(), PayloadFieldValue::String("test".to_string()));
    
    println!("   📝 Adding vector with payload...");
    index.add_vector_with_payload("vector1", test_vector.clone(), payload).unwrap();
    
    println!("   🔎 Searching with filter...");
    let results = index.search_with_filter(&test_vector, 1, None).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, EmbeddingId::from("vector1"));
    println!("   ✅ PASS: Hybrid index works with payloads");
}

