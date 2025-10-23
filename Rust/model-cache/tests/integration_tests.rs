/// Integration tests for model-cache
/// 
/// REAL TESTS - NO MOCKS:
/// - Downloads actual models from HuggingFace
/// - Tests real chunking and storage
/// - Uses real sled database
/// - Cleans up with tempfile
/// 
/// Test models defined in test_models.rs module

mod test_models;

use tabagent_model_cache::{ModelCache, ModelDownloader, ChunkStorage, ModelCatalog, ModelCatalogEntry};
use tempfile::TempDir;
use std::path::PathBuf;
use test_models::*;

#[tokio::test]
async fn test_real_download_and_cache() {
    println!("\n🧪 Testing REAL model download and caching...");
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cache = ModelCache::new(temp_dir.path()).expect("Failed to create cache");
    
    println!("📥 Downloading REAL model: {}/{}", TINY_GGUF_REPO, TINY_GGUF_FILE);
    println!("   (This is a real {}MB download - be patient!)", TINY_GGUF_SIZE_MB);
    
    // REAL download with progress
    let result = cache.download_and_cache(
        TINY_GGUF_REPO,
        TINY_GGUF_FILE,
        Some(Box::new(|downloaded, total| {
            if total > 0 {
                let percent = (downloaded as f64 / total as f64) * 100.0;
                if downloaded % (5 * 1024 * 1024) == 0 { // Print every 5MB
                    println!("   Progress: {:.1}% ({} MB / {} MB)", 
                        percent, 
                        downloaded / (1024 * 1024),
                        total / (1024 * 1024)
                    );
                }
            }
        }))
    ).await;
    
    assert!(result.is_ok(), "Real download failed: {:?}", result.err());
    println!("✅ Real model downloaded successfully");
    
    // Verify file is actually cached
    let cached = cache.get_file(TINY_GGUF_REPO, TINY_GGUF_FILE)
        .expect("Failed to get cached file");
    assert!(cached.is_some(), "Model not found in cache after download");
    
    let file_data = cached.expect("Cached file should exist");
    assert!(file_data.len() > 1_000_000, "Downloaded file too small ({}B) - not a real model", file_data.len());
    println!("✅ Model in cache: {} MB", file_data.len() / (1024 * 1024));
    
    // Test streaming (zero-copy read)
    let stream = cache.stream_file_chunks(TINY_GGUF_REPO, TINY_GGUF_FILE)
        .expect("Failed to create stream");
    assert!(stream.is_some(), "Stream should exist for cached model");
    
    let mut chunk_count = 0;
    let mut total_bytes = 0u64;
    
    for chunk_result in stream.expect("Stream should be available") {
        let chunk = chunk_result.expect("Failed to read chunk");
        total_bytes += chunk.len() as u64;
        chunk_count += 1;
    }
    
    assert!(chunk_count > 0, "Should have streamed at least one chunk");
    assert_eq!(total_bytes as usize, file_data.len(), "Streamed size mismatch");
    println!("✅ Streaming works: {} chunks, {} MB", chunk_count, total_bytes / (1024 * 1024));
}

#[tokio::test]
async fn test_real_manifest_scan() {
    println!("\n🧪 Testing REAL manifest scan...");
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cache = ModelCache::new(temp_dir.path()).expect("Failed to create cache");
    
    println!("🔍 Scanning REAL repo: {}", TINY_GGUF_REPO);
    
    let manifest = cache.scan_repo(TINY_GGUF_REPO).await;
    assert!(manifest.is_ok(), "Real repo scan failed: {:?}", manifest.err());
    
    let manifest = manifest.expect("Manifest should be available after scan");
    assert!(!manifest.files.is_empty(), "Should find files in real repo");
    
    println!("✅ Found {} file groups:", manifest.files.len());
    for (variant, files) in &manifest.files {
        println!("   - {}: {} files", variant, files.len());
    }
}

#[test]
fn test_real_chunking() {
    println!("\n🧪 Testing REAL chunk storage...");
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let storage = ChunkStorage::new(temp_dir.path()).expect("Failed to create storage");
    
    // Create a realistic "model file" (10MB of pseudo-random data)
    let mut fake_model = Vec::with_capacity(10 * 1024 * 1024);
    for i in 0..(10 * 1024 * 1024) {
        fake_model.push((i % 256) as u8);
    }
    
    println!("💾 Storing 10MB pseudo-model in chunks...");
    
    let result = storage.store_file("test/repo", "model.gguf", &fake_model);
    assert!(result.is_ok(), "Failed to store file: {:?}", result.err());
    
    println!("✅ Stored in {} chunks", (fake_model.len() + 5 * 1024 * 1024 - 1) / (5 * 1024 * 1024));
    
    // Retrieve and verify
    let retrieved = storage.get_file("test/repo", "model.gguf")
        .expect("Failed to get file");
    assert!(retrieved.is_some(), "File not found after storage");
    
    let retrieved_data = retrieved.expect("Retrieved data should be available");
    assert_eq!(retrieved_data.len(), fake_model.len(), "Size mismatch after retrieval");
    assert_eq!(retrieved_data, fake_model, "Data corrupted during storage/retrieval");
    
    println!("✅ Retrieved correctly: {} MB", retrieved_data.len() / (1024 * 1024));
}

#[test]
fn test_catalog_crud() {
    println!("\n🧪 Testing catalog CRUD operations...");
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let catalog = ModelCatalog::open(temp_dir.path()).expect("Failed to open catalog");
    
    // Add a test model entry
    let test_model = ModelCatalogEntry {
        id: "test-model".to_string(),
        name: "Test Model".to_string(),
        repo_id: TINY_GGUF_REPO.to_string(),
        file_path: Some(TINY_GGUF_FILE.to_string()),
        model_type: "gguf".to_string(),
        size_gb: (TINY_GGUF_SIZE_MB as f64) / 1024.0,
        tags: vec!["test".to_string()],
        suggested: false,
        downloaded: false,
        description: Some("Test model for unit tests".to_string()),
        default_quant: Some("q4_k_m".to_string()),
        source: Some("test".to_string()),
        requires_token: Some(false),
    };
    
    println!("➕ Adding test model to catalog...");
    catalog.insert_model(test_model.clone()).expect("Failed to insert model");
    
    // Read it back
    println!("🔍 Reading model from catalog...");
    let retrieved = catalog.get_model("test-model").expect("Failed to get model");
    assert!(retrieved.is_some(), "Model not found after insert");
    
    let retrieved_model = retrieved.expect("Retrieved model should be available");
    assert_eq!(retrieved_model.id, test_model.id);
    assert_eq!(retrieved_model.repo_id, test_model.repo_id);
    println!("✅ Model retrieved correctly");
    
    // Update download status
    println!("📝 Marking as downloaded...");
    catalog.mark_downloaded("test-model", true).expect("Failed to mark downloaded");
    
    let updated = catalog.get_model("test-model").expect("Failed to get model");
    assert!(updated.expect("Updated model should be available").downloaded, "Download status not updated");
    println!("✅ Status updated correctly");
    
    // Query by type
    println!("🔍 Querying by type...");
    let gguf_models = catalog.get_models_by_type("gguf").expect("Failed to query by type");
    assert!(gguf_models.iter().any(|m| m.id == "test-model"), "Model not found in type query");
    println!("✅ Type query works: found {} gguf models", gguf_models.len());
    
    // Delete
    println!("🗑️  Deleting model...");
    catalog.delete_model("test-model").expect("Failed to delete model");
    
    let deleted = catalog.get_model("test-model").expect("Failed to check deletion");
    assert!(deleted.is_none(), "Model still exists after deletion");
    println!("✅ Model deleted successfully");
}

#[tokio::test]
#[ignore] // Only run with: cargo test --package tabagent-model-cache --test integration_tests -- --ignored
async fn test_large_model_download() {
    println!("\n🧪 Testing LARGE model download (TinyLlama 1.1B ~600MB)...");
    println!("⚠️  This test downloads 600MB - run with --ignored flag");
    
    const LARGE_MODEL_REPO: &str = "TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF";
    const LARGE_MODEL_FILE: &str = "tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf";
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cache = ModelCache::new(temp_dir.path()).expect("Failed to create cache");
    
    println!("📥 Downloading large REAL model...");
    
    let start = std::time::Instant::now();
    let result = cache.download_and_cache(
        LARGE_MODEL_REPO,
        LARGE_MODEL_FILE,
        Some(Box::new(|downloaded, total| {
            if total > 0 && downloaded % (10 * 1024 * 1024) == 0 {
                let percent = (downloaded as f64 / total as f64) * 100.0;
                println!("   {:.1}% - {} MB / {} MB", 
                    percent, 
                    downloaded / (1024 * 1024),
                    total / (1024 * 1024)
                );
            }
        }))
    ).await;
    
    assert!(result.is_ok(), "Large model download failed: {:?}", result.err());
    let duration = start.elapsed();
    
    println!("✅ Downloaded 600MB model in {:.2}s", duration.as_secs_f64());
    
    // Verify it's actually there and correct size
    let cached = cache.get_file(LARGE_MODEL_REPO, LARGE_MODEL_FILE)
        .expect("Failed to get cached file");
    assert!(cached.is_some());
    
    let size_mb = cached.expect("Cached file should exist").len() / (1024 * 1024);
    assert!(size_mb > 500 && size_mb < 700, "Size {} MB seems wrong for this model", size_mb);
    println!("✅ Verified: {} MB in cache", size_mb);
}

