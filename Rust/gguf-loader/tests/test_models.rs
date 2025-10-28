use gguf_loader::{Model, ModelConfig, Context, GenerationParams, auto_select_variant, select_library_path, Variant, StandardCpuVariant};
use tabagent_model_cache::ModelCache;
use tempfile::tempdir;
use std::path::PathBuf;
use std::sync::Arc;

/// Test model constants - smallest available models for CI
const TEST_GGUF_REPO: &str = "ggml-org/smollm-135M-gguf";
const TEST_GGUF_FILE: &str = "smollm-135m-q4_k_m.gguf";

const TEST_PROMPT: &str = "Once upon a time";
const MIN_GENERATED_LENGTH: usize = 5;

/// Get base path to BitnetRelease directory
fn get_bitnet_base_path() -> PathBuf {
    // Assuming tests run from workspace root
    PathBuf::from("../../External/BitNet")
}

/// Download test model to temp directory
async fn download_test_model() -> Result<(PathBuf, PathBuf), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let cache_dir = temp_dir.path().to_path_buf();
    
    let cache = ModelCache::new(&cache_dir)?;
    
    // Download GGUF model
    eprintln!("Downloading test model: {}/{}", TEST_GGUF_REPO, TEST_GGUF_FILE);
    cache.download_file(TEST_GGUF_REPO, TEST_GGUF_FILE, None).await?;
    
    let model_path = cache.get_file_path(TEST_GGUF_REPO, TEST_GGUF_FILE).await?
        .ok_or("Model file not found in cache")?;
    
    Ok((cache_dir, model_path))
}

#[tokio::test]
async fn test_variant_selection() {
    env_logger::init();
    
    // Test auto-selection
    let variant = auto_select_variant(false);
    assert!(variant.is_ok(), "Failed to auto-select variant");
    
    let variant = variant.unwrap();
    eprintln!("Auto-selected variant: {:?}", variant);
    
    // Verify we can get a library path
    let base_path = get_bitnet_base_path();
    if base_path.exists() {
        let lib_path = select_library_path(&base_path, variant);
        if let Ok(path) = lib_path {
            eprintln!("Library path: {:?}", path);
            assert!(path.exists(), "Library file should exist at: {:?}", path);
        } else {
            eprintln!("Warning: Could not find library for selected variant (may need to build BitnetRelease first)");
        }
    } else {
        eprintln!("Warning: BitnetRelease not found at {:?}, skipping library check", base_path);
    }
}

#[tokio::test]
async fn test_load_gguf_model() {
    env_logger::init();
    
    // Download test model
    let (_cache_dir, model_path) = download_test_model().await
        .expect("Failed to download test model");
    
    eprintln!("Model downloaded to: {:?}", model_path);
    assert!(model_path.exists(), "Model file should exist");
    
    // Get library path
    let base_path = get_bitnet_base_path();
    if !base_path.exists() {
        eprintln!("Warning: BitnetRelease not found, skipping model load test");
        return;
    }
    
    // Use Standard CPU variant (most compatible)
    let variant = Variant::StandardCpu(StandardCpuVariant);
    let library_path = match select_library_path(&base_path, variant) {
        Ok(path) => path,
        Err(_) => {
            eprintln!("Warning: Could not find standard CPU library, skipping test");
            return;
        }
    };
    
    eprintln!("Using library: {:?}", library_path);
    
    // Load model
    let config = ModelConfig::new(&model_path);
    let model = Model::load(&library_path, config)
        .expect("Failed to load model");
    
    eprintln!("Model loaded successfully");
    
    // Verify model properties
    assert!(model.vocab_size() > 0, "Model should have vocabulary");
    assert!(model.context_train_size() > 0, "Model should have context size");
    
    eprintln!("Model vocab size: {}", model.vocab_size());
    eprintln!("Model context train size: {}", model.context_train_size());
}

#[tokio::test]
async fn test_generate_text() {
    env_logger::init();
    
    // Download test model
    let (_cache_dir, model_path) = download_test_model().await
        .expect("Failed to download test model");
    
    // Get library path
    let base_path = get_bitnet_base_path();
    if !base_path.exists() {
        eprintln!("Warning: BitnetRelease not found, skipping generation test");
        return;
    }
    
    let variant = Variant::StandardCpu(StandardCpuVariant);
    let library_path = match select_library_path(&base_path, variant) {
        Ok(path) => path,
        Err(_) => {
            eprintln!("Warning: Could not find standard CPU library, skipping test");
            return;
        }
    };
    
    // Load model
    let config = ModelConfig::new(&model_path);
    let model = Model::load(&library_path, config)
        .expect("Failed to load model");
    
    // Create context with default generation parameters
    let model_arc = Arc::new(model);
    let params = GenerationParams::default();
    let mut context = Context::new(model_arc, params)
        .expect("Failed to create context");
    
    eprintln!("Generating text with prompt: '{}'", TEST_PROMPT);
    
    // Generate text (just a few tokens for testing)
    let output = context.generate(TEST_PROMPT)
        .expect("Failed to generate text");
    
    eprintln!("Generated output: '{}'", output);
    
    // Verify output
    assert!(!output.is_empty(), "Generated text should not be empty");
    assert!(output.len() >= MIN_GENERATED_LENGTH, 
        "Generated text should be at least {} characters, got {}", 
        MIN_GENERATED_LENGTH, output.len());
}

#[tokio::test]
async fn test_model_with_auto_select() {
    env_logger::init();
    
    // Download test model
    let (_cache_dir, model_path) = download_test_model().await
        .expect("Failed to download test model");
    
    let base_path = get_bitnet_base_path();
    if !base_path.exists() {
        eprintln!("Warning: BitnetRelease not found, skipping auto-select test");
        return;
    }
    
    // Load with auto-select
    let config = ModelConfig::new(&model_path);
    let model = Model::load_with_auto_select(&base_path, config, false);
    
    match model {
        Ok(model) => {
            eprintln!("Model loaded with auto-select");
            eprintln!("Vocab size: {}", model.vocab_size());
            assert!(model.vocab_size() > 0);
        }
        Err(e) => {
            eprintln!("Warning: Auto-select failed (may need to build BitnetRelease): {}", e);
        }
    }
}

#[test]
fn test_list_available_variants() {
    env_logger::init();
    
    let base_path = get_bitnet_base_path();
    if !base_path.exists() {
        eprintln!("Warning: BitnetRelease not found, skipping variant list test");
        return;
    }
    
    let variants = gguf_loader::list_available_variants(&base_path);
    
    eprintln!("Found {} available variants:", variants.len());
    for (variant, path) in &variants {
        eprintln!("  - {:?} at {:?}", variant, path);
        assert!(path.exists(), "Variant library should exist");
    }
    
    if variants.is_empty() {
        eprintln!("Warning: No variants found. Build BitnetRelease first.");
    }
}

