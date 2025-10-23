"""
MediaPipe LLM Inference backend for TabAgent.

Optimized on-device inference using Google MediaPipe LLM task.
Supports Gemma models in .task bundle format.
"""

from .manager import MediaPipeManager
from .config import MediaPipeConfig

__all__ = [
    'MediaPipeManager',
    'MediaPipeConfig',
]

