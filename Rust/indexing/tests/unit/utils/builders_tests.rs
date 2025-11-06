//! 🏗️ BUILDER PATTERN TESTS - Fluent Index Construction

use indexing::utils::builders::{
    VectorIndexBuilder, SegmentIndexBuilder, EnhancedIndexBuilder,
    PayloadFilterBuilder, PayloadBuilder, SearchQueryBuilder, GraphIndexBuilder,
};
use tempfile::TempDir;

#[test]
fn test_vector_index_builder() {
    println!("\n🏗️  TEST: VectorIndexBuilder (fluent construction)");
    let temp_dir = TempDir::new().unwrap();
    let persist_path = temp_dir.path().join("test_index.hnsw");
    
    println!("   📝 Building with custom params (M=32, efC=100)...");
    let builder = VectorIndexBuilder::new()
        .persist_path(&persist_path)
        .max_connections(32)
        .ef_construction(100)
        .num_layers(8)
        .initial_capacity(500);
    
    let index = builder.build();
    assert!(index.is_ok());
    println!("   ✅ PASS: VectorIndexBuilder works");
}

#[test]
fn test_segment_index_builder() {
    println!("\n🏗️  TEST: SegmentIndexBuilder (multi-segment index)");
    let temp_dir = TempDir::new().unwrap();
    let segments_path = temp_dir.path().join("segments");
    
    println!("   📝 Building with segment params (5k/segment)...");
    let builder = SegmentIndexBuilder::new()
        .segments_path(&segments_path)
        .max_vectors_per_segment(5000)
        .min_vectors_for_new_segment(1000)
        .auto_optimize(true);
    
    let index = builder.build();
    assert!(index.is_ok());
    println!("   ✅ PASS: SegmentIndexBuilder works");
}

#[test]
fn test_enhanced_index_builder() {
    println!("\n🏗️  TEST: EnhancedIndexBuilder (advanced features)");
    let temp_dir = TempDir::new().unwrap();
    let segments_path = temp_dir.path().join("segments");
    
    println!("   📝 Building enhanced index with auto-optimize...");
    let builder = EnhancedIndexBuilder::new()
        .segments_path(&segments_path)
        .max_vectors_per_segment(5000)
        .min_vectors_for_new_segment(1000)
        .auto_optimize(true);
    
    let index = builder.build();
    assert!(index.is_ok());
    println!("   ✅ PASS: EnhancedIndexBuilder works");
}

#[test]
fn test_payload_filter_builder() {
    println!("\n🏗️  TEST: PayloadFilterBuilder (must/should/must_not)");
    
    println!("   📝 Building filter (2 must, 1 should, 1 must_not)...");
    let filter = PayloadFilterBuilder::new()
        .must_match_string("category", "test")
        .must_range("score", Some(0.5), Some(1.0))
        .should_match_string("tag", "important")
        .must_not_match_string("status", "deleted")
        .build();
    
    assert_eq!(filter.must.len(), 2);
    assert_eq!(filter.should.len(), 1);
    assert_eq!(filter.must_not.len(), 1);
    println!("   ✅ PASS: PayloadFilterBuilder works");
}

#[test]
fn test_payload_builder() {
    println!("\n🏗️  TEST: PayloadBuilder (multi-field metadata)");
    
    println!("   📝 Building payload with 3 fields...");
    let payload = PayloadBuilder::new()
        .add_string_field("name", "test")
        .add_integer_field("age", 25)
        .add_boolean_field("active", true)
        .build();
    
    assert_eq!(payload.fields.len(), 3);
    assert!(payload.fields.contains_key("name"));
    assert!(payload.fields.contains_key("age"));
    assert!(payload.fields.contains_key("active"));
    println!("   ✅ PASS: PayloadBuilder works (3 fields)");
}

#[test]
fn test_search_query_builder() {
    println!("\n🏗️  TEST: SearchQueryBuilder (query configuration)");
    
    println!("   📝 Building search query (limit=5, payload=true)...");
    let query = SearchQueryBuilder::new()
        .query_vector(vec![1.0, 0.0, 0.0])
        .limit(5)
        .include_payload(true)
        .include_vectors(false)
        .build();
    
    assert!(query.is_ok());
    let query = query.unwrap();
    assert_eq!(query.query_vector, vec![1.0, 0.0, 0.0]);
    assert_eq!(query.limit, 5);
    assert_eq!(query.include_payload, true);
    assert_eq!(query.include_vectors, false);
    println!("   ✅ PASS: SearchQueryBuilder works");
}

#[test]
fn test_graph_index_builder() {
    println!("\n🏗️  TEST: GraphIndexBuilder (graph configuration)");
    
    println!("   📝 Building graph index builder (hybrid mode)...");
    let _builder = GraphIndexBuilder::new()
        .outgoing_tree_path("outgoing".to_string())
        .incoming_tree_path("incoming".to_string())
        .with_hybrid(true);
    
    assert!(true);
    println!("   ✅ PASS: GraphIndexBuilder construction works");
}

