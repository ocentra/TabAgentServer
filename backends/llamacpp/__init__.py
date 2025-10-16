"""
llama.cpp inference backend for TabAgent.

Supports multiple acceleration backends:
- CPU
- CUDA (NVIDIA)
- Vulkan (cross-platform)
- ROCm (AMD - Linux)
- Metal (Apple)
"""

from .manager import LlamaCppManager
from .config import LlamaCppConfig

__all__ = [
    'LlamaCppManager',
    'LlamaCppConfig',
]

