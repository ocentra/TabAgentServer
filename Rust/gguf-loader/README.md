# gguf-loader

**FFI bindings to llama.cpp for GGUF/BitNet model inference with hardware-optimized variant selection.**

## Purpose

The `gguf-loader` crate provides safe Rust bindings to `llama.dll/libllama.so/libllama.dylib` for loading and running GGUF quantized models (BitNet 1.58-bit, Llama, Phi, SmolLM, etc.). It bridges Rust code with native C++ inference engines and automatically selects the optimal library variant based on detected hardware.

### Problem This Solves

**Before:** Python spawned `llama-server.exe` as subprocess, communicated via HTTP. Slow startup, process overhead, no direct memory access, no hardware optimization.

**After:** Direct FFI to optimized binaries with:
- In-process model loading (no subprocess)
- Direct memory access to model data
- Hardware-optimized variant selection (BitNet TL1/TL2 kernels for AMD Zen/Intel, GPU acceleration)
- Rust-safe wrappers around C API
- Full inference pipeline (tokenization → generation)

## Variant System

Maps exactly to `External/BitNet/BitnetRelease/` structure:

```
BitnetRelease/
  ├── cpu/{os}/
  │   ├── bitnet-amd-zen1/      # TL1/TL2 optimized for Zen1
  │   ├── bitnet-amd-zen2/      # TL1/TL2 optimized for Zen2
  │   ├── bitnet-amd-zen3/      # TL1/TL2 optimized for Zen3
  │   ├── bitnet-amd-zen4/      # TL1/TL2 optimized for Zen4
  │   ├── bitnet-amd-zen5/      # TL1/TL2 optimized for Zen5
  │   ├── bitnet-intel-*        # Intel Haswell→Alderlake
  │   ├── bitnet-arm/           # Apple Silicon, ARM
  │   └── standard/             # Generic (all GGUF formats)
  └── gpu/{os}/
      ├── bitnet-cuda/          # BitNet GPU (NVIDIA, Windows/Linux)
      ├── standard-cuda-vulkan/ # NVIDIA + AMD (Vulkan)
      ├── standard-opencl/      # Intel GPUs
      └── standard-metal/       # macOS/Apple Silicon

Selection Priority: BitNet GPU > Standard GPU > BitNet CPU > Standard CPU
```

## Architecture

Pattern inspired by llama.cpp's `simple.cpp` example with Rust safety

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
gguf-loader/
  ├── variant.rs        # Hardware-optimized variant selection
  │   ├── LibraryVariant trait (DRY architecture)
  │   ├── BitNetCpuVariant (13 architecture-specific variants)
  │   ├── BitNetGpuVariant (CUDA-only, NVIDIA)
  │   ├── StandardCpuVariant (generic, all GGUF formats)
  │   ├── StandardGpuVariant (CUDA/Vulkan/Metal/OpenCL)
  │   └── auto_select_variant() (hardware detection)
  │
  ├── ffi.rs            # 100+ llama.cpp C API bindings
  │   ├── Model lifecycle (load/free)
  │   ├── Context management (create/free)
  │   ├── Tokenization (encode/decode)
  │   ├── Inference (decode/logits/sampling)
  │   ├── Batch API (llama_batch_*)
  │   ├── Embeddings API (llama_get_embeddings_*)
  │   ├── KV cache management
  │   ├── LoRA adapters
  │   ├── State save/load
  │   └── Model metadata inspection
  │
  ├── model.rs          # RAII wrapper around llama_model*
  │   ├── Model::load() (with specific variant)
  │   ├── Model::load_with_auto_select() (hardware detection)
  │   ├── load_functions() (100+ FFI symbols)
  │   ├── vocab_size(), context_size(), embedding_dim()
  │   └── Special tokens (BOS/EOS/NL)
  │
  ├── context.rs        # RAII wrapper around llama_context*
  │   ├── Context::new() (creates inference context)
  │   ├── tokenize() / token_to_text()
  │   ├── generate() - Full autoregressive generation
  │   │   ├── Prompt encoding
  │   │   ├── Batch decoding
  │   │   ├── Logits extraction
  │   │   ├── Token sampling (greedy)
  │   │   └── EOG detection
  │   └── RAII cleanup (llama_free)
  │
  └── error.rs          # Custom error types
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

### Rust API (Full End-to-End)

```rust
use gguf_loader::{Model, ModelConfig, Context};
use std::path::Path;

// 1. Auto-select optimal variant based on hardware
let base_path = Path::new("./External/BitNet");
let config = ModelConfig::new("model.gguf")
    .with_gpu_layers(-1);  // Offload all layers

// 2. Load model with auto-selected variant
let model = Model::load_with_auto_select(base_path, config, true)?;

// 3. Get model info
println!("Vocab size: {}", model.vocab_size());
println!("Context size: {}", model.context_size());
println!("Embedding dim: {}", model.embedding_dim());

// 4. Create inference context
let context = Context::new(&model)?;

// 5. Generate text
let output = context.generate("Once upon a time")?;
println!("Generated: {}", output);
```

### Manual Variant Selection

```rust
use gguf_loader::{Variant, BitNetCpuVariant, select_library_path};

// Select specific variant manually
let variant = Variant::BitNetCpu(BitNetCpuVariant::AmdZen3);
let library_path = select_library_path(base_path, variant)?;

// Load with specific library
let config = ModelConfig::new("model.gguf");
let model = Model::load(&library_path, config)?;
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

## Testing

### Unit Tests (No Hardware Required)
```bash
cargo test -p gguf-loader --test variant_selection_tests
```
Tests:
- Variant selection logic for all CPU architectures
- GPU vendor mapping (NVIDIA/AMD/Intel/Apple)
- Selection priority (BitNet GPU > Standard GPU > BitNet CPU)
- Path construction
- Multi-GPU systems

### Integration Tests (Downloads Models)
```bash
cargo test -p gguf-loader --test test_models
```
Tests:
- Downloads smollm-135M-gguf (82MB)
- Loads with Standard CPU variant
- Full text generation pipeline
- Auto-selection with real hardware

## Performance

| Operation | Timing | Notes |
|-----------|--------|-------|
| Variant selection | <1ms | Hardware detection |
| Load library | ~50ms | One-time cost |
| Load GGUF model | ~1-5s | Depends on model size |
| Create context | ~100ms | Allocates KV cache |
| Token inference | ~10-100ms | Depends on model/GPU/variant |

### BitNet TL1/TL2 Performance
- **1.58-bit models**: ~3-5x faster than standard on optimized CPU variants
- **Memory**: ~8x smaller than FP16 (e.g., 7B model ~1GB vs 8GB)
- **AMD Zen3/4**: Best performance with `bitnet-amd-zen3/4`
- **Intel Skylake+**: Best performance with `bitnet-intel-skylake`

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
- `common` - Shared types (NodeId, EdgeId, DbError)
- `storage` - Model metadata
- `tabagent-hardware` - Hardware detection (CPU architecture, GPU vendor)

## Future: TabAgentDist Release Structure

This variant system will be replicated in `TabAgentDist/Release/` for server distribution:

```
TabAgentDist/Release/
  ├── tabagent-server.exe / tabagent-server
  ├── cpu/{os}/{variant}/llama.{dll|so|dylib}
  └── gpu/{os}/{variant}/llama.{dll|so|dylib}
```

The installer will:
1. Detect user's hardware (CPU architecture, GPU vendor)
2. Install only the relevant variant (e.g., `bitnet-amd-zen4` + `bitnet-cuda`)
3. Keep installation size minimal (~50-200MB vs full ~2GB)

## See Also

- **Parent**: [Main README](../../README.md)
- **Hardware Detection**: [hardware/README.md](../hardware/README.md)
- **ONNX Loader**: [onnx-loader/README.md](../onnx-loader/README.md)
- **Native Handler**: [native-handler/README.md](../native-handler/README.md)
- **Binary Source**: `../../External/BitNet/BitnetRelease/`

