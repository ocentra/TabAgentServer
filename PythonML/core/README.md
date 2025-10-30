# Core - Shared ML Utilities

**Foundation layer for ML services**

Provides file handling and stream conversion utilities shared across all ML services.

---

## Components

### `rust_file_provider.py` - Model File Provider

**Purpose**: Intercepts HuggingFace model downloads and fetches from Rust's ModelCache via gRPC.

**Why**: 
- Prevents duplicate downloads (Rust already manages models)
- Ensures consistent model versions
- Mirrors browser extension's IndexedDB caching pattern
- Enables offline operation

**How It Works**:
```
HuggingFace Transformers needs config.json
    ↓ (intercept)
RustFileProvider.get_file("microsoft/Florence-2-base", "config.json")
    ↓ (gRPC: GetModelFile)
Rust ModelCache serves file (streaming)
    ↓ (gRPC: stream ModelFileChunk)
RustFileProvider assembles chunks
    ↓ (return bytes)
HuggingFace continues loading
```

**Usage**:
```python
from core import RustFileProvider
import grpc

# Connect to Rust gRPC server
channel = grpc.aio.insecure_channel('localhost:50052')
provider = RustFileProvider(channel)

# Fetch file
file_data = provider.get_file(
    model_id="microsoft/Florence-2-base",
    file_path="config.json"
)

# Use with pipelines
pipeline.file_provider = provider
pipeline.load(model_id="microsoft/Florence-2-base")
```

**Interface**:
```python
class RustFileProvider:
    def __init__(self, rust_grpc_channel: grpc.aio.Channel)
    
    def get_file(self, model_id: str, file_path: str) -> bytes
        """Fetch file from Rust ModelCache (streaming)"""
```

---

### `stream_handler.py` - Video/Audio Stream Converter

**Purpose**: Converts various stream sources to unified `VideoFrame` format for MediaPipe processing.

**Supported Sources**:
1. **WebRTC Data Channels** - From Rust server
2. **Native Messaging** - From Chrome extension  
3. **System Capture** - Camera/screen via OpenCV/mss
4. **File Streams** - Video files

**Format**: All converted to `ml_inference_pb2.VideoFrame`:
```protobuf
message VideoFrame {
    bytes frame_data = 1;
    string format = 2;        // "rgb", "bgr", "h264", "jpeg"
    int32 width = 3;
    int32 height = 4;
    int64 timestamp_ms = 5;
    int32 frame_number = 6;
    map<string, string> metadata = 7;
}
```

**WebRTC Stream Protocol**:
```
Header (17 bytes):
    [0-7]   timestamp_ms (uint64)
    [8-11]  width (uint32)
    [12-15] height (uint32)
    [16]    format (uint8: 0=RGB, 1=BGR, 2=H264, 3=VP8, 4=JPEG)
    [17+]   frame_data
```

**Usage**:
```python
from core import StreamHandler

handler = StreamHandler()

# WebRTC stream from Rust
async for frame in handler.handle_webrtc_stream(
    stream_id="cam-001",
    data_channel=webrtc_channel
):
    # frame is VideoFrame, ready for MediaPipe
    results = mediapipe_service.process(frame)

# System camera
async for frame in handler.capture_camera(
    camera_id=0,
    width=640,
    height=480,
    fps=30
):
    # Process frame
    pass

# Screen capture
async for frame in handler.capture_screen(
    region=(0, 0, 1920, 1080),
    fps=30
):
    # Process frame
    pass

# Video file
async for frame in handler.read_video_file(
    file_path="video.mp4",
    start_frame=0,
    end_frame=100
):
    # Process frame
    pass
```

**Interface**:
```python
class StreamHandler:
    async def handle_webrtc_stream(
        stream_id: str,
        data_channel: AsyncGenerator[bytes, None]
    ) -> AsyncGenerator[VideoFrame, None]
    
    async def handle_native_stream(
        stream_id: str,
        pipe_reader: AsyncGenerator[bytes, None]
    ) -> AsyncGenerator[VideoFrame, None]
    
    async def capture_camera(
        camera_id: int = 0,
        width: int = 640,
        height: int = 480,
        fps: int = 30
    ) -> AsyncGenerator[VideoFrame, None]
    
    async def capture_screen(
        region: Optional[tuple] = None,
        fps: int = 30
    ) -> AsyncGenerator[VideoFrame, None]
    
    async def read_video_file(
        file_path: str,
        start_frame: int = 0,
        end_frame: Optional[int] = None
    ) -> AsyncGenerator[VideoFrame, None]
```

---

## Dependencies

- `grpcio` - gRPC client
- `protobuf` - Protocol buffers
- `numpy` - Array operations
- `opencv-python` - Video I/O (optional, for system capture)
- `mss` - Screen capture (optional)

---

## Examples

### Complete Pipeline with File Provider

```python
from core import RustFileProvider
from pipelines import PipelineFactory
import grpc

# 1. Connect to Rust
channel = grpc.aio.insecure_channel('localhost:50052')
provider = RustFileProvider(channel)

# 2. Create pipeline
pipeline = PipelineFactory.create_pipeline(
    task="florence2",
    model_id="microsoft/Florence-2-base"
)

# 3. Set file provider (intercepts downloads)
pipeline.file_provider = provider

# 4. Load model (files fetched from Rust)
pipeline.load(model_id="microsoft/Florence-2-base")

# 5. Generate
result = pipeline.generate({"prompt": "Describe this image", "image": img})
```

### Real-time Camera → MediaPipe

```python
from core import StreamHandler
from mediapipe import FaceDetector

handler = StreamHandler()
detector = FaceDetector()

async def process_camera():
    async for frame in handler.capture_camera(fps=30):
        # Decode frame
        img = np.frombuffer(
            frame.frame_data, 
            dtype=np.uint8
        ).reshape((frame.height, frame.width, 3))
        
        # Detect faces
        faces = detector.detect_single(img)
        print(f"Frame {frame.frame_number}: {len(faces)} faces")

await process_camera()
```

---

## Testing

```bash
# Unit tests
pytest tests/test_core.py -v

# Integration tests (requires Rust server running)
pytest tests/test_file_provider.py -v
```

---

## See Also

- **[ModelCache (Rust)](../../Rust/model-cache/README.md)** - File serving implementation
- **[MlClient (Rust)](../../Rust/common/README.md)** - Rust gRPC client
- **[Services](../services/README.md)** - How services use these utilities

