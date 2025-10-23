"""
ONNX Runtime backend configuration.
"""

from dataclasses import dataclass
from typing import List, Optional
from enum import Enum


class ONNXProvider(str, Enum):
    """ONNX Runtime execution providers"""
    CPU = "CPUExecutionProvider"
    CUDA = "CUDAExecutionProvider"
    DIRECTML = "DmlExecutionProvider"
    ROCM = "ROCmExecutionProvider"
    VITISAI = "VitisAIExecutionProvider"  # AMD Ryzen AI NPU
    TENSORRT = "TensorrtExecutionProvider"


class ONNXOptimizationLevel(int, Enum):
    """ONNX graph optimization levels"""
    DISABLE_ALL = 0
    ENABLE_BASIC = 1
    ENABLE_EXTENDED = 2
    ENABLE_ALL = 99


@dataclass(frozen=True)
class ONNXRTConfig:
    """
    ONNX Runtime backend configuration.
    
    Attributes:
        providers: List of execution providers in priority order
        optimization_level: Graph optimization level
        enable_profiling: Enable performance profiling
        log_severity_level: Logging level (0=Verbose, 4=Error)
        intra_op_num_threads: Number of intra-op threads (0=auto)
        inter_op_num_threads: Number of inter-op threads (0=auto)
    """
    providers: List[str]
    optimization_level: ONNXOptimizationLevel = ONNXOptimizationLevel.ENABLE_EXTENDED
    enable_profiling: bool = False
    log_severity_level: int = 3  # Warning
    intra_op_num_threads: int = 0  # Auto
    inter_op_num_threads: int = 0  # Auto


# Default configurations for different scenarios
DEFAULT_CPU_CONFIG = ONNXRTConfig(
    providers=[ONNXProvider.CPU.value]
)

DEFAULT_CUDA_CONFIG = ONNXRTConfig(
    providers=[ONNXProvider.CUDA.value, ONNXProvider.CPU.value]
)

DEFAULT_DIRECTML_CONFIG = ONNXRTConfig(
    providers=[ONNXProvider.DIRECTML.value, ONNXProvider.CPU.value]
)

DEFAULT_NPU_CONFIG = ONNXRTConfig(
    providers=[ONNXProvider.VITISAI.value, ONNXProvider.DIRECTML.value, ONNXProvider.CPU.value]
)

DEFAULT_ROCM_CONFIG = ONNXRTConfig(
    providers=[ONNXProvider.ROCM.value, ONNXProvider.CPU.value]
)

