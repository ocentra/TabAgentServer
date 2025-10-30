"""
WhisperPipeline - Automatic Speech Recognition

Specialized for: Speech-to-text, audio transcription
Architecture-specific: Whisper model handling

Uses Hugging Face Transformers for Whisper inference.
"""

import logging
from typing import Any, Dict, Optional
from .base import BasePipeline

logger = logging.getLogger(__name__)


class WhisperPipeline(BasePipeline):
    """
    Whisper Automatic Speech Recognition pipeline.
    
    Uses transformers WhisperForConditionalGeneration for audio transcription.
    Supports multilingual transcription and translation.
    """
    
    def pipeline_type(self) -> str:
        return "automatic-speech-recognition"
    
    def load(self, model_id: str, options: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """
        Load Whisper model using transformers.
        
        Args:
            model_id: HuggingFace model ID (e.g., "openai/whisper-small")
            options: Loading options (device, dtype, etc.)
        
        Returns:
            Status dict with 'status' and 'message'
        """
        try:
            logger.info(f"[Whisper] Loading model: {model_id}")
            
            from transformers import WhisperProcessor, WhisperForConditionalGeneration
            import torch
            
            opts = options or {}
            
            # Determine device
            device = opts.get("device", "cuda" if torch.cuda.is_available() else "cpu")
            logger.info(f"[Whisper] Using device: {device}")
            
            # Load processor (handles audio preprocessing and tokenization)
            logger.info(f"[Whisper] Loading processor...")
            self.processor = WhisperProcessor.from_pretrained(
                model_id,
                trust_remote_code=opts.get("trust_remote_code", False)
            )
            
            # Load model
            logger.info(f"[Whisper] Loading model...")
            torch_dtype = torch.float16 if device == "cuda" else torch.float32
            
            self.model = WhisperForConditionalGeneration.from_pretrained(
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
            logger.info(f"[Whisper] ✅ Model loaded successfully on {device}")
            
            return {
                "status": "success",
                "message": f"Model {model_id} loaded on {device}",
                "device": device,
                "dtype": str(torch_dtype)
            }
            
        except Exception as e:
            logger.error(f"[Whisper] ❌ Load failed: {e}", exc_info=True)
            self._loaded = False
            return {
                "status": "error",
                "message": f"Failed to load model: {str(e)}"
            }
    
    def generate(self, input_data: Dict[str, Any]) -> Dict[str, Any]:
        """
        Run Whisper transcription.
        
        Args:
            input_data: Dict with:
                - audio: Audio data as numpy array (float32, 16kHz)
                         or dict with 'array' and 'sampling_rate'
                - task: 'transcribe' or 'translate' (default: 'transcribe')
                - language: Language code (e.g., 'en', 'es') or None for auto-detect
                - return_timestamps: Whether to return timestamps (default: False)
        
        Returns:
            Dict with 'status', 'text', and optionally 'chunks' if timestamps requested
        """
        if not self.is_loaded():
            return {"status": "error", "message": "Model not loaded"}
        
        try:
            import torch
            import numpy as np
            
            # Get audio data
            audio = input_data.get("audio")
            if audio is None:
                return {"status": "error", "message": "No audio data provided"}
            
            # Handle different audio formats
            if isinstance(audio, dict):
                audio_array = audio.get("array")
                sampling_rate = audio.get("sampling_rate", 16000)
            elif isinstance(audio, np.ndarray):
                audio_array = audio
                sampling_rate = 16000
            elif isinstance(audio, list):
                audio_array = np.array(audio, dtype=np.float32)
                sampling_rate = 16000
            else:
                return {"status": "error", "message": "Invalid audio format"}
            
            # Get generation parameters
            task = input_data.get("task", "transcribe")
            language = input_data.get("language")
            return_timestamps = input_data.get("return_timestamps", False)
            
            logger.debug(f"[Whisper] Transcribing audio (task={task}, lang={language})")
            
            # Process audio input
            input_features = self.processor(
                audio_array,
                sampling_rate=sampling_rate,
                return_tensors="pt"
            ).input_features
            
            input_features = input_features.to(self.device)
            
            # Prepare generation kwargs
            gen_kwargs = {}
            if language:
                gen_kwargs["language"] = language
            if task:
                gen_kwargs["task"] = task
            
            # Generate transcription
            with torch.no_grad():
                if return_timestamps:
                    # Generate with timestamps
                    predicted_ids = self.model.generate(
                        input_features,
                        return_timestamps=True,
                        **gen_kwargs
                    )
                else:
                    # Standard generation
                    predicted_ids = self.model.generate(
                        input_features,
                        **gen_kwargs
                    )
            
            # Decode output
            transcription = self.processor.batch_decode(
                predicted_ids,
                skip_special_tokens=True
            )[0]
            
            result = {
                "status": "success",
                "text": transcription.strip(),
                "language": language or "auto-detected"
            }
            
            # Add timestamps if requested
            if return_timestamps:
                # Parse timestamps from output
                # Note: Timestamp parsing would need additional logic
                # For now, we return the raw text
                result["timestamps_available"] = False
            
            logger.debug(f"[Whisper] ✅ Transcribed {len(transcription)} chars")
            
            return result
            
        except Exception as e:
            logger.error(f"[Whisper] ❌ Transcription failed: {e}", exc_info=True)
            return {
                "status": "error",
                "message": f"Transcription failed: {str(e)}"
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
            logger.info("[Whisper] Model unloaded")
            
        except Exception as e:
            logger.error(f"[Whisper] Error during unload: {e}")
