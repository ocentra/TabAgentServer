"""
MediaPipe Inference Backend Manager.

Comprehensive on-device AI using Google MediaPipe framework.

Supports:
- GenAI: LLM inference (Gemma models) in .task bundles
- Vision: Image classification, embeddings, object detection, face/hand/pose tracking
- Text: Classification, embeddings, language detection  
- Audio: Audio classification

TabAgent is more than chatbot - full multimodal AI agent.
"""

import logging
from pathlib import Path
from typing import List, Optional, Dict, Any, Union
from enum import Enum
from dataclasses import dataclass

from Python.core.message_types import (
    ChatMessage,
    InferenceSettings,
    BackendType,
    AccelerationBackend,
)

from .config import (
    MediaPipeConfig,
    MediaPipeDelegate,
)
from Python.core.performance_tracker import PerformanceTracker


logger = logging.getLogger(__name__)


class MediaPipeStatus(str, Enum):
    """MediaPipe task status"""
    NOT_LOADED = "not_loaded"
    LOADING = "loading"
    READY = "ready"
    ERROR = "error"


class MediaPipeTaskCategory(str, Enum):
    """MediaPipe task categories"""
    GENAI = "genai"      # LLM inference
    VISION = "vision"    # Image/video tasks
    TEXT = "text"        # Text tasks
    AUDIO = "audio"      # Audio tasks


class VisionTaskType(str, Enum):
    """Supported vision task types"""
    IMAGE_CLASSIFICATION = "image_classification"
    IMAGE_EMBEDDING = "image_embedding"
    OBJECT_DETECTION = "object_detection"
    FACE_DETECTION = "face_detection"
    FACE_LANDMARKER = "face_landmarker"
    HAND_LANDMARKER = "hand_landmarker"
    POSE_LANDMARKER = "pose_landmarker"
    GESTURE_RECOGNITION = "gesture_recognition"
    IMAGE_SEGMENTATION = "image_segmentation"
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
    Comprehensive MediaPipe Inference Backend Manager.
    
    Supports multimodal on-device AI (TabAgent is more than chatbot):
    - GenAI: LLM inference (Gemma models)
    - Vision: Classification, detection, tracking, segmentation, embeddings
    - Text: Classification, embeddings, language detection
    - Audio: Classification
    
    Uses MediaPipe .task bundles with GPU/NPU acceleration.
    """
    
    def __init__(self):
        """Initialize MediaPipe manager"""
        # GenAI (LLM) instances
        self.llm_inference: Optional[Any] = None
        self.llm_session: Optional[Any] = None
        
        # Vision task instances
        self.vision_tasks: Dict[VisionTaskType, Any] = {}
        
        # Text task instances
        self.text_tasks: Dict[TextTaskType, Any] = {}
        
        # Audio task instances
        self.audio_tasks: Dict[AudioTaskType, Any] = {}
        
        # Common state
        self.model_path: Optional[Path] = None
        self.config: Optional[MediaPipeConfig] = None
        self.status: MediaPipeStatus = MediaPipeStatus.NOT_LOADED
        self.backend_type: Optional[BackendType] = None
        self.task_category: Optional[MediaPipeTaskCategory] = None
        self._performance_tracker = PerformanceTracker()
        
        # Lazy import MediaPipe
        self._mediapipe = None
        
        logger.info("MediaPipeManager initialized (multimodal: vision, text, audio, genai)")
    
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
    
    def is_model_loaded(self) -> bool:
        """
        Check if a model is currently loaded.
        
        Returns:
            True if model is loaded and ready
        """
        return (
            self.status == MediaPipeStatus.READY and
            self.llm_inference is not None
        )
    
    def load_model(
        self,
        model_path: str,
        delegate: MediaPipeDelegate = MediaPipeDelegate.CPU,
        config: Optional[MediaPipeConfig] = None
    ) -> bool:
        """
        Load MediaPipe .task bundle model.
        
        Args:
            model_path: Path to .task bundle file
            delegate: Inference delegate (CPU/GPU/NPU)
            config: MediaPipe configuration (optional)
            
        Returns:
            True if model loaded successfully
            
        Raises:
            FileNotFoundError: If model file not found
            RuntimeError: If MediaPipe not available
        """
        self._ensure_mediapipe()
        
        path = Path(model_path)
        if not path.exists():
            raise FileNotFoundError(f"Model file not found: {model_path}")
        
        if not path.suffix == '.task':
            logger.warning(f"Model file is not .task bundle: {model_path}")
        
        # Use provided config or create default
        if config is None:
            config = MediaPipeConfig(
                model_path=model_path,
                delegate=delegate
            )
        
        logger.info(
            f"Loading MediaPipe model: {model_path} "
            f"(delegate: {config.delegate.value})"
        )
        
        self.status = MediaPipeStatus.LOADING
        
        try:
            # Import MediaPipe LLM task
            from mediapipe.tasks.python import genai
            
            # Create task options
            task_options = genai.LlmInference.LlmInferenceOptions(
                model_asset_path=str(path)
            )
            
            # Create LLM Inference task
            self.llm_inference = genai.LlmInference.create_from_options(task_options)
            
            # Create session options
            session_options = {
                "max_tokens": config.max_tokens,
                "top_k": config.top_k,
                "top_p": config.top_p,
                "temperature": config.temperature,
            }
            
            if config.random_seed is not None:
                session_options["random_seed"] = config.random_seed
            
            # Create inference session
            self.llm_session = self.llm_inference.create_session(**session_options)
            
            self.model_path = path
            self.config = config
            self.backend_type = self._map_backend_type(delegate)
            self.status = MediaPipeStatus.READY
            
            logger.info(f"MediaPipe model loaded successfully")
            return True
            
        except Exception as e:
            logger.error(f"Failed to load MediaPipe model: {e}")
            self.status = MediaPipeStatus.ERROR
            self.llm_inference = None
            self.llm_session = None
            return False
    
    def unload_model(self) -> bool:
        """
        Unload current model.
        
        Returns:
            True if unloaded successfully
        """
        if self.llm_session is None and self.llm_inference is None:
            logger.info("No model loaded, nothing to unload")
            return True
        
        try:
            # Release task
            if self.llm_inference is not None:
                # Close the LLM inference instance
                self.llm_inference.close()
                self.llm_inference = None
            
            self.model_path = None
            self.config = None
            self.backend_type = None
            self.status = MediaPipeStatus.NOT_LOADED
            
            logger.info("MediaPipe model unloaded")
            return True
            
        except Exception as e:
            logger.error(f"Error unloading model: {e}")
            return False
    
    async def generate(
        self,
        messages: List[ChatMessage],
        settings: InferenceSettings
    ) -> str:
        """
        Generate text using MediaPipe LLM.
        
        Args:
            messages: Chat messages
            settings: Inference settings
            
        Returns:
            Generated text
            
        Raises:
            RuntimeError: If no model loaded
        """
        if not self.is_model_loaded():
            raise RuntimeError("No model loaded or not ready")
        
        try:
            # Build prompt from messages
            prompt = self._build_prompt(messages)
            
            # Generate response using MediaPipe
            response = self.llm_inference.generate_response(prompt)
            
            if response:
                logger.info(f"Generated {len(response)} characters")
                return response
            else:
                logger.error("Empty response from MediaPipe")
                return ""
                
        except Exception as e:
            logger.error(f"Generation failed: {e}")
            raise RuntimeError(f"MediaPipe generation failed: {e}")
    
    async def generate_stream(
        self,
        messages: List[ChatMessage],
        settings: InferenceSettings
    ):
        """
        Generate text with streaming output.
        
        Args:
            messages: Chat messages
            settings: Inference settings
            
        Yields:
            Generated tokens
            
        Raises:
            RuntimeError: If no model loaded
        """
        if not self.is_model_loaded():
            raise RuntimeError("No model loaded or not ready")
        
        try:
            # Build prompt
            prompt = self._build_prompt(messages)
            
            # MediaPipe generates in chunks via generator
            for response_chunk in self.llm_inference.generate_response_async(prompt):
                yield response_chunk
            
        except Exception as e:
            logger.error(f"Streaming generation failed: {e}")
            raise RuntimeError(f"MediaPipe streaming failed: {e}")
    
    def get_model_info(self) -> Dict[str, Any]:
        """
        Get information about loaded model.
        
        Returns:
            Dictionary with model information
        """
        if not self.is_model_loaded():
            return {
                "loaded": False,
                "status": self.status.value,
                "error": "No model loaded"
            }
        
        return {
            "loaded": True,
            "status": self.status.value,
            "model_path": str(self.model_path),
            "backend": self.backend_type.value if self.backend_type else None,
            "delegate": self.config.delegate.value if self.config else None,
            "max_tokens": self.config.max_tokens if self.config else None,
        }
    
    @staticmethod
    def _build_prompt(messages: List[ChatMessage]) -> str:
        """
        Build prompt string from chat messages.
        
        Uses Gemma chat format (most common for MediaPipe .task models).
        
        Args:
            messages: List of chat messages
            
        Returns:
            Formatted prompt string for Gemma models
        """
        # Gemma chat template format
        # Format: <start_of_turn>role\ncontent<end_of_turn>\n
        prompt_parts = []
        
        for msg in messages:
            role = msg.role.value
            content = msg.content
            
            # Gemma format
            prompt_parts.append(f"<start_of_turn>{role}\n{content}<end_of_turn>")
        
        # Add model turn marker
        prompt_parts.append("<start_of_turn>model\n")
        
        return "\n".join(prompt_parts)
    
    @staticmethod
    def _map_backend_type(delegate: MediaPipeDelegate) -> BackendType:
        """
        Map MediaPipe delegate to BackendType.
        
        Args:
            delegate: MediaPipe delegate
            
        Returns:
            BackendType enum value
        """
        mapping = {
            MediaPipeDelegate.CPU: BackendType.MEDIAPIPE_CPU,
            MediaPipeDelegate.GPU: BackendType.MEDIAPIPE_GPU,
            MediaPipeDelegate.NPU: BackendType.MEDIAPIPE_NPU,
        }
        
        return mapping.get(delegate, BackendType.MEDIAPIPE_CPU)
    
    # Vision Tasks (TabAgent multimodal capabilities)
    
    async def classify_image(self, image_path: str, task_model_path: str) -> Dict[str, Any]:
        """
        Classify image using MediaPipe image classifier.
        
        Args:
            image_path: Path to image file
            task_model_path: Path to image classifier .task bundle
            
        Returns:
            Classification results with categories and scores
        """
        self._ensure_mediapipe()
        
        try:
            from mediapipe.tasks.python import vision
            from mediapipe.tasks.python.core import base_options
            
            # Load classifier if not loaded
            if VisionTaskType.IMAGE_CLASSIFICATION not in self.vision_tasks:
                options = vision.ImageClassifierOptions(
                    base_options=base_options.BaseOptions(model_asset_path=task_model_path),
                    running_mode=vision.RunningMode.IMAGE
                )
                classifier = vision.ImageClassifier.create_from_options(options)
                self.vision_tasks[VisionTaskType.IMAGE_CLASSIFICATION] = classifier
            
            # Load and classify image
            image = self._mediapipe.Image.create_from_file(image_path)
            result = self.vision_tasks[VisionTaskType.IMAGE_CLASSIFICATION].classify(image)
            
            return {
                "task": "image_classification",
                "classifications": [
                    {
                        "category": cat.category_name,
                        "score": cat.score,
                        "index": cat.index
                    }
                    for classification in result.classifications
                    for cat in classification.categories
                ]
            }
        
        except Exception as e:
            logger.error(f"Image classification failed: {e}")
            raise
    
    async def generate_image_embeddings(self, image_path: str, task_model_path: str) -> List[float]:
        """
        Generate embeddings for image using MediaPipe.
        
        Args:
            image_path: Path to image file
            task_model_path: Path to image embedder .task bundle
            
        Returns:
            Embedding vector
        """
        self._ensure_mediapipe()
        
        try:
            from mediapipe.tasks.python import vision
            from mediapipe.tasks.python.core import base_options
            
            # Load embedder if not loaded
            if VisionTaskType.IMAGE_EMBEDDING not in self.vision_tasks:
                options = vision.ImageEmbedderOptions(
                    base_options=base_options.BaseOptions(model_asset_path=task_model_path),
                    running_mode=vision.RunningMode.IMAGE,
                    l2_normalize=True
                )
                embedder = vision.ImageEmbedder.create_from_options(options)
                self.vision_tasks[VisionTaskType.IMAGE_EMBEDDING] = embedder
            
            # Load and embed image
            image = self._mediapipe.Image.create_from_file(image_path)
            result = self.vision_tasks[VisionTaskType.IMAGE_EMBEDDING].embed(image)
            
            # Extract embedding vector
            if result.embeddings and len(result.embeddings) > 0:
                return result.embeddings[0].embedding.tolist()
            
            return []
        
        except Exception as e:
            logger.error(f"Image embedding failed: {e}")
            raise
    
    async def detect_objects(self, image_path: str, task_model_path: str) -> Dict[str, Any]:
        """
        Detect objects in image using MediaPipe.
        
        Args:
            image_path: Path to image file
            task_model_path: Path to object detector .task bundle
            
        Returns:
            Detection results with bounding boxes and labels
        """
        self._ensure_mediapipe()
        
        try:
            from mediapipe.tasks.python import vision
            from mediapipe.tasks.python.core import base_options
            
            # Load detector if not loaded
            if VisionTaskType.OBJECT_DETECTION not in self.vision_tasks:
                options = vision.ObjectDetectorOptions(
                    base_options=base_options.BaseOptions(model_asset_path=task_model_path),
                    running_mode=vision.RunningMode.IMAGE
                )
                detector = vision.ObjectDetector.create_from_options(options)
                self.vision_tasks[VisionTaskType.OBJECT_DETECTION] = detector
            
            # Load and detect objects
            image = self._mediapipe.Image.create_from_file(image_path)
            result = self.vision_tasks[VisionTaskType.OBJECT_DETECTION].detect(image)
            
            return {
                "task": "object_detection",
                "detections": [
                    {
                        "category": det.categories[0].category_name if det.categories else None,
                        "score": det.categories[0].score if det.categories else 0.0,
                        "bounding_box": {
                            "x": det.bounding_box.origin_x,
                            "y": det.bounding_box.origin_y,
                            "width": det.bounding_box.width,
                            "height": det.bounding_box.height
                        }
                    }
                    for det in result.detections
                ]
            }
        
        except Exception as e:
            logger.error(f"Object detection failed: {e}")
            raise
    
    async def generate_text_embeddings(self, text: str, task_model_path: str) -> List[float]:
        """
        Generate embeddings for text using MediaPipe.
        
        Args:
            text: Input text
            task_model_path: Path to text embedder .task bundle
            
        Returns:
            Embedding vector
        """
        self._ensure_mediapipe()
        
        try:
            from mediapipe.tasks.python import text
            from mediapipe.tasks.python.core import base_options
            
            # Load embedder if not loaded
            if TextTaskType.TEXT_EMBEDDING not in self.text_tasks:
                options = text.TextEmbedderOptions(
                    base_options=base_options.BaseOptions(model_asset_path=task_model_path),
                    l2_normalize=True
                )
                embedder = text.TextEmbedder.create_from_options(options)
                self.text_tasks[TextTaskType.TEXT_EMBEDDING] = embedder
            
            # Generate embedding
            result = self.text_tasks[TextTaskType.TEXT_EMBEDDING].embed(text)
            
            # Extract embedding vector
            if result.embeddings and len(result.embeddings) > 0:
                return result.embeddings[0].embedding.tolist()
            
            return []
        
        except Exception as e:
            logger.error(f"Text embedding failed: {e}")
            raise
    
    def get_state(self) -> Dict[str, Any]:
        """
        Get current manager state.
        
        Returns:
            Dictionary with state information including performance metrics
        """
        state = {
            "isReady": self.status == MediaPipeStatus.LOADED,
            "backend": self.backend_type.value if self.backend_type else None,
            "modelPath": str(self.model_path) if self.model_path else None,
            "taskCategory": self.task_category.value if self.task_category else None,
            "status": self.status.value
        }
        
        # Add performance metrics
        stats = self._performance_tracker.get_current_stats()
        state.update(stats)
        
        return state

