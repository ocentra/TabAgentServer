# Python ML Bridge (python-ml-bridge) - TODO

**Rust → Python calls for ML models (reverse direction)**

## ✅ Phase 6 Complete (Current State)

- [x] PyO3 integration
- [x] MlBridge trait implementation
- [x] Python ml_funcs module
- [x] Embedding generation (sentence-transformers)
- [x] Entity extraction (spaCy)
- [x] Summarization (BART)
- [x] Error handling and type conversion
- [x] Async support via spawn_blocking
- [x] Comprehensive tests (30 tests passing: 2 unit + 28 PyO3 integration + 1 doc)
- [x] Documentation

## 🔄 Phase 6.5: Enhancements (Optional)

### Alternative Backends
- [ ] **ONNX Runtime Support**:
  - Implement `OnnxMlBridge` for native Rust inference
  - No Python dependency
  - Faster startup, lower memory

- [ ] **OpenAI API Support**:
  - Implement `OpenAiMlBridge` for cloud inference
  - High-quality embeddings (text-embedding-3-small: 1536 dims)
  - GPT-4 for summarization

- [ ] **Local LLM Support**:
  - Llama.cpp integration
  - Phi-3, Mistral for summarization
  - Fully offline capable

### Performance
- [ ] **Batch Inference**:
  - `generate_embeddings_batch(&self, texts: &[String]) -> Vec<Vec<f32>>`
  - ~10x faster for bulk operations
  - Utilize GPU batching

- [ ] **Model Caching**:
  - Load models on first use (lazy)
  - Configurable model unloading
  - Memory pressure monitoring

- [ ] **GPU Support**:
  - Auto-detect CUDA/ROCm
  - Device selection
  - Multi-GPU support

## 📋 Testing & Quality

- [ ] Integration tests with actual Python models
- [ ] Performance benchmarks
- [ ] Memory leak tests (long-running)
- [ ] Error recovery tests
- [ ] Python exception handling improvements

## 🚀 Future Features

### Model Management
- [ ] Model versioning and updates
- [ ] Model download/cache management
- [ ] Multiple model support (switch per request)
- [ ] Model metrics (latency, throughput)

### Advanced Features
- [ ] Streaming embeddings for long texts
- [ ] Multilingual support (different spaCy models)
- [ ] Custom fine-tuned model support
- [ ] Model quantization for speed/size

## 🐛 Known Issues

- PyO3 0.20 requires compatibility flag for Python 3.13
- First model load is slow (~1-5 seconds)
- BART model is large (~1.6GB) - consider distilled version
- No GPU support yet (all CPU inference)

## 📝 Notes

- Python models are global/cached - safe for multithreading
- `spawn_blocking` used for CPU-bound Python calls
- Consider `rayon` for parallel batch processing
- Future: Explore `pyo3-asyncio` for better async integration

