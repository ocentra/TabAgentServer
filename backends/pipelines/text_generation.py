"""
TextGenerationPipeline - Generic text generation

For standard LLMs without special architecture requirements.
Supports: Llama, Mistral, Qwen, Phi, etc.

Format-agnostic: Supports GGUF, ONNX, SafeTensors
"""

import logging
from typing import Any, Dict, Optional
from .base import BasePipeline

logger = logging.getLogger(__name__)


class TextGenerationPipeline(BasePipeline):
    """
    Generic text generation pipeline
    
    For standard LLMs that don't need specialized preprocessing.
    Delegates to backend based on format.
    """
    
    def pipeline_type(self) -> str:
        return "text-generation"
    
    def load(self, model_id: str, options: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """Load text generation model - format-agnostic"""
        try:
            logger.info(f"[TextGen] Loading model: {model_id}")
            
            opts = options or {}
            model_info = opts.get("model_info", {})
            backend_decision = model_info.get("backend", {})
            architecture = model_info.get("architecture")
            
            # Delegate to appropriate backend based on format
            if backend_decision.get("Rust"):
                from native_host import rust_handle_message
                result = rust_handle_message({
                    "action": "load_model",
                    "modelPath": model_id,
                    "task": "text-generation",
                    "architecture": architecture  # Pass architecture if detected
                })
                
                if result.get("status") == "error":
                    logger.error(f"[TextGen] Rust load failed: {result.get('message')}")
                    return result
                
                self.backend_type = "rust"
                self.model_id = model_id
                logger.info(f"[TextGen] Loaded via Rust (GGUF/BitNet)")
                
            elif backend_decision.get("Python"):
                # Determine Python backend (ONNX, MediaPipe, or Transformers)
                if model_info.get("model_type") == "onnx":
                    from backends.onnx_backend import ONNXRuntimeManager
                    self.python_backend = ONNXRuntimeManager()
                    result = self.python_backend.load_model(model_id, opts)
                    self.backend_type = "onnx"
                    
                elif model_info.get("model_type") == "mediapipe":
                    from backends.mediapipe_backend import MediaPipeManager
                    self.python_backend = MediaPipeManager()
                    result = self.python_backend.load_model(model_id, opts)
                    self.backend_type = "mediapipe"
                    
                else:  # SafeTensors/PyTorch via Transformers
                    from backends.transformers_backend import TransformersTextGenBackend
                    self.python_backend = TransformersTextGenBackend()
                    result = self.python_backend.load_model(model_id, opts)
                    self.backend_type = "transformers"
                
                if result.get("status") == "error":
                    logger.error(f"[TextGen] Python backend load failed: {result.get('message')}")
                    return result
                
                self.model_id = model_id
                logger.info(f"[TextGen] Loaded via Python ({self.backend_type})")
            
            else:
                return {
                    "status": "error",
                    "message": "No backend selected for text generation model"
                }
            
            return {"status": "success", "message": "Text generation model loaded"}
            
        except Exception as e:
            logger.error(f"[TextGen] Load failed: {e}", exc_info=True)
            return {"status": "error", "message": str(e)}
    
    def generate(self, input_data: Dict[str, Any]) -> Dict[str, Any]:
        """Run text generation inference - format-agnostic"""
        if not self.is_loaded():
            return {"status": "error", "message": "Model not loaded"}
        
        try:
            # Delegate to appropriate backend
            if self.backend_type == "rust":
                from native_host import rust_handle_message
                result = rust_handle_message({
                    "action": "generate",
                    "text": input_data.get("text") or input_data.get("prompt"),
                    "task": "text-generation"
                })
                return result
            
            elif self.backend_type in ["onnx", "mediapipe", "transformers"]:
                # Python backend delegation
                result = self.python_backend.generate(input_data)
                return result
            
            else:
                return {
                    "status": "error",
                    "message": f"Unknown backend type: {self.backend_type}"
                }
                
        except Exception as e:
            logger.error(f"[TextGen] Generation failed: {e}", exc_info=True)
            return {"status": "error", "message": str(e)}

