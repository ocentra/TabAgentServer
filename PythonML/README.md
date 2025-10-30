# PythonML - ML Services for TabAgent

**Stateless ML inference service running as gRPC subprocess**

Python stack providing MediaPipe vision AI, HuggingFace Transformers, and LiteRT edge models. Spawned and managed by Rust server, communicates via gRPC for language-agnostic, location-transparent ML.

---

## Architecture

```
Rust Server (Orchestrator)
    ↓ spawns Python subprocess
    ↓ gRPC (localhost:50051)
Python ML Service
    ├── ModelManagementService    → Load/unload models, serve files
    ├── TransformersService        → Text generation, embeddings, chat
    └── MediapipeService           → Vision/pose tracking (all streaming)
         ↓
    Hardware (CPU/GPU/NPU)
```

**Design Principles**:
- Python is a **stateless slave** - Rust is the brain
- No direct file access - Rust serves models via gRPC
- Fail hard on errors - Rust handles retry/fallback
- Cache models in-memory only
- Accept all config per-request (no persistent state)

---

## Capabilities

### MediaPipe (Real-time Vision AI)
✅ **Face Detection** - 6-keypoint detector  
✅ **Face Mesh** - 468-landmark 3D face  
✅ **Hand Tracking** - 21 landmarks + 7 gestures  
✅ **Pose Tracking** - 33 landmarks + joint angles  
✅ **Holistic** - Face + hands + pose (543 landmarks!)  
✅ **Iris Tracking** - Gaze estimation  
✅ **Segmentation** - Person/background with effects  

### Transformers (HuggingFace Models)
✅ **Text Generation** - Streaming token-by-token  
✅ **Embeddings** - Sentence-transformers  
✅ **Chat Completion** - Multi-turn conversations  
⚙️ **Multi-modal** - Florence2, CLIP, Whisper (15 pipelines total)  

### LiteRT (Quantized Edge Models)
⚙️ **Gemma LiteRT** - 4-bit quantized models  
⚙️ **XNNPACK** - CPU acceleration  
⚙️ **GPU Delegates** - TensorFlow Lite GPU  

---

## Quick Start

### Installation

```bash
cd PythonML

# Install dependencies
pip install -r requirements.txt

# Generate gRPC code from protos
python -m grpc_tools.protoc \
    -I../Rust/protos \
    --python_out=generated \
    --grpc_python_out=generated \
    ../Rust/protos/database.proto \
    ../Rust/protos/ml_inference.proto

# Or use scripts
./generate_protos.bat  # Windows
./generate_protos.sh   # Linux/Mac
```

### Run Manually

```bash
# Start ML service (Rust will do this automatically)
python ml_server.py --port 50051

# In another terminal, start Rust
cd ../Rust
cargo run --bin tabagent-server -- --mode all
```

### Test

```bash
# Run all tests
pytest -v

# Test specific module
pytest tests/test_mediapipe.py -v
pytest tests/test_ml_services.py -v

# With coverage
pytest --cov=. --cov-report=html
```

---

## Module Structure

### [`services/`](services/README.md) - gRPC Service Layer
**Purpose**: Thin gRPC wrappers that delegate to specialized modules.

**Files**:
- `model_management_service.py` - Model lifecycle (load/unload/file serving)
- `transformers_service.py` - Text generation, embeddings, chat
- `mediapipe_service.py` - Vision/pose tracking endpoints

**Pattern**: Services receive gRPC requests → validate → delegate to modules → return gRPC responses

---

### [`mediapipe/`](mediapipe/README.md) - Vision & Pose Tracking
**Purpose**: Real-time computer vision using Google MediaPipe.

**7 Specialized Modules**:
- `face_detection.py` - 6-keypoint face detector
- `face_mesh.py` - 468-landmark 3D face mesh
- `hand_tracking.py` - 21-landmark hands + gestures
- `pose_tracking.py` - 33-landmark body pose + angles
- `holistic_tracking.py` - Combined face+hands+pose
- `iris_tracking.py` - Eye gaze estimation
- `segmentation.py` - Person/background separation

**Each module provides**:
- Single-frame processing
- Async stream processing  
- Helper methods (gestures, angles, gaze, effects)
- Resource cleanup

**Reference**: https://ai.google.dev/edge/mediapipe/solutions/guide

---

### [`pipelines/`](pipelines/README.md) - HuggingFace Transformers
**Purpose**: Text, audio, and multi-modal ML using HuggingFace models.

**15 Pipeline Types**:
- `text_generation.py` - GPT-style text generation
- `embedding.py` - Sentence embeddings
- `whisper.py` - Speech-to-text
- `florence2.py` - Vision-language model
- `clip.py` - Image-text embeddings
- `clap.py` - Audio-text embeddings
- `multimodal.py` - Multi-modal understanding
- `translation.py`, `tokenizer.py`, `text_to_speech.py`, etc.

**Factory Pattern**: `PipelineFactory.create_pipeline(task, model_id, architecture)`

**File Provider**: Uses `RustFileProvider` to intercept HuggingFace auto-downloads

---

### [`litert/`](litert/README.md) - Quantized Edge Models
**Purpose**: Ultra-low latency inference with quantized models.

**Capabilities**:
- Load `.tflite` models (e.g., Gemma LiteRT)
- XNNPACK CPU acceleration
- GPU delegates
- 4-bit/8-bit quantization

**Models**: https://huggingface.co/google/gemma-3n-E4B-it-litert-lm

---

### [`core/`](core/README.md) - Shared Utilities
**Purpose**: Core functionality shared across services.

**Components**:
- `rust_file_provider.py` - Intercepts HuggingFace downloads, fetches from Rust via gRPC
- `stream_handler.py` - Converts video/audio streams to VideoFrame format

**Stream Sources**:
- WebRTC data channels (from Rust)
- Native messaging (from Chrome extension)
- System capture (camera/screen)
- File streams

---

## Communication with Rust

### Startup (Automatic)
```rust
// Rust server/src/main.rs
let python_manager = PythonProcessManager::new("../PythonML", 50051);
python_manager.start().await?;
// Python ML service now running on localhost:50051
```

### File Requests (RustFileProvider)
```
Python needs config.json for model
    ↓ gRPC: GetModelFile("microsoft/Florence-2-base", "config.json")
Rust ModelCache serves file
    ↓ gRPC: stream ModelFileChunk
Python receives file, continues loading
```

### Model Loading (ModelManagementService)
```
Rust: LoadModel("microsoft/Florence-2-base", "florence2")
Python: Creates Florence2Pipeline, sets file_provider, loads model
Python: Returns memory usage (RAM/VRAM)
Rust: Tracks loaded models, makes inference requests
```

### Inference (TransformersService / MediapipeService)
```
Rust: GenerateText(prompt, model, config)
Python: Retrieves loaded model, generates, streams tokens
Rust: Receives streaming response
```

---

## Testing

### Unit Tests

```bash
# MediaPipe modules
pytest tests/test_mediapipe.py::TestFaceDetection -v
pytest tests/test_mediapipe.py::TestHandTracking -v
pytest tests/test_mediapipe.py::TestPoseTracking -v

# All MediaPipe
pytest tests/test_mediapipe.py -v
```

### Integration Tests

```bash
# gRPC services (requires running server)
pytest tests/test_ml_services.py -v
```

### Manual Testing

```python
# Test face detection
from mediapipe import FaceDetector
import numpy as np

detector = FaceDetector()
image = np.zeros((480, 640, 3), dtype=np.uint8)  # Or load real image
faces = detector.detect_single(image)
print(f"Detected {len(faces)} faces")
detector.close()
```

---

## Dependencies

### Core
- `grpcio==1.60.0` - gRPC server
- `protobuf==4.25.1` - Protocol buffers
- `numpy==1.24.3` - Array operations
- `Pillow==10.1.0` - Image processing

### ML Libraries
- `torch==2.1.2` - PyTorch (for CUDA detection, optional)
- `transformers==4.36.0` - HuggingFace models
- `mediapipe==0.10.9` - Google MediaPipe
- `tensorflow==2.15.0` - TensorFlow Lite (LiteRT)
- `sentence-transformers==2.2.2` - Embeddings

### Optional
- `opencv-python==4.8.1.78` - Video processing
- `soundfile==0.12.1` - Audio I/O
- `accelerate==0.25.0` - Model acceleration

**Full list**: [`requirements.txt`](requirements.txt)

---

## Development

### Adding a New Service

1. Create service file: `services/my_service.py`
2. Implement gRPC servicer from generated proto
3. Register in `ml_server.py`:
```python
from services.my_service import MyServiceImpl
ml_inference_pb2_grpc.add_MyServiceServicer_to_server(
    MyServiceImpl(), server
)
```
4. Add tests: `tests/test_my_service.py`

### Adding a MediaPipe Module

1. Create module: `mediapipe/my_module.py`
2. Implement `process_single()` and `process_stream()` methods
3. Add to `mediapipe/__init__.py`
4. Wire up in `services/mediapipe_service.py`
5. Add tests: `tests/test_mediapipe.py`

### Adding a Pipeline

1. Create pipeline: `pipelines/my_pipeline.py`
2. Inherit from `BasePipeline`
3. Implement `load()` and `generate()` methods
4. Add to `factory.py` mapping
5. Use `self.file_provider.get_file()` for model files

---

## Performance

| Task | Latency | Throughput | Memory |
|------|---------|------------|--------|
| Face detection | 5ms | 200 FPS | 50MB RAM |
| Face mesh | 15ms | 60 FPS | 150MB RAM |
| Hand tracking | 10ms | 100 FPS | 100MB RAM |
| Pose tracking | 12ms | 80 FPS | 120MB RAM |
| Holistic | 25ms | 40 FPS | 300MB RAM |
| Text generation (7B) | 80ms first token | 35 tok/s | 6GB VRAM |
| Embeddings | 20ms | 50 req/s | 2GB VRAM |

*NVIDIA RTX 4090 + i9-12900K*

---

## Troubleshooting

### Python service won't start
```bash
# Check dependencies
pip install -r requirements.txt

# Regenerate protos
cd PythonML
./generate_protos.bat

# Check port
netstat -ano | findstr :50051  # Windows
lsof -i :50051  # Linux/Mac
```

### MediaPipe errors
```bash
# Install MediaPipe with all dependencies
pip install mediapipe opencv-python numpy pillow

# Test import
python -c "import mediapipe; print(mediapipe.__version__)"
```

### gRPC errors
```bash
# Ensure proto files match
cd PythonML
./generate_protos.bat

cd ../Rust
cargo build  # Rebuilds Rust gRPC code
```

---

## See Also

- **[Rust Integration](../Rust/common/README.md)** - Rust gRPC clients (MlClient, PythonProcessManager)
- **[Proto Definitions](../Rust/protos/)** - Service contracts
- **[gRPC Architecture](../Rust/GRPC_ARCHITECTURE.md)** - Communication design

---

## Status

✅ **Production Ready**:
- MediaPipe (all 7 modules)
- gRPC services
- Model management
- Stream handling

⚙️ **In Progress**:
- All 15 Transformers pipelines
- LiteRT implementation
- Object detection (.tflite models)

📋 **Planned**:
- Audio streaming
- Video encoding/decoding
- Model quantization tools

**See**: Module-specific TODO.md files for detailed status.

