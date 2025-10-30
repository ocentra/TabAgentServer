"""
Holistic Tracking - Combined face mesh + hands + pose

MediaPipe Holistic for full-body tracking with face details.
Reference: https://ai.google.dev/edge/mediapipe/solutions/vision/holistic_landmarker
"""

import logging
import numpy as np
from typing import Optional, AsyncGenerator

logger = logging.getLogger(__name__)


class HolisticTracker:
    """
    MediaPipe Holistic Tracking.
    
    Combines:
    - Face mesh (468 landmarks)
    - Pose (33 landmarks)
    - Left hand (21 landmarks)
    - Right hand (21 landmarks)
    
    Total: 543 landmarks in a single pass!
    """
    
    def __init__(
        self,
        model_complexity: int = 2,
        smooth_landmarks: bool = True,
        min_detection_confidence: float = 0.5,
        min_tracking_confidence: float = 0.5
    ):
        """
        Initialize holistic tracker.
        
        Args:
            model_complexity: 0 = lite, 1 = full, 2 = heavy
            smooth_landmarks: Enable temporal smoothing
            min_detection_confidence: Minimum confidence for detection
            min_tracking_confidence: Minimum confidence for tracking
        """
        self.model_complexity = model_complexity
        self.smooth_landmarks = smooth_landmarks
        self.min_detection_confidence = min_detection_confidence
        self.min_tracking_confidence = min_tracking_confidence
        self._holistic = None
        self._mp = None
        
        logger.info(f"HolisticTracker initialized (complexity={model_complexity})")
    
    def _ensure_initialized(self):
        """Lazy initialization of MediaPipe"""
        if self._holistic is None:
            import mediapipe as mp
            self._mp = mp
            self._holistic = mp.solutions.holistic.Holistic(
                static_image_mode=False,
                model_complexity=self.model_complexity,
                smooth_landmarks=self.smooth_landmarks,
                min_detection_confidence=self.min_detection_confidence,
                min_tracking_confidence=self.min_tracking_confidence
            )
            logger.info("MediaPipe Holistic loaded")
    
    def track_single(self, image: np.ndarray) -> dict:
        """
        Track all body parts in a single image.
        
        Args:
            image: RGB image as numpy array (H, W, 3)
        
        Returns:
            {
                'face': {...} or None,
                'pose': {...} or None,
                'left_hand': {...} or None,
                'right_hand': {...} or None
            }
        """
        self._ensure_initialized()
        
        results = self._holistic.process(image)
        
        # Face mesh (468 landmarks)
        face = None
        if results.face_landmarks:
            face_landmarks = []
            for lm in results.face_landmarks.landmark:
                face_landmarks.append({
                    'x': lm.x,
                    'y': lm.y,
                    'z': lm.z,
                    'visibility': getattr(lm, 'visibility', 1.0),
                    'presence': getattr(lm, 'presence', 1.0)
                })
            face = {
                'landmarks': face_landmarks,
                'confidence': 1.0
            }
        
        # Pose (33 landmarks)
        pose = None
        if results.pose_landmarks:
            pose_landmarks = []
            for lm in results.pose_landmarks.landmark:
                pose_landmarks.append({
                    'x': lm.x,
                    'y': lm.y,
                    'z': lm.z,
                    'visibility': lm.visibility,
                    'presence': getattr(lm, 'presence', 1.0)
                })
            
            # World landmarks
            world_landmarks = []
            if results.pose_world_landmarks:
                for lm in results.pose_world_landmarks.landmark:
                    world_landmarks.append({
                        'x': lm.x,
                        'y': lm.y,
                        'z': lm.z,
                        'visibility': lm.visibility,
                        'presence': getattr(lm, 'presence', 1.0)
                    })
            
            confidence = sum(lm['visibility'] for lm in pose_landmarks) / len(pose_landmarks)
            
            pose = {
                'landmarks': pose_landmarks,
                'world_landmarks': world_landmarks,
                'confidence': confidence
            }
        
        # Left hand (21 landmarks)
        left_hand = None
        if results.left_hand_landmarks:
            left_landmarks = []
            for lm in results.left_hand_landmarks.landmark:
                left_landmarks.append({
                    'x': lm.x,
                    'y': lm.y,
                    'z': lm.z,
                    'visibility': getattr(lm, 'visibility', 1.0),
                    'presence': getattr(lm, 'presence', 1.0)
                })
            left_hand = {
                'landmarks': left_landmarks,
                'handedness': 'Left',
                'confidence': 1.0
            }
        
        # Right hand (21 landmarks)
        right_hand = None
        if results.right_hand_landmarks:
            right_landmarks = []
            for lm in results.right_hand_landmarks.landmark:
                right_landmarks.append({
                    'x': lm.x,
                    'y': lm.y,
                    'z': lm.z,
                    'visibility': getattr(lm, 'visibility', 1.0),
                    'presence': getattr(lm, 'presence', 1.0)
                })
            right_hand = {
                'landmarks': right_landmarks,
                'handedness': 'Right',
                'confidence': 1.0
            }
        
        return {
            'face': face,
            'pose': pose,
            'left_hand': left_hand,
            'right_hand': right_hand
        }
    
    async def track_stream(self, frames: AsyncGenerator[np.ndarray, None]) -> AsyncGenerator[dict, None]:
        """
        Track all body parts in video stream.
        
        Args:
            frames: Async generator yielding RGB frames
        
        Yields:
            Holistic tracking results per frame
        """
        self._ensure_initialized()
        
        async for frame in frames:
            results = self.track_single(frame)
            yield results
    
    def close(self):
        """Release resources"""
        if self._holistic:
            self._holistic.close()
            self._holistic = None
            logger.info("HolisticTracker closed")

