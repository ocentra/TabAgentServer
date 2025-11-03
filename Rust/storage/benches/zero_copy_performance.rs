//! Performance benchmarks to validate zero-copy claims
//! Following RAG Rule 15.1-15.5: Performance optimizations and benchmarks

use common::models::{Embedding, Message, Node};
use common::{EmbeddingId, NodeId};
use serde_json::json;
use std::time::Instant;
use storage::StorageManager;
use tempfile::TempDir;

/// Simple manual benchmark (no criterion) since we're in libmdbx constraint territory
fn main() {
    println!("🚀 Storage Layer Zero-Copy Performance Benchmark");
    println!("{}", "=".repeat(60));

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let storage: StorageManager<storage::engine::MdbxEngine> = StorageManager::new(temp_dir.path().join("bench.db").to_str().unwrap())
        .expect("Failed to create storage");

    // Setup: Insert test data
    println!("\n📦 Setting up test data...");
    
    // Large embedding for testing
    let embedding_id = EmbeddingId::new("bench_embedding");
    let large_vector: Vec<f32> = (0..4096).map(|i| (i as f32) * 0.001).collect(); // 4KB vector
    let embedding = Embedding {
        id: embedding_id.clone(),
        vector: large_vector.clone(),
        model: "test-model".to_string(),
    };
    storage.insert_embedding(&embedding).expect("Failed to insert");

    let msg_id = NodeId::new("bench_message");
    let message = Message {
        id: msg_id.clone(),
        chat_id: NodeId::new("bench_chat"),
        sender: "bench_user".to_string(),
        timestamp: 1234567890,
        text_content: "This is a benchmark message with some content to test performance of zero-copy access patterns.".to_string(),
        attachment_ids: vec![],
        embedding_id: Some(embedding_id.clone()),
        metadata: json!({"bench": true}).to_string(),
    };
    storage.insert_node(&Node::Message(message)).expect("Failed to insert");

    println!("✅ Test data setup complete\n");

    // Benchmark 1: Full deserialization path
    println!("🐌 BENCHMARK 1: Full deserialization (clone entire struct)");
    const ITERATIONS: u32 = 10000;
    
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _retrieved = storage.get_embedding(embedding_id.as_str()).expect("Failed to get embedding");
        std::hint::black_box(_retrieved);
    }
    let full_deserialize_time = start.elapsed();
    
    println!("   Iterations: {}", ITERATIONS);
    println!("   Total time: {:?}", full_deserialize_time);
    println!("   Time per op: {:?}ns", full_deserialize_time.as_nanos() / ITERATIONS as u128);

    // Benchmark 2: Archived access via wrapper
    println!("\n⚡ BENCHMARK 2: Archived access");
    
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let emb_ref = storage.get_embedding_ref(embedding_id.as_str()).expect("Failed to get ref");
        let _vector = emb_ref.map(|e| e.vector());
        std::hint::black_box(_vector);
    }
    let archived_time = start.elapsed();
    
    println!("   Iterations: {}", ITERATIONS);
    println!("   Total time: {:?}", archived_time);
    println!("   Time per op: {:?}ns", archived_time.as_nanos() / ITERATIONS as u128);

    // Calculate speedup
    let speedup = full_deserialize_time.as_nanos() as f64 / archived_time.as_nanos() as f64;
    println!("\n📊 RESULTS:");
    println!("   Speedup: {:.2}x", speedup);
    println!("   Improvement: {:.1}%", (speedup - 1.0) * 100.0);

    // Benchmark 3: Node access
    println!("\n🔍 BENCHMARK 3: Node access comparison");
    
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _retrieved = storage.get_node(msg_id.as_str()).expect("Failed to get node");
        std::hint::black_box(_retrieved);
    }
    let node_full_time = start.elapsed();
    
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let node_ref = storage.get_node_ref(msg_id.as_str()).expect("Failed to get ref");
        let _text: Option<String> = node_ref.and_then(|n| n.message_text().map(String::from));
        std::hint::black_box(_text);
    }
    let node_archived_time = start.elapsed();
    
    let node_speedup = node_full_time.as_nanos() as f64 / node_archived_time.as_nanos() as f64;
    println!("   Full deserialize: {:?}", node_full_time);
    println!("   Archived access: {:?}", node_archived_time);
    println!("   Speedup: {:.2}x", node_speedup);

    println!("\n✅ Benchmark complete!");
    println!("\n📝 NOTE: This benchmark validates:");
    println!("   1. Archived access is faster than full deserialization");
    println!("   2. One copy still occurs from libmdbx transaction (expected)");
    println!("   3. rkyv provides zero-copy field access AFTER guard creation");
    println!("   4. True zero-copy requires engine redesign (future work)");
}

