"""
Pose Tracking - Real-time 33-point body pose landmarks

MediaPipe Pose with world coordinates.
Reference: https://ai.google.dev/edge/mediapipe/solutions/vision/pose_landmarker
"""

import logging
import numpy as np
from typing import Optional, AsyncGenerator

logger = logging.getLogger(__name__)


class PoseTracker:
    """
    MediaPipe Pose Tracking.
    
    Provides 33 3D landmarks for full-body pose:
    - Face (nose, eyes, ears, mouth)
    - Upper body (shoulders, elbows, wrists, hands)
    - Torso (hips)
    - Lower body (knees, ankles, feet)
    
    Also provides world landmarks (real-world 3D coordinates).
    """
    
    # Landmark indices
    NOSE = 0
    LEFT_EYE_INNER = 1
    LEFT_EYE = 2
    LEFT_EYE_OUTER = 3
    RIGHT_EYE_INNER = 4
    RIGHT_EYE = 5
    RIGHT_EYE_OUTER = 6
    LEFT_EAR = 7
    RIGHT_EAR = 8
    MOUTH_LEFT = 9
    MOUTH_RIGHT = 10
    LEFT_SHOULDER = 11
    RIGHT_SHOULDER = 12
    LEFT_ELBOW = 13
    RIGHT_ELBOW = 14
    LEFT_WRIST = 15
    RIGHT_WRIST = 16
    LEFT_PINKY = 17
    RIGHT_PINKY = 18
    LEFT_INDEX = 19
    RIGHT_INDEX = 20
    LEFT_THUMB = 21
    RIGHT_THUMB = 22
    LEFT_HIP = 23
    RIGHT_HIP = 24
    LEFT_KNEE = 25
    RIGHT_KNEE = 26
    LEFT_ANKLE = 27
    RIGHT_ANKLE = 28
    LEFT_HEEL = 29
    RIGHT_HEEL = 30
    LEFT_FOOT_INDEX = 31
    RIGHT_FOOT_INDEX = 32
    
    def __init__(
        self,
        model_complexity: int = 2,
        smooth_landmarks: bool = True,
        min_detection_confidence: float = 0.5,
        min_tracking_confidence: float = 0.5
    ):
        """
        Initialize pose tracker.
        
        Args:
            model_complexity: 0 = lite, 1 = full, 2 = heavy (most accurate)
            smooth_landmarks: Enable temporal smoothing
            min_detection_confidence: Minimum confidence for detection
            min_tracking_confidence: Minimum confidence for tracking
        """
        self.model_complexity = model_complexity
        self.smooth_landmarks = smooth_landmarks
        self.min_detection_confidence = min_detection_confidence
        self.min_tracking_confidence = min_tracking_confidence
        self._pose = None
        self._mp = None
        
        logger.info(f"PoseTracker initialized (complexity={model_complexity}, smooth={smooth_landmarks})")
    
    def _ensure_initialized(self):
        """Lazy initialization of MediaPipe"""
        if self._pose is None:
            import mediapipe as mp
            self._mp = mp
            self._pose = mp.solutions.pose.Pose(
                static_image_mode=False,
                model_complexity=self.model_complexity,
                smooth_landmarks=self.smooth_landmarks,
                min_detection_confidence=self.min_detection_confidence,
                min_tracking_confidence=self.min_tracking_confidence
            )
            logger.info("MediaPipe Pose loaded")
    
    def track_single(self, image: np.ndarray) -> Optional[dict]:
        """
        Track pose in a single image.
        
        Args:
            image: RGB image as numpy array (H, W, 3)
        
        Returns:
            Pose detection with format:
            {
                'landmarks': [{'x': float, 'y': float, 'z': float, 'visibility': float}, ...],
                'world_landmarks': [{'x': float, 'y': float, 'z': float, 'visibility': float}, ...],
                'confidence': float
            }
            Or None if no pose detected.
        """
        self._ensure_initialized()
        
        results = self._pose.process(image)
        
        if not results.pose_landmarks:
            return None
        
        # Image landmarks (normalized [0, 1])
        landmarks = []
        for lm in results.pose_landmarks.landmark:
            landmarks.append({
                'x': lm.x,
                'y': lm.y,
                'z': lm.z,
                'visibility': lm.visibility,
                'presence': getattr(lm, 'presence', 1.0)
            })
        
        # World landmarks (meters, origin at hip center)
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
        
        # Calculate confidence as average visibility
        confidence = sum(lm['visibility'] for lm in landmarks) / len(landmarks)
        
        return {
            'landmarks': landmarks,
            'world_landmarks': world_landmarks,
            'confidence': confidence
        }
    
    async def track_stream(self, frames: AsyncGenerator[np.ndarray, None]) -> AsyncGenerator[Optional[dict], None]:
        """
        Track pose in video stream.
        
        Args:
            frames: Async generator yielding RGB frames
        
        Yields:
            Pose detection per frame (or None if no pose)
        """
        self._ensure_initialized()
        
        async for frame in frames:
            pose = self.track_single(frame)
            yield pose
    
    def calculate_angles(self, landmarks: list) -> dict:
        """
        Calculate joint angles from landmarks.
        
        Args:
            landmarks: List of 33 pose landmarks
        
        Returns:
            Dictionary of joint angles in degrees
        """
        if len(landmarks) < 33:
            return {}
        
        def angle_between_points(p1, p2, p3):
            """Calculate angle at p2 formed by p1-p2-p3"""
            v1 = np.array([p1['x'] - p2['x'], p1['y'] - p2['y']])
            v2 = np.array([p3['x'] - p2['x'], p3['y'] - p2['y']])
            
            cos_angle = np.dot(v1, v2) / (np.linalg.norm(v1) * np.linalg.norm(v2) + 1e-6)
            angle = np.arccos(np.clip(cos_angle, -1.0, 1.0))
            return np.degrees(angle)
        
        angles = {}
        
        # Left arm
        angles['left_elbow'] = angle_between_points(
            landmarks[self.LEFT_SHOULDER],
            landmarks[self.LEFT_ELBOW],
            landmarks[self.LEFT_WRIST]
        )
        
        # Right arm
        angles['right_elbow'] = angle_between_points(
            landmarks[self.RIGHT_SHOULDER],
            landmarks[self.RIGHT_ELBOW],
            landmarks[self.RIGHT_WRIST]
        )
        
        # Left leg
        angles['left_knee'] = angle_between_points(
            landmarks[self.LEFT_HIP],
            landmarks[self.LEFT_KNEE],
            landmarks[self.LEFT_ANKLE]
        )
        
        # Right leg
        angles['right_knee'] = angle_between_points(
            landmarks[self.RIGHT_HIP],
            landmarks[self.RIGHT_KNEE],
            landmarks[self.RIGHT_ANKLE]
        )
        
        # Left shoulder
        angles['left_shoulder'] = angle_between_points(
            landmarks[self.LEFT_HIP],
            landmarks[self.LEFT_SHOULDER],
            landmarks[self.LEFT_ELBOW]
        )
        
        # Right shoulder
        angles['right_shoulder'] = angle_between_points(
            landmarks[self.RIGHT_HIP],
            landmarks[self.RIGHT_SHOULDER],
            landmarks[self.RIGHT_ELBOW]
        )
        
        # Left hip
        angles['left_hip'] = angle_between_points(
            landmarks[self.LEFT_SHOULDER],
            landmarks[self.LEFT_HIP],
            landmarks[self.LEFT_KNEE]
        )
        
        # Right hip
        angles['right_hip'] = angle_between_points(
            landmarks[self.RIGHT_SHOULDER],
            landmarks[self.RIGHT_HIP],
            landmarks[self.RIGHT_KNEE]
        )
        
        return angles
    
    def close(self):
        """Release resources"""
        if self._pose:
            self._pose.close()
            self._pose = None
            logger.info("PoseTracker closed")

