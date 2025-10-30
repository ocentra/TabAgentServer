"""
Iris Tracking - Real-time eye and iris landmark detection

MediaPipe Iris (part of Face Mesh with refinement).
Reference: https://ai.google.dev/edge/mediapipe/solutions/vision/face_landmarker
"""

import logging
import numpy as np
from typing import Optional, AsyncGenerator

logger = logging.getLogger(__name__)


class IrisTracker:
    """
    MediaPipe Iris Tracking.
    
    Provides detailed eye tracking with iris landmarks:
    - Left eye region: 71 landmarks
    - Right eye region: 71 landmarks
    - Left iris: 5 landmarks
    - Right iris: 5 landmarks
    
    Useful for:
    - Gaze estimation
    - Eye movement tracking
    - Pupil dilation detection
    - AR face filters
    """
    
    def __init__(
        self,
        max_num_faces: int = 1,
        min_detection_confidence: float = 0.5,
        min_tracking_confidence: float = 0.5
    ):
        """
        Initialize iris tracker.
        
        Args:
            max_num_faces: Maximum number of faces to track
            min_detection_confidence: Minimum confidence for detection
            min_tracking_confidence: Minimum confidence for tracking
        """
        self.max_num_faces = max_num_faces
        self.min_detection_confidence = min_detection_confidence
        self.min_tracking_confidence = min_tracking_confidence
        self._face_mesh = None
        self._mp = None
        
        logger.info("IrisTracker initialized")
    
    def _ensure_initialized(self):
        """Lazy initialization of MediaPipe"""
        if self._face_mesh is None:
            import mediapipe as mp
            self._mp = mp
            # Iris tracking requires refine_landmarks=True
            self._face_mesh = mp.solutions.face_mesh.FaceMesh(
                static_image_mode=False,
                max_num_faces=self.max_num_faces,
                refine_landmarks=True,  # Enables 468 → 478 landmarks (adds iris)
                min_detection_confidence=self.min_detection_confidence,
                min_tracking_confidence=self.min_tracking_confidence
            )
            logger.info("MediaPipe Face Mesh (with iris) loaded")
    
    def track_single(self, image: np.ndarray) -> list:
        """
        Track iris in a single image.
        
        Args:
            image: RGB image as numpy array (H, W, 3)
        
        Returns:
            List of eye/iris detections with format:
            {
                'eye': 'left' | 'right',
                'iris_landmarks': [5 iris landmarks],
                'eye_landmarks': [eye region landmarks],
                'confidence': float,
                'gaze_direction': {'x': float, 'y': float}
            }
        """
        self._ensure_initialized()
        
        results = self._face_mesh.process(image)
        
        eyes = []
        if results.multi_face_landmarks:
            for face_landmarks in results.multi_face_landmarks:
                all_landmarks = face_landmarks.landmark
                
                # Only 478-landmark meshes have iris data
                if len(all_landmarks) < 478:
                    continue
                
                # Left iris: indices 468-472
                left_iris = []
                for i in range(468, 473):
                    lm = all_landmarks[i]
                    left_iris.append({
                        'x': lm.x,
                        'y': lm.y,
                        'z': lm.z
                    })
                
                # Right iris: indices 473-477
                right_iris = []
                for i in range(473, 478):
                    lm = all_landmarks[i]
                    right_iris.append({
                        'x': lm.x,
                        'y': lm.y,
                        'z': lm.z
                    })
                
                # Left eye region landmarks (approximate)
                left_eye_indices = [
                    33, 7, 163, 144, 145, 153, 154, 155, 133,  # Upper contour
                    173, 157, 158, 159, 160, 161, 246,  # Lower contour
                    130, 25, 110, 24, 23, 22, 26, 112, 243  # Eye corners and surrounds
                ]
                left_eye_landmarks = []
                for idx in left_eye_indices:
                    if idx < len(all_landmarks):
                        lm = all_landmarks[idx]
                        left_eye_landmarks.append({
                            'x': lm.x,
                            'y': lm.y,
                            'z': lm.z
                        })
                
                # Right eye region landmarks (approximate)
                right_eye_indices = [
                    362, 382, 381, 380, 374, 373, 390, 249, 263,  # Upper contour
                    466, 388, 387, 386, 385, 384, 398,  # Lower contour
                    359, 255, 339, 254, 253, 252, 256, 341, 463  # Eye corners and surrounds
                ]
                right_eye_landmarks = []
                for idx in right_eye_indices:
                    if idx < len(all_landmarks):
                        lm = all_landmarks[idx]
                        right_eye_landmarks.append({
                            'x': lm.x,
                            'y': lm.y,
                            'z': lm.z
                        })
                
                # Calculate gaze direction from iris center
                left_gaze = self._calculate_gaze(left_iris, left_eye_landmarks)
                right_gaze = self._calculate_gaze(right_iris, right_eye_landmarks)
                
                eyes.append({
                    'eye': 'left',
                    'iris_landmarks': left_iris,
                    'eye_landmarks': left_eye_landmarks,
                    'confidence': 1.0,
                    'gaze_direction': left_gaze
                })
                
                eyes.append({
                    'eye': 'right',
                    'iris_landmarks': right_iris,
                    'eye_landmarks': right_eye_landmarks,
                    'confidence': 1.0,
                    'gaze_direction': right_gaze
                })
        
        return eyes
    
    async def track_stream(self, frames: AsyncGenerator[np.ndarray, None]) -> AsyncGenerator[list, None]:
        """
        Track iris in video stream.
        
        Args:
            frames: Async generator yielding RGB frames
        
        Yields:
            List of eye/iris detections per frame
        """
        self._ensure_initialized()
        
        async for frame in frames:
            eyes = self.track_single(frame)
            yield eyes
    
    def _calculate_gaze(self, iris_landmarks: list, eye_landmarks: list) -> dict:
        """
        Calculate gaze direction from iris and eye landmarks.
        
        Args:
            iris_landmarks: 5 iris landmarks
            eye_landmarks: Eye region landmarks
        
        Returns:
            {'x': float, 'y': float} - Normalized gaze direction [-1, 1]
        """
        if not iris_landmarks or len(iris_landmarks) < 5:
            return {'x': 0.0, 'y': 0.0}
        
        # Iris center is the first landmark
        iris_center = iris_landmarks[0]
        
        # Calculate eye bounding box
        if not eye_landmarks:
            return {'x': 0.0, 'y': 0.0}
        
        eye_xs = [lm['x'] for lm in eye_landmarks]
        eye_ys = [lm['y'] for lm in eye_landmarks]
        
        eye_min_x = min(eye_xs)
        eye_max_x = max(eye_xs)
        eye_min_y = min(eye_ys)
        eye_max_y = max(eye_ys)
        
        eye_center_x = (eye_min_x + eye_max_x) / 2
        eye_center_y = (eye_min_y + eye_max_y) / 2
        
        eye_width = eye_max_x - eye_min_x
        eye_height = eye_max_y - eye_min_y
        
        # Normalize iris position relative to eye center
        gaze_x = (iris_center['x'] - eye_center_x) / (eye_width / 2) if eye_width > 0 else 0.0
        gaze_y = (iris_center['y'] - eye_center_y) / (eye_height / 2) if eye_height > 0 else 0.0
        
        # Clamp to [-1, 1]
        gaze_x = max(-1.0, min(1.0, gaze_x))
        gaze_y = max(-1.0, min(1.0, gaze_y))
        
        return {'x': gaze_x, 'y': gaze_y}
    
    def close(self):
        """Release resources"""
        if self._face_mesh:
            self._face_mesh.close()
            self._face_mesh = None
            logger.info("IrisTracker closed")

