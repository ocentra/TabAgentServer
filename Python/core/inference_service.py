"""
Inference Service - Unified Router for ALL Communication Channels

Delegates to native_host.py unified pipeline architecture.
Used by THREE channels (DRY - single implementation):
1. HTTP API (FastAPI) 
2. Native messaging (stdin/stdout Chrome extension)
3. WebRTC (future)

All channels speak the same language via core.message_types.
All channels route through the same pipeline loading logic.
"""

import logging
from typing import Optional, Callable, Dict, Any
from pathlib import Path

from Python.core.message_types import (
    ChatMessage,
    InferenceSettings,
    BackendType,
    ModelType,
    LoadingStatus,
    EventType,
)

logger = logging.getLogger(__name__)

# Type aliases from native_host.py
ProgressCallback = Callable[[LoadingStatus, int, str], None]
StreamCallback = Callable[[str, Optional[str], int], None]


class InferenceService:
    """
    Unified inference service for all backends.
    
    Used by both HTTP API and native messaging.
    Manages backend lifecycle and routing.
    """
    
    def __init__(self):
        """
        Initialize inference service.
        
        NOTE: This is now a THIN ROUTER that delegates to native_host.py
        All actual loading/generation logic lives in native_host.py
        """
        # Legacy manager references (kept for backward compatibility with get_active_manager)
        # Note: BitNet and llama.cpp are now handled by Rust (model-loader via FFI)
        self.lmstudio_manager: Optional[Any] = None
        self.onnx_manager: Optional[Any] = None
        self.mediapipe_manager: Optional[Any] = None
        
        # Track last loaded model for HTTP API
        self._last_loaded_source: Optional[str] = None
        self._last_backend_type: Optional[str] = None
        
        logger.info("InferenceService initialized (delegates to native_host.py unified pipeline)")
    
    def load_model(
        self,
        model_path: str,
        progress_callback: Optional[ProgressCallback] = None,
        auth_token: Optional[str] = None
    ) -> Dict[str, Any]:
        """
        Load model using UNIFIED PIPELINE ARCHITECTURE.
        
        Delegates to native_host.load_model_unified() for:
        - Rust-based detection (model-cache::detect_model_py)
        - Task detection (image-to-text, speech-to-text, text-generation)
        - Pipeline routing (Florence2, Whisper, TextGen, CLIP, etc.)
        - Specialized backends (transformers-based pipelines)
        
        This ensures ALL channels (HTTP, Native Messaging, WebRTC) use SAME code!
        
        Args:
            model_path: Path to model file or HuggingFace repo
            progress_callback: Optional progress callback (not used in pipeline flow yet)
            auth_token: Optional HuggingFace token for private models
            
        Returns:
            Response dict with status and payload
        """
        try:
            logger.info(f"[InferenceService] Loading model via unified pipeline: {model_path}")
            
            # Import from native_host (DRY - single source of truth!)
            from native_host import load_model_unified, _loaded_pipelines
            
            # Call unified pipeline loader
            result = load_model_unified(
                source=model_path,
                auth_token=auth_token
            )
            
            if result["status"] == "success":
                # Track loaded model
                self._last_loaded_source = model_path
                self._last_backend_type = result.get("backend", "unknown")
                
                logger.info(f"[InferenceService] Model loaded successfully: {self._last_backend_type}")
                
                # Update legacy manager references if needed for backward compatibility
                backend_type = result.get("backend", "")
                if backend_type == "python-pipeline":
                    # Pipeline-based backend - check _loaded_pipelines
                    if model_path in _loaded_pipelines:
                        # Store reference for get_active_manager()
                        pipeline = _loaded_pipelines[model_path]
                        # Map to legacy manager slots based on pipeline type
                        pipeline_type = result.get("task", "")
                        if "image" in pipeline_type or "vision" in pipeline_type:
                            self.mediapipe_manager = pipeline  # Reuse slot for vision models
                        else:
                            self.onnx_manager = pipeline  # Default slot
                
                elif "gguf" in backend_type.lower() or "llama" in backend_type.lower():
                    # GGUF/llama.cpp - manager already set by load_model_unified
                    pass
                elif "bitnet" in backend_type.lower():
                    # BitNet - manager already set
                    pass
                
                return result
            else:
                logger.error(f"[InferenceService] Model load failed: {result.get('message', 'Unknown error')}")
                return result
        
        except Exception as e:
            logger.error(f"[InferenceService] Load error: {e}", exc_info=True)
            return {
                "status": "error",
                "message": f"Failed to load model: {str(e)}"
            }
    
    async def generate(
        self,
        messages: list[ChatMessage],
        settings: Optional[InferenceSettings] = None,
        stream_callback: Optional[StreamCallback] = None
    ) -> Dict[str, Any]:
        """
        Generate text using UNIFIED PIPELINE ARCHITECTURE.
        
        Delegates to native_host.generate_via_pipeline() for pipeline-based backends.
        Falls back to direct manager calls for legacy backends (BitNet, llama.cpp).
        
        Args:
            messages: Chat messages
            settings: Inference settings
            stream_callback: Optional streaming callback
            
        Returns:
            Response dict with status and generated text
        """
        if settings is None:
            settings = InferenceSettings()
        
        try:
            logger.info(f"[InferenceService] Generate request ({len(messages)} messages)")
            
            # Check if we have a pipeline-loaded model
            if self._last_loaded_source and self._last_backend_type == "python-pipeline":
                # Use unified pipeline generation
                from native_host import generate_via_pipeline
                
                # Convert ChatMessage to dict format expected by pipelines
                input_data = {
                    "messages": [{"role": msg.role, "content": msg.content} for msg in messages],
                    "settings": settings.__dict__ if hasattr(settings, '__dict__') else {}
                }
                
                result = generate_via_pipeline(
                    source=self._last_loaded_source,
                    input_data=input_data
                )
                
                if result["status"] == "success":
                    generated_text = result.get("output", result.get("text", ""))
                    return {
                        "status": "success",
                        "type": EventType.GENERATION_COMPLETE.value,
                        "payload": {
                            "output": generated_text,
                            "generatedText": generated_text
                        }
                    }
                else:
                    return result
            
            # Otherwise fall back to legacy manager routing
            backend_name = None
            manager = None
            use_async = False
            
            # Check ONNX Runtime
            if self.onnx_manager is not None and hasattr(self.onnx_manager, 'is_model_loaded'):
                # Could be legacy ONNX or pipeline stored in slot
                if hasattr(self.onnx_manager, 'generate'):
                    backend_name = "ONNX Runtime"
                    manager = self.onnx_manager
                    use_async = True
            
            # llama.cpp and BitNet now handled by Rust (no manager needed)
            
            # Check MediaPipe
            elif self.mediapipe_manager is not None and hasattr(self.mediapipe_manager, 'is_model_loaded'):
                backend_name = "MediaPipe"
                manager = self.mediapipe_manager
                use_async = True
            
            # Check external service
            elif self.lmstudio_manager is not None and self.lmstudio_manager.is_server_running:
                backend_name = "External Service"
                manager = self.lmstudio_manager
                use_async = False
            
            else:
                return {
                    "status": "error",
                    "message": "No model loaded. Load a model first."
                }
            
            logger.info(f"[InferenceService] Using {backend_name}")
            
            # Generate text using active backend
            if use_async:
                # Async backends (ONNX, llama.cpp, MediaPipe, pipelines)
                generated_text = await manager.generate(
                    messages=messages,
                    settings=settings
                )
            else:
                # Sync backends (BitNet, external services)
                if backend_name == "BitNet":
                    generated_text = manager.generate(
                        messages=messages,
                        settings=settings,
                        stream_callback=stream_callback
                    )
                else:  # External service
                    generated_text = manager.proxy_chat_completion(
                        messages=messages,
                        settings=settings,
                        stream_callback=stream_callback
                    )
            
            # Return success
            return {
                "status": "success",
                "type": EventType.GENERATION_COMPLETE.value,
                "payload": {
                    "output": generated_text,
                    "generatedText": generated_text
                }
            }
        
        except Exception as e:
            logger.error(f"[InferenceService] Generation error: {e}", exc_info=True)
            return {
                "status": "error",
                "type": EventType.GENERATION_ERROR.value,
                "message": str(e)
            }
    
    def get_active_manager(self) -> Optional[Any]:
        """
        Get currently active backend manager.
        
        Checks all backends in priority order.
        
        Returns:
            Active manager or None
        """
        # Priority order: ONNX > MediaPipe > External
        # Note: GGUF/BitNet handled by Rust (no manager)
        if self.onnx_manager and self.onnx_manager.is_model_loaded():
            return self.onnx_manager
        elif self.mediapipe_manager and hasattr(self.mediapipe_manager, 'is_model_loaded') and self.mediapipe_manager.is_model_loaded():
            return self.mediapipe_manager
        elif self.lmstudio_manager and hasattr(self.lmstudio_manager, 'is_server_running') and self.lmstudio_manager.is_server_running:
            return self.lmstudio_manager
        
        return None
    
    def get_backend_type(self) -> Optional[BackendType]:
        """Get active backend type"""
        manager = self.get_active_manager()
        if manager and hasattr(manager, 'backend'):
            return manager.backend
        return None
    
    def is_model_loaded(self) -> bool:
        """Check if any model is loaded"""
        return self.get_active_manager() is not None


# Global singleton
_inference_service: Optional[InferenceService] = None


def get_inference_service() -> InferenceService:
    """Get global InferenceService singleton"""
    global _inference_service
    if _inference_service is None:
        _inference_service = InferenceService()
    return _inference_service

