"""
ImageClassificationPipeline - Image classification

For: Vision models that classify images into predefined categories
Examples: ViT, ResNet, EfficientNet, DINOv2, ConvNeXT

Uses Hugging Face Transformers for image classification.
"""

import logging
from typing import Any, Dict, Optional
from .base import BasePipeline

logger = logging.getLogger(__name__)


class ImageClassificationPipeline(BasePipeline):
    """
    Image classification pipeline.
    
    Classifies images into predefined categories using vision transformers
    or convolutional neural networks.
    """
    
    def pipeline_type(self) -> str:
        return "image-classification"
    
    def load(self, model_id: str, options: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """
        Load image classification model using transformers.
        
        Args:
            model_id: HuggingFace model ID (e.g., "google/vit-base-patch16-224")
            options: Loading options (device, dtype, etc.)
        
        Returns:
            Status dict with 'status' and 'message'
        """
        try:
            logger.info(f"[ImageClassification] Loading model: {model_id}")
            
            from transformers import AutoImageProcessor, AutoModelForImageClassification
            import torch
            
            opts = options or {}
            
            # Determine device
            device = opts.get("device", "cuda" if torch.cuda.is_available() else "cpu")
            logger.info(f"[ImageClassification] Using device: {device}")
            
            # Load image processor
            logger.info(f"[ImageClassification] Loading image processor...")
            self.processor = AutoImageProcessor.from_pretrained(
                model_id,
                trust_remote_code=opts.get("trust_remote_code", False)
            )
            
            # Load model
            logger.info(f"[ImageClassification] Loading model...")
            torch_dtype = torch.float16 if device == "cuda" else torch.float32
            
            self.model = AutoModelForImageClassification.from_pretrained(
                model_id,
                torch_dtype=torch_dtype,
                low_cpu_mem_usage=True,
                trust_remote_code=opts.get("trust_remote_code", False)
            )
            
            self.model = self.model.to(device)
            self.model.eval()
            
            # Get label information
            self.id2label = self.model.config.id2label
            self.num_labels = len(self.id2label)
            
            self._loaded = True
            logger.info(f"[ImageClassification] ✅ Model loaded successfully on {device} ({self.num_labels} classes)")
            
            return {
                "status": "success",
                "message": f"Model {model_id} loaded on {device}",
                "device": device,
                "dtype": str(torch_dtype),
                "num_labels": self.num_labels
            }
            
        except Exception as e:
            logger.error(f"[ImageClassification] ❌ Load failed: {e}", exc_info=True)
            self._loaded = False
            return {
                "status": "error",
                "message": f"Failed to load model: {str(e)}"
            }
    
    def generate(self, input_data: Dict[str, Any]) -> Dict[str, Any]:
        """
        Classify image.
        
        Args:
            input_data: Dict with:
                - image: PIL Image, numpy array, or base64 string
                - top_k: Return top K predictions (default: 5)
        
        Returns:
            Dict with 'status', 'predictions', and 'top_prediction'
        """
        if not self.is_loaded():
            return {"status": "error", "message": "Model not loaded"}
        
        try:
            from PIL import Image
            import torch
            import base64
            from io import BytesIO
            import numpy as np
            
            # Get image input
            image_input = input_data.get("image")
            if image_input is None:
                return {"status": "error", "message": "No image provided"}
            
            # Convert to PIL Image
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
            
            # Ensure RGB
            if image.mode != "RGB":
                image = image.convert("RGB")
            
            top_k = input_data.get("top_k", 5)
            
            logger.debug(f"[ImageClassification] Classifying image (top_k={top_k})")
            
            # Process image
            inputs = self.processor(
                images=image,
                return_tensors="pt"
            )
            
            device = next(self.model.parameters()).device
            inputs = {k: v.to(device) for k, v in inputs.items()}
            
            # Classify
            with torch.no_grad():
                outputs = self.model(**inputs)
                logits = outputs.logits
                probs = torch.nn.functional.softmax(logits, dim=-1)
            
            # Get top K predictions
            top_probs, top_indices = torch.topk(probs[0], k=min(top_k, self.num_labels))
            
            predictions = [
                {
                    "label": self.id2label[idx.item()],
                    "score": float(prob.item())
                }
                for prob, idx in zip(top_probs, top_indices)
            ]
            
            logger.debug(f"[ImageClassification] ✅ Top prediction: {predictions[0]['label']} ({predictions[0]['score']:.2%})")
            
            return {
                "status": "success",
                "predictions": predictions,
                "top_prediction": predictions[0]["label"] if predictions else None
            }
            
        except Exception as e:
            logger.error(f"[ImageClassification] ❌ Classification failed: {e}", exc_info=True)
            return {
                "status": "error",
                "message": f"Classification failed: {str(e)}"
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
            logger.info("[ImageClassification] Model unloaded")
            
        except Exception as e:
            logger.error(f"[ImageClassification] Error during unload: {e}")
