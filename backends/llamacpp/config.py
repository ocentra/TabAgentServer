"""
llama.cpp backend configuration.
"""

from dataclasses import dataclass
from typing import Optional
from enum import Enum


class LlamaCppBackend(str, Enum):
    """llama.cpp acceleration backends"""
    CPU = "cpu"
    CUDA = "cuda"
    VULKAN = "vulkan"
    ROCM = "rocm"
    METAL = "metal"


class LlamaCppBinaryName(str, Enum):
    """llama-server binary names by platform"""
    WINDOWS = "llama-server.exe"
    LINUX = "llama-server"
    MACOS = "llama-server"


@dataclass(frozen=True)
class LlamaCppConfig:
    """
    llama.cpp backend configuration.
    
    Attributes:
        binary_path: Path to llama-server executable
        backend: Acceleration backend to use
        ngl: Number of GPU layers to offload
        context_size: Context window size
        n_batch: Batch size for prompt processing
        n_threads: Number of CPU threads
        port: HTTP server port
        host: HTTP server host
        timeout: Request timeout in seconds
    """
    binary_path: str
    backend: LlamaCppBackend
    ngl: int = 0
    context_size: int = 4096
    n_batch: int = 512
    n_threads: Optional[int] = None  # None = auto
    port: int = 8081
    host: str = "127.0.0.1"
    timeout: int = 300

