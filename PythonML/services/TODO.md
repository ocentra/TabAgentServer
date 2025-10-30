# Services - TODO

## Current State

✅ **Implemented**:
- ModelManagementService (load/unload/file serving)
- TransformersService (text generation, embeddings, chat)
- MediapipeService (all streaming vision endpoints)
- Proper error handling with gRPC status codes
- Lazy module initialization

⚙️ **Tested**:
- Single-frame MediaPipe processing
- Model loading/unloading
- File serving via RustFileProvider

---

## Planned Features

### High Priority

- [ ] **Token Streaming (TransformersService)**
  - Use TextIteratorStreamer from transformers
  - True token-by-token streaming (not just full text)
  - Proper done signaling

- [ ] **LiteRT Service**
  - New gRPC service for quantized models
  - Load .tflite models
  - Inference with XNNPACK/GPU delegates

- [ ] **Error Recovery**
  - Graceful degradation on model load failure
  - Automatic unload on OOM
  - Request timeout handling

### Medium Priority

- [ ] **Performance Metrics**
  - Request latency tracking
  - Token throughput monitoring
  - FPS for vision streams
  - Memory usage per model

- [ ] **Batch Processing**
  - Batch embeddings (multiple texts)
  - Batch image processing
  - Dynamic batching for efficiency

- [ ] **Advanced MediaPipe**
  - Object detection (requires .tflite models)
  - Object tracking with trajectories
  - Template matching
  - AutoFlip (intelligent cropping)

### Low Priority

- [ ] **Health Checks**
  - Service health endpoint
  - Model warmup verification
  - GPU availability check

- [ ] **Rate Limiting**
  - Per-client request limits
  - Queue management
  - Backpressure handling

- [ ] **Caching**
  - Cache embedding results
  - Deduplicate identical requests
  - TTL-based invalidation

---

## Known Issues

- **Token streaming**: Currently returns full text, not token-by-token
- **H264 streams**: Decoding not implemented, passed as raw bytes
- **Large models**: No automatic offloading if VRAM insufficient
- **Concurrent requests**: No request queuing, may cause OOM

---

## Optimizations

- [ ] **Module Pooling**
  - Reuse initialized MediaPipe modules
  - Pool per-stream instances
  - Thread-safe access

- [ ] **Async Batching**
  - Accumulate requests for batch processing
  - Configurable batch size/timeout
  - Fairness across clients

- [ ] **GPU Sharing**
  - Multi-stream GPU inference
  - CUDA graph optimization
  - Memory pool management

---

## Documentation

- [ ] **API Examples**
  - Complete gRPC client examples (Python, Rust)
  - Streaming patterns
  - Error handling best practices

- [ ] **Performance Guide**
  - Optimal configuration per hardware
  - Benchmarking methodology
  - Tuning parameters

---

## Notes

- Services must remain thin - logic belongs in modules
- Always fail hard - Rust handles retry/fallback
- Status codes must match gRPC conventions
- Streaming endpoints must handle client disconnect gracefully

