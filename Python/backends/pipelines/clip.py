"""
ClipPipeline - Feature extraction and zero-shot classification

Specialized for: Image embeddings, text embeddings, zero-shot classification
Architecture-specific: CLIP dual-encoder handling

Format-agnostic: Supports GGUF, ONNX, SafeTensors
"""

import logging
from typing import Any, Dict, Optional
from .base import BasePipeline

logger = logging.getLogger(__name__)


class ClipPipeline(BasePipeline):
    """
    CLIP - Contrastive Language-Image Pre-training
    
    Handles CLIP-specific dual-encoder processing.
    Delegates actual loading/inference to backend based on format.
    """
    
    def pipeline_type(self) -> str:
        return "feature-extraction"
    
    def load(self, model_id: str, options: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """Load CLIP model - format-agnostic"""
        try:
            logger.info(f"[CLIP] Loading model: {model_id}")
            
            opts = options or {}
            model_info = opts.get("model_info", {})
            backend_decision = model_info.get("backend", {})
            
            # Delegate to appropriate backend based on format
            if backend_decision.get("Rust"):
                from native_host import rust_handle_message
                result = rust_handle_message({
                    "action": "load_model",
                    "modelPath": model_id,
                    "task": "feature-extraction",
                    "architecture": "clip"  # Pass architecture to Rust
                })
                
                if result.get("status") == "error":
                    logger.error(f"[CLIP] Rust load failed: {result.get('message')}")
                    return result
                
                self.backend_type = "rust"
                self.model_id = model_id
                logger.info(f"[CLIP] Loaded via Rust (GGUF/BitNet)")
                
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
                    logger.error(f"[CLIP] Python backend load failed: {result.get('message')}")
                    return result
                
                self.model_id = model_id
                logger.info(f"[CLIP] Loaded via Python ({self.backend_type})")
            
            else:
                return {
                    "status": "error",
                    "message": "No backend selected for CLIP model"
                }
            
            return {"status": "success", "message": "CLIP model loaded"}
            
        except Exception as e:
            logger.error(f"[CLIP] Load failed: {e}", exc_info=True)
            return {"status": "error", "message": str(e)}
    
    def generate(self, input_data: Dict[str, Any]) -> Dict[str, Any]:
        """Run CLIP inference (embeddings or classification) - format-agnostic"""
        if not self.is_loaded():
            return {"status": "error", "message": "Model not loaded"}
        
        try:
            # Delegate to appropriate backend
            if self.backend_type == "rust":
                from native_host import rust_handle_message
                result = rust_handle_message({
                    "action": "generate",
                    "text": input_data.get("text"),
                    "image": input_data.get("image"),
                    "architecture": "clip",
                    "task": "feature-extraction"
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
            logger.error(f"[CLIP] Generation failed: {e}", exc_info=True)
            return {"status": "error", "message": str(e)}

