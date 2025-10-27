# TabAgent ONNX Loader

Pure Rust ONNX Runtime inference with automatic hardware acceleration.

## Features

- **Automatic Execution Provider Selection**: Detects available hardware (CUDA, TensorRT, DirectML, CoreML, OpenVINO, ROCm) and selects optimal providers
- **Integrated Tokenization**: HuggingFace tokenizers support via `tabagent-tokenization`
- **Thread-Safe**: Arc-wrapped session for safe cloning across threads
- **Text Generation**: Full inference pipeline with tokenization
- **Embeddings**: Single and batch embedding generation

## Architecture

```
onnx-loader/
├── src/
│   ├── lib.rs           # Public API exports
│   ├── session.rs       # OnnxSession - main inference interface
│   ├── providers.rs     # Execution provider detection & config
│   └── error.rs         # Error types
└── tests/
    └── integration/     # Integration tests
```

## Usage

```rust
use tabagent_onnx_loader::OnnxSession;

// Load model with auto-detected providers
let mut session = OnnxSession::load("model.onnx")?;

// Load tokenizer
session.load_tokenizer("tokenizer.json")?;

// Generate text
let output = session.generate("Hello, world!")?;

// Generate embeddings
let embedding = session.generate_embedding("Some text")?;
let batch = session.generate_embeddings(&["Text 1", "Text 2"])?;
```

## Dependencies

- **ort**: ONNX Runtime Rust bindings (v2.0.0-rc.10)
- **tabagent-tokenization**: HuggingFace tokenizers wrapper
- **tabagent-hardware**: Hardware detection for provider selection

## Current Status

✅ Complete API surface  
✅ Real ort::Session integration  
✅ Execution provider auto-selection  
✅ Tokenization integration  
✅ Thread-safe session management  
✅ Working embedding generation  
✅ Mean pooling for sentence embeddings  
✅ Integration tests with real models  

The crate is fully functional with real ONNX Runtime inference for embedding models. Tested with sentence-transformers/all-MiniLM-L6-v2.

## Testing

```bash
cargo test -p tabagent-onnx-loader
```

Integration tests download real models from HuggingFace and validate:
- Model loading
- Tokenizer integration
- Embedding generation
- Batch processing
- Error handling

