"""
TextGenerationPipeline - Generic text generation

For standard LLMs without special architecture requirements.
Supports: Llama, Mistral, Qwen, Phi, etc.

Uses Hugging Face Transformers for inference.
"""

import logging
from typing import Any, Dict, Optional
from .base import BasePipeline

logger = logging.getLogger(__name__)


class TextGenerationPipeline(BasePipeline):
    """
    Generic text generation pipeline.
    
    Uses transformers AutoModelForCausalLM for standard LLMs.
    Supports streaming generation token by token.
    """
    
    def pipeline_type(self) -> str:
        return "text-generation"
    
    def load(self, model_id: str, options: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """
        Load text generation model using transformers.
        
        Args:
            model_id: HuggingFace model ID (e.g., "meta-llama/Llama-2-7b-hf")
            options: Loading options (device, dtype, etc.)
        
        Returns:
            Status dict with 'status' and 'message'
        """
        try:
            logger.info(f"[TextGen] Loading model: {model_id}")
            
            from transformers import AutoModelForCausalLM, AutoTokenizer
            import torch
            
            opts = options or {}
            
            # Determine device (GPU if available)
            device = opts.get("device", "cuda" if torch.cuda.is_available() else "cpu")
            logger.info(f"[TextGen] Using device: {device}")
            
            # Load tokenizer
            logger.info(f"[TextGen] Loading tokenizer...")
            self.tokenizer = AutoTokenizer.from_pretrained(
                model_id,
                trust_remote_code=opts.get("trust_remote_code", False),
                use_fast=opts.get("use_fast_tokenizer", True)
            )
            
            # Ensure padding token is set
            if self.tokenizer.pad_token is None:
                self.tokenizer.pad_token = self.tokenizer.eos_token
            
            # Load model
            logger.info(f"[TextGen] Loading model...")
            torch_dtype = torch.float16 if device == "cuda" else torch.float32
            
            self.model = AutoModelForCausalLM.from_pretrained(
                model_id,
                torch_dtype=torch_dtype,
                device_map="auto" if device == "cuda" else None,
                trust_remote_code=opts.get("trust_remote_code", False),
                low_cpu_mem_usage=True
            )
            
            # Move to device if CPU
            if device == "cpu":
                self.model = self.model.to(device)
            
            self.model.eval()  # Set to eval mode
            
            self._loaded = True
            logger.info(f"[TextGen] ✅ Model loaded successfully on {device}")
            
            return {
                "status": "success",
                "message": f"Model {model_id} loaded on {device}",
                "device": device,
                "dtype": str(torch_dtype)
            }
            
        except Exception as e:
            logger.error(f"[TextGen] ❌ Load failed: {e}", exc_info=True)
            self._loaded = False
            return {
                "status": "error",
                "message": f"Failed to load model: {str(e)}"
            }
    
    def generate(self, input_data: Dict[str, Any]) -> Dict[str, Any]:
        """
        Run text generation inference.
        
        Args:
            input_data: Dict with:
                - text or prompt: Input text
                - max_new_tokens: Max tokens to generate (default: 100)
                - temperature: Sampling temperature (default: 0.7)
                - top_p: Nucleus sampling parameter (default: 0.9)
                - top_k: Top-k sampling parameter (default: 50)
                - do_sample: Whether to sample (default: True)
                - stream: Whether to stream tokens (default: False)
        
        Returns:
            Dict with 'status', 'text', and optionally 'tokens' for streaming
        """
        if not self.is_loaded():
            return {"status": "error", "message": "Model not loaded"}
        
        try:
            import torch
            
            # Get input text
            prompt = input_data.get("text") or input_data.get("prompt")
            if not prompt:
                return {"status": "error", "message": "No input text provided"}
            
            # Get generation parameters
            max_new_tokens = input_data.get("max_new_tokens", 100)
            temperature = input_data.get("temperature", 0.7)
            top_p = input_data.get("top_p", 0.9)
            top_k = input_data.get("top_k", 50)
            do_sample = input_data.get("do_sample", True)
            
            logger.debug(f"[TextGen] Generating with max_tokens={max_new_tokens}, temp={temperature}")
            
            # Tokenize input
            inputs = self.tokenizer(
                prompt,
                return_tensors="pt",
                padding=True,
                truncation=True
            )
            
            # Move to same device as model
            device = next(self.model.parameters()).device
            inputs = {k: v.to(device) for k, v in inputs.items()}
            
            # Generate
            with torch.no_grad():
                outputs = self.model.generate(
                    **inputs,
                    max_new_tokens=max_new_tokens,
                    temperature=temperature,
                    top_p=top_p,
                    top_k=top_k,
                    do_sample=do_sample,
                    pad_token_id=self.tokenizer.pad_token_id,
                    eos_token_id=self.tokenizer.eos_token_id
                )
            
            # Decode output
            generated_text = self.tokenizer.decode(
                outputs[0],
                skip_special_tokens=True
            )
            
            # Remove input prompt from output
            if generated_text.startswith(prompt):
                generated_text = generated_text[len(prompt):].strip()
            
            logger.debug(f"[TextGen] ✅ Generated {len(generated_text)} chars")
            
            return {
                "status": "success",
                "text": generated_text,
                "tokens_generated": len(outputs[0]) - len(inputs["input_ids"][0])
            }
            
        except Exception as e:
            logger.error(f"[TextGen] ❌ Generation failed: {e}", exc_info=True)
            return {
                "status": "error",
                "message": f"Generation failed: {str(e)}"
            }
    
    def unload(self):
        """Unload model from memory"""
        try:
            if hasattr(self, 'model'):
                del self.model
            if hasattr(self, 'tokenizer'):
                del self.tokenizer
            
            # Clear CUDA cache if using GPU
            import torch
            if torch.cuda.is_available():
                torch.cuda.empty_cache()
            
            self._loaded = False
            logger.info("[TextGen] Model unloaded")
            
        except Exception as e:
            logger.error(f"[TextGen] Error during unload: {e}")
