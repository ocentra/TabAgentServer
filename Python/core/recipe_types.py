"""
Recipe Types for Model Loading

A "recipe" defines HOW a model runs:
- Backend (BitNet, ONNX, llama.cpp, etc.)
- Hardware acceleration (CPU, GPU, NPU)
- Quantization strategy

Inspired by Lemonade's recipe system but adapted for TabAgent's backends.
"""

from enum import Enum
from typing import Optional, List
from dataclasses import dataclass

from .message_types import BackendType, AccelerationBackend


class RecipeType(str, Enum):
    """
    Recipe types for model loading.
    
    Format: {backend}-{acceleration}
    """
    # BitNet recipes
    BITNET_CPU = "bitnet-cpu"
    BITNET_GPU = "bitnet-gpu"
    
    # ONNX Runtime recipes (like Lemonade's oga-*)
    ONNX_CPU = "onnx-cpu"           # Like oga-cpu
    ONNX_DIRECTML = "onnx-directml"  # Like oga-igpu (AMD iGPU/dGPU)
    ONNX_NPU = "onnx-npu"           # Like oga-npu (AMD Ryzen AI NPU)
    ONNX_HYBRID = "onnx-hybrid"     # Like oga-hybrid (NPU + iGPU)
    ONNX_CUDA = "onnx-cuda"         # NVIDIA GPU
    ONNX_ROCM = "onnx-rocm"         # AMD GPU (Linux)
    
    # llama.cpp recipes
    LLAMA_CPU = "llama-cpu"
    LLAMA_CUDA = "llama-cuda"       # NVIDIA GPU
    LLAMA_VULKAN = "llama-vulkan"   # AMD GPU (Vulkan)
    LLAMA_ROCM = "llama-rocm"       # AMD GPU (ROCm)
    LLAMA_METAL = "llama-metal"     # Apple Silicon
    
    # MediaPipe recipes
    MEDIAPIPE = "mediapipe"
    MEDIAPIPE_GPU = "mediapipe-gpu"
    
    # Future: HuggingFace Transformers (like Lemonade's hf-*)
    HF_CPU = "hf-cpu"               # PyTorch CPU
    HF_DGPU = "hf-dgpu"             # PyTorch GPU


@dataclass
class RecipeInfo:
    """
    Information about a recipe.
    
    Attributes:
        recipe: Recipe type
        backend: Backend to use
        acceleration: Acceleration backend
        file_format: Expected model file format
        description: Human-readable description
        hardware_required: Hardware requirements
        os_support: Supported operating systems
    """
    recipe: RecipeType
    backend: BackendType
    acceleration: AccelerationBackend
    file_format: str
    description: str
    hardware_required: str
    os_support: List[str]


class RecipeRegistry:
    """Registry of available recipes and their configurations"""
    
    RECIPES = {
        # BitNet recipes
        RecipeType.BITNET_CPU: RecipeInfo(
            recipe=RecipeType.BITNET_CPU,
            backend=BackendType.BITNET_CPU,
            acceleration=AccelerationBackend.CPU,
            file_format=".gguf (BitNet)",
            description="BitNet 1.58-bit models on CPU. Ultra-efficient.",
            hardware_required="Any CPU",
            os_support=["Windows", "Linux", "macOS"]
        ),
        RecipeType.BITNET_GPU: RecipeInfo(
            recipe=RecipeType.BITNET_GPU,
            backend=BackendType.BITNET_GPU,
            acceleration=AccelerationBackend.CUDA,
            file_format=".gguf (BitNet)",
            description="BitNet 1.58-bit models on NVIDIA GPU. Very fast.",
            hardware_required="NVIDIA GPU with CUDA",
            os_support=["Windows", "Linux"]
        ),
        
        # ONNX Runtime recipes
        RecipeType.ONNX_CPU: RecipeInfo(
            recipe=RecipeType.ONNX_CPU,
            backend=BackendType.ONNX_CPU,
            acceleration=AccelerationBackend.CPU,
            file_format=".onnx",
            description="ONNX models on CPU. Broad compatibility.",
            hardware_required="Any CPU",
            os_support=["Windows", "Linux", "macOS"]
        ),
        RecipeType.ONNX_DIRECTML: RecipeInfo(
            recipe=RecipeType.ONNX_DIRECTML,
            backend=BackendType.ONNX_DIRECTML,
            acceleration=AccelerationBackend.DIRECTML,
            file_format=".onnx",
            description="ONNX models on DirectML (AMD/Intel/NVIDIA GPU). Windows optimized.",
            hardware_required="DirectX 12 compatible GPU",
            os_support=["Windows"]
        ),
        RecipeType.ONNX_NPU: RecipeInfo(
            recipe=RecipeType.ONNX_NPU,
            backend=BackendType.ONNX_NPU,
            acceleration=AccelerationBackend.NPU,
            file_format=".onnx",
            description="ONNX models on AMD Ryzen AI NPU. Power-efficient.",
            hardware_required="AMD Ryzen AI 300+ series",
            os_support=["Windows"]
        ),
        RecipeType.ONNX_HYBRID: RecipeInfo(
            recipe=RecipeType.ONNX_HYBRID,
            backend=BackendType.ONNX_HYBRID,
            acceleration=AccelerationBackend.HYBRID,
            file_format=".onnx",
            description="ONNX models on NPU + iGPU hybrid. Best AMD Ryzen AI performance.",
            hardware_required="AMD Ryzen AI 300+ series",
            os_support=["Windows"]
        ),
        RecipeType.ONNX_CUDA: RecipeInfo(
            recipe=RecipeType.ONNX_CUDA,
            backend=BackendType.ONNX_CUDA,
            acceleration=AccelerationBackend.CUDA,
            file_format=".onnx",
            description="ONNX models on NVIDIA GPU via CUDA.",
            hardware_required="NVIDIA GPU with CUDA",
            os_support=["Windows", "Linux"]
        ),
        
        # llama.cpp recipes
        RecipeType.LLAMA_CPU: RecipeInfo(
            recipe=RecipeType.LLAMA_CPU,
            backend=BackendType.LLAMA_CPP_CPU,
            acceleration=AccelerationBackend.CPU,
            file_format=".gguf",
            description="GGUF models on CPU via llama.cpp.",
            hardware_required="Any CPU",
            os_support=["Windows", "Linux", "macOS"]
        ),
        RecipeType.LLAMA_CUDA: RecipeInfo(
            recipe=RecipeType.LLAMA_CUDA,
            backend=BackendType.LLAMA_CPP_CUDA,
            acceleration=AccelerationBackend.CUDA,
            file_format=".gguf",
            description="GGUF models on NVIDIA GPU via llama.cpp.",
            hardware_required="NVIDIA GPU with CUDA",
            os_support=["Windows", "Linux"]
        ),
        RecipeType.LLAMA_VULKAN: RecipeInfo(
            recipe=RecipeType.LLAMA_VULKAN,
            backend=BackendType.LLAMA_CPP_VULKAN,
            acceleration=AccelerationBackend.VULKAN,
            file_format=".gguf",
            description="GGUF models on AMD GPU via Vulkan.",
            hardware_required="Vulkan-compatible GPU",
            os_support=["Windows", "Linux"]
        ),
        RecipeType.LLAMA_METAL: RecipeInfo(
            recipe=RecipeType.LLAMA_METAL,
            backend=BackendType.LLAMA_CPP_METAL,
            acceleration=AccelerationBackend.METAL,
            file_format=".gguf",
            description="GGUF models on Apple Silicon via Metal.",
            hardware_required="Apple M1/M2/M3",
            os_support=["macOS"]
        ),
        
        # MediaPipe recipes
        RecipeType.MEDIAPIPE: RecipeInfo(
            recipe=RecipeType.MEDIAPIPE,
            backend=BackendType.MEDIAPIPE,
            acceleration=AccelerationBackend.CPU,
            file_format=".task",
            description="MediaPipe tasks (multimodal: vision, text, audio).",
            hardware_required="Any CPU",
            os_support=["Windows", "Linux", "macOS"]
        ),
    }
    
    @classmethod
    def get_recipe_info(cls, recipe: RecipeType) -> Optional[RecipeInfo]:
        """Get information about a recipe"""
        return cls.RECIPES.get(recipe)
    
    @classmethod
    def get_all_recipes(cls) -> List[RecipeInfo]:
        """Get all available recipes"""
        return list(cls.RECIPES.values())
    
    @classmethod
    def get_recipes_for_file_format(cls, file_format: str) -> List[RecipeInfo]:
        """Get recipes that support a file format"""
        return [
            info for info in cls.RECIPES.values()
            if file_format.lower() in info.file_format.lower()
        ]
    
    @classmethod
    def auto_detect_recipe(
        cls,
        file_path: str,
        has_cuda: bool = False,
        has_npu: bool = False,
        has_directml: bool = False,
        has_vulkan: bool = False,
        has_metal: bool = False
    ) -> Optional[RecipeType]:
        """
        Auto-detect best recipe based on file and hardware.
        
        Args:
            file_path: Path to model file
            has_cuda: NVIDIA GPU available
            has_npu: NPU available (AMD Ryzen AI)
            has_directml: DirectML available
            has_vulkan: Vulkan available
            has_metal: Metal available (Apple Silicon)
            
        Returns:
            Best recipe for hardware or None
        """
        file_path_lower = file_path.lower()
        
        # Detect file format
        if file_path_lower.endswith('.gguf'):
            # llama.cpp or BitNet
            # Check if BitNet model
            if 'bitnet' in file_path_lower:
                return RecipeType.BITNET_GPU if has_cuda else RecipeType.BITNET_CPU
            # Standard GGUF
            if has_cuda:
                return RecipeType.LLAMA_CUDA
            elif has_vulkan:
                return RecipeType.LLAMA_VULKAN
            elif has_metal:
                return RecipeType.LLAMA_METAL
            else:
                return RecipeType.LLAMA_CPU
        
        elif file_path_lower.endswith('.onnx'):
            # ONNX Runtime
            if has_npu and has_directml:
                return RecipeType.ONNX_HYBRID  # Best for AMD Ryzen AI
            elif has_npu:
                return RecipeType.ONNX_NPU
            elif has_cuda:
                return RecipeType.ONNX_CUDA
            elif has_directml:
                return RecipeType.ONNX_DIRECTML
            else:
                return RecipeType.ONNX_CPU
        
        elif file_path_lower.endswith('.task'):
            # MediaPipe
            return RecipeType.MEDIAPIPE
        
        return None


@dataclass
class ModelCapabilities:
    """
    Model capability flags.
    
    Tracks what a model can do beyond basic text generation.
    """
    reasoning: bool = False      # DeepSeek-style reasoning models
    vision: bool = False         # Image input support
    audio: bool = False          # Audio input support
    video: bool = False          # Video input support
    function_calling: bool = False  # Function/tool calling
    mmproj_path: Optional[str] = None  # Multimodal projector for vision


def recipe_to_backend_type(recipe: RecipeType) -> BackendType:
    """
    Map recipe to backend type.
    
    Args:
        recipe: Recipe type
        
    Returns:
        Corresponding backend type
    """
    mapping = {
        RecipeType.BITNET_CPU: BackendType.BITNET_CPU,
        RecipeType.BITNET_GPU: BackendType.BITNET_GPU,
        
        RecipeType.ONNX_CPU: BackendType.ONNX_CPU,
        RecipeType.ONNX_DIRECTML: BackendType.ONNX_DIRECTML,
        RecipeType.ONNX_NPU: BackendType.ONNX_NPU,
        RecipeType.ONNX_HYBRID: BackendType.ONNX_HYBRID,
        RecipeType.ONNX_CUDA: BackendType.ONNX_CUDA,
        RecipeType.ONNX_ROCM: BackendType.ONNX_ROCM,
        
        RecipeType.LLAMA_CPU: BackendType.LLAMA_CPP_CPU,
        RecipeType.LLAMA_CUDA: BackendType.LLAMA_CPP_CUDA,
        RecipeType.LLAMA_VULKAN: BackendType.LLAMA_CPP_VULKAN,
        RecipeType.LLAMA_ROCM: BackendType.LLAMA_CPP_ROCM,
        RecipeType.LLAMA_METAL: BackendType.LLAMA_CPP_METAL,
        
        RecipeType.MEDIAPIPE: BackendType.MEDIAPIPE,
        RecipeType.MEDIAPIPE_GPU: BackendType.MEDIAPIPE,
    }
    
    return mapping.get(recipe, BackendType.BITNET_CPU)

