"""
Test model constants for integration tests
These are TINY models optimized for fast CI/testing
"""

# GGUF Models (smallest available)
TINY_GGUF_REPO = "ggml-org/smollm-135M-gguf"
TINY_GGUF_FILE = "smollm-135m-q4_k_m.gguf"
TINY_GGUF_SIZE_MB = 82  # ~82MB

SMALL_GGUF_REPO = "ggml-org/SmolLM3-3B-GGUF"
SMALL_GGUF_FILE = "smollm3-3b-q4_k_m.gguf"
SMALL_GGUF_SIZE_MB = 1800  # ~1.8GB

# BitNet Models (smallest available)
TINY_BITNET_REPO = "microsoft/bitnet-b1.58-2B-4T"
TINY_BITNET_FILE = "model.safetensors"
TINY_BITNET_SIZE_MB = 300  # ~300MB

# ONNX Models (smallest available)
TINY_ONNX_REPO = "HuggingFaceTB/SmolLM3-3B-ONNX"
TINY_ONNX_FILE = "model.onnx"
TINY_ONNX_SIZE_MB = 1500  # ~1.5GB

# MediaPipe Models (smallest available)
TINY_MEDIAPIPE_REPO = "google/gemma-3n-E4B-it-litert-lm"
TINY_MEDIAPIPE_FILE = "model.tflite"
TINY_MEDIAPIPE_SIZE_MB = 500  # ~500MB


class TestModelConfig:
    """Test model configuration"""
    def __init__(self, repo_id: str, file_path: str, expected_size_mb: int, model_type: str):
        self.repo_id = repo_id
        self.file_path = file_path
        self.expected_size_mb = expected_size_mb
        self.model_type = model_type

    def to_dict(self):
        return {
            "repoId": self.repo_id,
            "modelFile": self.file_path,
            "expectedSizeMb": self.expected_size_mb,
            "modelType": self.model_type,
        }


# Predefined test model configurations
TEST_MODELS = [
    TestModelConfig(TINY_GGUF_REPO, TINY_GGUF_FILE, TINY_GGUF_SIZE_MB, "gguf"),
    TestModelConfig(TINY_BITNET_REPO, TINY_BITNET_FILE, TINY_BITNET_SIZE_MB, "bitnet"),
    TestModelConfig(TINY_ONNX_REPO, TINY_ONNX_FILE, TINY_ONNX_SIZE_MB, "onnx"),
    TestModelConfig(TINY_MEDIAPIPE_REPO, TINY_MEDIAPIPE_FILE, TINY_MEDIAPIPE_SIZE_MB, "mediapipe"),
]


def get_test_model(model_type: str) -> TestModelConfig:
    """Get test model by type"""
    model_map = {
        "gguf-tiny": TestModelConfig(TINY_GGUF_REPO, TINY_GGUF_FILE, TINY_GGUF_SIZE_MB, "gguf"),
        "gguf-small": TestModelConfig(SMALL_GGUF_REPO, SMALL_GGUF_FILE, SMALL_GGUF_SIZE_MB, "gguf"),
        "bitnet": TestModelConfig(TINY_BITNET_REPO, TINY_BITNET_FILE, TINY_BITNET_SIZE_MB, "bitnet"),
        "onnx": TestModelConfig(TINY_ONNX_REPO, TINY_ONNX_FILE, TINY_ONNX_SIZE_MB, "onnx"),
        "mediapipe": TestModelConfig(TINY_MEDIAPIPE_REPO, TINY_MEDIAPIPE_FILE, TINY_MEDIAPIPE_SIZE_MB, "mediapipe"),
    }
    if model_type not in model_map:
        raise ValueError(f"Unknown test model type: {model_type}")
    return model_map[model_type]

