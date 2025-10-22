"""
BitNet backend configuration.
"""

from dataclasses import dataclass
from typing import Optional
from enum import Enum


class BitNetBackend(str, Enum):
    """BitNet acceleration backends"""
    CPU_TL1 = "cpu_tl1"  # Intel optimized (Tiled Layout 1)
    CPU_TL2 = "cpu_tl2"  # AMD Ryzen optimized (Tiled Layout 2)
    GPU_CUDA = "gpu_cuda"  # CUDA with BitLinear kernels


class BitNetBinaryName(str, Enum):
    """llama-server-bitnet binary names by platform"""
    WINDOWS = "llama-server-bitnet.exe"
    LINUX = "llama-server-bitnet"
    MACOS = "llama-server-bitnet"


@dataclass(frozen=True)
class BitNetConfig:
    """
    BitNet backend configuration.
    
    Attributes:
        binary_path: Path to llama-server-bitnet executable
        backend: BitNet optimization variant (TL1/TL2/CUDA)
        ngl: Number of GPU layers to offload (0 for CPU-only)
        context_size: Context window size
        n_batch: Batch size for prompt processing
        n_threads: Number of CPU threads
        port: HTTP server port
        host: HTTP server host
        timeout: Request timeout in seconds
    """
    binary_path: str
    backend: BitNetBackend
    ngl: int = 0
    context_size: int = 4096
    n_batch: int = 512
    n_threads: Optional[int] = None  # None = auto
    port: int = 8082
    host: str = "127.0.0.1"
    timeout: int = 300

