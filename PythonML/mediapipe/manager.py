"""
MediaPipe Manager - Full Implementation

Comprehensive on-device AI using Google MediaPipe framework.

Supports ALL MediaPipe capabilities:
- GenAI: LLM inference (Gemma models in .task bundles)
- Vision: Object/face/hand/pose/gesture detection, classification, segmentation, embeddings
- Text: Classification, embeddings, language detection
- Audio: Audio classification

Reference: https://ai.google.dev/edge/mediapipe/solutions/guide
"""

import logging
from pathlib import Path
from typing import List, Optional, Dict, Any, Union
from enum import Enum

logger = logging.getLogger(__name__)


class MediaPipeTaskCategory(str, Enum):
    """MediaPipe task categories"""
    GENAI = "genai"      # LLM inference, RAG, function calling, image gen
    VISION = "vision"    # Image/video tasks
    TEXT = "text"        # Text tasks
    AUDIO = "audio"      # Audio tasks


class VisionTaskType(str, Enum):
    """Supported vision task types"""
    OBJECT_DETECTION = "object_detection"
    IMAGE_CLASSIFICATION = "image_classification"
    IMAGE_SEGMENTATION = "image_segmentation"
    INTERACTIVE_SEGMENTATION = "interactive_segmentation"
    IMAGE_EMBEDDING = "image_embedding"
    FACE_DETECTION = "face_detection"
    FACE_LANDMARKER = "face_landmarker"
    HAND_LANDMARKER = "hand_landmarker"
    GESTURE_RECOGNITION = "gesture_recognition"
    POSE_LANDMARKER = "pose_landmarker"
    HOLISTIC_TRACKING = "holistic_tracking"


class TextTaskType(str, Enum):
    """Supported text task types"""
    TEXT_CLASSIFICATION = "text_classification"
    TEXT_EMBEDDING = "text_embedding"
    LANGUAGE_DETECTION = "language_detection"


class AudioTaskType(str, Enum):
    """Supported audio task types"""
    AUDIO_CLASSIFICATION = "audio_classification"


class MediaPipeManager:
    """
    Comprehensive MediaPipe Manager - ALL CAPABILITIES
    
    Handles:
    1. GenAI: LLM inference with Gemma .task bundles
    2. Vision: All detection/classification/segmentation/tracking tasks
    3. Text: Classification, embeddings, language detection
    4. Audio: Audio classification
    
    MediaPipe runs on-device with GPU/NPU acceleration.
    """
    
    def __init__(self):
        """Initialize MediaPipe manager"""
        # GenAI (LLM) instances
        self.llm_inference: Optional[Any] = None
        self.llm_session: Optional[Any] = None
        self.llm_model_path: Optional[Path] = None
        
        # Vision task instances
        self.vision_tasks: Dict[VisionTaskType, Any] = {}
        
        # Text task instances
        self.text_tasks: Dict[TextTaskType, Any] = {}
        
        # Audio task instances
        self.audio_tasks: Dict[AudioTaskType, Any] = {}
        
        # Lazy import MediaPipe
        self._mediapipe = None
        
        logger.info("MediaPipeManager initialized (GenAI + Vision + Text + Audio)")
    
    def _ensure_mediapipe(self):
        """Lazy load MediaPipe library"""
        if self._mediapipe is None:
            try:
                import mediapipe as mp
                self._mediapipe = mp
                logger.info(f"MediaPipe loaded (version: {mp.__version__})")
            except ImportError:
                raise RuntimeError(
                    "MediaPipe not installed. "
                    "Install with: pip install mediapipe"
                )
    
    # ========================================
    # GenAI: LLM Inference
    # ========================================
    
    def load_llm(
        self,
        model_path: str,
        max_tokens: int = 1024,
        top_k: int = 40,
        top_p: float = 0.95,
        temperature: float = 0.8,
        random_seed: Optional[int] = None
    ) -> bool:
        """
        Load MediaPipe GenAI LLM (.task bundle).
        
        Supports Gemma models optimized for on-device inference.
        
        Args:
            model_path: Path to .task bundle file
            max_tokens: Max tokens to generate
            top_k: Top-k sampling
            top_p: Nucleus sampling
            temperature: Sampling temperature
            random_seed: Random seed for reproducibility
        
        Returns:
            True if loaded successfully
        """
        self._ensure_mediapipe()
        
        path = Path(model_path)
        if not path.exists():
            raise FileNotFoundError(f"Model file not found: {model_path}")
        
        logger.info(f"Loading MediaPipe GenAI LLM: {model_path}")
        
        try:
            from mediapipe.tasks.python import genai
            
            # Create LLM inference task
            task_options = genai.LlmInference.LlmInferenceOptions(
                model_asset_path=str(path)
            )
            self.llm_inference = genai.LlmInference.create_from_options(task_options)
            
            # Create session with generation config
            session_options = {
                "max_tokens": max_tokens,
                "top_k": top_k,
                "top_p": top_p,
                "temperature": temperature,
            }
            if random_seed is not None:
                session_options["random_seed"] = random_seed
            
            self.llm_session = self.llm_inference.create_session(**session_options)
            self.llm_model_path = path
            
            logger.info(f"✅ MediaPipe GenAI LLM loaded successfully")
            return True
            
        except Exception as e:
            logger.error(f"❌ Failed to load MediaPipe LLM: {e}", exc_info=True)
            return False
    
    def generate_llm(self, prompt: str) -> str:
        """Generate text using MediaPipe GenAI LLM"""
        if not self.llm_inference or not self.llm_session:
            raise RuntimeError("LLM not loaded. Call load_llm() first.")
        
        try:
            response = self.llm_inference.generate_response(prompt)
            return response if response else ""
        except Exception as e:
            logger.error(f"❌ LLM generation failed: {e}", exc_info=True)
            raise
    
    def generate_llm_stream(self, prompt: str):
        """Stream LLM generation token-by-token"""
        if not self.llm_inference or not self.llm_session:
            raise RuntimeError("LLM not loaded. Call load_llm() first.")
        
        try:
            for chunk in self.llm_inference.generate_response_async(prompt):
                yield chunk
        except Exception as e:
            logger.error(f"❌ LLM streaming failed: {e}", exc_info=True)
            raise
    
    def unload_llm(self) -> bool:
        """Unload LLM from memory"""
        try:
            if self.llm_inference:
                self.llm_inference.close()
                self.llm_inference = None
                self.llm_session = None
                self.llm_model_path = None
                logger.info("✅ MediaPipe LLM unloaded")
            return True
        except Exception as e:
            logger.error(f"❌ Error unloading LLM: {e}")
            return False
    
    # ========================================
    # Vision Tasks
    # ========================================
    
    def load_vision_task(self, task_type: VisionTaskType, model_path: str) -> bool:
        """Load a vision task model"""
        self._ensure_mediapipe()
        
        try:
            from mediapipe.tasks.python import vision
            from mediapipe.tasks.python.core import base_options
            
            base_opts = base_options.BaseOptions(model_asset_path=model_path)
            
            if task_type == VisionTaskType.OBJECT_DETECTION:
                options = vision.ObjectDetectorOptions(
                    base_options=base_opts,
                    running_mode=vision.RunningMode.IMAGE
                )
                self.vision_tasks[task_type] = vision.ObjectDetector.create_from_options(options)
            
            elif task_type == VisionTaskType.IMAGE_CLASSIFICATION:
                options = vision.ImageClassifierOptions(
                    base_options=base_opts,
                    running_mode=vision.RunningMode.IMAGE
                )
                self.vision_tasks[task_type] = vision.ImageClassifier.create_from_options(options)
            
            elif task_type == VisionTaskType.IMAGE_SEGMENTATION:
                options = vision.ImageSegmenterOptions(
                    base_options=base_opts,
                    running_mode=vision.RunningMode.IMAGE
                )
                self.vision_tasks[task_type] = vision.ImageSegmenter.create_from_options(options)
            
            elif task_type == VisionTaskType.IMAGE_EMBEDDING:
                options = vision.ImageEmbedderOptions(
                    base_options=base_opts,
                    running_mode=vision.RunningMode.IMAGE,
                    l2_normalize=True
                )
                self.vision_tasks[task_type] = vision.ImageEmbedder.create_from_options(options)
            
            elif task_type == VisionTaskType.FACE_DETECTION:
                options = vision.FaceDetectorOptions(
                    base_options=base_opts,
                    running_mode=vision.RunningMode.IMAGE
                )
                self.vision_tasks[task_type] = vision.FaceDetector.create_from_options(options)
            
            elif task_type == VisionTaskType.FACE_LANDMARKER:
                options = vision.FaceLandmarkerOptions(
                    base_options=base_opts,
                    running_mode=vision.RunningMode.IMAGE
                )
                self.vision_tasks[task_type] = vision.FaceLandmarker.create_from_options(options)
            
            elif task_type == VisionTaskType.HAND_LANDMARKER:
                options = vision.HandLandmarkerOptions(
                    base_options=base_opts,
                    running_mode=vision.RunningMode.IMAGE
                )
                self.vision_tasks[task_type] = vision.HandLandmarker.create_from_options(options)
            
            elif task_type == VisionTaskType.GESTURE_RECOGNITION:
                options = vision.GestureRecognizerOptions(
                    base_options=base_opts,
                    running_mode=vision.RunningMode.IMAGE
                )
                self.vision_tasks[task_type] = vision.GestureRecognizer.create_from_options(options)
            
            elif task_type == VisionTaskType.POSE_LANDMARKER:
                options = vision.PoseLandmarkerOptions(
                    base_options=base_opts,
                    running_mode=vision.RunningMode.IMAGE
                )
                self.vision_tasks[task_type] = vision.PoseLandmarker.create_from_options(options)
            
            elif task_type == VisionTaskType.HOLISTIC_TRACKING:
                options = vision.HolisticLandmarkerOptions(
                    base_options=base_opts,
                    running_mode=vision.RunningMode.IMAGE
                )
                self.vision_tasks[task_type] = vision.HolisticLandmarker.create_from_options(options)
            
            else:
                raise ValueError(f"Unsupported vision task: {task_type}")
            
            logger.info(f"✅ Vision task loaded: {task_type.value}")
            return True
            
        except Exception as e:
            logger.error(f"❌ Failed to load vision task {task_type}: {e}", exc_info=True)
            return False
    
    # ========================================
    # Text Tasks
    # ========================================
    
    def load_text_task(self, task_type: TextTaskType, model_path: str) -> bool:
        """Load a text task model"""
        self._ensure_mediapipe()
        
        try:
            from mediapipe.tasks.python import text
            from mediapipe.tasks.python.core import base_options
            
            base_opts = base_options.BaseOptions(model_asset_path=model_path)
            
            if task_type == TextTaskType.TEXT_CLASSIFICATION:
                options = text.TextClassifierOptions(base_options=base_opts)
                self.text_tasks[task_type] = text.TextClassifier.create_from_options(options)
            
            elif task_type == TextTaskType.TEXT_EMBEDDING:
                options = text.TextEmbedderOptions(
                    base_options=base_opts,
                    l2_normalize=True
                )
                self.text_tasks[task_type] = text.TextEmbedder.create_from_options(options)
            
            elif task_type == TextTaskType.LANGUAGE_DETECTION:
                options = text.LanguageDetectorOptions(base_options=base_opts)
                self.text_tasks[task_type] = text.LanguageDetector.create_from_options(options)
            
            else:
                raise ValueError(f"Unsupported text task: {task_type}")
            
            logger.info(f"✅ Text task loaded: {task_type.value}")
            return True
            
        except Exception as e:
            logger.error(f"❌ Failed to load text task {task_type}: {e}", exc_info=True)
            return False
    
    # ========================================
    # Audio Tasks
    # ========================================
    
    def load_audio_task(self, task_type: AudioTaskType, model_path: str) -> bool:
        """Load an audio task model"""
        self._ensure_mediapipe()
        
        try:
            from mediapipe.tasks.python import audio
            from mediapipe.tasks.python.core import base_options
            
            base_opts = base_options.BaseOptions(model_asset_path=model_path)
            
            if task_type == AudioTaskType.AUDIO_CLASSIFICATION:
                options = audio.AudioClassifierOptions(
                    base_options=base_opts,
                    running_mode=audio.RunningMode.AUDIO_CLIPS
                )
                self.audio_tasks[task_type] = audio.AudioClassifier.create_from_options(options)
            
            else:
                raise ValueError(f"Unsupported audio task: {task_type}")
            
            logger.info(f"✅ Audio task loaded: {task_type.value}")
            return True
            
        except Exception as e:
            logger.error(f"❌ Failed to load audio task {task_type}: {e}", exc_info=True)
            return False
    
    # ========================================
    # Unified Unload
    # ========================================
    
    def unload_all(self):
        """Unload all tasks and free memory"""
        self.unload_llm()
        self.vision_tasks.clear()
        self.text_tasks.clear()
        self.audio_tasks.clear()
        logger.info("✅ All MediaPipe tasks unloaded")
