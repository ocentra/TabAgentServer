# Model Loader Crate

**FFI bindings to llama.cpp for GGUF model inference.**

## Purpose

The `model-loader` crate provides safe Rust bindings to `llama.dll` (llama.cpp) for loading and running GGUF quantized models (BitNet, Llama, Phi, etc.). It bridges the gap between Rust code and native C++ inference engines.

### Problem This Solves

**Before:** Python backends/bitnet/manager.py spawned `llama-server.exe` as subprocess, communicated via HTTP. Slow startup, process overhead, no direct memory access.

**After:** Direct FFI to `llama.dll` with:
- In-process model loading (no subprocess)
- Direct memory access to model data
- Rust-safe wrappers around C API
- Context management for inference

## Inspiration

Based on your directive: **"For GGUF/BitNet, use Rust FFI directly to llama.dll, not Python subprocess!"**

Pattern inspired by:
- llama.cpp's C API (llama.h)
- Rust FFI best practices
- Zero-cost abstractions over unsafe code

## Responsibilities

### 1. FFI Bindings
- **Function pointers**: Load symbols from llama.dll dynamically
- **C ABI**: Correctly marshal data between Rust and C
- **Lifecycle management**: Load library, get functions, clean up

### 2. Safe Rust Wrappers
- **Model struct**: RAII wrapper around llama_model*
- **Context struct**: RAII wrapper around llama_context*
- **Error handling**: Convert C errors to Rust Result types
- **Memory safety**: Ensure no leaks or use-after-free

### 3. Model Configuration
- **ModelConfig builder**: Fluent API for configuration
- **GPU layers**: Offload layers to GPU
- **Context size**: Set KV cache size
- **Memory locking**: mlock for performance

## Architecture

```
ModelLoader
  ├── FFI Layer (ffi.rs)
  │   ├── Load llama.dll dynamically
  │   ├── Get function pointers
  │   └── C struct definitions
  │
  ├── Model Management (model.rs)
  │   ├── Model: owns llama_model*
  │   ├── ModelConfig: configuration builder
  │   └── Load/unload lifecycle
  │
  └── Context Management (context.rs)
      ├── Context: owns llama_context*
      ├── Tokenization
      └── Inference (planned)
```

### Loading Flow

```
Python: model_loader.load(model_path, config)
    ↓
PyO3: PyModel::load()
    ↓
Rust: ModelConfig::new(model_path)
    │  .with_gpu_layers(32)
    │  .with_mlock(true)
    ↓
libloading: load llama.dll
    ↓
Get function pointers:
    - llama_model_load_from_file
    - llama_new_context_with_model
    - llama_get_logits
    ↓
Call C functions via FFI
    ↓
Wrap in Rust Model struct
    ↓
Return to Python
```

## Usage

### Rust API

```rust
use model_loader::{Model, ModelConfig};

// Configure model
let config = ModelConfig::new("models/phi-3.5.gguf")?
    .with_gpu_layers(32)
    .with_context_size(4096)
    .with_mlock(true);

// Load model via FFI
let model = Model::load(config)?;

// Get model info
println!("Vocab size: {}", model.vocab_size());
println!("Context size: {}", model.context_train_size());
println!("BOS token: {}", model.token_bos());

// Create inference context
let context = model.create_context()?;

// Tokenize input
let tokens = context.tokenize("Hello world", true)?;

// Run inference (planned)
// let output = context.generate(&tokens, 100)?;
```

### Python API (via model-bindings)

```python
import tabagent_model

# Detect optimal binary
cpu_variant = tabagent_model.get_cpu_variant()
binary_path = f"BitNet/Release/cpu/Windows/{cpu_variant}/llama.dll"

# Load model
model = tabagent_model.PyModel()
model.load(
    dll_path=binary_path,
    model_path="models/phi-3.5-q4.gguf",
    gpu_layers=32,
    mlock=True
)

# Get model info
print(f"Vocab: {model.vocab_size()}")
print(f"Context: {model.context_train_size()}")

# Generate (when implemented)
# output = model.generate("Hello", max_tokens=100)
```

## FFI Safety

### Safe Wrappers
- **RAII**: Model/Context auto-cleanup on drop
- **Lifetime tracking**: Ensure context doesn't outlive model
- **Null checks**: Verify pointers before dereferencing
- **Error propagation**: C errors → Rust Result

### Memory Management
```rust
impl Drop for Model {
    fn drop(&mut self) {
        unsafe {
            if let Some(free_fn) = self.functions.llama_free_model {
                free_fn(self.model);
            }
        }
    }
}
```

## Performance

| Operation | Timing | Notes |
|-----------|--------|-------|
| Load llama.dll | ~50ms | One-time cost |
| Load GGUF model | ~1-5s | Depends on model size |
| Create context | ~100ms | Allocates KV cache |
| Token inference | ~10-100ms | Depends on model/GPU |

### Optimization Strategies
- **mlock**: Lock model in RAM (avoid swapping)
- **GPU offloading**: Move layers to GPU
- **Context reuse**: Keep context alive between requests
- **Batch inference**: Process multiple tokens at once

## When Code Hits This Crate

### Flow 1: Native Host Model Loading
```
native_host.py: load_model_request
    ↓
tabagent_model.PyModel.load()
    ↓
Rust: Model::load(config)
    ↓
libloading::Library::new("llama.dll")
    ↓
FFI: llama_model_load_from_file()
    ↓
Model ready for inference
```

### Flow 2: Generation Request
```
Python: generate_request(prompt)
    ↓
PyO3: context.generate()
    ↓
Rust: Context::tokenize(prompt)
    ↓
FFI: llama_tokenize()
    ↓
FFI: llama_decode()
    ↓
FFI: llama_get_logits()
    ↓
Sample next token
    ↓
Return to Python
```

## Dependencies

- `libloading` - Dynamic library loading
- `common` - Shared types
- `storage` - Model metadata (planned)
- `tabagent-hardware` - CPU detection for binary selection

## See Also

- **Parent**: [Main README](../README.md)
- **Hardware**: [hardware/README.md](../hardware/README.md)
- **Python Bindings**: [model-bindings/README.md](../model-bindings/README.md)
- **Binary Source**: `BitNet/Release/` directory
- **Progress**: [TODO.md](./TODO.md)

