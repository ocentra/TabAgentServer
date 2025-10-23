"""
TokenizerPipeline - Standalone tokenizer

For: Using tokenizers without loading full models
Useful for token counting, preprocessing

Format-agnostic: Works with any tokenizer format
"""

import logging
from typing import Any, Dict, Optional
from .base import BasePipeline

logger = logging.getLogger(__name__)


class TokenizerPipeline(BasePipeline):
    """
    Standalone tokenizer pipeline
    
    Loads only the tokenizer for token counting and preprocessing.
    """
    
    def pipeline_type(self) -> str:
        return "tokenizer"
    
    def load(self, model_id: str, options: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """Load tokenizer - format-agnostic"""
        try:
            logger.info(f"[Tokenizer] Loading model: {model_id}")
            
            opts = options or {}
            
            # Load only tokenizer (no model)
            from transformers import AutoTokenizer
            auth_token = opts.get("auth_token")
            self.tokenizer = AutoTokenizer.from_pretrained(
                model_id,
                trust_remote_code=True,
                token=auth_token
            )
            
            self._loaded = True
            self.model_id = model_id
            self.backend_type = "python-tokenizer"
            
            return {
                "status": "success",
                "pipeline_type": self.pipeline_type(),
                "backend_type": self.backend_type,
                "model_id": model_id
            }
            
        except Exception as e:
            logger.error(f"[Tokenizer] Load failed: {e}", exc_info=True)
            return {"status": "error", "message": str(e)}
    
    def generate(self, input_data: Dict[str, Any]) -> Dict[str, Any]:
        """Tokenize text"""
        if not self.is_loaded():
            return {"status": "error", "message": "Tokenizer not loaded"}
        
        try:
            text = input_data.get("text", "")
            
            # Tokenize
            tokens = self.tokenizer.encode(text)
            
            return {
                "status": "success",
                "tokens": tokens,
                "token_count": len(tokens),
                "pipeline_type": self.pipeline_type()
            }
        except Exception as e:
            logger.error(f"[Tokenizer] Tokenization failed: {e}", exc_info=True)
            return {"status": "error", "message": str(e)}

