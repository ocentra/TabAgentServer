"""
ZeroShotClassificationPipeline - Zero-shot classification

For: Models that classify text into arbitrary categories without training
Uses NLI (Natural Language Inference) models for zero-shot classification

Uses Hugging Face Transformers zero-shot classification.
"""

import logging
from typing import Any, Dict, List, Optional
from .base import BasePipeline

logger = logging.getLogger(__name__)


class ZeroShotClassificationPipeline(BasePipeline):
    """
    Zero-shot classification pipeline.
    
    Classifies text into arbitrary categories without requiring training data.
    Uses NLI (Natural Language Inference) models like BART or DeBERTa.
    """
    
    def pipeline_type(self) -> str:
        return "zero-shot-classification"
    
    def load(self, model_id: str, options: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """
        Load zero-shot classification model using transformers.
        
        Args:
            model_id: HuggingFace model ID (e.g., "facebook/bart-large-mnli")
            options: Loading options (device, dtype, etc.)
        
        Returns:
            Status dict with 'status' and 'message'
        """
        try:
            logger.info(f"[ZeroShot] Loading model: {model_id}")
            
            from transformers import AutoTokenizer, AutoModelForSequenceClassification
            import torch
            
            opts = options or {}
            
            # Determine device
            device = opts.get("device", "cuda" if torch.cuda.is_available() else "cpu")
            logger.info(f"[ZeroShot] Using device: {device}")
            
            # Load tokenizer
            logger.info(f"[ZeroShot] Loading tokenizer...")
            self.tokenizer = AutoTokenizer.from_pretrained(
                model_id,
                trust_remote_code=opts.get("trust_remote_code", False),
                use_fast=opts.get("use_fast_tokenizer", True)
            )
            
            # Load model
            logger.info(f"[ZeroShot] Loading model...")
            torch_dtype = torch.float16 if device == "cuda" else torch.float32
            
            self.model = AutoModelForSequenceClassification.from_pretrained(
                model_id,
                torch_dtype=torch_dtype,
                device_map="auto" if device == "cuda" else None,
                trust_remote_code=opts.get("trust_remote_code", False),
                low_cpu_mem_usage=True
            )
            
            if device == "cpu":
                self.model = self.model.to(device)
            
            self.model.eval()
            self.device = device
            
            # Detect entailment/contradiction label IDs for NLI models
            self.entailment_id = self._get_label_id("entailment")
            self.contradiction_id = self._get_label_id("contradiction")
            
            self._loaded = True
            logger.info(f"[ZeroShot] ✅ Model loaded successfully on {device}")
            
            return {
                "status": "success",
                "message": f"Model {model_id} loaded on {device}",
                "device": device,
                "dtype": str(torch_dtype)
            }
            
        except Exception as e:
            logger.error(f"[ZeroShot] ❌ Load failed: {e}", exc_info=True)
            self._loaded = False
            return {
                "status": "error",
                "message": f"Failed to load model: {str(e)}"
            }
    
    def _get_label_id(self, label: str) -> Optional[int]:
        """Get the token ID for a specific label (entailment/contradiction)"""
        if hasattr(self.model.config, 'label2id') and label in self.model.config.label2id:
            return self.model.config.label2id[label]
        # Try variations
        variations = [label.lower(), label.upper(), label.capitalize()]
        for var in variations:
            if hasattr(self.model.config, 'label2id') and var in self.model.config.label2id:
                return self.model.config.label2id[var]
        return None
    
    def generate(self, input_data: Dict[str, Any]) -> Dict[str, Any]:
        """
        Perform zero-shot classification.
        
        Args:
            input_data: Dict with:
                - text: Input text to classify
                - candidate_labels: List of possible labels
                - hypothesis_template: Template for hypothesis (default: "This example is {}")
                - multi_label: Whether text can have multiple labels (default: False)
        
        Returns:
            Dict with 'status', 'labels', 'scores', and 'top_label'
        """
        if not self.is_loaded():
            return {"status": "error", "message": "Model not loaded"}
        
        try:
            import torch
            import numpy as np
            
            # Get inputs
            text = input_data.get("text")
            candidate_labels = input_data.get("candidate_labels")
            
            if not text:
                return {"status": "error", "message": "No text provided"}
            if not candidate_labels:
                return {"status": "error", "message": "No candidate_labels provided"}
            
            # Get parameters
            hypothesis_template = input_data.get("hypothesis_template", "This example is {}")
            multi_label = input_data.get("multi_label", False)
            
            logger.debug(f"[ZeroShot] Classifying text with {len(candidate_labels)} labels")
            
            # Create hypothesis for each label
            hypotheses = [hypothesis_template.format(label) for label in candidate_labels]
            
            # Create premise-hypothesis pairs
            pairs = [[text, hypothesis] for hypothesis in hypotheses]
            
            # Tokenize all pairs
            inputs = self.tokenizer(
                pairs,
                padding=True,
                truncation=True,
                return_tensors="pt"
            )
            
            inputs = {k: v.to(self.device) for k, v in inputs.items()}
            
            # Get predictions
            with torch.no_grad():
                outputs = self.model(**inputs)
                logits = outputs.logits
            
            # Extract entailment scores
            if self.entailment_id is not None:
                # NLI model - use entailment probability
                probs = torch.nn.functional.softmax(logits, dim=-1)
                scores = probs[:, self.entailment_id].cpu().numpy()
            else:
                # Fallback to first logit if no entailment label found
                scores = logits[:, 0].cpu().numpy()
            
            # Normalize scores
            if multi_label:
                # For multi-label, keep raw probabilities
                pass
            else:
                # For single-label, softmax over all labels
                scores = np.exp(scores) / np.sum(np.exp(scores))
            
            # Sort by score
            sorted_indices = np.argsort(scores)[::-1]
            sorted_labels = [candidate_labels[i] for i in sorted_indices]
            sorted_scores = [float(scores[i]) for i in sorted_indices]
            
            logger.debug(f"[ZeroShot] ✅ Top label: {sorted_labels[0]} ({sorted_scores[0]:.2%})")
            
            return {
                "status": "success",
                "labels": sorted_labels,
                "scores": sorted_scores,
                "top_label": sorted_labels[0],
                "multi_label": multi_label
            }
            
        except Exception as e:
            logger.error(f"[ZeroShot] ❌ Classification failed: {e}", exc_info=True)
            return {
                "status": "error",
                "message": f"Classification failed: {str(e)}"
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
            logger.info("[ZeroShot] Model unloaded")
            
        except Exception as e:
            logger.error(f"[ZeroShot] Error during unload: {e}")
