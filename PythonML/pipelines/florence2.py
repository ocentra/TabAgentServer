"""
Florence2Pipeline - Image-to-text vision model

Specialized for: OCR, captioning, visual QA, region detection, object detection
Architecture-specific: Florence2 special tokens (<OCR>, <CAPTION>, <OD>, etc.)

Uses Hugging Face Transformers for Florence2 inference.
"""

import logging
from typing import Any, Dict, Optional
from .base import BasePipeline

logger = logging.getLogger(__name__)


class Florence2Pipeline(BasePipeline):
    """
    Florence2 Multi-task Vision Model pipeline.
    
    Supports multiple vision tasks via special tokens:
    - <CAPTION>: General image captioning
    - <DETAILED_CAPTION>: Detailed image description
    - <MORE_DETAILED_CAPTION>: Very detailed description
    - <OD>: Object detection
    - <OCR>: Optical character recognition
    - <OCR_WITH_REGION>: OCR with bounding boxes
    - <DENSE_REGION_CAPTION>: Region-based captioning
    - <REGION_PROPOSAL>: Region proposals
    
    Uses transformers AutoModelForCausalLM with Florence2 architecture.
    """
    
    def pipeline_type(self) -> str:
        return "image-to-text"
    
    def load(self, model_id: str, options: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """
        Load Florence2 model using transformers.
        
        Args:
            model_id: HuggingFace model ID (e.g., "microsoft/Florence-2-base")
            options: Loading options (device, dtype, trust_remote_code, etc.)
        
        Returns:
            Status dict with 'status' and 'message'
        """
        try:
            logger.info(f"[Florence2] Loading model: {model_id}")
            
            from transformers import AutoModelForCausalLM, AutoProcessor
            import torch
            
            opts = options or {}
            
            # Determine device
            device = opts.get("device", "cuda" if torch.cuda.is_available() else "cpu")
            logger.info(f"[Florence2] Using device: {device}")
            
            # Load processor (handles image preprocessing and special tokens)
            logger.info(f"[Florence2] Loading processor...")
            self.processor = AutoProcessor.from_pretrained(
                model_id,
                trust_remote_code=True,  # Florence2 requires this
                token=opts.get("auth_token")
            )
            
            # Load model
            logger.info(f"[Florence2] Loading model...")
            torch_dtype = torch.float16 if device == "cuda" else torch.float32
            
            self.model = AutoModelForCausalLM.from_pretrained(
                model_id,
                torch_dtype=torch_dtype,
                device_map="auto" if device == "cuda" else None,
                trust_remote_code=True,  # Florence2 requires this
                low_cpu_mem_usage=True
            )
            
            # Move to device if CPU
            if device == "cpu":
                self.model = self.model.to(device)
            
            self.model.eval()
            
            # Store device for later use
            self.device = device
            
            self._loaded = True
            logger.info(f"[Florence2] ✅ Model loaded successfully on {device}")
            
            return {
                "status": "success",
                "message": f"Model {model_id} loaded on {device}",
                "device": device,
                "dtype": str(torch_dtype),
                "supported_tasks": [
                    "<CAPTION>", "<DETAILED_CAPTION>", "<MORE_DETAILED_CAPTION>",
                    "<OD>", "<OCR>", "<OCR_WITH_REGION>", "<DENSE_REGION_CAPTION>",
                    "<REGION_PROPOSAL>"
                ]
            }
            
        except Exception as e:
            logger.error(f"[Florence2] ❌ Load failed: {e}", exc_info=True)
            self._loaded = False
            return {
                "status": "error",
                "message": f"Failed to load model: {str(e)}"
            }
    
    def generate(self, input_data: Dict[str, Any]) -> Dict[str, Any]:
        """
        Run Florence2 inference.
        
        Args:
            input_data: Dict with:
                - image: PIL Image, numpy array, or base64 string
                - task: Task token (e.g., "<CAPTION>", "<OCR>") (default: "<CAPTION>")
                - text: Optional additional text prompt
                - max_new_tokens: Max tokens to generate (default: 1024)
                - num_beams: Number of beams for beam search (default: 3)
                - do_sample: Whether to sample (default: False for deterministic)
        
        Returns:
            Dict with 'status', 'text', and task-specific results
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
            
            # Convert image to PIL Image
            if isinstance(image_input, str):
                if image_input.startswith("data:image"):
                    # Base64 encoded
                    image_data = image_input.split(",")[1]
                    image = Image.open(BytesIO(base64.b64decode(image_data)))
                else:
                    # File path
                    image = Image.open(image_input)
            elif isinstance(image_input, np.ndarray):
                # Numpy array
                image = Image.fromarray(image_input)
            elif isinstance(image_input, Image.Image):
                # Already PIL Image
                image = image_input
            else:
                return {"status": "error", "message": "Invalid image format"}
            
            # Ensure RGB mode
            if image.mode != "RGB":
                image = image.convert("RGB")
            
            # Get task and text prompt
            task = input_data.get("task", "<CAPTION>")
            text_input = input_data.get("text", task)
            
            # Ensure task token is in the prompt
            if not any(token in text_input for token in [
                "<CAPTION>", "<DETAILED_CAPTION>", "<MORE_DETAILED_CAPTION>",
                "<OD>", "<OCR>", "<OCR_WITH_REGION>", "<DENSE_REGION_CAPTION>",
                "<REGION_PROPOSAL>"
            ]):
                text_input = task
            
            logger.debug(f"[Florence2] Processing task: {text_input}")
            
            # Process inputs
            inputs = self.processor(
                text=text_input,
                images=image,
                return_tensors="pt"
            )
            
            # Move to device
            inputs = {k: v.to(self.device) for k, v in inputs.items()}
            
            # Get generation parameters
            max_new_tokens = input_data.get("max_new_tokens", 1024)
            num_beams = input_data.get("num_beams", 3)
            do_sample = input_data.get("do_sample", False)
            
            # Generate
            with torch.no_grad():
                generated_ids = self.model.generate(
                    **inputs,
                    max_new_tokens=max_new_tokens,
                    num_beams=num_beams,
                    do_sample=do_sample
                )
            
            # Decode output
            generated_text = self.processor.batch_decode(
                generated_ids,
                skip_special_tokens=False
            )[0]
            
            # Post-process output based on task
            result = self._post_process_output(generated_text, task)
            
            logger.debug(f"[Florence2] ✅ Generated output for task {task}")
            
            return {
                "status": "success",
                "text": result,
                "task": task,
                "pipeline_type": self.pipeline_type()
            }
            
        except Exception as e:
            logger.error(f"[Florence2] ❌ Generation failed: {e}", exc_info=True)
            return {
                "status": "error",
                "message": f"Generation failed: {str(e)}"
            }
    
    def _post_process_output(self, text: str, task: str) -> str:
        """
        Post-process Florence2 output based on task type.
        
        Florence2 outputs structured data for certain tasks (e.g., JSON for <OD>).
        This method cleans up the output and returns it.
        """
        # Remove task tokens from output
        for token in [
            "<CAPTION>", "<DETAILED_CAPTION>", "<MORE_DETAILED_CAPTION>",
            "<OD>", "<OCR>", "<OCR_WITH_REGION>", "<DENSE_REGION_CAPTION>",
            "<REGION_PROPOSAL>"
        ]:
            text = text.replace(token, "")
        
        # Remove other special tokens
        text = text.replace("<s>", "").replace("</s>", "").strip()
        
        return text
    
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
            logger.info("[Florence2] Model unloaded")
            
        except Exception as e:
            logger.error(f"[Florence2] Error during unload: {e}")
