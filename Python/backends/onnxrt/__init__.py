"""
ONNX Runtime inference backend for TabAgent.

Supports multiple execution providers:
- CPU (default)
- CUDA (NVIDIA GPU)
- DirectML (Windows GPU/NPU)
- VitisAI (AMD Ryzen AI NPU)
- ROCm (AMD GPU - Linux)
"""

from .manager import ONNXRuntimeManager
from .config import ONNXRTConfig

__all__ = [
    'ONNXRuntimeManager',
    'ONNXRTConfig',
]

