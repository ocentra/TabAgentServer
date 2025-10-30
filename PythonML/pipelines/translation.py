"""
TranslationPipeline - Language translation

For: Translation models that convert text from one language to another
Examples: NLLB, M2M100, MarianMT, OpusMT

Uses Hugging Face Transformers for translation.
"""

import logging
from typing import Any, Dict, List, Optional
from .base import BasePipeline

logger = logging.getLogger(__name__)


class TranslationPipeline(BasePipeline):
    """
    Translation pipeline for language-to-language conversion.
    
    Supports both direction-specific (e.g., en-fr) and multilingual models (e.g., NLLB).
    Uses transformers AutoModelForSeq2SeqLM for sequence-to-sequence translation.
    """
    
    def pipeline_type(self) -> str:
        return "translation"
    
    def load(self, model_id: str, options: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """
        Load translation model using transformers.
        
        Args:
            model_id: HuggingFace model ID (e.g., "facebook/nllb-200-distilled-600M")
            options: Loading options (device, dtype, etc.)
        
        Returns:
            Status dict with 'status' and 'message'
        """
        try:
            logger.info(f"[Translation] Loading model: {model_id}")
            
            from transformers import AutoTokenizer, AutoModelForSeq2SeqLM
            import torch
            
            opts = options or {}
            
            # Determine device
            device = opts.get("device", "cuda" if torch.cuda.is_available() else "cpu")
            logger.info(f"[Translation] Using device: {device}")
            
            # Load tokenizer
            logger.info(f"[Translation] Loading tokenizer...")
            self.tokenizer = AutoTokenizer.from_pretrained(
                model_id,
                trust_remote_code=opts.get("trust_remote_code", False),
                use_fast=opts.get("use_fast_tokenizer", True)
            )
            
            # Load model
            logger.info(f"[Translation] Loading model...")
            torch_dtype = torch.float16 if device == "cuda" else torch.float32
            
            self.model = AutoModelForSeq2SeqLM.from_pretrained(
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
            logger.info(f"[Translation] ✅ Model loaded successfully on {device}")
            
            return {
                "status": "success",
                "message": f"Model {model_id} loaded on {device}",
                "device": device,
                "dtype": str(torch_dtype)
            }
            
        except Exception as e:
            logger.error(f"[Translation] ❌ Load failed: {e}", exc_info=True)
            self._loaded = False
            return {
                "status": "error",
                "message": f"Failed to load model: {str(e)}"
            }
    
    def generate(self, input_data: Dict[str, Any]) -> Dict[str, Any]:
        """
        Translate text.
        
        Args:
            input_data: Dict with:
                - text: Input text or list of texts to translate
                - source_lang: Source language code (e.g., "eng_Latn") - optional for some models
                - target_lang: Target language code (e.g., "fra_Latn") - required for multilingual models
                - max_length: Max length of translation (default: 512)
                - num_beams: Beam search beams (default: 5)
        
        Returns:
            Dict with 'status', 'translated_text', and metadata
        """
        if not self.is_loaded():
            return {"status": "error", "message": "Model not loaded"}
        
        try:
            import torch
            
            # Get input text
            text = input_data.get("text")
            if not text:
                return {"status": "error", "message": "No text provided"}
            
            # Handle single string or list of strings
            single_input = isinstance(text, str)
            if single_input:
                text = [text]
            
            # Get language parameters
            source_lang = input_data.get("source_lang")
            target_lang = input_data.get("target_lang")
            
            # Get generation parameters
            max_length = input_data.get("max_length", 512)
            num_beams = input_data.get("num_beams", 5)
            
            logger.debug(f"[Translation] Translating {len(text)} text(s) ({source_lang or 'auto'} → {target_lang})")
            
            # Set tokenizer language tokens for multilingual models (e.g., NLLB)
            if source_lang and hasattr(self.tokenizer, 'src_lang'):
                self.tokenizer.src_lang = source_lang
            if target_lang and hasattr(self.tokenizer, 'tgt_lang'):
                self.tokenizer.tgt_lang = target_lang
            
            # Tokenize input
            inputs = self.tokenizer(
                text,
                return_tensors="pt",
                padding=True,
                truncation=True,
                max_length=max_length
            )
            
            device = next(self.model.parameters()).device
            inputs = {k: v.to(device) for k, v in inputs.items()}
            
            # Generate forced_bos_token_id for target language if needed
            gen_kwargs = {}
            if target_lang:
                # For NLLB-style models
                target_lang_id = self.tokenizer.convert_tokens_to_ids(target_lang)
                if target_lang_id is not None and target_lang_id != self.tokenizer.unk_token_id:
                    gen_kwargs["forced_bos_token_id"] = target_lang_id
            
            # Translate
            with torch.no_grad():
                outputs = self.model.generate(
                    **inputs,
                    max_length=max_length,
                    num_beams=num_beams,
                    **gen_kwargs
                )
            
            # Decode
            translations = self.tokenizer.batch_decode(
                outputs,
                skip_special_tokens=True
            )
            
            # Return single translation if single input
            if single_input:
                translations = translations[0]
            
            logger.debug(f"[Translation] ✅ Translated {len(text)} text(s)")
            
            return {
                "status": "success",
                "translated_text": translations,
                "source_lang": source_lang,
                "target_lang": target_lang,
                "count": len(text) if not single_input else 1
            }
            
        except Exception as e:
            logger.error(f"[Translation] ❌ Translation failed: {e}", exc_info=True)
            return {
                "status": "error",
                "message": f"Translation failed: {str(e)}"
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
            logger.info("[Translation] Model unloaded")
            
        except Exception as e:
            logger.error(f"[Translation] Error during unload: {e}")
