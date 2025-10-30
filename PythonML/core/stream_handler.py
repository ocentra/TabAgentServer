"""
Stream Handler - Converts various input sources to VideoFrame format

Handles real-time streaming from:
1. WebRTC data channels (from Rust server)
2. Native messaging (from Chrome extension)
3. System capture (camera/screen/audio)
4. File streams (video/audio files)

Converts all formats to ml_inference_pb2.VideoFrame for MediaPipe processing.
"""

import logging
from typing import Optional, AsyncGenerator
from enum import Enum

logger = logging.getLogger(__name__)


class StreamSource(str, Enum):
    """Source of video/audio stream"""
    WEBRTC = "webrtc"          # WebRTC data channel from Rust
    NATIVE = "native"          # Native messaging from extension
    CAMERA = "camera"          # System camera
    SCREEN = "screen"          # Screen capture
    FILE = "file"              # Video file
    AUDIO = "audio"            # Audio stream


class StreamHandler:
    """
    Converts various stream sources to VideoFrame format.
    
    Usage from Rust:
    
    1. WebRTC Data Channel:
       - Rust captures video/audio
       - Encodes frames as H264/VP8 or raw RGB
       - Sends via WebRTC data channel
       - This handler decodes and forwards to MediaPipe
    
    2. Native Messaging:
       - Extension captures tab video/audio
       - Sends via native messaging pipe
       - This handler decodes and forwards to MediaPipe
    
    3. Direct System Capture:
       - Rust requests Python to capture directly
       - Python uses OpenCV/PyAudio
       - Forwards to MediaPipe
    """
    
    def __init__(self):
        """Initialize stream handler"""
        self.active_streams = {}
        logger.info("StreamHandler initialized")
    
    async def handle_webrtc_stream(
        self,
        stream_id: str,
        data_channel: AsyncGenerator[bytes, None]
    ) -> AsyncGenerator:
        """
        Handle WebRTC data channel stream from Rust.
        
        Format:
        - Header: 8 bytes (timestamp_ms: uint64)
        - Width: 4 bytes (uint32)
        - Height: 4 bytes (uint32)
        - Format: 1 byte (0=RGB, 1=BGR, 2=H264, 3=VP8)
        - Data: remaining bytes
        
        Args:
            stream_id: Unique stream identifier
            data_channel: WebRTC data channel from Rust
        
        Yields:
            ml_inference_pb2.VideoFrame objects
        """
        try:
            import struct
            from generated import ml_inference_pb2
            
            frame_number = 0
            
            async for chunk in data_channel:
                if len(chunk) < 17:  # Header size
                    logger.warning(f"Incomplete frame header: {len(chunk)} bytes")
                    continue
                
                # Parse header
                timestamp_ms = struct.unpack('<Q', chunk[0:8])[0]
                width = struct.unpack('<I', chunk[8:12])[0]
                height = struct.unpack('<I', chunk[12:16])[0]
                format_byte = chunk[16]
                frame_data = chunk[17:]
                
                # Map format byte to string
                format_map = {
                    0: "rgb",
                    1: "bgr",
                    2: "h264",
                    3: "vp8",
                    4: "jpeg",
                }
                format_str = format_map.get(format_byte, "rgb")
                
                # Create VideoFrame
                yield ml_inference_pb2.VideoFrame(
                    frame_data=frame_data,
                    format=format_str,
                    width=width,
                    height=height,
                    timestamp_ms=timestamp_ms,
                    frame_number=frame_number,
                    metadata={"stream_id": stream_id, "source": "webrtc"}
                )
                
                frame_number += 1
        
        except Exception as e:
            logger.error(f"Error handling WebRTC stream {stream_id}: {e}", exc_info=True)
    
    async def handle_native_stream(
        self,
        stream_id: str,
        pipe_reader: AsyncGenerator[bytes, None]
    ) -> AsyncGenerator:
        """
        Handle native messaging stream from Chrome extension.
        
        Extension sends:
        - Tab video capture
        - Screen capture
        - Audio from tab
        
        Format is similar to WebRTC but uses native messaging pipe.
        """
        # Similar to WebRTC but receives from stdin/pipe
        async for frame in self.handle_webrtc_stream(stream_id, pipe_reader):
            yield frame
    
    async def capture_camera(
        self,
        camera_id: int = 0,
        width: int = 640,
        height: int = 480,
        fps: int = 30
    ) -> AsyncGenerator:
        """
        Capture from system camera using OpenCV.
        
        Args:
            camera_id: Camera device ID
            width: Frame width
            height: Frame height
            fps: Frames per second
        
        Yields:
            ml_inference_pb2.VideoFrame objects
        """
        try:
            import cv2
            import time
            from generated import ml_inference_pb2
            
            cap = cv2.VideoCapture(camera_id)
            cap.set(cv2.CAP_PROP_FRAME_WIDTH, width)
            cap.set(cv2.CAP_PROP_FRAME_HEIGHT, height)
            cap.set(cv2.CAP_PROP_FPS, fps)
            
            if not cap.isOpened():
                raise RuntimeError(f"Failed to open camera {camera_id}")
            
            logger.info(f"Camera {camera_id} opened: {width}x{height} @ {fps}fps")
            
            frame_number = 0
            frame_interval = 1.0 / fps
            
            while True:
                start_time = time.time()
                
                ret, frame = cap.read()
                if not ret:
                    logger.warning("Failed to read camera frame")
                    break
                
                # Convert BGR to RGB
                frame_rgb = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)
                
                # Create VideoFrame
                yield ml_inference_pb2.VideoFrame(
                    frame_data=frame_rgb.tobytes(),
                    format="rgb",
                    width=frame.shape[1],
                    height=frame.shape[0],
                    timestamp_ms=int(time.time() * 1000),
                    frame_number=frame_number,
                    metadata={"source": "camera", "camera_id": str(camera_id)}
                )
                
                frame_number += 1
                
                # Maintain target FPS
                elapsed = time.time() - start_time
                if elapsed < frame_interval:
                    await asyncio.sleep(frame_interval - elapsed)
            
            cap.release()
        
        except Exception as e:
            logger.error(f"Error capturing camera: {e}", exc_info=True)
    
    async def capture_screen(
        self,
        region: Optional[tuple] = None,
        fps: int = 30
    ) -> AsyncGenerator:
        """
        Capture screen using mss or pyautogui.
        
        Args:
            region: (x, y, width, height) or None for full screen
            fps: Frames per second
        
        Yields:
            ml_inference_pb2.VideoFrame objects
        """
        try:
            import mss
            import numpy as np
            import time
            from generated import ml_inference_pb2
            
            with mss.mss() as sct:
                if region:
                    monitor = {
                        "top": region[1],
                        "left": region[0],
                        "width": region[2],
                        "height": region[3]
                    }
                else:
                    monitor = sct.monitors[1]  # Primary monitor
                
                logger.info(f"Screen capture started: {monitor}")
                
                frame_number = 0
                frame_interval = 1.0 / fps
                
                while True:
                    start_time = time.time()
                    
                    # Capture screen
                    img = sct.grab(monitor)
                    frame = np.array(img)
                    
                    # Convert BGRA to RGB
                    frame_rgb = frame[:, :, :3]  # Drop alpha
                    frame_rgb = frame_rgb[:, :, ::-1]  # BGR to RGB
                    
                    yield ml_inference_pb2.VideoFrame(
                        frame_data=frame_rgb.tobytes(),
                        format="rgb",
                        width=frame_rgb.shape[1],
                        height=frame_rgb.shape[0],
                        timestamp_ms=int(time.time() * 1000),
                        frame_number=frame_number,
                        metadata={"source": "screen"}
                    )
                    
                    frame_number += 1
                    
                    # Maintain target FPS
                    elapsed = time.time() - start_time
                    if elapsed < frame_interval:
                        await asyncio.sleep(frame_interval - elapsed)
        
        except ImportError:
            logger.error("mss library not installed: pip install mss")
        except Exception as e:
            logger.error(f"Error capturing screen: {e}", exc_info=True)
    
    async def read_video_file(
        self,
        file_path: str,
        start_frame: int = 0,
        end_frame: Optional[int] = None
    ) -> AsyncGenerator:
        """
        Read video file frame by frame.
        
        Args:
            file_path: Path to video file
            start_frame: Starting frame number
            end_frame: Ending frame number (None = all frames)
        
        Yields:
            ml_inference_pb2.VideoFrame objects
        """
        try:
            import cv2
            from generated import ml_inference_pb2
            
            cap = cv2.VideoCapture(file_path)
            
            if not cap.isOpened():
                raise RuntimeError(f"Failed to open video file: {file_path}")
            
            total_frames = int(cap.get(cv2.CAP_PROP_FRAME_COUNT))
            fps = cap.get(cv2.CAP_PROP_FPS)
            
            logger.info(f"Video file: {file_path} ({total_frames} frames @ {fps}fps)")
            
            # Seek to start frame
            cap.set(cv2.CAP_PROP_POS_FRAMES, start_frame)
            
            frame_number = start_frame
            
            while True:
                if end_frame is not None and frame_number >= end_frame:
                    break
                
                ret, frame = cap.read()
                if not ret:
                    break
                
                # Convert BGR to RGB
                frame_rgb = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)
                
                # Calculate timestamp from frame number and FPS
                timestamp_ms = int((frame_number / fps) * 1000)
                
                yield ml_inference_pb2.VideoFrame(
                    frame_data=frame_rgb.tobytes(),
                    format="rgb",
                    width=frame.shape[1],
                    height=frame.shape[0],
                    timestamp_ms=timestamp_ms,
                    frame_number=frame_number,
                    metadata={"source": "file", "file_path": file_path}
                )
                
                frame_number += 1
            
            cap.release()
            logger.info(f"Finished reading video file: {frame_number - start_frame} frames")
        
        except Exception as e:
            logger.error(f"Error reading video file: {e}", exc_info=True)


# Example usage from Rust:
#
# 1. WebRTC Stream (Rust → Python):
#    - Rust opens WebRTC data channel to Python gRPC
#    - Rust sends video frames in custom format
#    - Python StreamHandler.handle_webrtc_stream() converts to VideoFrame
#    - Python forwards to MediaPipe streaming RPCs
#
# 2. System Capture (Rust → Python):
#    - Rust calls Python gRPC to start capture
#    - Python StreamHandler.capture_camera() or .capture_screen()
#    - Python forwards directly to MediaPipe
#    - Python streams results back to Rust via gRPC
#
# 3. Native Messaging (Extension → Rust → Python):
#    - Extension captures tab video
#    - Sends to Rust native messaging
#    - Rust forwards to Python via gRPC
#    - Python processes and returns results

