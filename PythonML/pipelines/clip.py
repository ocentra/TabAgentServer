"""
ClipPipeline - Contrastive Language-Image Pre-training

Specialized for: Image embeddings, text embeddings, zero-shot classification, image-text similarity
Architecture-specific: CLIP dual-encoder handling

Uses Hugging Face Transformers for CLIP inference.
"""

import logging
from typing import Any, Dict, List, Optional, Union
from .base import BasePipeline

logger = logging.getLogger(__name__)


class ClipPipeline(BasePipeline):
    """
    CLIP Contrastive Language-Image Pre-training pipeline.
    
    Supports:
    - Text embedding generation
    - Image embedding generation
    - Zero-shot image classification
    - Image-text similarity scoring
    
    Uses transformers CLIPModel with vision and text encoders.
    """
    
    def pipeline_type(self) -> str:
        return "feature-extraction"
    
    def load(self, model_id: str, options: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """
        Load CLIP model using transformers.
        
        Args:
            model_id: HuggingFace model ID (e.g., "openai/clip-vit-base-patch32")
            options: Loading options (device, dtype, etc.)
        
        Returns:
            Status dict with 'status' and 'message'
        """
        try:
            logger.info(f"[CLIP] Loading model: {model_id}")
            
            from transformers import CLIPModel, CLIPProcessor
            import torch
            
            opts = options or {}
            
            # Determine device
            device = opts.get("device", "cuda" if torch.cuda.is_available() else "cpu")
            logger.info(f"[CLIP] Using device: {device}")
            
            # Load processor (handles image and text preprocessing)
            logger.info(f"[CLIP] Loading processor...")
            self.processor = CLIPProcessor.from_pretrained(
                model_id,
                trust_remote_code=opts.get("trust_remote_code", False)
            )
            
            # Load model
            logger.info(f"[CLIP] Loading model...")
            torch_dtype = torch.float16 if device == "cuda" else torch.float32
            
            self.model = CLIPModel.from_pretrained(
                model_id,
                torch_dtype=torch_dtype,
                low_cpu_mem_usage=True,
                trust_remote_code=opts.get("trust_remote_code", False)
            )
            
            self.model = self.model.to(device)
            self.model.eval()
            
            # Store device for later use
            self.device = device
            
            self._loaded = True
            logger.info(f"[CLIP] ✅ Model loaded successfully on {device}")
            
            return {
                "status": "success",
                "message": f"Model {model_id} loaded on {device}",
                "device": device,
                "dtype": str(torch_dtype)
            }
            
        except Exception as e:
            logger.error(f"[CLIP] ❌ Load failed: {e}", exc_info=True)
            self._loaded = False
            return {
                "status": "error",
                "message": f"Failed to load model: {str(e)}"
            }
    
    def generate(self, input_data: Dict[str, Any]) -> Dict[str, Any]:
        """
        Run CLIP inference.
        
        Args:
            input_data: Dict with:
                - mode: 'text_embedding', 'image_embedding', 'similarity', or 'zero_shot' (default: 'similarity')
                - text: Text or list of texts
                - image: PIL Image, numpy array, or base64 string (or list of images)
                - candidates: List of candidate labels for zero-shot classification
                - normalize: Whether to L2 normalize embeddings (default: True)
        
        Returns:
            Dict with 'status' and mode-specific results
        """
        if not self.is_loaded():
            return {"status": "error", "message": "Model not loaded"}
        
        try:
            import torch
            
            mode = input_data.get("mode", "similarity")
            
            if mode == "text_embedding":
                return self._encode_text(input_data)
            elif mode == "image_embedding":
                return self._encode_image(input_data)
            elif mode == "similarity":
                return self._compute_similarity(input_data)
            elif mode == "zero_shot":
                return self._zero_shot_classification(input_data)
            else:
                return {"status": "error", "message": f"Unknown mode: {mode}"}
            
        except Exception as e:
            logger.error(f"[CLIP] ❌ Generation failed: {e}", exc_info=True)
            return {
                "status": "error",
                "message": f"Generation failed: {str(e)}"
            }
    
    def _encode_text(self, input_data: Dict[str, Any]) -> Dict[str, Any]:
        """Encode text(s) to embeddings"""
        import torch
        
        text = input_data.get("text")
        if not text:
            return {"status": "error", "message": "No text provided"}
        
        # Handle single string input
        single_input = isinstance(text, str)
        if single_input:
            text = [text]
        
        normalize = input_data.get("normalize", True)
        
        # Process text
        inputs = self.processor(
            text=text,
            return_tensors="pt",
            padding=True,
            truncation=True
        )
        
        inputs = {k: v.to(self.device) for k, v in inputs.items()}
        
        # Encode
        with torch.no_grad():
            text_features = self.model.get_text_features(**inputs)
        
        # Normalize if requested
        if normalize:
            text_features = text_features / text_features.norm(dim=-1, keepdim=True)
        
        # Convert to list
        embeddings = text_features.cpu().numpy().tolist()
        
        # Return single embedding if single input
        if single_input:
            embeddings = embeddings[0]
        
        logger.debug(f"[CLIP] ✅ Encoded {len(text)} text(s)")
        
        return {
            "status": "success",
            "embeddings": embeddings,
            "count": len(text) if not single_input else 1,
            "dimension": len(embeddings[0]) if not single_input else len(embeddings)
        }
    
    def _encode_image(self, input_data: Dict[str, Any]) -> Dict[str, Any]:
        """Encode image(s) to embeddings"""
        import torch
        from PIL import Image
        import base64
        from io import BytesIO
        import numpy as np
        
        image_input = input_data.get("image")
        if image_input is None:
            return {"status": "error", "message": "No image provided"}
        
        # Handle single image or list of images
        single_input = not isinstance(image_input, list)
        if single_input:
            image_input = [image_input]
        
        # Convert all images to PIL
        images = []
        for img in image_input:
            if isinstance(img, str):
                if img.startswith("data:image"):
                    # Base64
                    image_data = img.split(",")[1]
                    images.append(Image.open(BytesIO(base64.b64decode(image_data))))
                else:
                    # File path
                    images.append(Image.open(img))
            elif isinstance(img, np.ndarray):
                images.append(Image.fromarray(img))
            elif isinstance(img, Image.Image):
                images.append(img)
            else:
                return {"status": "error", "message": "Invalid image format"}
        
        # Ensure RGB
        images = [img.convert("RGB") if img.mode != "RGB" else img for img in images]
        
        normalize = input_data.get("normalize", True)
        
        # Process images
        inputs = self.processor(
            images=images,
            return_tensors="pt"
        )
        
        inputs = {k: v.to(self.device) for k, v in inputs.items()}
        
        # Encode
        with torch.no_grad():
            image_features = self.model.get_image_features(**inputs)
        
        # Normalize if requested
        if normalize:
            image_features = image_features / image_features.norm(dim=-1, keepdim=True)
        
        # Convert to list
        embeddings = image_features.cpu().numpy().tolist()
        
        # Return single embedding if single input
        if single_input:
            embeddings = embeddings[0]
        
        logger.debug(f"[CLIP] ✅ Encoded {len(images)} image(s)")
        
        return {
            "status": "success",
            "embeddings": embeddings,
            "count": len(images) if not single_input else 1,
            "dimension": len(embeddings[0]) if not single_input else len(embeddings)
        }
    
    def _compute_similarity(self, input_data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute image-text similarity"""
        import torch
        from PIL import Image
        import base64
        from io import BytesIO
        import numpy as np
        
        text = input_data.get("text")
        image_input = input_data.get("image")
        
        if not text or image_input is None:
            return {"status": "error", "message": "Both text and image required"}
        
        # Handle single string input
        if isinstance(text, str):
            text = [text]
        
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
        
        # Process inputs
        inputs = self.processor(
            text=text,
            images=image,
            return_tensors="pt",
            padding=True
        )
        
        inputs = {k: v.to(self.device) for k, v in inputs.items()}
        
        # Compute similarity
        with torch.no_grad():
            outputs = self.model(**inputs)
            logits_per_image = outputs.logits_per_image  # image-text similarity scores
            probs = logits_per_image.softmax(dim=1)  # probabilities
        
        similarities = probs.cpu().numpy().tolist()[0]
        
        logger.debug(f"[CLIP] ✅ Computed similarity for {len(text)} text(s)")
        
        return {
            "status": "success",
            "similarities": similarities,
            "text": text
        }
    
    def _zero_shot_classification(self, input_data: Dict[str, Any]) -> Dict[str, Any]:
        """Zero-shot image classification"""
        import torch
        from PIL import Image
        import base64
        from io import BytesIO
        import numpy as np
        
        image_input = input_data.get("image")
        candidates = input_data.get("candidates")
        
        if image_input is None or not candidates:
            return {"status": "error", "message": "Both image and candidates required"}
        
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
        
        # Format candidate labels as "a photo of {label}"
        text_prompts = [f"a photo of a {label}" for label in candidates]
        
        # Process inputs
        inputs = self.processor(
            text=text_prompts,
            images=image,
            return_tensors="pt",
            padding=True
        )
        
        inputs = {k: v.to(self.device) for k, v in inputs.items()}
        
        # Compute predictions
        with torch.no_grad():
            outputs = self.model(**inputs)
            logits_per_image = outputs.logits_per_image
            probs = logits_per_image.softmax(dim=1)
        
        probabilities = probs.cpu().numpy().tolist()[0]
        
        # Sort by probability
        results = list(zip(candidates, probabilities))
        results.sort(key=lambda x: x[1], reverse=True)
        
        logger.debug(f"[CLIP] ✅ Classified image with {len(candidates)} candidates")
        
        return {
            "status": "success",
            "predictions": [
                {"label": label, "score": float(score)}
                for label, score in results
            ],
            "top_prediction": results[0][0] if results else None
        }
    
    def unload(self):
        """Unload model from memory"""
        try:
            if hasattr(self, 'model'):
                del self.model
            if hasattr(self, 'processor'):
                del self.processor
            
            # Clear CUDA cache if using GPU
            import torch
            if torch.cuda.is_available():
                torch.cuda.empty_cache()
            
            self._loaded = False
            logger.info("[CLIP] Model unloaded")
            
        except Exception as e:
            logger.error(f"[CLIP] Error during unload: {e}")
