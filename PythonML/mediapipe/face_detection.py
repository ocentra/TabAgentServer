"""
Face Detection - Real-time implementation

MediaPipe Face Detection with 6 keypoints per face.
Reference: https://ai.google.dev/edge/mediapipe/solutions/vision/face_detector
"""

import logging
import numpy as np
from typing import Optional, AsyncGenerator
import cv2

logger = logging.getLogger(__name__)


class FaceDetector:
    """
    MediaPipe Face Detection.
    
    Detects faces with bounding boxes and 6 keypoints:
    - Right eye
    - Left eye
    - Nose tip
    - Mouth center
    - Right ear
    - Left ear
    """
    
    def __init__(self, model_selection: int = 1, min_detection_confidence: float = 0.5):
        """
        Initialize face detector.
        
        Args:
            model_selection: 0 = close-range (2m), 1 = full-range (5m+)
            min_detection_confidence: Minimum confidence threshold
        """
        self.model_selection = model_selection
        self.min_detection_confidence = min_detection_confidence
        self._detector = None
        self._mp = None
        
        logger.info(f"FaceDetector initialized (model={model_selection}, threshold={min_detection_confidence})")
    
    def _ensure_initialized(self):
        """Lazy initialization of MediaPipe"""
        if self._detector is None:
            import mediapipe as mp
            self._mp = mp
            self._detector = mp.solutions.face_detection.FaceDetection(
                model_selection=self.model_selection,
                min_detection_confidence=self.min_detection_confidence
            )
            logger.info("MediaPipe Face Detection loaded")
    
    def detect_single(self, image: np.ndarray) -> list:
        """
        Detect faces in a single image.
        
        Args:
            image: RGB image as numpy array (H, W, 3)
        
        Returns:
            List of detections with format:
            {
                'bbox': {'x': float, 'y': float, 'width': float, 'height': float},
                'keypoints': [{'x': float, 'y': float, 'name': str, 'confidence': float}, ...],
                'confidence': float
            }
        """
        self._ensure_initialized()
        
        results = self._detector.process(image)
        
        detections = []
        if results.detections:
            keypoint_names = [
                "right_eye", "left_eye", "nose_tip", 
                "mouth_center", "right_ear", "left_ear"
            ]
            
            for detection in results.detections:
                bbox = detection.location_data.relative_bounding_box
                
                keypoints = []
                for i, kp in enumerate(detection.location_data.relative_keypoints):
                    keypoints.append({
                        'x': kp.x,
                        'y': kp.y,
                        'name': keypoint_names[i] if i < len(keypoint_names) else f"keypoint_{i}",
                        'confidence': 1.0
                    })
                
                detections.append({
                    'bbox': {
                        'x': bbox.xmin,
                        'y': bbox.ymin,
                        'width': bbox.width,
                        'height': bbox.height
                    },
                    'keypoints': keypoints,
                    'confidence': detection.score[0] if detection.score else 0.0
                })
        
        return detections
    
    async def detect_stream(self, frames: AsyncGenerator[np.ndarray, None]) -> AsyncGenerator[list, None]:
        """
        Detect faces in video stream.
        
        Args:
            frames: Async generator yielding RGB frames
        
        Yields:
            List of detections per frame
        """
        self._ensure_initialized()
        
        async for frame in frames:
            detections = self.detect_single(frame)
            yield detections
    
    def close(self):
        """Release resources"""
        if self._detector:
            self._detector.close()
            self._detector = None
            logger.info("FaceDetector closed")

