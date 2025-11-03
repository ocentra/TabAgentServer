//! Integration tests for ONNX Runtime inference
//!
//! Tests with real models from HuggingFace:
//! - sentence-transformers/all-MiniLM-L6-v2 (90MB, 384-dim embeddings)
//!
//! These tests download models on first run and cache them.

use tabagent_onnx_loader::{OnnxSession, OnnxError};
use std::path::PathBuf;
use tempfile::TempDir;

/// Model ID for testing
const TEST_MODEL_REPO: &str = "sentence-transformers/all-MiniLM-L6-v2";
const TEST_MODEL_FILE: &str = "onnx/model.onnx";
const TEST_TOKENIZER_FILE: &str = "tokenizer.json";

/// Download a file from HuggingFace using model-cache
async fn download_test_model(repo_id: &str, file_path: &str, cache_dir: &PathBuf) -> Result<PathBuf, Box<dyn std::error::Error>> {
    use tabagent_model_cache::ModelCache;
    
    // Initialize cache
    let registry = std::sync::Arc::new(storage::StorageRegistry::new(cache_dir));
    let cache = ModelCache::new(registry, cache_dir)?;
    
    // Download file (returns path to cached file)
    cache.download_file(repo_id, file_path, None).await?;
    
    // Get the cached file path
    let cached_path = cache.get_file_path(repo_id, file_path).await?
        .ok_or_else(|| format!("Failed to get cached file path for {}/{}", repo_id, file_path))?;
    
    Ok(cached_path)
}

#[tokio::test]
async fn test_load_real_onnx_model() {
    // Create temp directory for caching
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cache_path = temp_dir.path().to_path_buf();
    
    // Download model
    let model_path = download_test_model(TEST_MODEL_REPO, TEST_MODEL_FILE, &cache_path)
        .await
        .expect("Failed to download test model");
    
    // Load ONNX session
    let session = OnnxSession::load(&model_path);
    
    assert!(session.is_ok(), "Failed to load ONNX model: {:?}", session.err());
    let session = session.unwrap();
    
    // Verify model path
    assert_eq!(session.model_path(), model_path.as_path());
}

#[tokio::test]
async fn test_tokenizer_integration() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cache_path = temp_dir.path().to_path_buf();
    
    // Download model and tokenizer
    let model_path = download_test_model(TEST_MODEL_REPO, TEST_MODEL_FILE, &cache_path)
        .await
        .expect("Failed to download test model");
    
    let tokenizer_path = download_test_model(TEST_MODEL_REPO, TEST_TOKENIZER_FILE, &cache_path)
        .await
        .expect("Failed to download tokenizer");
    
    // Load session
    let mut session = OnnxSession::load(&model_path).expect("Failed to load model");
    
    // Load tokenizer
    session.load_tokenizer(&tokenizer_path).expect("Failed to load tokenizer");
    
    assert!(session.has_tokenizer(), "Tokenizer not loaded");
}

#[tokio::test]
async fn test_embedding_generation() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cache_path = temp_dir.path().to_path_buf();
    
    // Download files
    let model_path = download_test_model(TEST_MODEL_REPO, TEST_MODEL_FILE, &cache_path)
        .await
        .expect("Failed to download test model");
    
    let tokenizer_path = download_test_model(TEST_MODEL_REPO, TEST_TOKENIZER_FILE, &cache_path)
        .await
        .expect("Failed to download tokenizer");
    
    // Load session with tokenizer
    let mut session = OnnxSession::load(&model_path).expect("Failed to load model");
    session.load_tokenizer(&tokenizer_path).expect("Failed to load tokenizer");
    
    // Generate embedding for test text
    let text = "This is a test sentence for embedding generation.";
    let embedding = session.generate_embedding(text).expect("Failed to generate embedding");
    
    // Verify embedding dimensions (all-MiniLM-L6-v2 produces 384-dim embeddings)
    assert_eq!(embedding.len(), 384, "Unexpected embedding dimension");
    
    // Verify embedding is not all zeros (actual inference happened)
    let sum: f32 = embedding.iter().sum();
    assert!(sum.abs() > 0.001, "Embedding appears to be all zeros");
}

#[tokio::test]
async fn test_batch_embedding_generation() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cache_path = temp_dir.path().to_path_buf();
    
    // Download files
    let model_path = download_test_model(TEST_MODEL_REPO, TEST_MODEL_FILE, &cache_path)
        .await
        .expect("Failed to download test model");
    
    let tokenizer_path = download_test_model(TEST_MODEL_REPO, TEST_TOKENIZER_FILE, &cache_path)
        .await
        .expect("Failed to download tokenizer");
    
    // Load session
    let mut session = OnnxSession::load(&model_path).expect("Failed to load model");
    session.load_tokenizer(&tokenizer_path).expect("Failed to load tokenizer");
    
    // Generate embeddings for multiple texts
    let texts = vec![
        "First test sentence.".to_string(),
        "Second test sentence.".to_string(),
        "Third test sentence.".to_string(),
    ];
    
    let embeddings = session.generate_embeddings(&texts).expect("Failed to generate batch embeddings");
    
    // Verify we got the right number of embeddings
    assert_eq!(embeddings.len(), 3, "Expected 3 embeddings");
    
    // Verify each embedding has correct dimensions
    for (i, emb) in embeddings.iter().enumerate() {
        assert_eq!(emb.len(), 384, "Embedding {} has wrong dimension", i);
        
        // Verify non-zero
        let sum: f32 = emb.iter().sum();
        assert!(sum.abs() > 0.001, "Embedding {} appears to be all zeros", i);
    }
}

#[tokio::test]
async fn test_external_data_handling() {
    // Test with a model that has external data files
    // If all-MiniLM-L6-v2 doesn't have one, this tests the detection logic
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cache_path = temp_dir.path().to_path_buf();
    
    let model_path = download_test_model(TEST_MODEL_REPO, TEST_MODEL_FILE, &cache_path)
        .await
        .expect("Failed to download test model");
    
    // Check if external data file exists
    let external_data_path = format!("{}_data", model_path.display());
    
    if std::path::Path::new(&external_data_path).exists() {
        // If external data exists, loading should handle it automatically
        let session = OnnxSession::load(&model_path);
        assert!(session.is_ok(), "Failed to load model with external data");
    } else {
        // If no external data, model should still load fine
        let session = OnnxSession::load(&model_path);
        assert!(session.is_ok(), "Failed to load model without external data");
    }
}

#[test]
fn test_provider_auto_selection() {
    use tabagent_onnx_loader::providers_bridge;
    
    let result = providers_bridge::auto_select_providers();
    
    // Should succeed
    assert!(result.is_ok(), "Provider auto-selection failed: {:?}", result.err());
    
    let providers = result.unwrap();
    
    // Should have at least one provider (CPU minimum)
    assert!(!providers.is_empty(), "No execution providers selected");
}

#[test]
fn test_error_on_missing_file() {
    let result = OnnxSession::load("/nonexistent/path/model.onnx");
    assert!(result.is_err(), "Should fail on missing file");
    
    match result {
        Err(OnnxError::ModelLoadFailed(msg)) => {
            assert!(msg.contains("not found"), "Error message should mention file not found");
        }
        _ => panic!("Wrong error type returned"),
    }
}

#[tokio::test]
async fn test_error_on_missing_tokenizer() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cache_path = temp_dir.path().to_path_buf();
    
    let model_path = download_test_model(TEST_MODEL_REPO, TEST_MODEL_FILE, &cache_path)
        .await
        .expect("Failed to download test model");
    
    let session = OnnxSession::load(&model_path).expect("Failed to load model");
    
    // Try to generate without tokenizer
    let result = session.generate_embedding("test");
    assert!(result.is_err(), "Should fail without tokenizer");
}

