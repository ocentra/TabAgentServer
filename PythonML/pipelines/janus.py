"""
JanusPipeline - Multimodal understanding

Specialized for: Janus multimodal models (dual vision-language understanding)
Architecture-specific: Janus-specific processing with separate encoders

Uses Hugging Face Transformers for Janus inference.
"""

import logging
from typing import Any, Dict, Optional
from .base import BasePipeline

logger = logging.getLogger(__name__)


class JanusPipeline(BasePipeline):
    """
    Janus Multimodal Model pipeline.
    
    Janus models have dual vision encoders for different aspects of visual understanding.
    Supports image captioning, visual QA, and multimodal reasoning.
    """
    
    def pipeline_type(self) -> str:
        return "image-to-text"
    
    def load(self, model_id: str, options: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """
        Load Janus model using transformers.
        
        Args:
            model_id: HuggingFace model ID (e.g., "deepseek-ai/Janus-1.3B")
            options: Loading options (device, dtype, etc.)
        
        Returns:
            Status dict with 'status' and 'message'
        """
        try:
            logger.info(f"[Janus] Loading model: {model_id}")
            
            from transformers import AutoModel, AutoProcessor
            import torch
            
            opts = options or {}
            
            # Determine device
            device = opts.get("device", "cuda" if torch.cuda.is_available() else "cpu")
            logger.info(f"[Janus] Using device: {device}")
            
            # Load processor
            logger.info(f"[Janus] Loading processor...")
            self.processor = AutoProcessor.from_pretrained(
                model_id,
                trust_remote_code=True  # Janus likely requires this
            )
            
            # Load model
            logger.info(f"[Janus] Loading model...")
            torch_dtype = torch.float16 if device == "cuda" else torch.float32
            
            self.model = AutoModel.from_pretrained(
                model_id,
                torch_dtype=torch_dtype,
                device_map="auto" if device == "cuda" else None,
                trust_remote_code=True,  # Janus likely requires this
                low_cpu_mem_usage=True
            )
            
            if device == "cpu":
                self.model = self.model.to(device)
            
            self.model.eval()
            self.device = device
            
            self._loaded = True
            logger.info(f"[Janus] ✅ Model loaded successfully on {device}")
            
            return {
                "status": "success",
                "message": f"Model {model_id} loaded on {device}",
                "device": device,
                "dtype": str(torch_dtype)
            }
            
        except Exception as e:
            logger.error(f"[Janus] ❌ Load failed: {e}", exc_info=True)
            self._loaded = False
            return {
                "status": "error",
                "message": f"Failed to load model: {str(e)}"
            }
    
    def generate(self, input_data: Dict[str, Any]) -> Dict[str, Any]:
        """
        Run Janus multimodal inference.
        
        Args:
            input_data: Dict with:
                - image: PIL Image, numpy array, or base64 string
                - text: Text prompt/question
                - max_new_tokens: Max tokens to generate (default: 512)
                - temperature: Sampling temperature (default: 0.7)
        
        Returns:
            Dict with 'status' and 'text'
        """
        if not self.is_loaded():
            return {"status": "error", "message": "Model not loaded"}
        
        try:
            from PIL import Image
            import torch
            import base64
            from io import BytesIO
            import numpy as np
            
            # Get inputs
            image_input = input_data.get("image")
            text_input = input_data.get("text", "Describe this image.")
            
            if image_input is None:
                return {"status": "error", "message": "No image provided"}
            
            # Convert image to PIL
            if isinstance(image_input, str):
                if image_input.startswith("data:image"):
                    image_data = image_input.split(",")[1]
                    image = Image.open(BytesIO(base64.b64decode(image_data)))
                else:
                    image = Image.open(image_input)
            elif isinstance(image_input, np.ndarray):
                image = Image.fromarray(image_input)
            elif isinstance(image_input, Image.Image):
                image = image_input
            else:
                return {"status": "error", "message": "Invalid image format"}
            
            if image.mode != "RGB":
                image = image.convert("RGB")
            
            # Get generation parameters
            max_new_tokens = input_data.get("max_new_tokens", 512)
            temperature = input_data.get("temperature", 0.7)
            
            logger.debug(f"[Janus] Processing with prompt: {text_input}")
            
            # Process inputs
            inputs = self.processor(
                text=text_input,
                images=image,
                return_tensors="pt"
            )
            
            inputs = {k: v.to(self.device) for k, v in inputs.items()}
            
            # Generate
            with torch.no_grad():
                outputs = self.model.generate(
                    **inputs,
                    max_new_tokens=max_new_tokens,
                    temperature=temperature,
                    do_sample=temperature > 0
                )
            
            # Decode
            generated_text = self.processor.batch_decode(
                outputs,
                skip_special_tokens=True
            )[0]
            
            # Clean up output (remove input prompt if present)
            if generated_text.startswith(text_input):
                generated_text = generated_text[len(text_input):].strip()
            
            logger.debug(f"[Janus] ✅ Generated {len(generated_text)} chars")
            
            return {
                "status": "success",
                "text": generated_text,
                "pipeline_type": self.pipeline_type()
            }
            
        except Exception as e:
            logger.error(f"[Janus] ❌ Generation failed: {e}", exc_info=True)
            return {
                "status": "error",
                "message": f"Generation failed: {str(e)}"
            }
    
    def unload(self):
        """Unload model from memory"""
        try:
            if hasattr(self, 'model'):
                del self.model
            if hasattr(self, 'processor'):
                del self.processor
            
            import torch
            if torch.cuda.is_available():
                torch.cuda.empty_cache()
            
            self._loaded = False
            logger.info("[Janus] Model unloaded")
            
        except Exception as e:
            logger.error(f"[Janus] Error during unload: {e}")
