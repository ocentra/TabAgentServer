"""
MediaPipe module - All vision/pose/hand tracking capabilities

Modular implementation with separate files for each capability:
- FaceDetector: 6-keypoint face detection
- FaceMesh: 468-point 3D face mesh
- HandTracker: 21-point hand tracking with gestures
- PoseTracker: 33-point body pose tracking
- HolisticTracker: Combined face + hands + pose
- IrisTracker: Eye gaze estimation
- Segmenter: Person/background segmentation
"""

from .manager import MediaPipeManager
from .face_detection import FaceDetector
from .face_mesh import FaceMesh
from .hand_tracking import HandTracker
from .pose_tracking import PoseTracker
from .holistic_tracking import HolisticTracker
from .iris_tracking import IrisTracker
from .segmentation import Segmenter

__all__ = [
    'MediaPipeManager',
    'FaceDetector',
    'FaceMesh',
    'HandTracker',
    'PoseTracker',
    'HolisticTracker',
    'IrisTracker',
    'Segmenter',
]
