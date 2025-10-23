"""
Smart backend selection and configuration for TabAgent.

Provides intelligent backend selection based on:
- Hardware capabilities
- Model requirements
- VRAM availability
- User preferences

Includes VRAM-aware GPU layer offloading (ngl) calculation.
"""

import logging
from typing import Optional, List
from enum import Enum
from dataclasses import dataclass

from Python.core.message_types import (
    BackendType,
    ModelType,
    AccelerationBackend,
    BackendConfig,
    GPUInfo,
)
from .hardware_detection import HardwareDetector, create_hardware_detector
from .engine_detection import AccelerationDetector, InferenceEngineDetector


logger = logging.getLogger(__name__)


class SelectionStrategy(str, Enum):
    """Backend selection strategies"""
    AUTO = "auto"
    FASTEST = "fastest"
    LOWEST_MEMORY = "lowest_memory"
    USER_OVERRIDE = "user_override"


class VRAMReservation(int, Enum):
    """VRAM reservation amounts in GB"""
    CONTEXT_BASE = 2  # Base reservation for context
    SYSTEM_OVERHEAD = 1  # Additional system overhead
    TOTAL_DEFAULT = 3  # Total default reservation (2 + 1)


class ModelSizeEstimate(float, Enum):
    """Common model size estimates in GB"""
    TINY_1B = 0.8
    SMALL_3B = 2.5
    MEDIUM_7B = 5.0
    LARGE_13B = 9.0
    EXTRA_LARGE_30B = 20.0


class LayerEstimate(int, Enum):
    """Layer count estimates for different model sizes"""
    TINY_1B = 16
    SMALL_3B = 26
    MEDIUM_7B = 32
    LARGE_13B = 40
    EXTRA_LARGE_30B = 60


@dataclass(frozen=True)
class BackendSelectionResult:
    """
    Result of backend selection process.
    
    Attributes:
        backend: Selected backend type
        acceleration: Acceleration backend to use
        gpu_index: GPU device index (0-based)
        ngl: Number of GPU layers to offload
        context_size: Recommended context size
        confidence: Selection confidence (0.0-1.0)
        reason: Human-readable reason for selection
    """
    backend: BackendType
    acceleration: AccelerationBackend
    gpu_index: int
    ngl: int
    context_size: int
    confidence: float
    reason: str


class GPULayerCalculator:
    """
    Calculates optimal GPU layer offloading (ngl) based on VRAM.
    
    Uses formula:
    - Reserve VRAM for context + overhead
    - Estimate layers per GB based on model size
    - Calculate safe layer count within VRAM limits
    """
    
    @staticmethod
    def calculate_optimal_ngl(
        model_size_gb: float,
        vram_mb: int,
        context_size: int = 4096,
        model_layers: Optional[int] = None
    ) -> int:
        """
        Calculate optimal GPU layer offloading.
        
        Args:
            model_size_gb: Model size in GB
            vram_mb: Available VRAM in MB
            context_size: Context size in tokens
            model_layers: Total model layers (if known)
            
        Returns:
            Number of layers to offload (0 if insufficient VRAM)
        """
        vram_gb = vram_mb / 1024.0
        
        # Calculate context memory requirement (rough estimate)
        # ~1.5 bytes per token for context storage
        context_gb = (context_size * 1.5) / (1024 ** 3)
        
        # Total reservation
        reserved_gb = VRAMReservation.CONTEXT_BASE.value + context_gb
        
        # Available VRAM for model
        available_gb = vram_gb - reserved_gb
        
        if available_gb <= 0:
            logger.warning(
                f"Insufficient VRAM: {vram_mb}MB VRAM, need {reserved_gb:.1f}GB reserved, "
                f"model size {model_size_gb:.1f}GB"
            )
            return 0
        
        # Can we fit entire model?
        if available_gb >= model_size_gb:
            # Full offload possible
            if model_layers:
                logger.info(
                    f"Full model offload possible: {vram_mb}MB VRAM, "
                    f"{model_layers} layers"
                )
                return model_layers
            else:
                # Estimate layers if not provided
                estimated_layers = GPULayerCalculator._estimate_layers(model_size_gb)
                logger.info(
                    f"Full model offload possible: {vram_mb}MB VRAM, "
                    f"~{estimated_layers} layers (estimated)"
                )
                return estimated_layers
        
        # Partial offload - calculate how many layers fit
        ratio = available_gb / model_size_gb
        
        if model_layers:
            layers_to_offload = int(model_layers * ratio)
        else:
            estimated_total = GPULayerCalculator._estimate_layers(model_size_gb)
            layers_to_offload = int(estimated_total * ratio)
        
        # Safety margin - offload slightly fewer layers
        layers_to_offload = int(layers_to_offload * 0.9)
        
        # Minimum viable offload
        if layers_to_offload < 4:
            logger.warning(
                f"Partial offload too small ({layers_to_offload} layers), "
                f"using CPU instead"
            )
            return 0
        
        logger.info(
            f"Partial offload: {layers_to_offload} layers "
            f"({available_gb:.1f}GB / {model_size_gb:.1f}GB = {ratio:.1%})"
        )
        
        return layers_to_offload
    
    @staticmethod
    def _estimate_layers(model_size_gb: float) -> int:
        """
        Estimate number of layers based on model size.
        
        Args:
            model_size_gb: Model size in GB
            
        Returns:
            Estimated layer count
        """
        # Rough estimates based on common model sizes
        if model_size_gb < 1.5:
            return LayerEstimate.TINY_1B.value
        elif model_size_gb < 4.0:
            return LayerEstimate.SMALL_3B.value
        elif model_size_gb < 8.0:
            return LayerEstimate.MEDIUM_7B.value
        elif model_size_gb < 15.0:
            return LayerEstimate.LARGE_13B.value
        else:
            return LayerEstimate.EXTRA_LARGE_30B.value


class BackendSelector:
    """
    Intelligent backend selection for model loading.
    
    Analyzes:
    - Hardware capabilities
    - Model type and size
    - VRAM availability
    - Acceleration backends
    
    Selects optimal backend with configuration.
    """
    
    def __init__(self):
        """Initialize backend selector with hardware detection"""
        self.hardware_detector: Optional[HardwareDetector] = None
        self.acceleration_detector = AccelerationDetector()
        self.engine_detector = InferenceEngineDetector()
        self._hw_info_cache = None
        
        try:
            self.hardware_detector = create_hardware_detector()
            self._hw_info_cache = self.hardware_detector.get_hardware_info()
            logger.info("BackendSelector initialized with hardware detection")
        except Exception as e:
            logger.warning(f"Hardware detection unavailable: {e}")
    
    def select_backend(
        self,
        model_type: ModelType,
        model_size_gb: Optional[float] = None,
        model_layers: Optional[int] = None,
        user_preference: Optional[BackendType] = None,
        strategy: SelectionStrategy = SelectionStrategy.AUTO
    ) -> BackendSelectionResult:
        """
        Select optimal backend for model.
        
        Args:
            model_type: Type of model
            model_size_gb: Model size in GB (if known)
            model_layers: Number of model layers (if known)
            user_preference: User's backend preference
            strategy: Selection strategy
            
        Returns:
            BackendSelectionResult with configuration
        """
        # User override takes precedence
        if user_preference and strategy == SelectionStrategy.USER_OVERRIDE:
            return self._validate_user_preference(
                user_preference,
                model_type,
                model_size_gb,
                model_layers
            )
        
        # Auto-selection based on model type
        if model_type == ModelType.BITNET_158:
            return self._select_bitnet_backend(model_size_gb, model_layers)
        elif model_type == ModelType.MEDIAPIPE_TASK:
            return self._select_mediapipe_backend()
        elif model_type == ModelType.ONNX:
            return self._select_onnx_backend()
        elif model_type in [ModelType.GGUF_REGULAR, ModelType.SAFETENSORS, ModelType.PYTORCH]:
            return self._select_llamacpp_backend(model_size_gb, model_layers)
        else:
            # Unknown model type - safe CPU fallback
            return BackendSelectionResult(
                backend=BackendType.LLAMA_CPP_CPU,
                acceleration=AccelerationBackend.CPU,
                gpu_index=0,
                ngl=0,
                context_size=4096,
                confidence=0.3,
                reason=f"Unknown model type {model_type.value}, using llama.cpp CPU fallback"
            )
    
    def _select_bitnet_backend(
        self,
        model_size_gb: Optional[float],
        model_layers: Optional[int]
    ) -> BackendSelectionResult:
        """
        Select backend for BitNet 1.58 models.
        
        Args:
            model_size_gb: Model size in GB
            model_layers: Number of layers
            
        Returns:
            BackendSelectionResult for BitNet
        """
        # Check for CUDA availability
        if self.acceleration_detector.has_cuda() and self._hw_info_cache:
            nvidia_gpus = self._hw_info_cache.nvidia_gpus
            if nvidia_gpus and len(nvidia_gpus) > 0:
                # Use GPU backend
                gpu = nvidia_gpus[0]
                return BackendSelectionResult(
                    backend=BackendType.BITNET_GPU,
                    acceleration=AccelerationBackend.CUDA,
                    gpu_index=0,
                    ngl=0,  # BitNet GPU does full GPU inference
                    context_size=4096,
                    confidence=1.0,
                    reason=f"BitNet GPU backend selected (CUDA available, {gpu.name})"
                )
        
        # Fallback to CPU
        return BackendSelectionResult(
            backend=BackendType.BITNET_CPU,
            acceleration=AccelerationBackend.CPU,
            gpu_index=0,
            ngl=0,
            context_size=4096,
            confidence=0.9,
            reason="BitNet CPU backend selected (no CUDA available)"
        )
    
    def _select_mediapipe_backend(self) -> BackendSelectionResult:
        """
        Select backend for MediaPipe .task bundle models.
        
        Returns:
            BackendSelectionResult for MediaPipe
        """
        backends = self.acceleration_detector.detect_all()
        
        # Try NPU first (best for on-device)
        if backends.get(AccelerationBackend.NPU):
            return BackendSelectionResult(
                backend=BackendType.MEDIAPIPE_NPU,
                acceleration=AccelerationBackend.NPU,
                gpu_index=0,
                ngl=0,
                context_size=4096,
                confidence=1.0,
                reason="MediaPipe NPU selected (optimal for on-device Gemma)"
            )
        
        # Try GPU
        if backends.get(AccelerationBackend.DIRECTML) or backends.get(AccelerationBackend.CUDA):
            return BackendSelectionResult(
                backend=BackendType.MEDIAPIPE_GPU,
                acceleration=AccelerationBackend.DIRECTML if backends.get(AccelerationBackend.DIRECTML) else AccelerationBackend.CUDA,
                gpu_index=0,
                ngl=0,
                context_size=4096,
                confidence=0.9,
                reason="MediaPipe GPU selected (GPU-accelerated on-device)"
            )
        
        # CPU fallback
        return BackendSelectionResult(
            backend=BackendType.MEDIAPIPE_CPU,
            acceleration=AccelerationBackend.CPU,
            gpu_index=0,
            ngl=0,
            context_size=4096,
            confidence=0.7,
            reason="MediaPipe CPU selected (on-device inference)"
        )
    
    def _select_onnx_backend(self) -> BackendSelectionResult:
        """
        Select backend for ONNX models.
        
        Returns:
            BackendSelectionResult for ONNX Runtime
        """
        backends = self.acceleration_detector.detect_all()
        
        # Try NPU first
        if backends.get(AccelerationBackend.NPU):
            return BackendSelectionResult(
                backend=BackendType.ONNX_NPU,
                acceleration=AccelerationBackend.NPU,
                gpu_index=0,
                ngl=0,
                context_size=4096,
                confidence=1.0,
                reason="ONNX NPU selected (VitisAI for AMD Ryzen AI)"
            )
        
        # Try DirectML
        if backends.get(AccelerationBackend.DIRECTML):
            return BackendSelectionResult(
                backend=BackendType.ONNX_DIRECTML,
                acceleration=AccelerationBackend.DIRECTML,
                gpu_index=0,
                ngl=0,
                context_size=4096,
                confidence=0.9,
                reason="ONNX DirectML selected (Windows GPU/NPU)"
            )
        
        # Try CUDA
        if backends.get(AccelerationBackend.CUDA):
            return BackendSelectionResult(
                backend=BackendType.ONNX_CUDA,
                acceleration=AccelerationBackend.CUDA,
                gpu_index=0,
                ngl=0,
                context_size=4096,
                confidence=0.9,
                reason="ONNX CUDA selected (NVIDIA GPU)"
            )
        
        # CPU fallback
        return BackendSelectionResult(
            backend=BackendType.ONNX_CPU,
            acceleration=AccelerationBackend.CPU,
            gpu_index=0,
            ngl=0,
            context_size=4096,
            confidence=0.7,
            reason="ONNX CPU selected"
        )
    
    def _select_llamacpp_backend(
        self,
        model_size_gb: Optional[float],
        model_layers: Optional[int]
    ) -> BackendSelectionResult:
        """
        Select backend for standard GGUF/Safetensors/ONNX models.
        
        Routes to llama.cpp for GGUF or ONNX Runtime for ONNX models.
        
        Args:
            model_size_gb: Model size in GB
            model_layers: Number of layers
            
        Returns:
            BackendSelectionResult for standard models
        """
        # Estimate model size if not provided
        if model_size_gb is None:
            model_size_gb = ModelSizeEstimate.MEDIUM_7B.value
            logger.info(f"Model size unknown, estimating {model_size_gb}GB")
        
        backends = self.acceleration_detector.detect_all()
        
        # Try NPU first (most efficient for supported models)
        if backends.get(AccelerationBackend.NPU):
            return BackendSelectionResult(
                backend=BackendType.ONNX_NPU,
                acceleration=AccelerationBackend.NPU,
                gpu_index=0,
                ngl=0,  # NPU doesn't use ngl
                context_size=4096,
                confidence=1.0,
                reason="NPU selected (AMD Ryzen AI or Intel VPU - best efficiency)"
            )
        
        # Try CUDA
        if backends.get(AccelerationBackend.CUDA) and self._hw_info_cache:
            nvidia_gpus = self._hw_info_cache.nvidia_gpus
            if nvidia_gpus and len(nvidia_gpus) > 0:
                gpu = nvidia_gpus[0]
                if gpu.vram_mb:
                    ngl = GPULayerCalculator.calculate_optimal_ngl(
                        model_size_gb=model_size_gb,
                        vram_mb=gpu.vram_mb,
                        context_size=4096,
                        model_layers=model_layers
                    )
                    
                    if ngl > 0:
                        return BackendSelectionResult(
                            backend=BackendType.LLAMA_CPP_CUDA,
                            acceleration=AccelerationBackend.CUDA,
                            gpu_index=0,
                            ngl=ngl,
                            context_size=4096,
                            confidence=1.0,
                            reason=f"CUDA selected with {ngl} layer offload ({gpu.name}, {gpu.vram_mb}MB)"
                        )
        
        # Try DirectML (Windows GPU/NPU)
        if backends.get(AccelerationBackend.DIRECTML):
            return BackendSelectionResult(
                backend=BackendType.ONNX_DIRECTML,
                acceleration=AccelerationBackend.DIRECTML,
                gpu_index=0,
                ngl=0,
                context_size=4096,
                confidence=0.9,
                reason="DirectML selected (Windows GPU/NPU acceleration)"
            )
        
        # Try Vulkan
        if backends.get(AccelerationBackend.VULKAN):
            return BackendSelectionResult(
                backend=BackendType.LLAMA_CPP_VULKAN,
                acceleration=AccelerationBackend.VULKAN,
                gpu_index=0,
                ngl=999,  # Vulkan handles layer management
                context_size=4096,
                confidence=0.8,
                reason="Vulkan selected (cross-platform GPU acceleration)"
            )
        
        # Try ROCm
        if backends.get(AccelerationBackend.ROCM):
            return BackendSelectionResult(
                backend=BackendType.LLAMA_CPP_ROCM,
                acceleration=AccelerationBackend.ROCM,
                gpu_index=0,
                ngl=999,  # ROCm handles layer management
                context_size=4096,
                confidence=0.8,
                reason="ROCm selected (AMD GPU acceleration)"
            )
        
        # Try Metal
        if backends.get(AccelerationBackend.METAL):
            return BackendSelectionResult(
                backend=BackendType.LLAMA_CPP_METAL,
                acceleration=AccelerationBackend.METAL,
                gpu_index=0,
                ngl=999,  # Metal handles layer management
                context_size=4096,
                confidence=0.8,
                reason="Metal selected (Apple GPU acceleration)"
            )
        
        # CPU fallback
        return BackendSelectionResult(
            backend=BackendType.LLAMA_CPP_CPU,
            acceleration=AccelerationBackend.CPU,
            gpu_index=0,
            ngl=0,
            context_size=4096,
            confidence=0.6,
            reason="CPU fallback selected (no GPU acceleration available)"
        )
    
    def _validate_user_preference(
        self,
        preferred_backend: BackendType,
        model_type: ModelType,
        model_size_gb: Optional[float],
        model_layers: Optional[int]
    ) -> BackendSelectionResult:
        """
        Validate and configure user's preferred backend.
        
        Args:
            preferred_backend: User's backend choice
            model_type: Type of model
            model_size_gb: Model size in GB
            model_layers: Number of layers
            
        Returns:
            BackendSelectionResult with user preference
        """
        # Validate BitNet model with non-BitNet backend
        if model_type == ModelType.BITNET_158:
            if preferred_backend not in [BackendType.BITNET_CPU, BackendType.BITNET_GPU]:
                logger.warning(
                    f"BitNet model requires BitNet backend, "
                    f"user requested {preferred_backend.value}"
                )
                return self._select_bitnet_backend(model_size_gb, model_layers)
        
        # Honor user preference with best effort configuration
        if preferred_backend == BackendType.BITNET_GPU:
            if self.acceleration_detector.has_cuda():
                return BackendSelectionResult(
                    backend=BackendType.BITNET_GPU,
                    acceleration=AccelerationBackend.CUDA,
                    gpu_index=0,
                    ngl=0,
                    context_size=4096,
                    confidence=0.9,
                    reason="User override: BitNet GPU"
                )
            else:
                logger.warning("User requested BitNet GPU but CUDA not available")
                return self._select_bitnet_backend(model_size_gb, model_layers)
        
        elif preferred_backend == BackendType.BITNET_CPU:
            return BackendSelectionResult(
                backend=BackendType.BITNET_CPU,
                acceleration=AccelerationBackend.CPU,
                gpu_index=0,
                ngl=0,
                context_size=4096,
                confidence=0.9,
                reason="User override: BitNet CPU"
            )
        
        elif preferred_backend in [
            BackendType.LLAMA_CPP_CPU,
            BackendType.LLAMA_CPP_CUDA,
            BackendType.LLAMA_CPP_VULKAN,
            BackendType.LLAMA_CPP_ROCM,
            BackendType.LLAMA_CPP_METAL,
            BackendType.ONNX_CPU,
            BackendType.ONNX_CUDA,
            BackendType.ONNX_DIRECTML,
            BackendType.ONNX_NPU,
            BackendType.LMSTUDIO,
        ]:
            return self._select_llamacpp_backend(model_size_gb, model_layers)
        
        # Unknown backend
        logger.warning(f"Unknown backend preference: {preferred_backend.value}")
        return self.select_backend(
            model_type=model_type,
            model_size_gb=model_size_gb,
            model_layers=model_layers,
            strategy=SelectionStrategy.AUTO
        )
    
    def get_available_backends(self) -> List[BackendType]:
        """
        Get list of available backends.
        
        Returns:
            List of available BackendType values
        """
        available: List[BackendType] = []
        
        backends = self.acceleration_detector.detect_all()
        
        # BitNet backends
        if backends.get(AccelerationBackend.CUDA):
            available.append(BackendType.BITNET_GPU)
        available.append(BackendType.BITNET_CPU)
        
        # ONNX Runtime backends
        if backends.get(AccelerationBackend.NPU):
            available.append(BackendType.ONNX_NPU)
        if backends.get(AccelerationBackend.DIRECTML):
            available.append(BackendType.ONNX_DIRECTML)
        if backends.get(AccelerationBackend.CUDA):
            available.append(BackendType.ONNX_CUDA)
        available.append(BackendType.ONNX_CPU)
        
        # llama.cpp backends
        if backends.get(AccelerationBackend.CUDA):
            available.append(BackendType.LLAMA_CPP_CUDA)
        if backends.get(AccelerationBackend.VULKAN):
            available.append(BackendType.LLAMA_CPP_VULKAN)
        if backends.get(AccelerationBackend.ROCM):
            available.append(BackendType.LLAMA_CPP_ROCM)
        if backends.get(AccelerationBackend.METAL):
            available.append(BackendType.LLAMA_CPP_METAL)
        available.append(BackendType.LLAMA_CPP_CPU)
        
        # MediaPipe backends
        if backends.get(AccelerationBackend.NPU):
            available.append(BackendType.MEDIAPIPE_NPU)
        available.append(BackendType.MEDIAPIPE_GPU)  # May work with GPU delegate
        available.append(BackendType.MEDIAPIPE_CPU)
        
        # LM Studio backend (external)
        available.append(BackendType.LMSTUDIO)
        
        return available

