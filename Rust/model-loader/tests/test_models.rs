/// Test model constants for integration tests
/// These are TINY models optimized for fast CI/testing

// GGUF Models (smallest available)
pub const TINY_GGUF_REPO: &str = "ggml-org/smollm-135M-gguf";
pub const TINY_GGUF_FILE: &str = "smollm-135m-q4_k_m.gguf";
pub const TINY_GGUF_SIZE_MB: u64 = 82; // ~82MB

pub const SMALL_GGUF_REPO: &str = "ggml-org/SmolLM3-3B-GGUF";
pub const SMALL_GGUF_FILE: &str = "smollm3-3b-q4_k_m.gguf";
pub const SMALL_GGUF_SIZE_MB: u64 = 1800; // ~1.8GB

// BitNet Models (smallest available)
pub const TINY_BITNET_REPO: &str = "microsoft/bitnet-b1.58-2B-4T";
pub const TINY_BITNET_FILE: &str = "model.safetensors";
pub const TINY_BITNET_SIZE_MB: u64 = 300; // ~300MB

// ONNX Models (smallest available)
pub const TINY_ONNX_REPO: &str = "HuggingFaceTB/SmolLM3-3B-ONNX";
pub const TINY_ONNX_FILE: &str = "model.onnx";
pub const TINY_ONNX_SIZE_MB: u64 = 1500; // ~1.5GB

// MediaPipe Models (smallest available)
pub const TINY_MEDIAPIPE_REPO: &str = "google/gemma-3n-E4B-it-litert-lm";
pub const TINY_MEDIAPIPE_FILE: &str = "model.tflite";
pub const TINY_MEDIAPIPE_SIZE_MB: u64 = 500; // ~500MB

// Helper to get model by type
pub fn get_test_model(model_type: &str) -> (&'static str, &'static str, u64) {
    match model_type {
        "gguf-tiny" => (TINY_GGUF_REPO, TINY_GGUF_FILE, TINY_GGUF_SIZE_MB),
        "gguf-small" => (SMALL_GGUF_REPO, SMALL_GGUF_FILE, SMALL_GGUF_SIZE_MB),
        "bitnet" => (TINY_BITNET_REPO, TINY_BITNET_FILE, TINY_BITNET_SIZE_MB),
        "onnx" => (TINY_ONNX_REPO, TINY_ONNX_FILE, TINY_ONNX_SIZE_MB),
        "mediapipe" => (TINY_MEDIAPIPE_REPO, TINY_MEDIAPIPE_FILE, TINY_MEDIAPIPE_SIZE_MB),
        _ => panic!("Unknown test model type: {}", model_type),
    }
}

// Predefined test configurations
pub struct TestModelConfig {
    pub repo_id: &'static str,
    pub file_path: &'static str,
    pub expected_size_mb: u64,
    pub model_type: &'static str,
}

pub const TEST_MODELS: &[TestModelConfig] = &[
    TestModelConfig {
        repo_id: TINY_GGUF_REPO,
        file_path: TINY_GGUF_FILE,
        expected_size_mb: TINY_GGUF_SIZE_MB,
        model_type: "gguf",
    },
    TestModelConfig {
        repo_id: TINY_BITNET_REPO,
        file_path: TINY_BITNET_FILE,
        expected_size_mb: TINY_BITNET_SIZE_MB,
        model_type: "bitnet",
    },
];

