"""
Unified Request Handler
=======================

Single source of truth for ALL request handling.
Used by BOTH HTTP API and native messaging.

Flow:
    HTTP Route       ┐
                     ├──→ UnifiedRequestHandler ──→ InferenceService ──→ Backends
    Native Messaging ┘

Benefits:
- DRY: One implementation for both interfaces
- Easy to debug: Single code path
- Modular: Clear separation of concerns
- Feature parity: HTTP and native always have same features
"""

import logging
from typing import Optional, Dict, Any, List, AsyncGenerator
import time

from .message_types import (
    ChatMessage,
    InferenceSettings,
    BackendType,
)
from .inference_service import get_inference_service
from .resource_manager import (
    get_resource_manager,
    ResourceManager,
    ModelResourceEstimate,
    OffloadStrategy,
)
from .model_tracker import get_model_tracker, ModelTracker

logger = logging.getLogger(__name__)


class UnifiedRequestHandler:
    """
    Unified handler for all inference requests.
    
    Provides consistent logic for both HTTP API and native messaging.
    All business logic lives here - interfaces just call these methods.
    """
    
    def __init__(self):
        """Initialize unified handler"""
        self._service = get_inference_service()
        self._global_settings = InferenceSettings()
        self._resource_manager = get_resource_manager()
        self._model_tracker = get_model_tracker()
        logger.info("UnifiedRequestHandler initialized")
    
    # ==========================================================================
    # CHAT & GENERATION
    # ==========================================================================
    
    async def chat(
        self,
        messages: List[ChatMessage],
        settings: Optional[InferenceSettings] = None
    ) -> Dict[str, Any]:
        """
        Generate chat response (non-streaming).
        
        Args:
            messages: Chat messages
            settings: Optional settings (uses global if not provided)
            
        Returns:
            Dictionary with text and metadata
        """
        if not self._service.is_model_loaded():
            raise RuntimeError("No model loaded")
        
        effective_settings = settings or self._global_settings
        
        # Generate using InferenceService
        result = await self._service.generate(
            messages=messages,
            settings=effective_settings
        )
        
        return {
            "text": result.get("text", ""),
            "finish_reason": result.get("finish_reason", "stop"),
            "backend": str(self._service.get_backend_type()),
        }
    
    async def chat_stream(
        self,
        messages: List[ChatMessage],
        settings: Optional[InferenceSettings] = None
    ) -> AsyncGenerator[str, None]:
        """
        Generate chat response (streaming).
        
        Args:
            messages: Chat messages
            settings: Optional settings
            
        Yields:
            Text chunks as they're generated
        """
        if not self._service.is_model_loaded():
            raise RuntimeError("No model loaded")
        
        effective_settings = settings or self._global_settings
        
        # Stream using InferenceService
        # Note: InferenceService.generate handles streaming via callback
        # For async generators, we'd need to refactor
        # For now, yield the complete result
        result = await self._service.generate(
            messages=messages,
            settings=effective_settings
        )
        
        yield result.get("text", "")
    
    def stop_generation(self) -> Dict[str, Any]:
        """
        Stop ongoing generation.
        
        Returns:
            Status dictionary
        """
        # InferenceService doesn't have halt yet - backends do
        manager = self._service.get_active_manager()
        if manager and hasattr(manager, 'halt_generation'):
            manager.halt_generation()
            return {"stopped": True, "backend": str(self._service.get_backend_type())}
        
        return {"stopped": False, "message": "No active generation"}
    
    # ==========================================================================
    # MODEL MANAGEMENT
    # ==========================================================================
    
    def load_model(
        self,
        model_path: str,
        recipe: Optional[str] = None
    ) -> Dict[str, Any]:
        """
        Load model for inference.
        
        Args:
            model_path: Path to model (HuggingFace repo or local path)
            recipe: Deprecated, kept for compatibility
            
        Returns:
            Status dictionary
        """
        # Load via InferenceService (uses Rust detection + pipelines)
        result = self._service.load_model(model_path)
        
        return {
            "status": result.get("status", "error"),
            "message": result.get("message", ""),
            "backend": str(self._service.get_backend_type())
        }
    
    def unload_model(self) -> Dict[str, Any]:
        """
        Unload current model.
        
        Returns:
            Status dictionary
        """
        manager = self._service.get_active_manager()
        if manager and hasattr(manager, 'unload_model'):
            manager.unload_model()
            return {"status": "success", "message": "Model unloaded"}
        
        return {"status": "error", "message": "No model to unload"}
    
    def get_model_state(self) -> Dict[str, Any]:
        """
        Get current model state.
        
        Returns:
            Model state dictionary
        """
        manager = self._service.get_active_manager()
        if manager and hasattr(manager, 'get_state'):
            state = manager.get_state()
            state["backend"] = str(self._service.get_backend_type())
            return state
        
        return {
            "isReady": False,
            "backend": None
        }
    
    # ==========================================================================
    # EMBEDDINGS & RAG
    # ==========================================================================
    
    async def generate_embeddings(
        self,
        texts: List[str],
        model: str = "default"
    ) -> Dict[str, Any]:
        """
        Generate embeddings for texts.
        
        Args:
            texts: List of texts to embed
            model: Model identifier
            
        Returns:
            Dictionary with embeddings
        """
        if not self._service.is_model_loaded():
            raise RuntimeError("No model loaded")
        
        manager = self._service.get_active_manager()
        
        if not manager or not hasattr(manager, 'generate_embeddings'):
            raise NotImplementedError("Backend does not support embeddings")
        
        embeddings = await manager.generate_embeddings(texts)
        
        return {
            "embeddings": embeddings,
            "model": model,
            "backend": str(self._service.get_backend_type())
        }
    
    async def rerank_documents(
        self,
        query: str,
        documents: List[str],
        model: str = "default",
        top_k: Optional[int] = None
    ) -> Dict[str, Any]:
        """
        Rerank documents by relevance.
        
        Args:
            query: Search query
            documents: Documents to rerank
            model: Model identifier
            top_k: Top K results
            
        Returns:
            Dictionary with ranked results
        """
        if not self._service.is_model_loaded():
            raise RuntimeError("No model loaded")
        
        manager = self._service.get_active_manager()
        
        # Check if backend has dedicated reranker
        if hasattr(manager, 'rerank_documents'):
            reranked = await manager.rerank_documents(query, documents, top_k)
            return reranked
        
        # Fallback: Use embeddings for similarity
        elif hasattr(manager, 'generate_embeddings'):
            from core.embedding_eval import EmbeddingEvaluator, SimilarityMetric
            
            all_texts = [query] + documents
            embeddings = await manager.generate_embeddings(all_texts)
            
            query_embedding = embeddings[0]
            doc_embeddings = embeddings[1:]
            
            # Find top-K similar
            similar_results = EmbeddingEvaluator.find_top_k_similar(
                query_embedding,
                doc_embeddings,
                k=top_k or len(documents),
                metric=SimilarityMetric.COSINE
            )
            
            reranked = [
                {
                    "index": result.index,
                    "document": documents[result.index],
                    "score": result.score
                }
                for result in similar_results
            ]
            
            return {
                "results": reranked,
                "total_tokens": len(all_texts) * 50
            }
        
        else:
            raise NotImplementedError("Backend does not support reranking or embeddings")
    
    # ==========================================================================
    # PARAMETERS & CONFIGURATION
    # ==========================================================================
    
    def get_params(self) -> Dict[str, Any]:
        """
        Get current generation parameters.
        
        Returns:
            Current parameter values
        """
        return {
            "temperature": self._global_settings.temperature,
            "top_p": self._global_settings.top_p,
            "top_k": self._global_settings.top_k,
            "max_new_tokens": self._global_settings.max_new_tokens,
            "repetition_penalty": self._global_settings.repetition_penalty,
            "do_sample": self._global_settings.do_sample,
        }
    
    def set_params(self, params: Dict[str, Any]) -> Dict[str, Any]:
        """
        Set generation parameters.
        
        Args:
            params: Parameter values to update
            
        Returns:
            Updated parameter values
        """
        # Update global settings
        for key, value in params.items():
            if hasattr(self._global_settings, key):
                setattr(self._global_settings, key, value)
        
        logger.info(f"Parameters updated: {params}")
        
        return self.get_params()
    
    def get_recipes(self) -> Dict[str, Any]:
        """
        List available recipes.
        
        Returns:
            Dictionary of recipes
        """
        from core.recipe_types import RecipeRegistry
        
        recipes = RecipeRegistry.get_all_recipes()
        
        return {
            "recipes": [
                {
                    "recipe": info.recipe.value,
                    "backend": info.backend.value,
                    "acceleration": info.acceleration.value,
                    "file_format": info.file_format,
                    "description": info.description,
                    "hardware_required": info.hardware_required,
                    "os_support": info.os_support
                }
                for info in recipes
            ],
            "total": len(recipes)
        }
    
    def get_registered_models(self) -> Dict[str, Any]:
        """
        List available models from catalog.
        
        Returns:
            Dictionary of available models
        """
        from Python.models import ModelLibrary
        
        library = ModelLibrary()
        all_models = library.list_models()
        
        return {
            "models": {
                model.name: {
                    "repo": model.repo,
                    "type": model.model_type.value,
                    "description": model.description,
                    "size_gb": model.size_gb,
                    "context_length": model.context_length,
                    "recommended": model.recommended,
                    "variants": model.variants,
                }
                for model in all_models
            },
            "total": len(all_models)
        }
    
    # ==========================================================================
    # RESOURCE MANAGEMENT (For Agentic Systems)
    # ==========================================================================
    
    def query_resources(self) -> Dict[str, Any]:
        """
        Query available VRAM/RAM resources.
        
        Critical for agentic systems to check before loading models.
        
        Returns:
            Resource status with available VRAM/RAM
        """
        status = self._resource_manager.get_resource_status()
        allocations = self._resource_manager.get_all_allocations()
        
        return {
            "resources": status.to_dict(),
            "allocations": {
                model_id: {
                    "vram_mb": alloc.vram_mb,
                    "ram_mb": alloc.ram_mb,
                    "backend": alloc.backend
                }
                for model_id, alloc in allocations.items()
            },
            "loaded_models_count": self._model_tracker.count_models(),
            "by_backend": self._model_tracker.count_by_backend()
        }
    
    def estimate_model_size(self, model_path: str) -> Dict[str, Any]:
        """
        Estimate memory requirements for a model.
        
        Agents can query this BEFORE loading to decide strategy.
        
        Args:
            model_path: Path to model file
            
        Returns:
            Size estimate with offload options
        """
        try:
            # Get estimate
            estimate = self._resource_manager.estimate_model_size(model_path)
            
            # Get current resources
            status = self._resource_manager.get_resource_status()
            
            # Get offload suggestions
            options = self._resource_manager.suggest_offload_strategies(
                estimate,
                status.vram_available_mb,
                status.ram_available_mb
            )
            
            return {
                "model_path": model_path,
                "estimate": {
                    "total_size_mb": estimate.total_size_mb,
                    "vram_required_mb": estimate.vram_required_mb,
                    "ram_required_mb": estimate.ram_required_mb,
                    "layer_count": estimate.layer_count,
                    "mb_per_layer": estimate.mb_per_layer
                },
                "available": {
                    "vram_mb": status.vram_available_mb,
                    "ram_mb": status.ram_available_mb
                },
                "can_load": len(options) > 0,
                "options": [
                    {
                        "strategy": opt.strategy,
                        "vram_layers": opt.vram_layers,
                        "ram_layers": opt.ram_layers,
                        "vram_mb": opt.vram_mb,
                        "ram_mb": opt.ram_mb,
                        "speed": opt.speed_rating,
                        "description": opt.description
                    }
                    for opt in options
                ]
            }
        
        except FileNotFoundError as e:
            return {
                "error": str(e),
                "can_load": False
            }
    
    def list_loaded_models(self) -> Dict[str, Any]:
        """
        List all currently loaded models.
        
        Agentic systems need to know what's already loaded.
        
        Returns:
            List of loaded models with states
        """
        models = self._model_tracker.list_models()
        active_id = self._model_tracker.get_active_model_id()
        
        return {
            "models": [model.to_dict() for model in models],
            "total": len(models),
            "active_model_id": active_id,
            "by_backend": self._model_tracker.count_by_backend()
        }
    
    def select_active_model(self, model_id: str) -> Dict[str, Any]:
        """
        Select which loaded model to use for inference.
        
        Args:
            model_id: Model identifier to activate
            
        Returns:
            Status dictionary
        """
        success = self._model_tracker.set_active_model(model_id)
        
        if success:
            return {
                "active_model_id": model_id,
                "backend": str(self._model_tracker.get_active_model().backend_type)
            }
        else:
            raise ValueError(f"Model not found: {model_id}")
    
    # ==========================================================================
    # STATE QUERIES
    # ==========================================================================
    
    def is_model_loaded(self) -> bool:
        """Check if model is loaded"""
        return self._service.is_model_loaded()
    
    def get_backend_type(self) -> Optional[BackendType]:
        """Get active backend type"""
        return self._service.get_backend_type()


# Global singleton
_unified_handler: Optional[UnifiedRequestHandler] = None


def get_unified_handler() -> UnifiedRequestHandler:
    """
    Get global unified handler instance.
    
    Returns:
        UnifiedRequestHandler singleton
    """
    global _unified_handler
    if _unified_handler is None:
        _unified_handler = UnifiedRequestHandler()
    return _unified_handler

