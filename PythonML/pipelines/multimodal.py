"""
MultimodalPipeline - Generic multimodal models

For: Models with both text and vision capabilities
Examples: Phi-3.5-vision, Llama-3.2-vision, Qwen2-VL, LLaVA

Uses Hugging Face Transformers for multimodal inference.
"""

import logging
from typing import Any, Dict, List, Optional, Union
from .base import BasePipeline

logger = logging.getLogger(__name__)


class MultimodalPipeline(BasePipeline):
    """
    Generic multimodal pipeline for vision-language models.
    
    Supports text-only and vision+text generation.
    Automatically routes based on whether images are provided.
    """
    
    def pipeline_type(self) -> str:
        return "text-generation"
    
    def load(self, model_id: str, options: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """
        Load multimodal model using transformers.
        
        Args:
            model_id: HuggingFace model ID (e.g., "microsoft/Phi-3-vision-128k-instruct")
            options: Loading options (device, dtype, etc.)
        
        Returns:
            Status dict with 'status' and 'message'
        """
        try:
            logger.info(f"[Multimodal] Loading model: {model_id}")
            
            from transformers import AutoProcessor, AutoModelForVision2Seq
            import torch
            
            opts = options or {}
            
            # Determine device
            device = opts.get("device", "cuda" if torch.cuda.is_available() else "cpu")
            logger.info(f"[Multimodal] Using device: {device}")
            
            # Load processor
            logger.info(f"[Multimodal] Loading processor...")
            self.processor = AutoProcessor.from_pretrained(
                model_id,
                trust_remote_code=opts.get("trust_remote_code", True)
            )
            
            # Load model
            logger.info(f"[Multimodal] Loading model...")
            torch_dtype = torch.float16 if device == "cuda" else torch.float32
            
            self.model = AutoModelForVision2Seq.from_pretrained(
                model_id,
                torch_dtype=torch_dtype,
                device_map="auto" if device == "cuda" else None,
                trust_remote_code=opts.get("trust_remote_code", True),
                low_cpu_mem_usage=True
            )
            
            if device == "cpu":
                self.model = self.model.to(device)
            
            self.model.eval()
            self.device = device
            
            self._loaded = True
            logger.info(f"[Multimodal] ✅ Model loaded successfully on {device}")
            
            return {
                "status": "success",
                "message": f"Model {model_id} loaded on {device}",
                "device": device,
                "dtype": str(torch_dtype),
                "supports_vision": True
            }
            
        except Exception as e:
            logger.error(f"[Multimodal] ❌ Load failed: {e}", exc_info=True)
            self._loaded = False
            return {
                "status": "error",
                "message": f"Failed to load model: {str(e)}"
            }
    
    def generate(self, input_data: Dict[str, Any]) -> Dict[str, Any]:
        """
        Run multimodal inference.
        
        Args:
            input_data: Dict with:
                - prompt: Text prompt
                - image: PIL Image, numpy array, base64 string (optional)
                - images: List of images for multi-image input (optional)
                - max_new_tokens: Max tokens to generate (default: 512)
                - temperature: Sampling temperature (default: 0.7)
                - top_p: Nucleus sampling (default: 0.9)
        
        Returns:
            Dict with 'status', 'text', and metadata
        """
        if not self.is_loaded():
            return {"status": "error", "message": "Model not loaded"}
        
        try:
            import torch
            from PIL import Image
            import base64
            from io import BytesIO
            import numpy as np
            
            # Get inputs
            prompt = input_data.get("prompt") or input_data.get("text")
            if not prompt:
                return {"status": "error", "message": "No prompt provided"}
            
            # Get image(s) if provided
            images = None
            if "images" in input_data:
                images = input_data["images"]
            elif "image" in input_data:
                images = [input_data["image"]]
            
            # Convert images to PIL if provided
            pil_images = []
            if images:
                for img in images:
                    if isinstance(img, str):
                        if img.startswith("data:image"):
                            image_data = img.split(",")[1]
                            pil_images.append(Image.open(BytesIO(base64.b64decode(image_data))))
                        else:
                            pil_images.append(Image.open(img))
                    elif isinstance(img, np.ndarray):
                        pil_images.append(Image.fromarray(img))
                    elif isinstance(img, Image.Image):
                        pil_images.append(img)
                    else:
                        return {"status": "error", "message": "Invalid image format"}
                
                # Ensure RGB
                pil_images = [img.convert("RGB") if img.mode != "RGB" else img for img in pil_images]
            
            # Get generation parameters
            max_new_tokens = input_data.get("max_new_tokens", 512)
            temperature = input_data.get("temperature", 0.7)
            top_p = input_data.get("top_p", 0.9)
            
            mode = "vision+text" if pil_images else "text-only"
            logger.debug(f"[Multimodal] Generating in {mode} mode")
            
            # Process inputs
            if pil_images:
                inputs = self.processor(
                    text=prompt,
                    images=pil_images,
                    return_tensors="pt",
                    padding=True
                )
            else:
                inputs = self.processor(
                    text=prompt,
                    return_tensors="pt",
                    padding=True
                )
            
            inputs = {k: v.to(self.device) for k, v in inputs.items()}
            
            # Generate
            with torch.no_grad():
                outputs = self.model.generate(
                    **inputs,
                    max_new_tokens=max_new_tokens,
                    temperature=temperature,
                    top_p=top_p,
                    do_sample=temperature > 0
                )
            
            # Decode
            generated_text = self.processor.batch_decode(
                outputs,
                skip_special_tokens=True
            )[0]
            
            # Remove input prompt if present
            if generated_text.startswith(prompt):
                generated_text = generated_text[len(prompt):].strip()
            
            logger.debug(f"[Multimodal] ✅ Generated {len(generated_text)} chars")
            
            return {
                "status": "success",
                "text": generated_text,
                "mode": mode,
                "num_images": len(pil_images) if pil_images else 0,
                "tokens_generated": len(outputs[0]) - len(inputs["input_ids"][0])
            }
            
        except Exception as e:
            logger.error(f"[Multimodal] ❌ Generation failed: {e}", exc_info=True)
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
            logger.info("[Multimodal] Model unloaded")
            
        except Exception as e:
            logger.error(f"[Multimodal] Error during unload: {e}")
