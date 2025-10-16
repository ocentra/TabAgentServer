"""
Model Registry

Pre-configured models with recipes and capabilities.
Supports both system models and user-registered models.
"""

from typing import Dict, Optional, List
from dataclasses import dataclass, field

from core.recipe_types import RecipeType, ModelCapabilities


@dataclass
class RegisteredModel:
    """
    A registered model with metadata.
    
    Attributes:
        model_name: Unique model name (e.g., "Phi-3.5-Mini-ONNX-NPU")
        checkpoint: HuggingFace checkpoint ID
        recipe: Recipe to use for loading
        capabilities: Model capabilities
        description: Human-readable description
        is_user_model: Whether this is a user-registered model
    """
    model_name: str
    checkpoint: str
    recipe: RecipeType
    capabilities: ModelCapabilities = field(default_factory=lambda: ModelCapabilities())
    description: Optional[str] = None
    is_user_model: bool = False


class ModelRegistry:
    """
    Registry of available models.
    
    Maintains both system models (pre-configured) and user models (custom).
    """
    
    # System models (pre-configured, high-quality models)
    SYSTEM_MODELS: Dict[str, RegisteredModel] = {
        # ONNX Runtime models
        "Phi-3.5-Mini-ONNX-NPU": RegisteredModel(
            model_name="Phi-3.5-Mini-ONNX-NPU",
            checkpoint="microsoft/Phi-3.5-mini-instruct",
            recipe=RecipeType.ONNX_NPU,
            description="Phi-3.5 Mini optimized for AMD Ryzen AI NPU"
        ),
        "Phi-3.5-Mini-ONNX-Hybrid": RegisteredModel(
            model_name="Phi-3.5-Mini-ONNX-Hybrid",
            checkpoint="microsoft/Phi-3.5-mini-instruct",
            recipe=RecipeType.ONNX_HYBRID,
            description="Phi-3.5 Mini on NPU + iGPU hybrid (best AMD Ryzen AI)"
        ),
        "Phi-3.5-Mini-ONNX-CPU": RegisteredModel(
            model_name="Phi-3.5-Mini-ONNX-CPU",
            checkpoint="microsoft/Phi-3.5-mini-instruct",
            recipe=RecipeType.ONNX_CPU,
            description="Phi-3.5 Mini on CPU"
        ),
        
        # llama.cpp models
        "Llama-3.2-1B-GGUF-CUDA": RegisteredModel(
            model_name="Llama-3.2-1B-GGUF-CUDA",
            checkpoint="meta-llama/Llama-3.2-1B-Instruct",
            recipe=RecipeType.LLAMA_CUDA,
            description="Llama 3.2 1B on NVIDIA GPU"
        ),
        "Llama-3.2-1B-GGUF-CPU": RegisteredModel(
            model_name="Llama-3.2-1B-GGUF-CPU",
            checkpoint="meta-llama/Llama-3.2-1B-Instruct",
            recipe=RecipeType.LLAMA_CPU,
            description="Llama 3.2 1B on CPU"
        ),
        
        # BitNet models
        "Llama-3.2-1B-BitNet-GPU": RegisteredModel(
            model_name="Llama-3.2-1B-BitNet-GPU",
            checkpoint="1bitLLM/bitnet_b1_58-3B",
            recipe=RecipeType.BITNET_GPU,
            description="BitNet 1.58-bit model on NVIDIA GPU. Ultra-efficient."
        ),
    }
    
    # User models (dynamically registered)
    _user_models: Dict[str, RegisteredModel] = {}
    
    @classmethod
    def register_model(
        cls,
        model_name: str,
        checkpoint: str,
        recipe: RecipeType,
        capabilities: Optional[ModelCapabilities] = None
    ) -> RegisteredModel:
        """
        Register a new user model.
        
        Args:
            model_name: Unique model name (will be prefixed with "user." if not already)
            checkpoint: HuggingFace checkpoint ID
            recipe: Recipe to use
            capabilities: Model capabilities
            
        Returns:
            Registered model
        """
        # Ensure user model has "user." prefix
        if not model_name.startswith("user."):
            model_name = f"user.{model_name}"
        
        model = RegisteredModel(
            model_name=model_name,
            checkpoint=checkpoint,
            recipe=recipe,
            capabilities=capabilities or ModelCapabilities(),
            is_user_model=True
        )
        
        cls._user_models[model_name] = model
        return model
    
    @classmethod
    def unregister_model(cls, model_name: str) -> bool:
        """
        Unregister a user model.
        
        Args:
            model_name: Model name to unregister
            
        Returns:
            True if unregistered, False if not found
        """
        if model_name in cls._user_models:
            del cls._user_models[model_name]
            return True
        return False
    
    @classmethod
    def get_model(cls, model_name: str) -> Optional[RegisteredModel]:
        """
        Get a registered model by name.
        
        Checks both system and user models.
        
        Args:
            model_name: Model name
            
        Returns:
            Registered model or None
        """
        # Check system models first
        if model_name in cls.SYSTEM_MODELS:
            return cls.SYSTEM_MODELS[model_name]
        
        # Check user models
        if model_name in cls._user_models:
            return cls._user_models[model_name]
        
        return None
    
    @classmethod
    def get_all_models(cls) -> Dict[str, RegisteredModel]:
        """
        Get all registered models (system + user).
        
        Returns:
            Dictionary of all models
        """
        all_models = {}
        all_models.update(cls.SYSTEM_MODELS)
        all_models.update(cls._user_models)
        return all_models
    
    @classmethod
    def get_system_models(cls) -> Dict[str, RegisteredModel]:
        """Get only system models"""
        return cls.SYSTEM_MODELS.copy()
    
    @classmethod
    def get_user_models(cls) -> Dict[str, RegisteredModel]:
        """Get only user models"""
        return cls._user_models.copy()
    
    @classmethod
    def get_models_by_recipe(cls, recipe: RecipeType) -> List[RegisteredModel]:
        """
        Get all models for a specific recipe.
        
        Args:
            recipe: Recipe type
            
        Returns:
            List of models using that recipe
        """
        all_models = cls.get_all_models()
        return [
            model for model in all_models.values()
            if model.recipe == recipe
        ]

