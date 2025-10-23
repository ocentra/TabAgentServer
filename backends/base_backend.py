"""
Base backend interface for all inference backends.

Provides common abstraction for different inference engines:
- Transformers (PyTorch/SafeTensors) - Python
- ONNX Runtime - Python (will migrate to Rust)
- LiteRT/MediaPipe - Python (will migrate to Rust)
- GGUF/llama.cpp - Rust
- BitNet - Rust
"""

from abc import ABC, abstractmethod
from typing import Any, Dict, List, Optional
from pathlib import Path
import logging

logger = logging.getLogger(__name__)


class BaseInferenceBackend(ABC):
    """
    Abstract base class for all inference backends.
    
    Provides common interface that all backends must implement.
    """
    
    def __init__(self):
        """Initialize base backend"""
        self.model: Optional[Any] = None
        self.tokenizer: Optional[Any] = None
        self.model_path: Optional[Path] = None
        self.config: Optional[Dict[str, Any]] = None
        self._is_loaded = False
    
    @abstractmethod
    def load_model(
        self,
        model_path: str,
        task: str,
        **kwargs
    ) -> bool:
        """
        Load model for inference.
        
        Args:
            model_path: Path to model file or HuggingFace repo ID
            task: Task type (text-generation, feature-extraction, etc.)
            **kwargs: Backend-specific configuration
            
        Returns:
            True if successful, False otherwise
        """
        pass
    
    @abstractmethod
    def generate(
        self,
        prompt: str,
        **generation_params
    ) -> Dict[str, Any]:
        """
        Generate text from prompt.
        
        Args:
            prompt: Input text
            **generation_params: Backend-specific generation parameters
            
        Returns:
            Dictionary with generated text and metadata
        """
        pass
    
    def is_model_loaded(self) -> bool:
        """
        Check if model is currently loaded.
        
        Returns:
            True if model is loaded
        """
        return self._is_loaded
    
    def unload_model(self) -> bool:
        """
        Unload model and free resources.
        
        Returns:
            True if successful
        """
        self.model = None
        self.tokenizer = None
        self.model_path = None
        self.config = None
        self._is_loaded = False
        logger.info(f"{self.__class__.__name__} unloaded")
        return True
    
    def get_model_info(self) -> Dict[str, Any]:
        """
        Get information about currently loaded model.
        
        Returns:
            Dictionary with model metadata
        """
        if not self._is_loaded:
            return {
                "loaded": False,
                "model_path": None
            }
        
        return {
            "loaded": True,
            "model_path": str(self.model_path) if self.model_path else None,
            "backend": self.__class__.__name__,
            "config": self.config
        }


class TextGenerationBackend(BaseInferenceBackend):
    """Base class for text generation backends"""
    pass


class EmbeddingBackend(BaseInferenceBackend):
    """Base class for embedding/feature extraction backends"""
    
    @abstractmethod
    def embed(self, text: str, **kwargs) -> List[float]:
        """
        Generate embeddings for text.
        
        Args:
            text: Input text
            **kwargs: Backend-specific parameters
            
        Returns:
            List of embedding values
        """
        pass


class MultimodalBackend(BaseInferenceBackend):
    """Base class for multimodal (vision + text) backends"""
    
    @abstractmethod
    def process_image(self, image_path: str, prompt: str, **kwargs) -> Dict[str, Any]:
        """
        Process image with text prompt.
        
        Args:
            image_path: Path to image file
            prompt: Text prompt
            **kwargs: Backend-specific parameters
            
        Returns:
            Dictionary with results
        """
        pass

