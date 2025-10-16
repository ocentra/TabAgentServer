"""
Backend implementations for different model types and inference engines.

TabAgent supports multiple powerful backends:
- BitNet: 1.58-bit quantized models (CPU/GPU)
- ONNX Runtime: Multi-provider support (CPU/CUDA/DirectML/NPU)
- llama.cpp: GGUF models with multiple accelerations
- MediaPipe: On-device Gemma models (.task bundles)
"""

from .bitnet import BitNetManager, BitNetConfig
from .onnxrt import ONNXRuntimeManager, ONNXRTConfig
from .llamacpp import LlamaCppManager, LlamaCppConfig
from .mediapipe import MediaPipeManager, MediaPipeConfig
from .lmstudio import LMStudioManager

__all__ = [
    # BitNet backend
    'BitNetManager',
    'BitNetConfig',
    
    # ONNX Runtime backend
    'ONNXRuntimeManager',
    'ONNXRTConfig',
    
    # llama.cpp backend
    'LlamaCppManager',
    'LlamaCppConfig',
    
    # MediaPipe backend
    'MediaPipeManager',
    'MediaPipeConfig',
    
    # LM Studio backend
    'LMStudioManager',
]

