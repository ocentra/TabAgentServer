"""
Backend Adapter

Direct pipeline adapter for HTTP API.
Uses pipelines directly - no middleman!
"""

import logging
from typing import Optional, AsyncGenerator

from core.message_types import (
    ChatMessage,
    InferenceSettings,
    BackendType,
)
from .backend_manager import BackendInterface
from .types import PerformanceStats

logger = logging.getLogger(__name__)


class InferenceServiceAdapter(BackendInterface):
    """
    Pipeline adapter for FastAPI.
    Uses pipelines directly (same as native_host.py)
    """
    
    def __init__(self):
        """Initialize adapter"""
        from native_host import _loaded_pipelines
        self._pipelines = _loaded_pipelines  # Reference to native_host's pipeline registry
    
    def is_loaded(self) -> bool:
        """Check if model is loaded"""
        return len(self._pipelines) > 0 and any(p.is_loaded() for p in self._pipelines.values())
    
    def get_backend_type(self) -> Optional[BackendType]:
        """Get current backend type"""
        # Return first loaded pipeline's type
        for pipeline in self._pipelines.values():
            if pipeline.is_loaded():
                return BackendType.PYTHON  # Generic type
        return None
    
    def get_model_path(self) -> Optional[str]:
        """Get loaded model path"""
        for pipeline in self._pipelines.values():
            if pipeline.is_loaded() and pipeline.model_id:
                return pipeline.model_id
        return None
    
    async def generate(
        self,
        messages: list[ChatMessage],
        settings: InferenceSettings,
    ) -> str:
        """
        Generate non-streaming response.
        Uses pipeline directly.
        
        Args:
            messages: Chat messages
            settings: Inference settings
            
        Returns:
            Generated text
        """
        # Get first loaded pipeline
        pipeline = next((p for p in self._pipelines.values() if p.is_loaded()), None)
        if not pipeline:
            raise RuntimeError("No model loaded")
        
        # Convert messages to text (simple concatenation for now)
        text = "\n".join([f"{m.role}: {m.content}" for m in messages])
        
        # Call pipeline generate
        result = pipeline.generate({
            "text": text,
            "max_new_tokens": settings.max_new_tokens,
            "temperature": settings.temperature,
        })
        
        if result["status"] == "success":
            return result.get("text", "")
        else:
            raise RuntimeError(result.get("message", "Generation failed"))
    
    async def generate_stream(
        self,
        messages: list[ChatMessage],
        settings: InferenceSettings,
    ) -> AsyncGenerator[str, None]:
        """
        Generate streaming response
        
        Args:
            messages: Chat messages
            settings: Inference settings
            
        Yields:
            Generated tokens
        """
        # Get first loaded pipeline
        pipeline = next((p for p in self._pipelines.values() if p.is_loaded()), None)
        if not pipeline:
            raise RuntimeError("No model loaded")
        
        # TODO: Implement streaming via pipeline
        # For now, fall back to non-streaming
        text = await self.generate(messages, settings)
        yield text
    
    def get_stats(self) -> Optional[PerformanceStats]:
        """Get performance statistics"""
        # TODO: Implement stats tracking in pipelines
        return None


def get_inference_adapter() -> InferenceServiceAdapter:
    """
    Get inference adapter using shared service.
    
    Returns:
        Adapter wrapping InferenceService
    """
    return InferenceServiceAdapter()

