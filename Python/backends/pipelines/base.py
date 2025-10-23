"""
BasePipeline - Abstract base class for all ML pipelines

Mirrors the extension's BasePipeline.ts structure.
Each specialized pipeline inherits from this and implements:
- pipeline_type() - Task identifier
- load() - Model loading logic
- generate() - Inference logic
"""

import logging
from typing import Any, Dict, Optional
from abc import ABC, abstractmethod

logger = logging.getLogger(__name__)


class BasePipeline(ABC):
    """
    Base class for all specialized pipelines.
    
    Provides shared functionality and enforces consistent API.
    Mirrors the Rust Pipeline trait and TypeScript BasePipeline.
    """
    
    def __init__(self):
        self.model = None
        self.processor = None
        self.tokenizer = None
        self._loaded = False
        self.backend = None
        self.backend_type = None
        self.model_id = None
    
    @abstractmethod
    def pipeline_type(self) -> str:
        """Return the pipeline type (e.g., 'image-to-text')"""
        pass
    
    def is_loaded(self) -> bool:
        """Check if model is loaded"""
        return self._loaded
    
    @abstractmethod
    def load(self, model_id: str, options: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """
        Load the model.
        
        Args:
            model_id: Model identifier (repo or path)
            options: Optional loading parameters including model_info
            
        Returns:
            Result dict with status
        """
        pass
    
    @abstractmethod
    def generate(self, input_data: Dict[str, Any]) -> Dict[str, Any]:
        """
        Run inference.
        
        Args:
            input_data: Input parameters (text, image, audio, etc.)
            
        Returns:
            Generated output
        """
        pass
    
    def unload(self) -> Dict[str, Any]:
        """Unload model to free resources"""
        self.model = None
        self.processor = None
        self.tokenizer = None
        self.backend = None
        self._loaded = False
        return {"status": "success", "message": "Model unloaded"}
    
    def get_config(self) -> Optional[Dict[str, Any]]:
        """Get current configuration"""
        if self.model_id:
            return {
                "model_id": self.model_id,
                "backend_type": self.backend_type,
                "pipeline_type": self.pipeline_type()
            }
        return None

