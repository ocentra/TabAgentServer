# MediaPipe - TODO

## Current State

✅ **Fully Implemented**:
- Face detection (6 keypoints)
- Face mesh (468 landmarks + iris)
- Hand tracking (21 landmarks)
- Pose tracking (33 landmarks + world coordinates)
- Holistic tracking (543 combined landmarks)
- Iris tracking (gaze estimation)
- Segmentation (person/background + effects)

✅ **Features**:
- Single-frame processing
- Async streaming support
- Gesture recognition (7 gestures)
- Joint angle calculation
- Gaze direction estimation
- Background blur/replacement
- Foreground extraction with alpha

⚙️ **Tested**:
- All modules with unit tests
- Integration tests
- Real camera/video streaming

---

## Planned Features

### High Priority

- [ ] **Object Detection**
  - Requires .tflite model file
  - COCO dataset (80 classes)
  - Bounding boxes + labels + confidence
  - Real-time detection (30+ FPS)

- [ ] **Object Tracking**
  - Track detected objects across frames
  - Assign unique IDs
  - Trajectory history
  - Handle occlusion

- [ ] **Multi-Object Tracking**
  - Track multiple objects simultaneously
  - Re-identification after occlusion
  - Kalman filter for smooth tracking

### Medium Priority

- [ ] **Template Matching**
  - Find template in video frames
  - Multi-scale matching
  - Rotation invariance (optional)
  - Real-time performance

- [ ] **AutoFlip**
  - Intelligent video cropping
  - Follow region of interest
  - Smooth transitions
  - Aspect ratio conversion

- [ ] **Advanced Segmentation**
  - Multi-class segmentation
  - Clothing/accessories segmentation
  - Real-time matting

- [ ] **Action Recognition**
  - Detect predefined actions (wave, clap, etc.)
  - Pose-based action classification
  - Temporal modeling

### Low Priority

- [ ] **3D Object Detection (Objectron)**
  - NOTE: Google ended support for Objectron
  - Alternative: Use other 3D detection methods
  - Consider ARKit/ARCore integration

- [ ] **Media Sequence Processing**
  - Batch video analysis
  - Activity recognition
  - Scene understanding

- [ ] **Face Effects**
  - AR face filters
  - Face morphing
  - Expression transfer

- [ ] **Hand Gesture Custom Training**
  - Train custom gestures
  - User-specific gesture profiles
  - Online learning

---

## Known Issues

- **H264/VP8 streams**: Not decoded yet (passed as raw bytes)
- **High-res images**: Performance degrades >1080p (recommend downscaling)
- **Multiple faces**: Face mesh limited to 2 faces max (configurable)
- **Lighting**: Poor lighting affects detection quality

---

## Optimizations

- [ ] **GPU Acceleration**
  - Investigate MediaPipe GPU inference
  - CUDA/TensorRT backends
  - Batch processing on GPU

- [ ] **Model Quantization**
  - Use quantized MediaPipe models
  - Reduce memory footprint
  - Faster inference

- [ ] **Multi-threading**
  - Parallel processing of multiple streams
  - Thread pool for inference
  - Lock-free queues

- [ ] **Memory Optimization**
  - Reuse buffers
  - Lazy initialization
  - Automatic cleanup on idle

---

## Hardware Support

✅ **Currently Supported**:
- CPU (all platforms)
- GPU (NVIDIA/AMD/Intel via OpenCL)

⚙️ **Planned**:
- NPU acceleration (AMD Ryzen AI, Intel Core Ultra)
- Apple Neural Engine (macOS)
- Qualcomm Hexagon DSP (Android)

---

## Documentation

- [ ] **Video Tutorials**
  - Getting started guide
  - Common use cases
  - Performance tuning

- [ ] **Cookbook**
  - Recipe examples for each module
  - Integration patterns
  - Best practices

- [ ] **Benchmark Suite**
  - Standardized benchmarks
  - Hardware comparison
  - FPS tracking across versions

---

## Integration

- [ ] **Rust Native**
  - Direct MediaPipe bindings in Rust
  - Avoid gRPC overhead for local inference
  - Investigate `mediapipe-rs` crate

- [ ] **Browser**
  - MediaPipe Web (WASM)
  - Client-side processing
  - Offload from server

- [ ] **Mobile**
  - Android MediaPipe SDK
  - iOS MediaPipe SDK
  - On-device inference

---

## Models to Add

### Vision
- [ ] Hair segmentation (dedicated model)
- [ ] Depth estimation
- [ ] Optical flow
- [ ] Image super-resolution

### Pose
- [ ] Full-body skeleton (40+ landmarks)
- [ ] Hand pose with finger joints
- [ ] Facial expression recognition

---

## Notes

- MediaPipe is actively developed by Google - monitor for new features
- `.task` bundles (MediaPipe GenAI) are separate from vision/pose models
- Consider migrating to MediaPipe Solutions API when stable
- Keep modules independent - don't create dependencies between them
- All modules must support both single-frame and streaming

---

## References

- **MediaPipe Solutions**: https://ai.google.dev/edge/mediapipe/solutions/guide
- **MediaPipe Source**: https://github.com/google/mediapipe
- **Desktop Examples**: `External/mediapipe/mediapipe/examples/desktop/`

