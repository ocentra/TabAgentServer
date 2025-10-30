"""
CodeCompletionPipeline - Code generation and completion

For: Code-specific LLMs
Examples: CodeLlama, StarCoder, DeepSeek-Coder, CodeGen

Uses Hugging Face Transformers for code generation.
"""

import logging
from typing import Any, Dict, Optional
from .base import BasePipeline

logger = logging.getLogger(__name__)


class CodeCompletionPipeline(BasePipeline):
    """
    Code completion and generation pipeline.
    
    Specialized for code-specific models with fill-in-the-middle (FIM) support.
    Uses transformers AutoModelForCausalLM optimized for code tasks.
    """
    
    def pipeline_type(self) -> str:
        return "text-generation"
    
    def load(self, model_id: str, options: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """
        Load code completion model using transformers.
        
        Args:
            model_id: HuggingFace model ID (e.g., "bigcode/starcoder")
            options: Loading options (device, dtype, etc.)
        
        Returns:
            Status dict with 'status' and 'message'
        """
        try:
            logger.info(f"[CodeCompletion] Loading model: {model_id}")
            
            from transformers import AutoModelForCausalLM, AutoTokenizer
            import torch
            
            opts = options or {}
            
            # Determine device
            device = opts.get("device", "cuda" if torch.cuda.is_available() else "cpu")
            logger.info(f"[CodeCompletion] Using device: {device}")
            
            # Load tokenizer
            logger.info(f"[CodeCompletion] Loading tokenizer...")
            self.tokenizer = AutoTokenizer.from_pretrained(
                model_id,
                trust_remote_code=opts.get("trust_remote_code", False),
                use_fast=opts.get("use_fast_tokenizer", True)
            )
            
            # Ensure padding token
            if self.tokenizer.pad_token is None:
                self.tokenizer.pad_token = self.tokenizer.eos_token
            
            # Detect FIM (Fill-In-the-Middle) tokens for models that support it
            self.supports_fim = hasattr(self.tokenizer, 'fim_prefix') or '<fim_' in str(self.tokenizer.vocab)
            
            # Load model
            logger.info(f"[CodeCompletion] Loading model...")
            torch_dtype = torch.float16 if device == "cuda" else torch.float32
            
            self.model = AutoModelForCausalLM.from_pretrained(
                model_id,
                torch_dtype=torch_dtype,
                device_map="auto" if device == "cuda" else None,
                trust_remote_code=opts.get("trust_remote_code", False),
                low_cpu_mem_usage=True
            )
            
            if device == "cpu":
                self.model = self.model.to(device)
            
            self.model.eval()
            
            self._loaded = True
            logger.info(f"[CodeCompletion] ✅ Model loaded successfully on {device} (FIM: {self.supports_fim})")
            
            return {
                "status": "success",
                "message": f"Model {model_id} loaded on {device}",
                "device": device,
                "dtype": str(torch_dtype),
                "supports_fim": self.supports_fim
            }
            
        except Exception as e:
            logger.error(f"[CodeCompletion] ❌ Load failed: {e}", exc_info=True)
            self._loaded = False
            return {
                "status": "error",
                "message": f"Failed to load model: {str(e)}"
            }
    
    def generate(self, input_data: Dict[str, Any]) -> Dict[str, Any]:
        """
        Generate code completion.
        
        Args:
            input_data: Dict with:
                - prompt: Code prompt/prefix
                - suffix: Code suffix (for FIM models, optional)
                - max_new_tokens: Max tokens to generate (default: 256)
                - temperature: Sampling temperature (default: 0.2, low for deterministic)
                - top_p: Nucleus sampling (default: 0.95)
                - stop_sequences: List of stop sequences (default: ["\n\n"])
        
        Returns:
            Dict with 'status', 'code', and 'tokens_generated'
        """
        if not self.is_loaded():
            return {"status": "error", "message": "Model not loaded"}
        
        try:
            import torch
            
            prompt = input_data.get("prompt", "")
            suffix = input_data.get("suffix")
            
            if not prompt:
                return {"status": "error", "message": "No prompt provided"}
            
            # Get generation parameters
            max_new_tokens = input_data.get("max_new_tokens", 256)
            temperature = input_data.get("temperature", 0.2)
            top_p = input_data.get("top_p", 0.95)
            stop_sequences = input_data.get("stop_sequences", ["\n\n"])
            
            # Format input for FIM if supported and suffix provided
            if self.supports_fim and suffix:
                # Format: <fim_prefix>PREFIX<fim_suffix>SUFFIX<fim_middle>
                input_text = f"<fim_prefix>{prompt}<fim_suffix>{suffix}<fim_middle>"
                logger.debug(f"[CodeCompletion] Using FIM mode")
            else:
                input_text = prompt
            
            logger.debug(f"[CodeCompletion] Generating with max_tokens={max_new_tokens}")
            
            # Tokenize
            inputs = self.tokenizer(
                input_text,
                return_tensors="pt",
                truncation=True
            )
            
            device = next(self.model.parameters()).device
            inputs = {k: v.to(device) for k, v in inputs.items()}
            
            # Generate
            with torch.no_grad():
                outputs = self.model.generate(
                    **inputs,
                    max_new_tokens=max_new_tokens,
                    temperature=temperature,
                    top_p=top_p,
                    do_sample=temperature > 0,
                    pad_token_id=self.tokenizer.pad_token_id,
                    eos_token_id=self.tokenizer.eos_token_id
                )
            
            # Decode
            generated_code = self.tokenizer.decode(
                outputs[0],
                skip_special_tokens=True
            )
            
            # Remove input prompt
            if generated_code.startswith(prompt):
                generated_code = generated_code[len(prompt):].strip()
            
            # Apply stop sequences
            for stop_seq in stop_sequences:
                if stop_seq in generated_code:
                    generated_code = generated_code.split(stop_seq)[0]
            
            logger.debug(f"[CodeCompletion] ✅ Generated {len(generated_code)} chars")
            
            return {
                "status": "success",
                "code": generated_code,
                "tokens_generated": len(outputs[0]) - len(inputs["input_ids"][0])
            }
            
        except Exception as e:
            logger.error(f"[CodeCompletion] ❌ Generation failed: {e}", exc_info=True)
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
            
            import torch
            if torch.cuda.is_available():
                torch.cuda.empty_cache()
            
            self._loaded = False
            logger.info("[CodeCompletion] Model unloaded")
            
        except Exception as e:
            logger.error(f"[CodeCompletion] Error during unload: {e}")
