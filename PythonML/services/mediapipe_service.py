"""
MediaPipe gRPC Service - Clean delegation to modular components

Connects gRPC endpoints to MediaPipe modules:
- Face detection → FaceDetector
- Face mesh → FaceMesh  
- Hand tracking → HandTracker
- Pose tracking → PoseTracker
- Holistic tracking → HolisticTracker
- Iris tracking → IrisTracker
- Segmentation → Segmenter

NO INLINE LOGIC - all processing delegated to specialized modules.
"""

import logging
import grpc
import numpy as np
import cv2
from typing import AsyncGenerator

import sys
import os
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))

from generated import ml_inference_pb2
from generated import ml_inference_pb2_grpc
from mediapipe import (
    FaceDetector, FaceMesh, HandTracker, PoseTracker,
    HolisticTracker, IrisTracker, Segmenter
)

logger = logging.getLogger(__name__)


class MediapipeService(ml_inference_pb2_grpc.MediapipeServiceServicer):
    """
    MediaPipe gRPC Service - delegates to modular components.
    
    Each capability is a separate, testable module.
    """
    
    def __init__(self):
        """Initialize MediaPipe service"""
        # Modules are lazy-initialized on first use
        self._face_detector = None
        self._face_mesh = None
        self._hand_tracker = None
        self._pose_tracker = None
        self._holistic_tracker = None
        self._iris_tracker = None
        self._selfie_segmenter = None
        self._hair_segmenter = None
        
        logger.info("MediapipeService initialized (all modules ready)")
    
    # ========================================
    # Helper: Decode Frame
    # ========================================
    
    def _decode_frame(self, frame: ml_inference_pb2.VideoFrame) -> np.ndarray:
        """Decode VideoFrame to numpy array"""
        try:
            if frame.format in ["rgb", "bgr"]:
                img = np.frombuffer(frame.frame_data, dtype=np.uint8).reshape(
                    (frame.height, frame.width, 3)
                )
                return cv2.cvtColor(img, cv2.COLOR_BGR2RGB) if frame.format == "bgr" else img
            
            elif frame.format in ["jpeg", "jpg", "png"]:
                nparr = np.frombuffer(frame.frame_data, np.uint8)
                img = cv2.imdecode(nparr, cv2.IMREAD_COLOR)
                return cv2.cvtColor(img, cv2.COLOR_BGR2RGB) if img is not None else None
            
            else:
                logger.error(f"Unsupported format: {frame.format}")
                return None
        except Exception as e:
            logger.error(f"Error decoding frame: {e}")
            return None
    
    # ========================================
    # Face Detection
    # ========================================
    
    async def DetectFaces(self, request, context):
        """Single frame face detection"""
        try:
            if not request.image_data:
                context.set_code(grpc.StatusCode.INVALID_ARGUMENT)
                context.set_details("image_data required")
                return ml_inference_pb2.FaceDetectionResponse()
            
            # Decode image
            nparr = np.frombuffer(request.image_data, np.uint8)
            image = cv2.imdecode(nparr, cv2.IMREAD_COLOR)
            if image is None:
                raise ValueError("Failed to decode image")
            image_rgb = cv2.cvtColor(image, cv2.COLOR_BGR2RGB)
            
            # Lazy init
            if not self._face_detector:
                self._face_detector = FaceDetector()
            
            # Detect
            detections = self._face_detector.detect_single(image_rgb)
            
            # Convert to protobuf
            faces = []
            for det in detections:
                keypoints = [
                    ml_inference_pb2.KeyPoint(
                        x=kp['x'], y=kp['y'],
                        confidence=kp['confidence'],
                        name=kp['name']
                    )
                    for kp in det['keypoints']
                ]
                
                faces.append(ml_inference_pb2.FaceDetection(
                    x=det['bbox']['x'],
                    y=det['bbox']['y'],
                    width=det['bbox']['width'],
                    height=det['bbox']['height'],
                    confidence=det['confidence'],
                    keypoints=keypoints
                ))
            
            return ml_inference_pb2.FaceDetectionResponse(faces=faces)
        
        except Exception as e:
            logger.error(f"Error in DetectFaces: {e}", exc_info=True)
            context.set_code(grpc.StatusCode.INTERNAL)
            context.set_details(str(e))
            return ml_inference_pb2.FaceDetectionResponse()
    
    async def StreamFaceDetection(
        self,
        request_iterator: AsyncGenerator,
        context
    ) -> AsyncGenerator:
        """Real-time face detection stream"""
        try:
            if not self._face_detector:
                self._face_detector = FaceDetector()
            
            async for frame in request_iterator:
                img = self._decode_frame(frame)
                if img is None:
                    continue
                
                detections = self._face_detector.detect_single(img)
                
                faces = []
                for det in detections:
                    keypoints = [
                        ml_inference_pb2.KeyPoint(
                            x=kp['x'], y=kp['y'],
                            confidence=kp['confidence'],
                            name=kp['name']
                        )
                        for kp in det['keypoints']
                    ]
                    
                    faces.append(ml_inference_pb2.FaceDetection(
                        x=det['bbox']['x'],
                        y=det['bbox']['y'],
                        width=det['bbox']['width'],
                        height=det['bbox']['height'],
                        confidence=det['confidence'],
                        keypoints=keypoints
                    ))
                
                yield ml_inference_pb2.FaceDetectionResponse(
                    faces=faces,
                    timestamp_ms=frame.timestamp_ms,
                    frame_number=frame.frame_number
                )
        
        except Exception as e:
            logger.error(f"Error in StreamFaceDetection: {e}", exc_info=True)
            context.set_code(grpc.StatusCode.INTERNAL)
            context.set_details(str(e))
    
    # ========================================
    # Face Mesh
    # ========================================
    
    async def StreamFaceMesh(
        self,
        request_iterator: AsyncGenerator,
        context
    ) -> AsyncGenerator:
        """Real-time face mesh stream"""
        try:
            if not self._face_mesh:
                self._face_mesh = FaceMesh(max_num_faces=2, refine_landmarks=True)
            
            async for frame in request_iterator:
                img = self._decode_frame(frame)
                if img is None:
                    continue
                
                faces = self._face_mesh.process_single(img)
                
                face_meshes = []
                for face in faces:
                    landmarks = [
                        ml_inference_pb2.Landmark(
                            x=lm['x'], y=lm['y'], z=lm['z'],
                            visibility=lm['visibility'],
                            presence=lm['presence']
                        )
                        for lm in face['landmarks']
                    ]
                    
                    face_meshes.append(ml_inference_pb2.FaceMesh(
                        landmarks=landmarks,
                        contours=[],
                        confidence=face['confidence']
                    ))
                
                yield ml_inference_pb2.FaceMeshResponse(
                    faces=face_meshes,
                    timestamp_ms=frame.timestamp_ms,
                    frame_number=frame.frame_number
                )
        
        except Exception as e:
            logger.error(f"Error in StreamFaceMesh: {e}", exc_info=True)
            context.set_code(grpc.StatusCode.INTERNAL)
            context.set_details(str(e))
    
    # ========================================
    # Hand Tracking
    # ========================================
    
    async def DetectHands(self, request, context):
        """Single frame hand detection"""
        try:
            if not request.image_data:
                context.set_code(grpc.StatusCode.INVALID_ARGUMENT)
                context.set_details("image_data required")
                return ml_inference_pb2.HandDetectionResponse()
            
            nparr = np.frombuffer(request.image_data, np.uint8)
            image = cv2.imdecode(nparr, cv2.IMREAD_COLOR)
            if image is None:
                raise ValueError("Failed to decode image")
            image_rgb = cv2.cvtColor(image, cv2.COLOR_BGR2RGB)
            
            if not self._hand_tracker:
                self._hand_tracker = HandTracker()
            
            hands = self._hand_tracker.track_single(image_rgb)
            
            hand_detections = []
            for hand in hands:
                landmarks = [
                    ml_inference_pb2.Landmark(
                        x=lm['x'], y=lm['y'], z=lm['z'],
                        visibility=lm['visibility'],
                        presence=lm['presence']
                    )
                    for lm in hand['landmarks']
                ]
                
                gestures = [
                    ml_inference_pb2.Gesture(
                        name=g['name'],
                        confidence=g['confidence']
                    )
                    for g in hand['gestures']
                ]
                
                hand_detections.append(ml_inference_pb2.HandDetection(
                    landmarks=landmarks,
                    handedness=hand['handedness'],
                    confidence=hand['confidence'],
                    gestures=gestures
                ))
            
            return ml_inference_pb2.HandDetectionResponse(hands=hand_detections)
        
        except Exception as e:
            logger.error(f"Error in DetectHands: {e}", exc_info=True)
            context.set_code(grpc.StatusCode.INTERNAL)
            context.set_details(str(e))
            return ml_inference_pb2.HandDetectionResponse()
    
    async def StreamHandTracking(
        self,
        request_iterator: AsyncGenerator,
        context
    ) -> AsyncGenerator:
        """Real-time hand tracking stream"""
        try:
            if not self._hand_tracker:
                self._hand_tracker = HandTracker()
            
            async for frame in request_iterator:
                img = self._decode_frame(frame)
                if img is None:
                    continue
                
                hands = self._hand_tracker.track_single(img)
                
                hand_detections = []
                for hand in hands:
                    landmarks = [
                        ml_inference_pb2.Landmark(
                            x=lm['x'], y=lm['y'], z=lm['z'],
                            visibility=lm['visibility'],
                            presence=lm['presence']
                        )
                        for lm in hand['landmarks']
                    ]
                    
                    gestures = [
                        ml_inference_pb2.Gesture(
                            name=g['name'],
                            confidence=g['confidence']
                        )
                        for g in hand['gestures']
                    ]
                    
                    hand_detections.append(ml_inference_pb2.HandDetection(
                        landmarks=landmarks,
                        handedness=hand['handedness'],
                        confidence=hand['confidence'],
                        gestures=gestures
                    ))
                
                yield ml_inference_pb2.HandDetectionResponse(
                    hands=hand_detections,
                    timestamp_ms=frame.timestamp_ms,
                    frame_number=frame.frame_number
                )
        
        except Exception as e:
            logger.error(f"Error in StreamHandTracking: {e}", exc_info=True)
            context.set_code(grpc.StatusCode.INTERNAL)
            context.set_details(str(e))
    
    # ========================================
    # Pose Tracking
    # ========================================
    
    async def DetectPose(self, request, context):
        """Single frame pose detection"""
        try:
            if not request.image_data:
                context.set_code(grpc.StatusCode.INVALID_ARGUMENT)
                context.set_details("image_data required")
                return ml_inference_pb2.PoseDetectionResponse()
            
            nparr = np.frombuffer(request.image_data, np.uint8)
            image = cv2.imdecode(nparr, cv2.IMREAD_COLOR)
            if image is None:
                raise ValueError("Failed to decode image")
            image_rgb = cv2.cvtColor(image, cv2.COLOR_BGR2RGB)
            
            if not self._pose_tracker:
                self._pose_tracker = PoseTracker()
            
            pose = self._pose_tracker.track_single(image_rgb)
            
            poses = []
            if pose:
                landmarks = [
                    ml_inference_pb2.Landmark(
                        x=lm['x'], y=lm['y'], z=lm['z'],
                        visibility=lm['visibility'],
                        presence=lm['presence']
                    )
                    for lm in pose['landmarks']
                ]
                
                world_landmarks = [
                    ml_inference_pb2.Landmark(
                        x=lm['x'], y=lm['y'], z=lm['z'],
                        visibility=lm['visibility'],
                        presence=lm['presence']
                    )
                    for lm in pose['world_landmarks']
                ]
                
                poses.append(ml_inference_pb2.PoseDetection(
                    landmarks=landmarks,
                    confidence=pose['confidence'],
                    world_landmarks=world_landmarks
                ))
            
            return ml_inference_pb2.PoseDetectionResponse(poses=poses)
        
        except Exception as e:
            logger.error(f"Error in DetectPose: {e}", exc_info=True)
            context.set_code(grpc.StatusCode.INTERNAL)
            context.set_details(str(e))
            return ml_inference_pb2.PoseDetectionResponse()
    
    async def StreamPoseTracking(
        self,
        request_iterator: AsyncGenerator,
        context
    ) -> AsyncGenerator:
        """Real-time pose tracking stream"""
        try:
            if not self._pose_tracker:
                self._pose_tracker = PoseTracker()
            
            async for frame in request_iterator:
                img = self._decode_frame(frame)
                if img is None:
                    continue
                
                pose = self._pose_tracker.track_single(img)
                
                poses = []
                if pose:
                    landmarks = [
                        ml_inference_pb2.Landmark(
                            x=lm['x'], y=lm['y'], z=lm['z'],
                            visibility=lm['visibility'],
                            presence=lm['presence']
                        )
                        for lm in pose['landmarks']
                    ]
                    
                    world_landmarks = [
                        ml_inference_pb2.Landmark(
                            x=lm['x'], y=lm['y'], z=lm['z'],
                            visibility=lm['visibility'],
                            presence=lm['presence']
                        )
                        for lm in pose['world_landmarks']
                    ]
                    
                    poses.append(ml_inference_pb2.PoseDetection(
                        landmarks=landmarks,
                        confidence=pose['confidence'],
                        world_landmarks=world_landmarks
                    ))
                
                yield ml_inference_pb2.PoseDetectionResponse(
                    poses=poses,
                    timestamp_ms=frame.timestamp_ms,
                    frame_number=frame.frame_number
                )
        
        except Exception as e:
            logger.error(f"Error in StreamPoseTracking: {e}", exc_info=True)
            context.set_code(grpc.StatusCode.INTERNAL)
            context.set_details(str(e))
    
    # ========================================
    # Holistic Tracking
    # ========================================
    
    async def StreamHolisticTracking(
        self,
        request_iterator: AsyncGenerator,
        context
    ) -> AsyncGenerator:
        """Real-time holistic tracking stream"""
        try:
            if not self._holistic_tracker:
                self._holistic_tracker = HolisticTracker()
            
            async for frame in request_iterator:
                img = self._decode_frame(frame)
                if img is None:
                    continue
                
                results = self._holistic_tracker.track_single(img)
                
                # Face
                face = None
                if results['face']:
                    face_landmarks = [
                        ml_inference_pb2.Landmark(
                            x=lm['x'], y=lm['y'], z=lm['z'],
                            visibility=lm['visibility'],
                            presence=lm['presence']
                        )
                        for lm in results['face']['landmarks']
                    ]
                    face = ml_inference_pb2.FaceMesh(
                        landmarks=face_landmarks,
                        contours=[],
                        confidence=results['face']['confidence']
                    )
                
                # Pose
                pose = None
                if results['pose']:
                    pose_landmarks = [
                        ml_inference_pb2.Landmark(
                            x=lm['x'], y=lm['y'], z=lm['z'],
                            visibility=lm['visibility'],
                            presence=lm['presence']
                        )
                        for lm in results['pose']['landmarks']
                    ]
                    world_landmarks = [
                        ml_inference_pb2.Landmark(
                            x=lm['x'], y=lm['y'], z=lm['z'],
                            visibility=lm['visibility'],
                            presence=lm['presence']
                        )
                        for lm in results['pose']['world_landmarks']
                    ]
                    pose = ml_inference_pb2.PoseDetection(
                        landmarks=pose_landmarks,
                        confidence=results['pose']['confidence'],
                        world_landmarks=world_landmarks
                    )
                
                # Hands
                hands = []
                for hand_key in ['left_hand', 'right_hand']:
                    if results[hand_key]:
                        hand_landmarks = [
                            ml_inference_pb2.Landmark(
                                x=lm['x'], y=lm['y'], z=lm['z'],
                                visibility=lm['visibility'],
                                presence=lm['presence']
                            )
                            for lm in results[hand_key]['landmarks']
                        ]
                        hands.append(ml_inference_pb2.HandDetection(
                            landmarks=hand_landmarks,
                            handedness=results[hand_key]['handedness'],
                            confidence=results[hand_key]['confidence'],
                            gestures=[]
                        ))
                
                yield ml_inference_pb2.HolisticTrackingResponse(
                    face=face,
                    pose=pose,
                    hands=hands,
                    timestamp_ms=frame.timestamp_ms,
                    frame_number=frame.frame_number
                )
        
        except Exception as e:
            logger.error(f"Error in StreamHolisticTracking: {e}", exc_info=True)
            context.set_code(grpc.StatusCode.INTERNAL)
            context.set_details(str(e))
    
    # ========================================
    # Iris Tracking
    # ========================================
    
    async def StreamIrisTracking(
        self,
        request_iterator: AsyncGenerator,
        context
    ) -> AsyncGenerator:
        """Real-time iris tracking stream"""
        try:
            if not self._iris_tracker:
                self._iris_tracker = IrisTracker()
            
            async for frame in request_iterator:
                img = self._decode_frame(frame)
                if img is None:
                    continue
                
                eyes = self._iris_tracker.track_single(img)
                
                iris_eyes = []
                for eye in eyes:
                    iris_landmarks = [
                        ml_inference_pb2.Landmark(x=lm['x'], y=lm['y'], z=lm['z'])
                        for lm in eye['iris_landmarks']
                    ]
                    eye_landmarks = [
                        ml_inference_pb2.Landmark(x=lm['x'], y=lm['y'], z=lm['z'])
                        for lm in eye['eye_landmarks']
                    ]
                    
                    iris_eyes.append(ml_inference_pb2.IrisTracking(
                        iris_landmarks=iris_landmarks,
                        eye_landmarks=eye_landmarks,
                        eye=eye['eye'],
                        confidence=eye['confidence']
                    ))
                
                yield ml_inference_pb2.IrisTrackingResponse(
                    eyes=iris_eyes,
                    timestamp_ms=frame.timestamp_ms,
                    frame_number=frame.frame_number
                )
        
        except Exception as e:
            logger.error(f"Error in StreamIrisTracking: {e}", exc_info=True)
            context.set_code(grpc.StatusCode.INTERNAL)
            context.set_details(str(e))
    
    # ========================================
    # Segmentation
    # ========================================
    
    async def StreamSelfieSegmentation(
        self,
        request_iterator: AsyncGenerator,
        context
    ) -> AsyncGenerator:
        """Real-time selfie segmentation stream"""
        try:
            if not self._selfie_segmenter:
                self._selfie_segmenter = Segmenter(model_selection=1, segmentation_type='selfie')
            
            async for frame in request_iterator:
                img = self._decode_frame(frame)
                if img is None:
                    continue
                
                result = self._selfie_segmenter.segment_single(img)
                
                yield ml_inference_pb2.SegmentationResponse(
                    mask=result['mask'].tobytes(),
                    width=result['width'],
                    height=result['height'],
                    format="binary",
                    timestamp_ms=frame.timestamp_ms,
                    frame_number=frame.frame_number
                )
        
        except Exception as e:
            logger.error(f"Error in StreamSelfieSegmentation: {e}", exc_info=True)
            context.set_code(grpc.StatusCode.INTERNAL)
            context.set_details(str(e))
    
    async def StreamHairSegmentation(
        self,
        request_iterator: AsyncGenerator,
        context
    ) -> AsyncGenerator:
        """Real-time hair segmentation stream"""
        try:
            if not self._hair_segmenter:
                self._hair_segmenter = Segmenter(model_selection=1, segmentation_type='hair')
            
            async for frame in request_iterator:
                img = self._decode_frame(frame)
                if img is None:
                    continue
                
                result = self._hair_segmenter.segment_single(img)
                
                yield ml_inference_pb2.SegmentationResponse(
                    mask=result['mask'].tobytes(),
                    width=result['width'],
                    height=result['height'],
                    format="binary",
                    timestamp_ms=frame.timestamp_ms,
                    frame_number=frame.frame_number
                )
        
        except Exception as e:
            logger.error(f"Error in StreamHairSegmentation: {e}", exc_info=True)
            context.set_code(grpc.StatusCode.INTERNAL)
            context.set_details(str(e))
    
    # ========================================
    # Object Detection (TODO: requires .tflite models)
    # ========================================
    
    async def StreamObjectDetection(self, request_iterator, context):
        """Real-time object detection"""
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details("Requires .tflite model - load via ModelManagementService first")
        return
        yield
    
    async def StreamObjectTracking(self, request_iterator, context):
        """Real-time object tracking"""
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details("Requires implementation")
        return
        yield
    
    async def StreamObjectDetection3D(self, request_iterator, context):
        """Real-time 3D object detection"""
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details("Objectron support ended by Google")
        return
        yield
    
    async def StreamTemplateMatching(self, request_iterator, context):
        """Real-time template matching"""
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details("Requires implementation")
        return
        yield
    
    async def StreamAutoFlip(self, request_iterator, context):
        """Intelligent video cropping"""
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details("Requires MediaPipe graph implementation")
        return
        yield
    
    async def ProcessMediaSequence(self, request_iterator, context):
        """Batch video sequence analysis"""
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details("Requires pre-trained model")
        return ml_inference_pb2.MediaSequenceResponse()
