"""
Face Mesh - Real-time 468-point 3D face landmarks

MediaPipe Face Mesh with iris refinement.
Reference: https://ai.google.dev/edge/mediapipe/solutions/vision/face_landmarker
"""

import logging
import numpy as np
from typing import Optional, AsyncGenerator

logger = logging.getLogger(__name__)


class FaceMesh:
    """
    MediaPipe Face Mesh.
    
    Provides 468 3D landmarks for detailed face tracking:
    - Face contours
    - Eyes (with optional iris refinement)
    - Eyebrows
    - Nose
    - Mouth
    - Face oval
    """
    
    def __init__(
        self,
        max_num_faces: int = 2,
        refine_landmarks: bool = True,
        min_detection_confidence: float = 0.5,
        min_tracking_confidence: float = 0.5
    ):
        """
        Initialize face mesh.
        
        Args:
            max_num_faces: Maximum number of faces to detect
            refine_landmarks: Include iris landmarks (468 → 478 landmarks)
            min_detection_confidence: Minimum confidence for detection
            min_tracking_confidence: Minimum confidence for tracking
        """
        self.max_num_faces = max_num_faces
        self.refine_landmarks = refine_landmarks
        self.min_detection_confidence = min_detection_confidence
        self.min_tracking_confidence = min_tracking_confidence
        self._mesh = None
        self._mp = None
        
        logger.info(f"FaceMesh initialized (max_faces={max_num_faces}, refine={refine_landmarks})")
    
    def _ensure_initialized(self):
        """Lazy initialization of MediaPipe"""
        if self._mesh is None:
            import mediapipe as mp
            self._mp = mp
            self._mesh = mp.solutions.face_mesh.FaceMesh(
                static_image_mode=False,
                max_num_faces=self.max_num_faces,
                refine_landmarks=self.refine_landmarks,
                min_detection_confidence=self.min_detection_confidence,
                min_tracking_confidence=self.min_tracking_confidence
            )
            logger.info("MediaPipe Face Mesh loaded")
    
    def process_single(self, image: np.ndarray) -> list:
        """
        Process a single image for face mesh.
        
        Args:
            image: RGB image as numpy array (H, W, 3)
        
        Returns:
            List of face meshes with format:
            {
                'landmarks': [{'x': float, 'y': float, 'z': float, 'visibility': float, 'presence': float}, ...],
                'confidence': float
            }
        """
        self._ensure_initialized()
        
        results = self._mesh.process(image)
        
        faces = []
        if results.multi_face_landmarks:
            for face_landmarks in results.multi_face_landmarks:
                landmarks = []
                for lm in face_landmarks.landmark:
                    landmarks.append({
                        'x': lm.x,
                        'y': lm.y,
                        'z': lm.z,
                        'visibility': getattr(lm, 'visibility', 1.0),
                        'presence': getattr(lm, 'presence', 1.0)
                    })
                
                faces.append({
                    'landmarks': landmarks,
                    'confidence': 1.0
                })
        
        return faces
    
    async def process_stream(self, frames: AsyncGenerator[np.ndarray, None]) -> AsyncGenerator[list, None]:
        """
        Process video stream for face mesh.
        
        Args:
            frames: Async generator yielding RGB frames
        
        Yields:
            List of face meshes per frame
        """
        self._ensure_initialized()
        
        async for frame in frames:
            faces = self.process_single(frame)
            yield faces
    
    def get_iris_landmarks(self, landmarks: list) -> dict:
        """
        Extract iris landmarks from full face mesh.
        
        Args:
            landmarks: Full 478-landmark list
        
        Returns:
            {
                'left_iris': [5 landmarks],
                'right_iris': [5 landmarks]
            }
        """
        if len(landmarks) < 478:
            return {'left_iris': [], 'right_iris': []}
        
        return {
            'left_iris': landmarks[468:473],   # Left iris: 468-472
            'right_iris': landmarks[473:478]   # Right iris: 473-477
        }
    
    def close(self):
        """Release resources"""
        if self._mesh:
            self._mesh.close()
            self._mesh = None
            logger.info("FaceMesh closed")

