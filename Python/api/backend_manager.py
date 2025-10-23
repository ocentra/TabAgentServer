"""
Backend Manager

Unified interface for managing inference backends.
Works for both HTTP server and native messaging (stdin/stdout).
"""

import logging
from typing import Optional, AsyncGenerator
from abc import ABC, abstractmethod

from Python.core.message_types import (
    ChatMessage,
    InferenceSettings,
    BackendType,
    ModelType,
    GenerationCompletePayload,
)
from .types import (
    ChatCompletionRequest,
    ChatCompletionResponse,
    ChatCompletionChunk,
    ChatCompletionChoice,
    ChatCompletionChunkChoice,
    ChatCompletionChunkDelta,
    ChatCompletionMessage,
    ChatCompletionUsage,
    PerformanceStats,
)
from .constants import OpenAIObject, FinishReason, MessageRole

logger = logging.getLogger(__name__)


class BackendInterface(ABC):
    """Abstract interface for inference backends"""
    
    @abstractmethod
    def is_loaded(self) -> bool:
        """Check if model is loaded"""
        pass
    
    @abstractmethod
    def get_backend_type(self) -> Optional[BackendType]:
        """Get current backend type"""
        pass
    
    @abstractmethod
    def get_model_path(self) -> Optional[str]:
        """Get loaded model path"""
        pass
    
    @abstractmethod
    async def generate(
        self,
        messages: list[ChatMessage],
        settings: InferenceSettings,
    ) -> str:
        """Generate non-streaming response"""
        pass
    
    @abstractmethod
    async def generate_stream(
        self,
        messages: list[ChatMessage],
        settings: InferenceSettings,
    ) -> AsyncGenerator[str, None]:
        """Generate streaming response"""
        pass
    
    @abstractmethod
    def get_stats(self) -> Optional[PerformanceStats]:
        """Get performance statistics"""
        pass


class BackendManager:
    """
    Manages multiple inference backends.
    
    Provides unified interface for both HTTP API and native messaging.
    """
    
    def __init__(self):
        self._current_backend: Optional[BackendInterface] = None
        self._model_id: str = "unknown"
        self._is_generating: bool = False
        self._generation_halt_requested: bool = False
        self._global_settings: InferenceSettings = InferenceSettings()  # Persistent settings
        logger.info("BackendManager initialized")
    
    def set_backend(self, backend: BackendInterface, model_id: str) -> None:
        """
        Set active backend
        
        Args:
            backend: Backend implementation
            model_id: Model identifier
        """
        self._current_backend = backend
        self._model_id = model_id
        logger.info(f"Backend set: {backend.get_backend_type()}, model: {model_id}")
    
    def is_model_loaded(self) -> bool:
        """Check if any model is loaded"""
        return (
            self._current_backend is not None 
            and self._current_backend.is_loaded()
        )
    
    def get_backend_type(self) -> Optional[BackendType]:
        """Get current backend type"""
        if self._current_backend:
            return self._current_backend.get_backend_type()
        return None
    
    def get_model_id(self) -> str:
        """Get current model identifier"""
        return self._model_id
    
    async def chat_completion(
        self,
        request: ChatCompletionRequest,
    ) -> ChatCompletionResponse:
        """
        Generate non-streaming chat completion
        
        Args:
            request: Chat completion request
            
        Returns:
            Chat completion response
            
        Raises:
            RuntimeError: If no model is loaded
        """
        if not self.is_model_loaded():
            raise RuntimeError("No model loaded")
        
        assert self._current_backend is not None
        
        # Convert request to inference settings
        settings = request.to_inference_settings()
        
        # Set generation flag
        self._is_generating = True
        self._generation_halt_requested = False
        
        try:
            # Generate response
            generated_text = await self._current_backend.generate(
                request.messages,
                settings,
            )
        finally:
            self._is_generating = False
        
        # Build OpenAI-compatible response
        import time
        
        response = ChatCompletionResponse(
            id=self._generate_id(),
            created=int(time.time()),
            model=request.model,
            choices=[
                ChatCompletionChoice(
                    index=0,
                    message=ChatCompletionMessage(
                        role=MessageRole.ASSISTANT,
                        content=generated_text,
                    ),
                    finish_reason=FinishReason.STOP,
                )
            ],
        )
        
        # Add usage stats if available
        stats = self._current_backend.get_stats()
        if stats and stats.input_tokens and stats.output_tokens:
            response.usage = ChatCompletionUsage(
                prompt_tokens=stats.input_tokens,
                completion_tokens=stats.output_tokens,
                total_tokens=stats.input_tokens + stats.output_tokens,
            )
        
        return response
    
    async def chat_completion_stream(
        self,
        request: ChatCompletionRequest,
    ) -> AsyncGenerator[ChatCompletionChunk, None]:
        """
        Generate streaming chat completion
        
        Args:
            request: Chat completion request
            
        Yields:
            Chat completion chunks
            
        Raises:
            RuntimeError: If no model is loaded
        """
        if not self.is_model_loaded():
            raise RuntimeError("No model loaded")
        
        assert self._current_backend is not None
        
        # Convert request to inference settings
        settings = request.to_inference_settings()
        
        # Set generation flag
        self._is_generating = True
        self._generation_halt_requested = False
        
        try:
            # Generate streaming response
            import time
            chunk_id = self._generate_id()
            
            async for token in self._current_backend.generate_stream(
                request.messages,
                settings,
            ):
                # Check if halt was requested
                if self._generation_halt_requested:
                    logger.info("Generation halted by user request")
                    break
                
                chunk = ChatCompletionChunk(
                    id=chunk_id,
                    created=int(time.time()),
                    model=request.model,
                    choices=[
                        ChatCompletionChunkChoice(
                            index=0,
                            delta=ChatCompletionChunkDelta(
                                content=token,
                            ),
                            finish_reason=None,
                        )
                    ],
                )
                yield chunk
            
            # Send finish chunk
            finish_reason = FinishReason.STOP if not self._generation_halt_requested else FinishReason.STOP
            finish_chunk = ChatCompletionChunk(
                id=chunk_id,
                created=int(time.time()),
                model=request.model,
                choices=[
                    ChatCompletionChunkChoice(
                        index=0,
                        delta=ChatCompletionChunkDelta(),
                        finish_reason=finish_reason,
                    )
                ],
            )
            yield finish_chunk
        finally:
            self._is_generating = False
            self._generation_halt_requested = False
    
    def get_performance_stats(self) -> Optional[PerformanceStats]:
        """Get current performance statistics"""
        if self._current_backend:
            return self._current_backend.get_stats()
        return None
    
    def is_generating(self) -> bool:
        """
        Check if generation is currently in progress.
        
        Returns:
            True if actively generating
        """
        return self._is_generating
    
    async def halt_generation(self) -> dict:
        """
        Halt in-progress generation.
        
        Sets halt flag that backends should check during generation.
        
        Returns:
            Dictionary with halt status and any partial output
        """
        if not self._is_generating:
            logger.warning("Halt requested but no generation in progress")
            return {
                "was_generating": False,
                "partial_output": None,
                "tokens_generated": 0
            }
        
        logger.info("Setting generation halt flag")
        self._generation_halt_requested = True
        
        # Halt flag is checked during generation loop
        # Backends honor this flag in their streaming methods
        
        return {
            "was_generating": True,
            "partial_output": None,  # Backends will populate this
            "tokens_generated": 0
        }
    
    def get_current_settings(self) -> InferenceSettings:
        """
        Get current global generation settings.
        
        Returns:
            Current InferenceSettings
        """
        return self._global_settings
    
    def update_global_settings(self, settings: InferenceSettings) -> None:
        """
        Update global generation settings.
        
        These settings will be used for all subsequent generation requests
        unless overridden in the request.
        
        Args:
            settings: New inference settings
        """
        self._global_settings = settings
        logger.info(f"Global settings updated: temp={settings.temperature}, top_p={settings.top_p}")
    
    async def generate_embeddings(
        self,
        texts: list[str],
        model: str
    ) -> dict:
        """
        Generate embeddings for input texts.
        
        Routes to appropriate backend via InferenceService.
        
        Args:
            texts: List of text strings to embed
            model: Model identifier
            
        Returns:
            Dictionary with embeddings list
            
        Raises:
            NotImplementedError: If backend doesn't support embeddings
        """
        if not self.is_model_loaded():
            raise RuntimeError("No model loaded")
        
        logger.info(f"Generating embeddings for {len(texts)} texts")
        
        # Get actual manager from InferenceService
        from core.inference_service import get_inference_service
        
        service = get_inference_service()
        active_manager = service.get_active_manager()
        
        if not active_manager:
            raise RuntimeError("No active backend manager")
        
        # Check if backend supports embeddings
        if hasattr(active_manager, 'generate_embeddings'):
            embeddings = await active_manager.generate_embeddings(texts)
            
            return {
                "embeddings": embeddings,
                "model": model,
                "backend": str(self.get_backend_type())
            }
        
        else:
            backend_name = type(active_manager).__name__
            raise NotImplementedError(
                f"Backend {backend_name} does not support embeddings. "
                "Load an embedding model with ONNX, llama.cpp, or MediaPipe backend."
            )
    
    async def rerank_documents(
        self,
        query: str,
        documents: list[str],
        model: str,
        top_k: Optional[int] = None
    ) -> dict:
        """
        Rerank documents based on relevance to query.
        
        Uses cross-encoder models to score document relevance.
        
        Args:
            query: Search query
            documents: List of documents to rerank
            model: Reranker model identifier
            top_k: Return top K results (optional)
            
        Returns:
            Dictionary with reranked results
            
        Raises:
            NotImplementedError: If backend doesn't support reranking
        """
        if not self.is_model_loaded():
            raise RuntimeError("No model loaded")
        
        logger.info(f"Reranking {len(documents)} documents")
        
        # Get actual manager from InferenceService
        from core.inference_service import get_inference_service
        
        service = get_inference_service()
        active_manager = service.get_active_manager()
        
        if not active_manager:
            raise RuntimeError("No active backend manager")
        
        # Check if backend supports reranking
        if hasattr(active_manager, 'rerank_documents'):
            reranked = await active_manager.rerank_documents(query, documents, top_k)
            return reranked
        
        # Fallback: Use embeddings for semantic similarity reranking
        elif hasattr(active_manager, 'generate_embeddings'):
            logger.info("Using embedding-based reranking (no dedicated reranker)")
            
            # Generate embeddings for query and documents
            all_texts = [query] + documents
            embeddings = await active_manager.generate_embeddings(all_texts)
            
            query_embedding = embeddings[0]
            doc_embeddings = embeddings[1:]
            
            # Use shared evaluation utilities (DRY)
            from core.embedding_eval import EmbeddingEvaluator, SimilarityMetric
            
            # Find top-K most similar documents
            similar_results = EmbeddingEvaluator.find_top_k_similar(
                query_embedding,
                doc_embeddings,
                k=top_k or len(documents),
                metric=SimilarityMetric.COSINE
            )
            
            # Build response
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
                "total_tokens": len(all_texts) * 50  # Approximate
            }
        
        else:
            backend_name = type(active_manager).__name__
            raise NotImplementedError(
                f"Backend {backend_name} does not support reranking or embeddings. "
                "Load a reranker model with ONNX backend."
            )
    
    @staticmethod
    def _generate_id() -> str:
        """Generate unique ID for responses"""
        import time
        import random
        timestamp = int(time.time())
        random_part = random.randint(1000, 9999)
        return f"chatcmpl-{timestamp}{random_part}"


# Global backend manager instance
_backend_manager: Optional[BackendManager] = None


def get_backend_manager() -> BackendManager:
    """
    Get global backend manager instance
    
    Returns:
        BackendManager singleton
    """
    global _backend_manager
    if _backend_manager is None:
        _backend_manager = BackendManager()
    return _backend_manager

