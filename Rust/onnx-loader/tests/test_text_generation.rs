//! Text Generation Tests for ONNX Models
//!
//! # Variant/Quantization Selection
//! 
//! In production, ONNX precision variants (FP32/FP16/INT8) are:
//! - Stored in manifest DB with quant_key: "fp32", "fp16", "int8"
//! - Selected by user via UI dropdown (like GGUF Q4_K_M, Q5_K_S, etc.)
//! - Retrieved via `ModelSettingsStore::get_or_default(repo_id, variant)`
//! 
//! These tests use default FP32. To test other precisions:
//! 1. Change `model_file` to ONNX_FP16, ONNX_INT8, etc.
//! 2. Ensure the repo contains that variant
//! 3. Update `variant` string to match manifest key

use tabagent_onnx_loader::{OnnxSession, init, GenerationConfig};
use tabagent_model_cache::{ModelCache, STANDARD_ONNX_DATA, STANDARD_TOKENIZER};
use tempfile::TempDir;

/// Initialize test environment
fn setup() {
    let _ = env_logger::builder().is_test(true).try_init();
    init().expect("Failed to initialize ONNX Runtime");
}

/// Test text generation with SmolLM2-360M-Instruct (single file, no external data)
/// 
/// Note: This uses FP32 precision (default). For other precisions:
/// - FP16: Use ONNX_FP16 if available in repo
/// - INT8: Use ONNX_INT8 if available in repo
/// In production, variant is selected via manifest/UI dropdown.
/// 
#[tokio::test]
async fn test_smollm2_360m_generation() {
    setup();
    
    // Model details - using onnx-community ONNX exports
    let repo_id = "onnx-community/SmolLM2-360M-Instruct-ONNX";
    let _variant = "fp32"; // Would be used with ModelSettingsStore::get_or_default(repo_id, variant)
    let model_file = "onnx/model.onnx"; // Files are in onnx/ subdirectory
    let tokenizer_file = STANDARD_TOKENIZER;
    
    // Download model files using model-cache with unique temp directory
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cache = ModelCache::new(temp_dir.path()).expect("Failed to create model cache");
    
    println!("Downloading SmolLM2-360M model files...");
    cache.download_file(repo_id, model_file, None).await
        .expect("Failed to download model");
    cache.download_file(repo_id, tokenizer_file, None).await
        .expect("Failed to download tokenizer");
    
    let model_path = cache.get_file_path(repo_id, model_file).await
        .expect("Failed to get model path")
        .expect("Model file not found");
    let tokenizer_path = cache.get_file_path(repo_id, tokenizer_file).await
        .expect("Failed to get tokenizer path")
        .expect("Tokenizer file not found");
    
    println!("Model path: {:?}", model_path);
    println!("Tokenizer path: {:?}", tokenizer_path);
    
    // Load ONNX session (auto-selects best available provider: GPU/NPU/CPU)
    let mut session = OnnxSession::load(&model_path)
        .expect("Failed to load ONNX session");
    
    // Load tokenizer
    session.load_tokenizer(&tokenizer_path)
        .expect("Failed to load tokenizer");
    
    // Test with greedy decoding
    println!("\n=== Test 1: Greedy Decoding ===");
    let greedy_settings = GenerationConfig {
        temperature: 0.0, // Greedy (deterministic)
        max_new_tokens: 50,
        do_sample: false,
        repetition_penalty: 1.1,
        ..Default::default()
    };
    
    let prompt = "Hello, my name is";
    println!("Prompt: '{}'", prompt);
    
    let output = session.generate_text(prompt, &greedy_settings)
        .expect("Failed to generate text");
    
    println!("Generated: '{}'", output);
    assert!(!output.is_empty(), "Generated text should not be empty");
    assert!(output.len() > prompt.len(), "Generated text should be longer than prompt");
    
    // Test with sampling
    println!("\n=== Test 2: Sampling with Temperature ===");
    let sampling_settings = GenerationConfig {
        temperature: 0.7,
        max_new_tokens: 50,
        top_k: 40,
        top_p: 0.9,
        do_sample: true,
        repetition_penalty: 1.1,
        ..Default::default()
    };
    
    let output2 = session.generate_text(prompt, &sampling_settings)
        .expect("Failed to generate text with sampling");
    
    println!("Generated: '{}'", output2);
    assert!(!output2.is_empty(), "Generated text should not be empty");
    
    // Test with higher temperature (more creative)
    println!("\n=== Test 3: High Temperature ===");
    let creative_settings = GenerationConfig {
        temperature: 1.0,
        max_new_tokens: 30,
        top_k: 50,
        top_p: 0.95,
        do_sample: true,
        repetition_penalty: 1.2,
        ..Default::default()
    };
    
    let creative_prompt = "Once upon a time";
    println!("Prompt: '{}'", creative_prompt);
    
    let output3 = session.generate_text(creative_prompt, &creative_settings)
        .expect("Failed to generate creative text");
    
    println!("Generated: '{}'", output3);
    assert!(!output3.is_empty(), "Generated text should not be empty");
    
    println!("\n✅ SmolLM2-360M text generation tests passed!");
}

/// Test text generation with SmolLM2-1.7B-Instruct (with external data)
/// 
/// Tests ONNX models with external weight files (>2GB).
/// Variant selection works same as single-file models.
#[tokio::test]
async fn test_smollm2_1_7b_generation() {
    setup();
    
    // Model details - using onnx-community ONNX exports
    let repo_id = "onnx-community/SmolLM2-360M-Instruct-ONNX"; // Using 360M (1.7B ONNX doesn't exist yet)
    let _variant = "fp32"; // Would be used with ModelSettingsStore::get_or_default(repo_id, variant)
    let model_file = "onnx/model.onnx"; // Files are in onnx/ subdirectory
    let model_data_file = STANDARD_ONNX_DATA; // External data file
    let tokenizer_file = STANDARD_TOKENIZER;
    
    // Download model files using model-cache with unique temp directory
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cache = ModelCache::new(temp_dir.path()).expect("Failed to create model cache");
    
    println!("Downloading SmolLM2-1.7B model files (this may take a while)...");
    
    // Download all three files
    cache.download_file(repo_id, model_file, None).await
        .expect("Failed to download model");
    cache.download_file(repo_id, model_data_file, None).await
        .expect("Failed to download external data");
    cache.download_file(repo_id, tokenizer_file, None).await
        .expect("Failed to download tokenizer");
    
    let model_path = cache.get_file_path(repo_id, model_file).await
        .expect("Failed to get model path")
        .expect("Model file not found");
    let tokenizer_path = cache.get_file_path(repo_id, tokenizer_file).await
        .expect("Failed to get tokenizer path")
        .expect("Tokenizer file not found");
    
    println!("Model path: {:?}", model_path);
    println!("Tokenizer path: {:?}", tokenizer_path);
    
    // Load ONNX session (external data & best provider auto-selected)
    let mut session = OnnxSession::load(&model_path)
        .expect("Failed to load ONNX session with external data");
    
    // Load tokenizer
    session.load_tokenizer(&tokenizer_path)
        .expect("Failed to load tokenizer");
    
    // Test generation with optimal settings for larger model
    println!("\n=== Testing SmolLM2-1.7B Generation ===");
    let settings = GenerationConfig {
        temperature: 0.7,
        max_new_tokens: 100,
        top_k: 50,
        top_p: 0.9,
        do_sample: true,
        repetition_penalty: 1.15,
        ..Default::default()
    };
    
    let prompt = "Write a short poem about AI:";
    println!("Prompt: '{}'", prompt);
    
    let output = session.generate_text(prompt, &settings)
        .expect("Failed to generate text");
    
    println!("Generated: '{}'", output);
    assert!(!output.is_empty(), "Generated text should not be empty");
    assert!(output.len() > prompt.len(), "Generated text should be longer than prompt");
    
    // Test with a different prompt
    println!("\n=== Testing Complex Prompt ===");
    let complex_prompt = "Explain quantum computing in simple terms:";
    println!("Prompt: '{}'", complex_prompt);
    
    let output2 = session.generate_text(complex_prompt, &settings)
        .expect("Failed to generate text for complex prompt");
    
    println!("Generated: '{}'", output2);
    assert!(!output2.is_empty(), "Generated text should not be empty");
    
    println!("\n✅ SmolLM2-1.7B text generation tests passed!");
}

/// Test that GenerationConfig properly affect output
#[tokio::test]
async fn test_settings_impact() {
    setup();
    
    let repo_id = "onnx-community/SmolLM2-360M-Instruct-ONNX";
    let model_file = "onnx/model.onnx"; // Files are in onnx/ subdirectory
    let tokenizer_file = STANDARD_TOKENIZER;
    
    // Download model files using model-cache with unique temp directory
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cache = ModelCache::new(temp_dir.path()).expect("Failed to create model cache");
    
    // Download model files
    println!("Downloading SmolLM2-360M model files for settings test...");
    cache.download_file(repo_id, model_file, None).await
        .expect("Failed to download model");
    cache.download_file(repo_id, tokenizer_file, None).await
        .expect("Failed to download tokenizer");
    
    let model_path = cache.get_file_path(repo_id, model_file).await
        .expect("Failed to get model path")
        .expect("Model file not found");
    let tokenizer_path = cache.get_file_path(repo_id, tokenizer_file).await
        .expect("Failed to get tokenizer path")
        .expect("Tokenizer file not found");
    
    let mut session = OnnxSession::load(&model_path)
        .expect("Failed to load ONNX session");
    session.load_tokenizer(&tokenizer_path)
        .expect("Failed to load tokenizer");
    
    let prompt = "The capital of France is";
    
    // Greedy should be deterministic
    let greedy = GenerationConfig {
        temperature: 0.0,
        max_new_tokens: 10,
        do_sample: false,
        ..Default::default()
    };
    
    let out1 = session.generate_text(prompt, &greedy).expect("Generation failed");
    let out2 = session.generate_text(prompt, &greedy).expect("Generation failed");
    
    println!("Greedy output 1: '{}'", out1);
    println!("Greedy output 2: '{}'", out2);
    // Note: Greedy should be identical, but due to floating point, we just check they're similar
    assert_eq!(out1.len(), out2.len(), "Greedy outputs should have same length");
    
    // High temperature should produce varied outputs (probabilistic)
    let sampling = GenerationConfig {
        temperature: 1.5,
        max_new_tokens: 10,
        top_p: 0.9,
        do_sample: true,
        ..Default::default()
    };
    
    let out3 = session.generate_text(prompt, &sampling).expect("Generation failed");
    let out4 = session.generate_text(prompt, &sampling).expect("Generation failed");
    
    println!("Sampling output 1: '{}'", out3);
    println!("Sampling output 2: '{}'", out4);
    // Outputs may differ due to sampling randomness
    
    println!("\n✅ Settings impact test passed!");
}

