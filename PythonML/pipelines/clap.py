"""
ClapPipeline - Contrastive Language-Audio Pretraining

Specialized for: Audio embeddings, audio-text similarity, zero-shot audio classification
Architecture-specific: CLAP (audio version of CLIP)

Uses Hugging Face Transformers for CLAP inference.
"""

import logging
from typing import Any, Dict, List, Optional
from .base import BasePipeline

logger = logging.getLogger(__name__)


class ClapPipeline(BasePipeline):
    """
    CLAP Contrastive Language-Audio Pre-training pipeline.
    
    Supports:
    - Audio embedding generation
    - Text embedding generation
    - Audio-text similarity scoring
    - Zero-shot audio classification
    
    Uses transformers ClapModel with audio and text encoders.
    """
    
    def pipeline_type(self) -> str:
        return "audio-classification"
    
    def load(self, model_id: str, options: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """
        Load CLAP model using transformers.
        
        Args:
            model_id: HuggingFace model ID (e.g., "laion/clap-htsat-unfused")
            options: Loading options (device, dtype, etc.)
        
        Returns:
            Status dict with 'status' and 'message'
        """
        try:
            logger.info(f"[CLAP] Loading model: {model_id}")
            
            from transformers import ClapModel, ClapProcessor
            import torch
            
            opts = options or {}
            
            # Determine device
            device = opts.get("device", "cuda" if torch.cuda.is_available() else "cpu")
            logger.info(f"[CLAP] Using device: {device}")
            
            # Load processor
            logger.info(f"[CLAP] Loading processor...")
            self.processor = ClapProcessor.from_pretrained(
                model_id,
                trust_remote_code=opts.get("trust_remote_code", False)
            )
            
            # Load model
            logger.info(f"[CLAP] Loading model...")
            torch_dtype = torch.float16 if device == "cuda" else torch.float32
            
            self.model = ClapModel.from_pretrained(
                model_id,
                torch_dtype=torch_dtype,
                low_cpu_mem_usage=True,
                trust_remote_code=opts.get("trust_remote_code", False)
            )
            
            self.model = self.model.to(device)
            self.model.eval()
            
            self.device = device
            
            self._loaded = True
            logger.info(f"[CLAP] ✅ Model loaded successfully on {device}")
            
            return {
                "status": "success",
                "message": f"Model {model_id} loaded on {device}",
                "device": device,
                "dtype": str(torch_dtype)
            }
            
        except Exception as e:
            logger.error(f"[CLAP] ❌ Load failed: {e}", exc_info=True)
            self._loaded = False
            return {
                "status": "error",
                "message": f"Failed to load model: {str(e)}"
            }
    
    def generate(self, input_data: Dict[str, Any]) -> Dict[str, Any]:
        """
        Run CLAP inference.
        
        Args:
            input_data: Dict with:
                - mode: 'audio_embedding', 'text_embedding', 'similarity', or 'zero_shot'
                - audio: Audio array (numpy) with shape (num_samples,)
                - text: Text or list of texts
                - sampling_rate: Audio sampling rate (default: 48000)
                - candidates: List of labels for zero-shot classification
        
        Returns:
            Dict with 'status' and mode-specific results
        """
        if not self.is_loaded():
            return {"status": "error", "message": "Model not loaded"}
        
        try:
            mode = input_data.get("mode", "similarity")
            
            if mode == "audio_embedding":
                return self._encode_audio(input_data)
            elif mode == "text_embedding":
                return self._encode_text(input_data)
            elif mode == "similarity":
                return self._compute_similarity(input_data)
            elif mode == "zero_shot":
                return self._zero_shot_classification(input_data)
            else:
                return {"status": "error", "message": f"Unknown mode: {mode}"}
            
        except Exception as e:
            logger.error(f"[CLAP] ❌ Generation failed: {e}", exc_info=True)
            return {"status": "error", "message": f"Generation failed: {str(e)}"}
    
    def _encode_audio(self, input_data: Dict[str, Any]) -> Dict[str, Any]:
        """Encode audio to embeddings"""
        import torch
        import numpy as np
        
        audio = input_data.get("audio")
        if audio is None:
            return {"status": "error", "message": "No audio provided"}
        
        if isinstance(audio, list):
            audio = np.array(audio, dtype=np.float32)
        
        sampling_rate = input_data.get("sampling_rate", 48000)
        normalize = input_data.get("normalize", True)
        
        # Process audio
        inputs = self.processor(
            audios=audio,
            sampling_rate=sampling_rate,
            return_tensors="pt"
        )
        
        inputs = {k: v.to(self.device) for k, v in inputs.items()}
        
        # Encode
        with torch.no_grad():
            audio_features = self.model.get_audio_features(**inputs)
        
        if normalize:
            audio_features = audio_features / audio_features.norm(dim=-1, keepdim=True)
        
        embeddings = audio_features.cpu().numpy().tolist()[0]
        
        return {
            "status": "success",
            "embeddings": embeddings,
            "dimension": len(embeddings)
        }
    
    def _encode_text(self, input_data: Dict[str, Any]) -> Dict[str, Any]:
        """Encode text to embeddings"""
        import torch
        
        text = input_data.get("text")
        if not text:
            return {"status": "error", "message": "No text provided"}
        
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
        
        if normalize:
            text_features = text_features / text_features.norm(dim=-1, keepdim=True)
        
        embeddings = text_features.cpu().numpy().tolist()
        
        if single_input:
            embeddings = embeddings[0]
        
        return {
            "status": "success",
            "embeddings": embeddings,
            "count": len(text) if not single_input else 1,
            "dimension": len(embeddings[0]) if not single_input else len(embeddings)
        }
    
    def _compute_similarity(self, input_data: Dict[str, Any]) -> Dict[str, Any]:
        """Compute audio-text similarity"""
        import torch
        import numpy as np
        
        audio = input_data.get("audio")
        text = input_data.get("text")
        
        if audio is None or not text:
            return {"status": "error", "message": "Both audio and text required"}
        
        if isinstance(audio, list):
            audio = np.array(audio, dtype=np.float32)
        
        if isinstance(text, str):
            text = [text]
        
        sampling_rate = input_data.get("sampling_rate", 48000)
        
        # Process inputs
        inputs = self.processor(
            text=text,
            audios=audio,
            sampling_rate=sampling_rate,
            return_tensors="pt",
            padding=True
        )
        
        inputs = {k: v.to(self.device) for k, v in inputs.items()}
        
        # Compute similarity
        with torch.no_grad():
            outputs = self.model(**inputs)
            logits_per_audio = outputs.logits_per_audio
            probs = logits_per_audio.softmax(dim=1)
        
        similarities = probs.cpu().numpy().tolist()[0]
        
        return {
            "status": "success",
            "similarities": similarities,
            "text": text
        }
    
    def _zero_shot_classification(self, input_data: Dict[str, Any]) -> Dict[str, Any]:
        """Zero-shot audio classification"""
        import torch
        import numpy as np
        
        audio = input_data.get("audio")
        candidates = input_data.get("candidates")
        
        if audio is None or not candidates:
            return {"status": "error", "message": "Both audio and candidates required"}
        
        if isinstance(audio, list):
            audio = np.array(audio, dtype=np.float32)
        
        sampling_rate = input_data.get("sampling_rate", 48000)
        
        # Format text prompts
        text_prompts = [f"sound of {label}" for label in candidates]
        
        # Process inputs
        inputs = self.processor(
            text=text_prompts,
            audios=audio,
            sampling_rate=sampling_rate,
            return_tensors="pt",
            padding=True
        )
        
        inputs = {k: v.to(self.device) for k, v in inputs.items()}
        
        # Compute predictions
        with torch.no_grad():
            outputs = self.model(**inputs)
            logits_per_audio = outputs.logits_per_audio
            probs = logits_per_audio.softmax(dim=1)
        
        probabilities = probs.cpu().numpy().tolist()[0]
        
        # Sort by probability
        results = list(zip(candidates, probabilities))
        results.sort(key=lambda x: x[1], reverse=True)
        
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
            
            import torch
            if torch.cuda.is_available():
                torch.cuda.empty_cache()
            
            self._loaded = False
            logger.info("[CLAP] Model unloaded")
            
        except Exception as e:
            logger.error(f"[CLAP] Error during unload: {e}")
