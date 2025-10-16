"""
Model Tracker
=============

Tracks multiple loaded models and their states.
Essential for agentic systems where multiple models run simultaneously.

State Management:
- AVAILABLE: Model in library (not downloaded)
- DOWNLOADED: Model in HF cache (not loaded)
- LOADING: Model being loaded into memory
- LOADED: Model ready for inference
- FAILED: Load failed
"""

import logging
from typing import Dict, List, Optional, Any
from dataclasses import dataclass, field
from enum import Enum
from datetime import datetime

from core.message_types import BackendType

logger = logging.getLogger(__name__)


class ModelState(str, Enum):
    """Model lifecycle states"""
    AVAILABLE = "available"       # In library, not downloaded
    DOWNLOADED = "downloaded"     # Downloaded, not loaded
    LOADING = "loading"           # Being loaded
    LOADED = "loaded"             # Ready for inference
    UNLOADING = "unloading"       # Being unloaded
    FAILED = "failed"             # Load failed


@dataclass
class LoadedModelInfo:
    """
    Information about a loaded model.
    
    Attributes:
        model_id: Unique identifier for this loaded instance
        model_path: Path to model file
        backend_type: Which backend is running it
        manager: Reference to backend manager instance
        state: Current state
        vram_mb: VRAM allocated
        ram_mb: RAM allocated
        vram_layers: Layers on GPU
        ram_layers: Layers on CPU
        loaded_at: When model was loaded
        last_used: Last inference time
        metadata: Additional metadata
    """
    model_id: str
    model_path: str
    backend_type: BackendType
    manager: Any
    state: ModelState = ModelState.LOADING
    vram_mb: int = 0
    ram_mb: int = 0
    vram_layers: int = 0
    ram_layers: int = 0
    loaded_at: datetime = field(default_factory=datetime.now)
    last_used: Optional[datetime] = None
    metadata: Dict[str, Any] = field(default_factory=dict)
    
    def mark_loaded(self) -> None:
        """Mark model as successfully loaded"""
        self.state = ModelState.LOADED
        logger.info(f"Model {self.model_id} marked as LOADED")
    
    def mark_failed(self, error: str) -> None:
        """Mark model load as failed"""
        self.state = ModelState.FAILED
        self.metadata["error"] = error
        logger.error(f"Model {self.model_id} marked as FAILED: {error}")
    
    def mark_used(self) -> None:
        """Update last used timestamp"""
        self.last_used = datetime.now()
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary"""
        return {
            "model_id": self.model_id,
            "model_path": self.model_path,
            "backend": self.backend_type.value,
            "state": self.state.value,
            "resources": {
                "vram_mb": self.vram_mb,
                "ram_mb": self.ram_mb,
                "vram_layers": self.vram_layers,
                "ram_layers": self.ram_layers,
            },
            "loaded_at": self.loaded_at.isoformat(),
            "last_used": self.last_used.isoformat() if self.last_used else None,
            "metadata": self.metadata
        }


class ModelTracker:
    """
    Tracks multiple loaded models.
    
    Enables multi-model scenarios:
    - Orchestrator + worker agents
    - Multiple specialized models
    - Model comparison
    """
    
    def __init__(self):
        """Initialize model tracker"""
        self._loaded_models: Dict[str, LoadedModelInfo] = {}
        self._active_model_id: Optional[str] = None
        logger.info("ModelTracker initialized")
    
    def add_model(self, model_info: LoadedModelInfo) -> None:
        """
        Add a loaded model to tracking.
        
        Args:
            model_info: Loaded model information
        """
        self._loaded_models[model_info.model_id] = model_info
        
        # Set as active if it's the first/only model
        if len(self._loaded_models) == 1:
            self._active_model_id = model_info.model_id
        
        logger.info(f"Model added to tracker: {model_info.model_id} ({model_info.backend_type.value})")
    
    def remove_model(self, model_id: str) -> bool:
        """
        Remove a model from tracking.
        
        Args:
            model_id: Model identifier
            
        Returns:
            True if removed, False if not found
        """
        if model_id in self._loaded_models:
            del self._loaded_models[model_id]
            
            # If active model was removed, switch to another
            if self._active_model_id == model_id:
                if self._loaded_models:
                    self._active_model_id = next(iter(self._loaded_models.keys()))
                else:
                    self._active_model_id = None
            
            logger.info(f"Model removed from tracker: {model_id}")
            return True
        return False
    
    def get_model(self, model_id: str) -> Optional[LoadedModelInfo]:
        """Get info for a specific model"""
        return self._loaded_models.get(model_id)
    
    def list_models(self, backend: Optional[BackendType] = None) -> List[LoadedModelInfo]:
        """
        List all loaded models.
        
        Args:
            backend: Filter by backend type (optional)
            
        Returns:
            List of loaded models
        """
        models = list(self._loaded_models.values())
        
        if backend:
            models = [m for m in models if m.backend_type == backend]
        
        return models
    
    def set_active_model(self, model_id: str) -> bool:
        """
        Set which model is active for inference.
        
        Args:
            model_id: Model identifier
            
        Returns:
            True if set, False if model not found
        """
        if model_id in self._loaded_models:
            self._active_model_id = model_id
            logger.info(f"Active model set to: {model_id}")
            return True
        return False
    
    def get_active_model(self) -> Optional[LoadedModelInfo]:
        """Get currently active model"""
        if self._active_model_id:
            return self._loaded_models.get(self._active_model_id)
        return None
    
    def get_active_model_id(self) -> Optional[str]:
        """Get active model ID"""
        return self._active_model_id
    
    def count_models(self) -> int:
        """Count loaded models"""
        return len(self._loaded_models)
    
    def count_by_backend(self) -> Dict[str, int]:
        """Count models per backend"""
        counts: Dict[str, int] = {}
        for model in self._loaded_models.values():
            backend = model.backend_type.value
            counts[backend] = counts.get(backend, 0) + 1
        return counts
    
    def clear(self) -> None:
        """Clear all tracked models"""
        self._loaded_models.clear()
        self._active_model_id = None
        logger.info("Model tracker cleared")


# Global singleton
_model_tracker: Optional[ModelTracker] = None


def get_model_tracker() -> ModelTracker:
    """Get global model tracker instance"""
    global _model_tracker
    if _model_tracker is None:
        _model_tracker = ModelTracker()
    return _model_tracker

