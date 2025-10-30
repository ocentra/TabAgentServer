# Services - gRPC Service Layer

**Thin gRPC wrappers connecting Rust to ML modules**

Service layer that receives gRPC requests from Rust and delegates to specialized ML modules. Pure delegation - no business logic.

---

## Architecture

```
Rust gRPC Client
    ↓ (gRPC request)
Service Layer (this layer)
    ↓ (delegates to)
ML Modules (mediapipe/, pipelines/, litert/)
    ↓ (returns results)
Service Layer
    ↓ (gRPC response)
Rust
```

**Pattern**: Thin adapter layer, business logic stays in modules.

---

## Services

### `model_management_service.py` - Model Lifecycle

**Purpose**: Load/unload models, serve files from Rust cache.

**gRPC Methods**:
- `LoadModel(model_id, pipeline_type, architecture, options) → LoadModelResponse`
  - Creates pipeline via PipelineFactory
  - Sets RustFileProvider for file access
  - Loads model into memory
  - Returns RAM/VRAM usage

- `UnloadModel(model_id) → StatusResponse`
  - Calls pipeline.unload()
  - Frees memory
  - Removes from tracking

- `GetModelFile(model_id, file_path) → stream ModelFileChunk`
  - Fetches file from Rust via file_provider
  - Streams back in chunks (100KB)
  - Used by Python when loading models

- `GetLoadedModels() → LoadedModelsResponse`
  - Returns list of loaded models
  - Includes pipeline type, memory usage, timestamp

**State Management**:
```python
self.loaded_models: Dict[str, BasePipeline] = {}
self.model_metadata: Dict[str, dict] = {}
self.file_provider: RustFileProvider = None
```

**Usage from Rust**:
```rust
// Load Florence2 model
let response = ml_client.load_model(
    "microsoft/Florence-2-base",
    "florence2",
    Some("Florence2"),
    HashMap::new()
).await?;

// Use for inference
let result = ml_client.generate_text(...).await?;

// Unload when done
ml_client.unload_model("microsoft/Florence-2-base").await?;
```

---

### `transformers_service.py` - Text & Multi-modal

**Purpose**: Text generation, embeddings, chat using HuggingFace models.

**gRPC Methods**:
- `GenerateText(prompt, model, config) → stream TextResponse`
  - Retrieves loaded pipeline
  - Calls pipeline.generate()
  - Streams tokens (TODO: proper streaming with TextIteratorStreamer)

- `GenerateEmbeddings(texts, model) → GeneratedEmbeddingsResponse`
  - Retrieves embedding pipeline
  - Calls pipeline.generate()
  - Returns float vectors

- `ChatCompletion(messages, model, temperature) → stream ChatResponse`
  - Retrieves chat pipeline
  - Formats messages
  - Streams response

**Design**:
- Does NOT load models (ModelManagementService does that)
- Retrieves loaded pipelines via `get_pipeline(model_id)`
- Validates pipeline types match request
- Delegates generation to pipeline

**Error Handling**:
```python
if not pipeline:
    context.set_code(grpc.StatusCode.FAILED_PRECONDITION)
    context.set_details("Model not loaded")
    return

if pipeline.pipeline_type() != expected_type:
    context.set_code(grpc.StatusCode.FAILED_PRECONDITION)
    context.set_details(f"Wrong pipeline type")
    return
```

---

### `mediapipe_service.py` - Vision & Pose Tracking

**Purpose**: Real-time vision AI streaming endpoints.

**gRPC Methods**:

**Legacy (single frame)**:
- `DetectFaces(ImageRequest) → FaceDetectionResponse`
- `DetectHands(ImageRequest) → HandDetectionResponse`
- `DetectPose(ImageRequest) → PoseDetectionResponse`

**Streaming (real-time)**:
- `StreamFaceDetection(stream VideoFrame) → stream FaceDetectionResponse`
- `StreamFaceMesh(stream VideoFrame) → stream FaceMeshResponse`
- `StreamHandTracking(stream VideoFrame) → stream HandDetectionResponse`
- `StreamPoseTracking(stream VideoFrame) → stream PoseDetectionResponse`
- `StreamHolisticTracking(stream VideoFrame) → stream HolisticTrackingResponse`
- `StreamIrisTracking(stream VideoFrame) → stream IrisTrackingResponse`
- `StreamSelfieSegmentation(stream VideoFrame) → stream SegmentationResponse`
- `StreamHairSegmentation(stream VideoFrame) → stream SegmentationResponse`

**Design**:
- Lazy-initializes MediaPipe modules on first use
- Decodes VideoFrame → numpy array
- Delegates to mediapipe modules (FaceDetector, HandTracker, etc.)
- Converts module output → protobuf messages
- Handles frame-by-frame streaming

**Frame Decoding**:
```python
def _decode_frame(self, frame: VideoFrame) -> np.ndarray:
    if frame.format in ["rgb", "bgr"]:
        # Raw pixel data
        return np.frombuffer(...).reshape(height, width, 3)
    elif frame.format in ["jpeg", "png"]:
        # Compressed image
        return cv2.imdecode(...)
    elif frame.format == "h264":
        # TODO: Video codec
        return None
```

**Streaming Pattern**:
```python
async def StreamFaceDetection(self, request_iterator, context):
    if not self._face_detector:
        self._face_detector = FaceDetector()
    
    async for frame in request_iterator:
        img = self._decode_frame(frame)
        faces = self._face_detector.detect_single(img)
        
        # Convert to protobuf
        pb_faces = [...]
        
        yield FaceDetectionResponse(
            faces=pb_faces,
            timestamp_ms=frame.timestamp_ms,
            frame_number=frame.frame_number
        )
```

---

## Service Registration

**In `ml_server.py`**:
```python
# Create service instances
model_mgmt = ModelManagementServiceImpl()
transformers = TransformersServiceImpl(model_mgmt)
mediapipe = MediapipeService()

# Register with gRPC server
ml_inference_pb2_grpc.add_ModelManagementServiceServicer_to_server(
    model_mgmt, server
)
ml_inference_pb2_grpc.add_TransformersServiceServicer_to_server(
    transformers, server
)
ml_inference_pb2_grpc.add_MediapipeServiceServicer_to_server(
    mediapipe, server
)
```

---

## Error Handling

**Pattern**: Fail hard, let Rust retry/handle.

```python
try:
    # Process request
    result = module.process(input)
    return Response(result)
except ValueError as e:
    # Client error
    context.set_code(grpc.StatusCode.INVALID_ARGUMENT)
    context.set_details(str(e))
    return EmptyResponse()
except RuntimeError as e:
    # Server error
    context.set_code(grpc.StatusCode.INTERNAL)
    context.set_details(str(e))
    return EmptyResponse()
```

**Status Codes Used**:
- `INVALID_ARGUMENT` - Bad input (missing fields, invalid format)
- `FAILED_PRECONDITION` - Model not loaded, wrong pipeline type
- `INTERNAL` - Unexpected errors
- `UNIMPLEMENTED` - Feature not yet implemented

---

## Testing

```bash
# Unit tests (mock modules)
pytest tests/test_services_unit.py -v

# Integration tests (requires running ml_server.py)
pytest tests/test_ml_services.py -v
```

---

## Adding a New Service

1. **Define proto** (`Rust/protos/ml_inference.proto`):
```protobuf
service MyService {
    rpc MyMethod(MyRequest) returns (MyResponse);
}
```

2. **Generate code**:
```bash
cd PythonML
./generate_protos.bat
```

3. **Implement service** (`services/my_service.py`):
```python
import ml_inference_pb2_grpc

class MyServiceImpl(ml_inference_pb2_grpc.MyServiceServicer):
    async def MyMethod(self, request, context):
        # Validate
        # Delegate to module
        # Return response
        pass
```

4. **Register** (`ml_server.py`):
```python
from services.my_service import MyServiceImpl
ml_inference_pb2_grpc.add_MyServiceServicer_to_server(
    MyServiceImpl(), server
)
```

5. **Test**:
```python
# tests/test_my_service.py
async def test_my_method():
    channel = grpc.aio.insecure_channel('localhost:50051')
    stub = ml_inference_pb2_grpc.MyServiceStub(channel)
    response = await stub.MyMethod(request)
    assert response.success
```

---

## See Also

- **[Proto Definitions](../../Rust/protos/)** - Service contracts
- **[MediaPipe Modules](../mediapipe/README.md)** - Vision/pose tracking
- **[Pipelines](../pipelines/README.md)** - HuggingFace models
- **[MlClient (Rust)](../../Rust/common/README.md)** - Rust gRPC client

