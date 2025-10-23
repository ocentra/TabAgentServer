"""
Backend implementations for different model types and inference engines.

TabAgent supports multiple powerful backends:
- GGUF/BitNet: [RUST FFI] tabagent-rs/model-loader (llama.dll)
- ONNX Runtime: [Python → Rust migration] Multi-provider support
- MediaPipe: [Python → Rust migration] On-device models
- Transformers: [Python] PyTorch/SafeTensors (HuggingFace)
- LM Studio: [Python] External API proxy
"""

from .base_backend import (
    BaseInferenceBackend,
    TextGenerationBackend,
    EmbeddingBackend,
    MultimodalBackend
)
from .transformers_backend import (
    TransformersTextGenBackend,
    TransformersEmbeddingBackend
)
# Legacy Python wrappers removed - now handled by Rust:
# - BitNetManager (→ Rust model-loader via FFI)
# - LlamaCppManager (→ Rust model-loader via FFI)
from .onnxrt import ONNXRuntimeManager, ONNXRTConfig
from .mediapipe import MediaPipeManager, MediaPipeConfig
from .lmstudio import LMStudioManager

__all__ = [
    # Base backend classes
    'BaseInferenceBackend',
    'TextGenerationBackend',
    'EmbeddingBackend',
    'MultimodalBackend',
    
    # Transformers backend (PyTorch/SafeTensors) - Python permanent
    'TransformersTextGenBackend',
    'TransformersEmbeddingBackend',
    
    # ONNX Runtime backend - Python temporary (migrating to Rust)
    'ONNXRuntimeManager',
    'ONNXRTConfig',
    
    # MediaPipe backend - Python temporary (migrating to Rust)
    'MediaPipeManager',
    'MediaPipeConfig',
    
    # LM Studio backend - External API proxy
    'LMStudioManager',
]

