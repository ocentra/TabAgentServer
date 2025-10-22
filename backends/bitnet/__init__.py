"""
BitNet 1.58-bit quantized model inference backend for TabAgent.

Supports BitNet 1.58 models with ultra-low bit quantization:
- CPU: TL1 (Intel) / TL2 (AMD Ryzen) optimized inference
- GPU: CUDA-accelerated BitLinear layers (future)
"""

from .manager import BitNetManager
from .config import BitNetConfig

__all__ = [
    'BitNetManager',
    'BitNetConfig',
]

