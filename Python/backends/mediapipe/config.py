"""
MediaPipe LLM Inference backend configuration.
"""

from dataclasses import dataclass
from typing import Optional
from enum import Enum


class MediaPipeDelegate(str, Enum):
    """MediaPipe inference delegates"""
    CPU = "cpu"
    GPU = "gpu"  # GPU acceleration where available
    NPU = "npu"  # NPU/EdgeTPU where available


class MediaPipeModelType(str, Enum):
    """MediaPipe model types"""
    GEMMA_2B = "gemma_2b"
    GEMMA_7B = "gemma_7b"
    GEMMA_NANO = "gemma_nano"
    CUSTOM = "custom"


@dataclass(frozen=True)
class MediaPipeConfig:
    """
    MediaPipe LLM Inference configuration.
    
    Attributes:
        model_path: Path to .task bundle file
        delegate: Inference delegate (CPU/GPU/NPU)
        max_tokens: Maximum tokens to generate
        top_k: Top-k sampling parameter
        top_p: Top-p (nucleus) sampling parameter
        temperature: Temperature for sampling
        random_seed: Random seed for reproducibility (optional)
    """
    model_path: str
    delegate: MediaPipeDelegate = MediaPipeDelegate.CPU
    max_tokens: int = 4096
    top_k: int = 64
    top_p: float = 0.95
    temperature: float = 1.0
    random_seed: Optional[int] = None

