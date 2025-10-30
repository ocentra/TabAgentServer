# Pipelines - TODO

## Current State

✅ **Structure**:
- Factory pattern implemented
- BasePipeline abstract class
- 15 pipeline files created
- Types defined (PipelineTask enum)

⚙️ **In Progress**:
- Full implementation of each pipeline
- RustFileProvider integration
- Model loading with file provider

🔴 **Needs Work**:
- All pipelines are currently stubs
- Need to implement load() and generate() methods
- Test coverage

---

## Implementation Priority

### Phase 1: Core Text Pipelines (High Priority)

- [ ] **Text Generation**
  - Implement with transformers.AutoModelForCausalLM
  - Support streaming with TextIteratorStreamer
  - Handle generation config properly

- [ ] **Embeddings**
  - Use sentence-transformers
  - Batch processing support
  - Normalization option

- [ ] **Chat Completion**
  - Apply chat templates
  - Multi-turn conversation support
  - System prompts

### Phase 2: Vision-Language (High Priority)

- [ ] **Florence2**
  - Object detection
  - Image captioning
  - OCR
  - Region understanding

- [ ] **CLIP**
  - Zero-shot image classification
  - Image-text similarity
  - Multimodal embeddings

- [ ] **Multimodal (LLaVA, Qwen-VL)**
  - Visual question answering
  - Image reasoning
  - Multi-image support

### Phase 3: Audio (Medium Priority)

- [ ] **Whisper**
  - Transcription
  - Translation
  - Language detection
  - Timestamps

- [ ] **CLAP**
  - Audio-text similarity
  - Zero-shot audio classification

- [ ] **Text-to-Speech**
  - Speech synthesis
  - Voice cloning (if model supports)

### Phase 4: Specialized (Medium Priority)

- [ ] **Translation**
  - Language pair support
  - Batch translation

- [ ] **Code Completion**
  - Code generation
  - Infilling
  - Multi-language support

- [ ] **Zero-Shot Classification**
  - Custom labels
  - Multi-label support

- [ ] **Cross-Encoder**
  - Semantic similarity
  - Re-ranking

### Phase 5: Advanced (Low Priority)

- [ ] **Image Classification**
  - Standard classifiers
  - Custom fine-tuned models

- [ ] **Janus**
  - Unified multimodal processing

- [ ] **Tokenizer**
  - Standalone tokenization
  - Token counting utilities

---

## Technical Tasks

### RustFileProvider Integration

- [ ] **Implement file interception**
  - Hook into transformers file download
  - Redirect to RustFileProvider
  - Handle caching properly

- [ ] **Error Handling**
  - Fallback to direct download if Rust unavailable
  - Retry logic
  - Timeout handling

- [ ] **Testing**
  - Mock RustFileProvider for unit tests
  - Integration tests with real Rust server
  - Verify no duplicate downloads

### Model Loading

- [ ] **Lazy Loading**
  - Load model on first use
  - Unload on idle timeout
  - Memory management

- [ ] **Quantization Support**
  - 8-bit, 4-bit quantization
  - GPTQ, AWQ, GGUF formats
  - Device mapping for large models

- [ ] **Multi-GPU Support**
  - Model parallelism
  - Pipeline parallelism
  - Automatic device assignment

### Inference Optimization

- [ ] **Batching**
  - Dynamic batching
  - Padding strategies
  - Throughput optimization

- [ ] **Streaming**
  - Token-by-token streaming for text
  - Chunk-by-chunk for audio
  - Progress callbacks

- [ ] **Caching**
  - KV cache for text generation
  - Result caching for deterministic inputs
  - Prompt caching

---

## Known Issues

- **Stubs**: All pipelines are currently stubs, not functional
- **No File Provider Integration**: Models will try to download directly
- **No Tests**: Unit/integration tests need to be written
- **No Error Handling**: Need proper exception handling

---

## Future Enhancements

### Advanced Features

- [ ] **LoRA Support**
  - Load LoRA adapters
  - Switch adapters dynamically
  - Merge adapters

- [ ] **Quantization Tools**
  - Quantize models on-the-fly
  - Convert formats (GPTQ ↔ AWQ)
  - Benchmark quantization quality

- [ ] **Model Ensemble**
  - Combine multiple models
  - Weighted averaging
  - Voting strategies

### Performance

- [ ] **Benchmark Suite**
  - Latency benchmarks per pipeline
  - Throughput benchmarks
  - Memory profiling

- [ ] **Optimization Guide**
  - Per-model tuning parameters
  - Hardware-specific configs
  - Best practices

---

## Dependencies to Add

```txt
# Core (already in requirements.txt)
transformers==4.36.0
torch==2.1.2
sentence-transformers==2.2.2

# For specific pipelines
# Audio
soundfile==0.12.1
librosa==0.10.1

# Vision
timm==0.9.12  # Image models
diffusers==0.25.0  # Image generation

# Optimization
optimum==1.16.1  # Model optimization
bitsandbytes==0.41.3  # Quantization
accelerate==0.25.0  # Multi-GPU

# Advanced
peft==0.7.1  # LoRA support
```

---

## Testing Strategy

### Unit Tests
- Mock RustFileProvider
- Test each pipeline in isolation
- Verify input/output formats

### Integration Tests
- Requires running Rust server
- Test with real models (small ones)
- Verify file provider integration

### End-to-End Tests
- Full workflow: Rust → Python → inference → results
- Multiple pipelines concurrently
- Error scenarios

---

## Documentation Needed

- [ ] **Per-Pipeline Guides**
  - Supported models
  - Configuration options
  - Example usage
  - Performance tips

- [ ] **Migration Guide**
  - From HuggingFace pipelines
  - From direct model usage
  - From other frameworks

- [ ] **Troubleshooting**
  - Common errors
  - Model compatibility
  - Memory issues

---

## Notes

- Keep pipelines independent - no cross-dependencies
- All file access must go through RustFileProvider
- Fail hard on errors - let Rust handle retry
- Support both sync and async inference where possible
- Document memory requirements per model size

