"""
TextToSpeechPipeline - Text-to-speech synthesis

For: TTS models that generate audio from text
Examples: SpeechT5, Bark, Microsoft TTS, Coqui XTTS

Uses Hugging Face Transformers for TTS inference.
"""

import logging
from typing import Any, Dict, Optional
from .base import BasePipeline

logger = logging.getLogger(__name__)


class TextToSpeechPipeline(BasePipeline):
    """
    Text-to-speech synthesis pipeline.
    
    Generates audio waveforms from text input.
    Supports various TTS architectures via transformers.
    """
    
    def pipeline_type(self) -> str:
        return "text-to-speech"
    
    def load(self, model_id: str, options: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """
        Load TTS model using transformers.
        
        Args:
            model_id: HuggingFace model ID (e.g., "microsoft/speecht5_tts")
            options: Loading options (device, vocoder_id, etc.)
        
        Returns:
            Status dict with 'status' and 'message'
        """
        try:
            logger.info(f"[TTS] Loading model: {model_id}")
            
            from transformers import AutoProcessor, AutoModel
            import torch
            
            opts = options or {}
            
            # Determine device
            device = opts.get("device", "cuda" if torch.cuda.is_available() else "cpu")
            logger.info(f"[TTS] Using device: {device}")
            
            # Load processor
            logger.info(f"[TTS] Loading processor...")
            self.processor = AutoProcessor.from_pretrained(
                model_id,
                trust_remote_code=opts.get("trust_remote_code", False)
            )
            
            # Load model
            logger.info(f"[TTS] Loading model...")
            torch_dtype = torch.float16 if device == "cuda" else torch.float32
            
            self.model = AutoModel.from_pretrained(
                model_id,
                torch_dtype=torch_dtype,
                low_cpu_mem_usage=True,
                trust_remote_code=opts.get("trust_remote_code", False)
            )
            
            self.model = self.model.to(device)
            self.model.eval()
            
            # Load vocoder if needed (for models like SpeechT5)
            vocoder_id = opts.get("vocoder_id")
            if vocoder_id:
                logger.info(f"[TTS] Loading vocoder: {vocoder_id}")
                from transformers import AutoModel as VocoderModel
                self.vocoder = VocoderModel.from_pretrained(vocoder_id)
                self.vocoder = self.vocoder.to(device)
                self.vocoder.eval()
            else:
                self.vocoder = None
            
            self.device = device
            
            self._loaded = True
            logger.info(f"[TTS] ✅ Model loaded successfully on {device}")
            
            return {
                "status": "success",
                "message": f"Model {model_id} loaded on {device}",
                "device": device,
                "dtype": str(torch_dtype),
                "has_vocoder": self.vocoder is not None
            }
            
        except Exception as e:
            logger.error(f"[TTS] ❌ Load failed: {e}", exc_info=True)
            self._loaded = False
            return {
                "status": "error",
                "message": f"Failed to load model: {str(e)}"
            }
    
    def generate(self, input_data: Dict[str, Any]) -> Dict[str, Any]:
        """
        Synthesize speech from text.
        
        Args:
            input_data: Dict with:
                - text: Input text to synthesize
                - speaker_embeddings: Optional speaker embeddings (for multi-speaker models)
                - sampling_rate: Desired sampling rate (default: model's default)
        
        Returns:
            Dict with 'status', 'audio' (numpy array), and 'sampling_rate'
        """
        if not self.is_loaded():
            return {"status": "error", "message": "Model not loaded"}
        
        try:
            import torch
            import numpy as np
            
            # Get text input
            text = input_data.get("text")
            if not text:
                return {"status": "error", "message": "No text provided"}
            
            logger.debug(f"[TTS] Synthesizing speech for text: {text[:50]}...")
            
            # Process text
            inputs = self.processor(
                text=text,
                return_tensors="pt"
            )
            
            inputs = {k: v.to(self.device) for k, v in inputs.items()}
            
            # Add speaker embeddings if provided
            speaker_embeddings = input_data.get("speaker_embeddings")
            if speaker_embeddings is not None:
                if isinstance(speaker_embeddings, list):
                    speaker_embeddings = torch.tensor(speaker_embeddings, dtype=torch.float32)
                speaker_embeddings = speaker_embeddings.to(self.device)
                inputs["speaker_embeddings"] = speaker_embeddings.unsqueeze(0) if speaker_embeddings.dim() == 1 else speaker_embeddings
            
            # Generate speech
            with torch.no_grad():
                if self.vocoder:
                    # Models with separate vocoder (e.g., SpeechT5)
                    speech = self.model.generate_speech(**inputs, vocoder=self.vocoder)
                else:
                    # End-to-end models (e.g., Bark)
                    outputs = self.model.generate(**inputs)
                    speech = outputs if isinstance(outputs, torch.Tensor) else outputs.audio
            
            # Convert to numpy
            audio_array = speech.cpu().numpy()
            
            # Get sampling rate
            sampling_rate = getattr(self.model.config, "sampling_rate", 16000)
            
            logger.debug(f"[TTS] ✅ Generated {len(audio_array)} samples at {sampling_rate}Hz")
            
            return {
                "status": "success",
                "audio": audio_array.tolist(),
                "sampling_rate": sampling_rate,
                "duration_seconds": len(audio_array) / sampling_rate
            }
            
        except Exception as e:
            logger.error(f"[TTS] ❌ Synthesis failed: {e}", exc_info=True)
            return {
                "status": "error",
                "message": f"Synthesis failed: {str(e)}"
            }
    
    def unload(self):
        """Unload model from memory"""
        try:
            if hasattr(self, 'model'):
                del self.model
            if hasattr(self, 'processor'):
                del self.processor
            if hasattr(self, 'vocoder'):
                del self.vocoder
            
            import torch
            if torch.cuda.is_available():
                torch.cuda.empty_cache()
            
            self._loaded = False
            logger.info("[TTS] Model unloaded")
            
        except Exception as e:
            logger.error(f"[TTS] Error during unload: {e}")
