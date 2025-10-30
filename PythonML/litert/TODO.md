# LiteRT - TODO

## Current State

⚙️ **Structure**:
- Basic manager class created
- Load/generate/unload methods defined

🔴 **Incomplete**:
- load_model() is a stub
- generate() is a stub
- No delegate support
- No tokenization utilities

---

## Implementation Priority

### Phase 1: Core Functionality (High Priority)

- [ ] **Model Loading**
  - Implement TFLite Interpreter
  - Load .tflite models
  - Allocate tensors
  - Verify input/output shapes

- [ ] **Inference**
  - Set input tensors
  - Run interpreter.invoke()
  - Get output tensors
  - Handle batching

- [ ] **Resource Management**
  - Proper cleanup
  - Memory profiling
  - Multi-model support

### Phase 2: Acceleration (High Priority)

- [ ] **XNNPACK Delegate**
  - Enable XNNPACK
  - Verify CPU acceleration
  - Benchmark performance

- [ ] **GPU Delegate**
  - Detect GPU availability
  - Load GPU delegate
  - Fallback to CPU if unavailable

- [ ] **Thread Optimization**
  - Optimal thread count detection
  - Per-model thread config
  - CPU affinity

### Phase 3: Gemma LiteRT Support (High Priority)

- [ ] **Tokenization**
  - SentencePiece tokenizer
  - Encode/decode utilities
  - Special tokens handling

- [ ] **Text Generation**
  - Autoregressive generation
  - Top-k/top-p sampling
  - Temperature control
  - Stop tokens

- [ ] **Streaming**
  - Token-by-token generation
  - Async streaming
  - Early stopping

### Phase 4: Advanced Features (Medium Priority)

- [ ] **NNAPI Delegate** (Android)
  - Detect NNAPI availability
  - Load NNAPI delegate
  - Benchmark vs. XNNPACK

- [ ] **Core ML Delegate** (iOS)
  - Detect Neural Engine
  - Load Core ML delegate
  - Performance profiling

- [ ] **Hexagon Delegate** (Qualcomm)
  - QNN integration
  - DSP acceleration

- [ ] **Model Caching**
  - Cache compiled models
  - Lazy loading
  - Warmup optimization

### Phase 5: Utilities (Low Priority)

- [ ] **Model Conversion Tools**
  - PyTorch → TFLite
  - ONNX → TFLite
  - Quantization helpers

- [ ] **Benchmarking Suite**
  - Latency benchmarks
  - Memory profiling
  - Throughput tests
  - Hardware comparison

- [ ] **Model Zoo**
  - Pre-converted popular models
  - Download utilities
  - Version management

---

## Technical Debt

- [ ] **Error Handling**
  - Proper exception types
  - Graceful degradation
  - Detailed error messages

- [ ] **Type Hints**
  - Complete type annotations
  - Return type documentation

- [ ] **Logging**
  - Structured logging
  - Performance metrics
  - Debug mode

---

## Testing

- [ ] **Unit Tests**
  - Model loading
  - Inference correctness
  - Delegate selection
  - Error cases

- [ ] **Integration Tests**
  - With gRPC service
  - With Rust ModelCache
  - End-to-end workflow

- [ ] **Performance Tests**
  - Latency benchmarks
  - Throughput tests
  - Memory usage
  - CPU/GPU comparison

---

## Known Issues

- **Stub Implementation**: All methods are currently stubs
- **No Delegates**: XNNPACK/GPU not implemented
- **No Tokenization**: Manual tokenization required
- **No Streaming**: Batch-only inference

---

## Dependencies

```txt
# Core (already in requirements.txt)
tensorflow==2.15.0

# For model conversion
tensorflow-addons==0.23.0
tf2onnx==1.16.0

# For tokenization
sentencepiece==0.1.99

# For optimization
flatbuffers==23.5.26  # TFLite format
```

---

## Platform Support

✅ **Planned**:
- Windows (CPU + GPU via DirectML)
- Linux (CPU + GPU via CUDA/ROCm)
- macOS (CPU + Metal)
- Android (NNAPI + GPU)
- iOS (Core ML + Metal)

⚙️ **Current**:
- Basic structure only

---

## Model Support

### Gemma LiteRT
- [ ] Gemma 2B LiteRT
- [ ] Gemma 3B LiteRT (E4B quantization)
- [ ] Gemma 7B LiteRT
- [ ] Gemma 2 variants

### Other Models
- [ ] MobileBERT
- [ ] DistilBERT
- [ ] MobileNet (vision)
- [ ] EfficientNet (vision)
- [ ] Custom quantized models

---

## Performance Targets

| Hardware | Target Latency | Target Throughput |
|----------|----------------|-------------------|
| Desktop CPU | <100ms first token | 40+ tok/s |
| Mobile CPU (flagship) | <150ms first token | 25+ tok/s |
| Desktop GPU | <50ms first token | 80+ tok/s |
| Mobile GPU | <80ms first token | 40+ tok/s |
| NPU/Neural Engine | <60ms first token | 50+ tok/s |

---

## Documentation Needed

- [ ] **Conversion Guide**
  - PyTorch → TFLite step-by-step
  - Quantization best practices
  - Testing converted models

- [ ] **Deployment Guide**
  - Mobile deployment (Android/iOS)
  - Edge device deployment
  - Cloud deployment

- [ ] **Troubleshooting**
  - Common conversion errors
  - Delegate selection issues
  - Performance debugging

---

## References

- **LiteRT**: https://ai.google.dev/edge/litert
- **TensorFlow Lite**: https://www.tensorflow.org/lite
- **Gemma LiteRT**: https://huggingface.co/collections/google/gemma-litert-models
- **XNNPACK**: https://github.com/google/XNNPACK
- **Model Conversion**: https://www.tensorflow.org/lite/models/convert

---

## Notes

- LiteRT is TensorFlow Lite rebranded for Google AI Edge
- Focus on Gemma LiteRT models first (official support)
- Delegates are critical for performance - prioritize XNNPACK
- Consider using TFLite Model Maker for custom models
- Test on real edge devices, not just desktop

