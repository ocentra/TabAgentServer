"""
Hand Tracking - Real-time 21-point hand landmarks with gesture recognition

MediaPipe Hands with custom gesture detection.
Reference: https://ai.google.dev/edge/mediapipe/solutions/vision/hand_landmarker
"""

import logging
import numpy as np
from typing import Optional, AsyncGenerator, List, Dict

logger = logging.getLogger(__name__)


class HandTracker:
    """
    MediaPipe Hand Tracking.
    
    Provides 21 3D landmarks per hand:
    - Wrist
    - Thumb (4 joints)
    - Index finger (4 joints)
    - Middle finger (4 joints)
    - Ring finger (4 joints)
    - Pinky finger (4 joints)
    
    Plus gesture recognition:
    - Thumb up
    - Victory / Peace sign
    - Open palm
    - Fist
    - Pointing
    """
    
    # Landmark indices
    WRIST = 0
    THUMB_CMC = 1
    THUMB_MCP = 2
    THUMB_IP = 3
    THUMB_TIP = 4
    INDEX_FINGER_MCP = 5
    INDEX_FINGER_PIP = 6
    INDEX_FINGER_DIP = 7
    INDEX_FINGER_TIP = 8
    MIDDLE_FINGER_MCP = 9
    MIDDLE_FINGER_PIP = 10
    MIDDLE_FINGER_DIP = 11
    MIDDLE_FINGER_TIP = 12
    RING_FINGER_MCP = 13
    RING_FINGER_PIP = 14
    RING_FINGER_DIP = 15
    RING_FINGER_TIP = 16
    PINKY_MCP = 17
    PINKY_PIP = 18
    PINKY_DIP = 19
    PINKY_TIP = 20
    
    def __init__(
        self,
        max_num_hands: int = 2,
        min_detection_confidence: float = 0.5,
        min_tracking_confidence: float = 0.5
    ):
        """
        Initialize hand tracker.
        
        Args:
            max_num_hands: Maximum number of hands to track
            min_detection_confidence: Minimum confidence for detection
            min_tracking_confidence: Minimum confidence for tracking
        """
        self.max_num_hands = max_num_hands
        self.min_detection_confidence = min_detection_confidence
        self.min_tracking_confidence = min_tracking_confidence
        self._hands = None
        self._mp = None
        
        logger.info(f"HandTracker initialized (max_hands={max_num_hands})")
    
    def _ensure_initialized(self):
        """Lazy initialization of MediaPipe"""
        if self._hands is None:
            import mediapipe as mp
            self._mp = mp
            self._hands = mp.solutions.hands.Hands(
                static_image_mode=False,
                max_num_hands=self.max_num_hands,
                min_detection_confidence=self.min_detection_confidence,
                min_tracking_confidence=self.min_tracking_confidence
            )
            logger.info("MediaPipe Hands loaded")
    
    def track_single(self, image: np.ndarray) -> list:
        """
        Track hands in a single image.
        
        Args:
            image: RGB image as numpy array (H, W, 3)
        
        Returns:
            List of hand detections with format:
            {
                'landmarks': [{'x': float, 'y': float, 'z': float}, ...],
                'handedness': 'Left' | 'Right',
                'confidence': float,
                'gestures': [{'name': str, 'confidence': float}, ...]
            }
        """
        self._ensure_initialized()
        
        results = self._hands.process(image)
        
        hands = []
        if results.multi_hand_landmarks and results.multi_handedness:
            for hand_landmarks, handedness in zip(results.multi_hand_landmarks, results.multi_handedness):
                landmarks = []
                for lm in hand_landmarks.landmark:
                    landmarks.append({
                        'x': lm.x,
                        'y': lm.y,
                        'z': lm.z,
                        'visibility': getattr(lm, 'visibility', 1.0),
                        'presence': getattr(lm, 'presence', 1.0)
                    })
                
                hand_label = handedness.classification[0].label
                hand_confidence = handedness.classification[0].score
                
                # Detect gestures
                gestures = self.detect_gestures(landmarks, hand_label)
                
                hands.append({
                    'landmarks': landmarks,
                    'handedness': hand_label,
                    'confidence': hand_confidence,
                    'gestures': gestures
                })
        
        return hands
    
    async def track_stream(self, frames: AsyncGenerator[np.ndarray, None]) -> AsyncGenerator[list, None]:
        """
        Track hands in video stream.
        
        Args:
            frames: Async generator yielding RGB frames
        
        Yields:
            List of hand detections per frame
        """
        self._ensure_initialized()
        
        async for frame in frames:
            hands = self.track_single(frame)
            yield hands
    
    def detect_gestures(self, landmarks: List[Dict], handedness: str) -> List[Dict]:
        """
        Detect hand gestures from landmarks.
        
        Args:
            landmarks: List of 21 hand landmarks
            handedness: 'Left' or 'Right'
        
        Returns:
            List of detected gestures with confidence
        """
        if len(landmarks) < 21:
            return []
        
        gestures = []
        
        # Extract key landmarks
        wrist = landmarks[self.WRIST]
        thumb_tip = landmarks[self.THUMB_TIP]
        thumb_ip = landmarks[self.THUMB_IP]
        index_tip = landmarks[self.INDEX_FINGER_TIP]
        index_pip = landmarks[self.INDEX_FINGER_PIP]
        index_mcp = landmarks[self.INDEX_FINGER_MCP]
        middle_tip = landmarks[self.MIDDLE_FINGER_TIP]
        middle_pip = landmarks[self.MIDDLE_FINGER_PIP]
        middle_mcp = landmarks[self.MIDDLE_FINGER_MCP]
        ring_tip = landmarks[self.RING_FINGER_TIP]
        ring_pip = landmarks[self.RING_FINGER_PIP]
        pinky_tip = landmarks[self.PINKY_TIP]
        pinky_pip = landmarks[self.PINKY_PIP]
        
        # Check finger extension states
        thumb_extended = thumb_tip['y'] < thumb_ip['y']
        index_extended = index_tip['y'] < index_pip['y']
        middle_extended = middle_tip['y'] < middle_pip['y']
        ring_extended = ring_tip['y'] < ring_pip['y']
        pinky_extended = pinky_tip['y'] < pinky_pip['y']
        
        # Thumb Up: Thumb extended, all fingers closed
        if thumb_extended and not index_extended and not middle_extended and not ring_extended and not pinky_extended:
            gestures.append({'name': 'Thumb_Up', 'confidence': 0.95})
        
        # Victory / Peace Sign: Index and middle extended, others closed
        elif index_extended and middle_extended and not ring_extended and not pinky_extended:
            # Check if index and middle are separated (V shape)
            finger_distance = abs(index_tip['x'] - middle_tip['x'])
            if finger_distance > 0.05:
                gestures.append({'name': 'Victory', 'confidence': 0.95})
        
        # Open Palm: All fingers extended
        elif index_extended and middle_extended and ring_extended and pinky_extended:
            gestures.append({'name': 'Open_Palm', 'confidence': 0.95})
        
        # Fist: All fingers closed
        elif not index_extended and not middle_extended and not ring_extended and not pinky_extended:
            gestures.append({'name': 'Fist', 'confidence': 0.95})
        
        # Pointing: Only index extended
        elif index_extended and not middle_extended and not ring_extended and not pinky_extended:
            gestures.append({'name': 'Pointing', 'confidence': 0.95})
        
        # OK Sign: Thumb and index touch, others extended
        thumb_index_dist = np.sqrt(
            (thumb_tip['x'] - index_tip['x'])**2 + 
            (thumb_tip['y'] - index_tip['y'])**2
        )
        if thumb_index_dist < 0.05 and middle_extended and ring_extended and pinky_extended:
            gestures.append({'name': 'OK_Sign', 'confidence': 0.90})
        
        # Rock Sign: Index and pinky extended, middle and ring closed
        elif index_extended and not middle_extended and not ring_extended and pinky_extended:
            gestures.append({'name': 'Rock_Sign', 'confidence': 0.90})
        
        return gestures
    
    def close(self):
        """Release resources"""
        if self._hands:
            self._hands.close()
            self._hands = None
            logger.info("HandTracker closed")

