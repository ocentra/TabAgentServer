# Tab Agent Native Host Configuration
"""
Strongly typed configuration for native host and inference backends.
"""

from typing import List, Literal
from dataclasses import dataclass


# Logging configuration
LOG_LEVEL: Literal["DEBUG", "INFO", "WARNING", "ERROR"] = "INFO"
LOG_FILE: str = "native_host.log"

# Security settings
# Whitelist of allowed commands (empty list means all commands allowed - FOR DEVELOPMENT ONLY)
ALLOWED_COMMANDS: List[str] = []

# Timeout for command execution (seconds)
COMMAND_TIMEOUT: int = 30

# Maximum message size (bytes)
MAX_MESSAGE_SIZE: int = 1024 * 1024  # 1MB


@dataclass
class BitNetConfig:
    """BitNet inference backend configuration"""
    
    # CPU inference settings
    cpu_port: int = 8765
    cpu_host: str = "127.0.0.1"
    cpu_context_size: int = 4096
    cpu_threads: int = 0  # 0 = auto-detect
    
    # GPU inference settings (CUDA only)
    gpu_device_id: int = 0
    
    # Inference defaults
    default_temperature: float = 0.7
    default_top_k: int = 40
    default_top_p: float = 0.9
    default_max_tokens: int = 512
    default_repetition_penalty: float = 1.0
    
    # Timeouts
    startup_timeout_seconds: float = 5.0
    inference_timeout_seconds: float = 120.0
    
    # Paths (relative to backends/bitnet/)
    binary_dir: str = "binaries"  # backends/bitnet/binaries/


# Global BitNet configuration instance
BITNET_CONFIG = BitNetConfig()

