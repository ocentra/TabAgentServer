"""
Florence2Pipeline - Image-to-text vision model

Specialized for: OCR, captioning, visual QA, region detection
Architecture-specific: Florence2 special tokens (<OCR>, <CAPTION>, etc.)

Format-agnostic: Supports GGUF, ONNX, SafeTensors, MediaPipe
Delegates loading to appropriate backend based on format
"""

import logging
from typing import Any, Dict, Optional
from .base import BasePipeline

logger = logging.getLogger(__name__)


class Florence2Pipeline(BasePipeline):
    """
    Florence2 - Multi-task vision model
    
    Handles Florence2-specific preprocessing and special tokens.
    Delegates actual loading/inference to backend based on format.
    """
    
    def pipeline_type(self) -> str:
        return "image-to-text"
    
    def load(self, model_id: str, options: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """
        Load Florence2 model - format-agnostic
        
        Delegates to:
        - GGUF/BitNet: Rust via rust_handle_message
        - ONNX: Python onnx_manager (→ Rust when flag flips)
        - SafeTensors: Python transformers
        - MediaPipe: Python mediapipe_manager (→ Rust when flag flips)
        """
        try:
            logger.info(f"[Florence2] Loading model: {model_id}")
            
            # Get options
            opts = options or {}
            model_info = opts.get("model_info", {})
            format = model_info.get("model_type", "SafeTensors")
            backend_decision = model_info.get("backend", {})
            
            logger.info(f"[Florence2] Format: {format}, Backend: {backend_decision}")
            
            # ✅ Delegate to appropriate backend
            if backend_decision.get("Rust"):
                # GGUF or BitNet - Rust handles everything
                engine = backend_decision["Rust"]["engine"]
                logger.info(f"[Florence2] Using Rust backend: {engine}")
                
                from native_host import rust_handle_message
                result = rust_handle_message({
                    "action": "load_model",
                    "modelPath": model_id,
                    "task": "image-to-text",
                    "architecture": "florence2"  # ← Tell Rust this is Florence2!
                })
                
                if result.get("status") == "success":
                    self.backend_type = "rust"
                    self.model_id = model_id
                    self._loaded = True
                else:
                    raise Exception(f"Rust loading failed: {result.get('message')}")
            
            elif backend_decision.get("Python"):
                engine = backend_decision["Python"]["engine"]
                logger.info(f"[Florence2] Using Python backend: {engine}")
                
                if engine == "onnxruntime":
                    # Python ONNX (will migrate to Rust)
                    from Python.backends.onnxrt.manager import ONNXRuntimeManager
                    self.backend = ONNXRuntimeManager()
                    self.backend.load_model(model_id, task="image-to-text")
                    self.backend_type = "python-onnx"
                
                elif engine == "mediapipe":
                    # Python MediaPipe (will migrate to Rust)
                    from Python.backends.mediapipe.manager import MediaPipeManager
                    self.backend = MediaPipeManager()
                    self.backend.load_model(model_id)
                    self.backend_type = "python-mediapipe"
                
                elif engine == "transformers":
                    # Python SafeTensors (stays Python)
                    from Python.backends.transformers_backend import TransformersTextGenBackend
                    self.backend = TransformersTextGenBackend()
                    self.backend.load_model(
                        model_path=model_id,
                        task="image-to-text",
                        trust_remote_code=True
                    )
                    self.backend_type = "python-transformers"
                else:
                    raise ValueError(f"Unsupported Python engine: {engine}")
                
                self._loaded = True
            else:
                raise ValueError(f"Unknown backend decision: {backend_decision}")
            
            # Load processor (Florence2-specific preprocessing)
            from transformers import AutoProcessor
            auth_token = opts.get("auth_token")
            self.processor = AutoProcessor.from_pretrained(
                model_id,
                trust_remote_code=True,
                token=auth_token
            )
            
            logger.info(f"[Florence2] Model loaded successfully: {self.backend_type}")
            
            return {
                "status": "success",
                "pipeline_type": self.pipeline_type(),
                "backend_type": self.backend_type,
                "model_id": model_id,
                "format": format
            }
            
        except Exception as e:
            logger.error(f"[Florence2] Load failed: {e}", exc_info=True)
            return {
                "status": "error",
                "message": str(e)
            }
    
    def generate(self, input_data: Dict[str, Any]) -> Dict[str, Any]:
        """
        Run Florence2 inference - delegates to backend
        
        Florence2-specific: Handles special tokens (<OCR>, <CAPTION>, etc.)
        Format-agnostic: Works with any backend (Rust GGUF, Python ONNX, SafeTensors, etc.)
        
        Input formats:
        - text: Prompt/task (e.g., "<OD>", "<CAPTION>", "<OCR>")
        - image: PIL Image or base64 string or path
        """
        if not self.is_loaded():
            return {"status": "error", "message": "Model not loaded"}
        
        try:
            # ✅ Florence2-SPECIFIC: Extract and format prompt with special tokens
            prompt = input_data.get("text", "<CAPTION>")  # Default task
            
            if self.backend_type == "rust":
                # ✅ Delegate to Rust with architecture context
                from native_host import rust_handle_message
                result = rust_handle_message({
                    "action": "generate",
                    "text": prompt,
                    "image": input_data.get("image"),
                    "max_new_tokens": input_data.get("max_new_tokens", 1024),
                    "architecture": "florence2",  # ← Tell Rust this is Florence2!
                    "task": "image-to-text"
                })
                
                if result.get("status") == "success":
                    return {
                        "status": "success",
                        "text": result.get("output"),
                        "pipeline_type": self.pipeline_type()
                    }
                else:
                    raise Exception(f"Rust generation failed: {result.get('message')}")
            
            elif self.backend_type in ["python-onnx", "python-mediapipe"]:
                # ✅ Delegate to Python backend
                result = self.backend.generate(
                    prompt=prompt,
                    image=input_data.get("image"),
                    max_tokens=input_data.get("max_new_tokens", 1024)
                )
                return {
                    "status": "success",
                    "text": result,
                    "pipeline_type": self.pipeline_type()
                }
            
            elif self.backend_type == "python-transformers":
                # ✅ Use transformers backend (SafeTensors)
                from PIL import Image
                import base64
                from io import BytesIO
                
                image_input = input_data.get("image")
                
                # Handle different image formats
                if isinstance(image_input, str):
                    if image_input.startswith("data:image"):
                        # Base64
                        image_data = image_input.split(",")[1]
                        image = Image.open(BytesIO(base64.b64decode(image_data)))
                    else:
                        # File path
                        image = Image.open(image_input)
                else:
                    # Already PIL Image
                    image = image_input
                
                # Use backend's generate method
                result = self.backend.generate(
                    prompt=prompt,
                    images=[image],
                    max_new_tokens=input_data.get("max_new_tokens", 1024)
                )
                
                return {
                    "status": "success",
                    "text": result,
                    "pipeline_type": self.pipeline_type()
                }
            else:
                raise ValueError(f"Unknown backend type: {self.backend_type}")
                
        except Exception as e:
            logger.error(f"[Florence2] Generation failed: {e}", exc_info=True)
            return {
                "status": "error",
                "message": str(e)
            }

