"""
Inference Service - Shared Logic Layer

Extracted from native_host.py to be used by BOTH:
- HTTP API (FastAPI)
- Native messaging (stdin/stdout)

DRY principle: Single source of truth for inference logic.
"""

import logging
from typing import Optional, Callable, Dict, Any
from pathlib import Path

from core.message_types import (
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
        """Initialize inference service"""
        self.bitnet_manager: Optional[Any] = None
        self.lmstudio_manager: Optional[Any] = None
        self.llamacpp_manager: Optional[Any] = None
        self.onnx_manager: Optional[Any] = None
        self.mediapipe_manager: Optional[Any] = None
        
        logger.info("InferenceService initialized")
    
    def load_model(
        self,
        model_path: str,
        progress_callback: Optional[ProgressCallback] = None
    ) -> Dict[str, Any]:
        """
        Load model - routes to appropriate backend.
        
        Auto-detects model type and selects best backend.
        Supports: BitNet (.gguf bitnet), ONNX (.onnx), llama.cpp (.gguf), MediaPipe (.task)
        
        Args:
            model_path: Path to model file
            progress_callback: Optional progress callback
            
        Returns:
            Response dict with status and payload
        """
        from backends.bitnet.validator import GGUFValidator
        from backends.bitnet import BitNetManager, BitNetConfig
        from backends.onnxrt import ONNXRuntimeManager
        from backends.llamacpp import LlamaCppManager
        from backends.llamacpp.config import LlamaCppConfig, LlamaCppBackend
        from pathlib import Path
        
        try:
            path = Path(model_path)
            file_extension = path.suffix.lower()
            
            logger.info(f"Loading model: {model_path} (extension: {file_extension})")
            
            # Route based on file extension
            if file_extension == ".onnx":
                # ONNX Runtime backend
                logger.info("Detected ONNX model, using ONNX Runtime")
                
                if self.onnx_manager is None:
                    self.onnx_manager = ONNXRuntimeManager()
                
                # Auto-detect best acceleration
                from core.message_types import AccelerationBackend
                from hardware.engine_detection import AccelerationDetector
                
                detector = AccelerationDetector()
                backends = detector.detect_all()
                
                # Priority: CUDA > DirectML > NPU > CPU
                if backends.get(AccelerationBackend.CUDA, False):
                    acceleration = AccelerationBackend.CUDA
                    logger.info("Using CUDA acceleration for ONNX")
                elif backends.get(AccelerationBackend.DIRECTML, False):
                    acceleration = AccelerationBackend.DIRECTML
                    logger.info("Using DirectML acceleration for ONNX")
                elif backends.get(AccelerationBackend.NPU, False):
                    acceleration = AccelerationBackend.NPU
                    logger.info("Using NPU acceleration for ONNX")
                else:
                    acceleration = AccelerationBackend.CPU
                    logger.info("Using CPU for ONNX (no GPU/NPU detected)")
                
                success = self.onnx_manager.load_model(
                    model_path,
                    acceleration=acceleration
                )
                
                if success:
                    return {
                        "status": "success",
                        "type": EventType.WORKER_READY.value,
                        "payload": {
                            "backend": "onnx_runtime",
                            "modelPath": model_path,
                            "executionProvider": "onnx_cpu"
                        }
                    }
                else:
                    return {
                        "status": "error",
                        "message": "Failed to load ONNX model"
                    }
            
            elif file_extension == ".gguf":
                # Detect if BitNet or standard GGUF
                model_type = GGUFValidator.detect_model_type(model_path)
                logger.info(f"Detected GGUF model type: {model_type.value}")
                
                if model_type == ModelType.BITNET:
                    # BitNet backend
                    logger.info("Using BitNet backend")
                    
                    if self.bitnet_manager is None:
                        self.bitnet_manager = BitNetManager(BitNetConfig())
                    
                    has_gpu = GGUFValidator.detect_cuda_available()
                    backend_type = GGUFValidator.get_backend_for_model(model_type, has_gpu=has_gpu)
                    
                    self.bitnet_manager.load_model(model_path, progress_callback)
                    
                    return {
                        "status": "success",
                        "type": EventType.WORKER_READY.value,
                        "payload": {
                            "backend": backend_type.value,
                            "modelPath": model_path,
                            "executionProvider": backend_type.value
                        }
                    }
                
                else:
                    # Standard GGUF - use llama.cpp
                    logger.info("Using llama.cpp backend")
                    
                    if self.llamacpp_manager is None:
                        self.llamacpp_manager = LlamaCppManager()
                    
                    # Auto-detect best backend for llama.cpp
                    from hardware.engine_detection import AccelerationDetector
                    
                    detector = AccelerationDetector()
                    backends = detector.detect_all()
                    
                    # Priority: CUDA > Vulkan > ROCm > Metal > CPU
                    if backends.get(AccelerationBackend.CUDA, False):
                        llama_backend = LlamaCppBackend.CUDA
                        logger.info("Using CUDA backend for llama.cpp")
                    elif backends.get(AccelerationBackend.VULKAN, False):
                        llama_backend = LlamaCppBackend.VULKAN
                        logger.info("Using Vulkan backend for llama.cpp")
                    elif backends.get(AccelerationBackend.ROCM, False):
                        llama_backend = LlamaCppBackend.ROCM
                        logger.info("Using ROCm backend for llama.cpp")
                    elif backends.get(AccelerationBackend.METAL, False):
                        llama_backend = LlamaCppBackend.METAL
                        logger.info("Using Metal backend for llama.cpp")
                    else:
                        llama_backend = LlamaCppBackend.CPU
                        logger.info("Using CPU for llama.cpp (no GPU detected)")
                    
                    config = LlamaCppConfig(
                        backend=llama_backend,
                        port=8766  # Different from BitNet
                    )
                    
                    success = self.llamacpp_manager.load_model(model_path, config)
                    
                    if success:
                        return {
                            "status": "success",
                            "type": EventType.WORKER_READY.value,
                            "payload": {
                                "backend": "llama_cpp",
                                "modelPath": model_path,
                                "executionProvider": "llama_cpp_cpu"
                            }
                        }
                    else:
                        return {
                            "status": "error",
                            "message": "Failed to load llama.cpp model"
                        }
            
            elif file_extension == ".task":
                # MediaPipe backend
                logger.info("Detected MediaPipe .task bundle, using MediaPipe")
                
                if self.mediapipe_manager is None:
                    from backends.mediapipe import MediaPipeManager
                    self.mediapipe_manager = MediaPipeManager()
                
                # Auto-detect delegate (GPU > CPU)
                from backends.mediapipe.config import MediaPipeDelegate
                from hardware.engine_detection import AccelerationDetector
                
                detector = AccelerationDetector()
                backends = detector.detect_all()
                
                # Priority: GPU > NPU > CPU for MediaPipe
                if backends.get(AccelerationBackend.CUDA, False) or backends.get(AccelerationBackend.DIRECTML, False):
                    delegate = MediaPipeDelegate.GPU
                    logger.info("Using GPU delegate for MediaPipe")
                elif backends.get(AccelerationBackend.NPU, False):
                    delegate = MediaPipeDelegate.NPU
                    logger.info("Using NPU delegate for MediaPipe")
                else:
                    delegate = MediaPipeDelegate.CPU
                    logger.info("Using CPU delegate for MediaPipe")
                
                success = self.mediapipe_manager.load_model(model_path, delegate)
                
                if success:
                    return {
                        "status": "success",
                        "type": EventType.WORKER_READY.value,
                        "payload": {
                            "backend": "mediapipe",
                            "modelPath": model_path,
                            "executionProvider": f"mediapipe_{delegate.value}"
                        }
                    }
                else:
                    return {
                        "status": "error",
                        "message": "Failed to load MediaPipe model"
                    }
            
            else:
                return {
                    "status": "error",
                    "message": f"Unsupported model format: {file_extension}"
                }
        
        except FileNotFoundError as e:
            logger.error(f"Model file not found: {e}")
            return {
                "status": "error",
                "message": str(e)
            }
        except Exception as e:
            logger.error(f"Error loading model: {e}")
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
        Generate text - routes to active backend.
        
        Routes to: BitNet, ONNX, llama.cpp, MediaPipe, or external service.
        
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
            # Determine which backend to use (priority order)
            backend_name = None
            manager = None
            use_async = False
            
            # Check ONNX Runtime
            if self.onnx_manager is not None and self.onnx_manager.is_model_loaded():
                backend_name = "ONNX Runtime"
                manager = self.onnx_manager
                use_async = True
            
            # Check llama.cpp
            elif self.llamacpp_manager is not None and self.llamacpp_manager.is_model_loaded():
                backend_name = "llama.cpp"
                manager = self.llamacpp_manager
                use_async = True
            
            # Check BitNet
            elif self.bitnet_manager is not None and self.bitnet_manager.is_model_loaded:
                backend_name = "BitNet"
                manager = self.bitnet_manager
                use_async = False  # BitNet uses sync generation
            
            # Check MediaPipe
            elif self.mediapipe_manager is not None and hasattr(self.mediapipe_manager, 'is_model_loaded') and self.mediapipe_manager.is_model_loaded():
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
            
            logger.info(f"Generate request using {backend_name} ({len(messages)} messages)")
            
            # Generate text using active backend
            if use_async:
                # Async backends (ONNX, llama.cpp, MediaPipe)
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
            logger.error(f"Generation error: {e}")
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
        # Priority order: ONNX > llama.cpp > BitNet > MediaPipe > External
        if self.onnx_manager and self.onnx_manager.is_model_loaded():
            return self.onnx_manager
        elif self.llamacpp_manager and self.llamacpp_manager.is_model_loaded():
            return self.llamacpp_manager
        elif self.bitnet_manager and self.bitnet_manager.is_model_loaded:
            return self.bitnet_manager
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

