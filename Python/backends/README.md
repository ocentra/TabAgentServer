# TabAgent Backends

**Inference backend implementations - Python & Rust**

---

## Architecture Overview

TabAgent uses a **DRY architecture** where:
1. **Rust** handles ALL detection logic (format + task)
2. **Python** executes inference based on Rust's routing decision
3. **No duplication** of detection/routing logic

---

## Backend Types

### **Rust Backends** (High-performance, FFI-based)
- ✅ **BitNet** - 1.58-bit quantized models (bitnet.dll)
- ✅ **GGUF** - Standard quantized models (llama.cpp)
- 🔜 **ONNX** - Multi-provider (migrating from Python)
- 🔜 **LiteRT** - On-device models (migrating from Python)

### **Python Backends** (Library-based)
- ✅ **Transformers** - PyTorch/SafeTensors (HuggingFace)
- ⏳ **ONNX** - onnxruntime (temporary, migrating to Rust)
- ⏳ **MediaPipe** - LiteRT models (temporary, migrating to Rust)

---

## Structure

```
backends/
├── base_backend.py         # Abstract base class for all backends
├── transformers_backend.py # PyTorch/SafeTensors inference (NEW!)
│
├── bitnet/                 # BitNet 1.58 backend [Rust]
│   ├── manager.py          # Uses BitNet/Release/cpu & gpu
│   └── validator.py        # Model type detection
│
├── llamacpp/               # General GGUF backend [Rust]
│   ├── manager.py          # Uses BitNet/Release/cpu (standard binary)
│   └── config.py
│
├── onnxrt/                 # ONNX Runtime backend [Python → Rust]
│   ├── manager.py          # DirectML, CUDA, NPU support
│   └── config.py
│
├── mediapipe/              # MediaPipe backend [Python → Rust]
│   └── manager.py
│
└── lmstudio/               # LM Studio proxy
    └── manager.py
```

---

## Binary Location (IMPORTANT!)

All backends now use `BitNet/Release/` for binaries:

### CPU Binaries
```
BitNet/Release/cpu/
├── linux/
│   ├── llama-server-bitnet      # BitNet 1.58 optimized (TL2)
│   └── llama-server-standard    # Regular GGUF (Vulkan)
├── macos/
│   ├── llama-server-bitnet      # BitNet 1.58 optimized (TL1)
│   └── llama-server-standard    # Regular GGUF (Metal)
└── windows/
    ├── llama-server-bitnet.exe  # BitNet 1.58 optimized (TL2)
    └── llama-server-standard.exe # Regular GGUF (CUDA/Vulkan)
```

### GPU Modules
```
BitNet/Release/gpu/
├── linux/
│   ├── bitlinear_cuda.so
│   └── *.py (model, generate, tokenizer, etc.)
├── macos/
│   └── *.py (no CUDA - Python only)
└── windows/
    ├── bitlinear_cuda.pyd
    └── *.py
```

---

## How Binaries Are Selected

### BitNetManager
```python
# Automatically chooses:
model_type = detect_model_type(model_path)

if model_type == ModelType.BITNET_158:
    binary = "llama-server-bitnet"    # 2-6x faster!
else:
    binary = "llama-server-standard"  # CUDA/Vulkan/Metal
```

### LlamaCppManager
```python
# Always uses standard:
binary = "llama-server-standard"  # Full GPU support
```

---

## Build Status

| Platform | CPU Binary | GPU Kernel | Status |
|----------|-----------|------------|--------|
| **Linux** | ✅ Built (WSL2) | ✅ Built | Ready |
| **macOS** | ✅ Built (Actions) | ✅ Built | Ready |
| **Windows** | ⏳ Pending | ⏳ Pending | Build locally |

Build Windows with: `cd BitNet && build-all-windows.bat`

---

## Why This Structure?

### Old (Messy):
```
backends/bitnet/binaries/
├── macos/
│   ├── cpu-macos/     ← Nested!
│   │   └── binary
│   └── gpu-macos/     ← Nested!
└── (no linux, no windows)
```

### New (Clean):
```
BitNet/Release/
├── cpu/{platform}/binary    ← Flat!
└── gpu/{platform}/modules   ← Flat!
```

**Benefits**:
- ✅ Single source of truth
- ✅ CI/CD populates directly
- ✅ Easy to manage
- ✅ Clear separation (cpu vs gpu)
- ✅ No nesting confusion

---

**Updated**: All backends now point to `BitNet/Release/`  
**Status**: Ready for Linux/macOS, Windows pending build

---

## Routing Flow (DRY Architecture)

### 1. Detection (Rust)
```rust
// detection.rs - Single source of truth
detect_model_py(source, auth_token) → {
    "model_type": "SafeTensors",
    "backend": { "Python": { "engine": "transformers" } },
    "task": "text-generation",
    "repo": "microsoft/phi-2"
}
```

### 2. Routing (Python)
```python
# native_host.py - Routes based on Rust's decision
if backend["Python"]["engine"] == "transformers":
    load_transformers_python(source, task, token)
elif backend["Rust"]["engine"] == "llama.cpp":
    rust_handle_message({"action": "load_model", "modelPath": source})
```

### 3. Execution (Backend-specific)
```python
# backends/transformers_backend.py
TransformersTextGenBackend().load_model(model_path, task)
→ Uses torch, transformers, accelerate
```

---

## Migration Plan (Python → Rust)

### Current State
| Backend | Implementation | Status |
|---------|---------------|--------|
| BitNet | Rust (bitnet.dll) | ✅ Stable |
| GGUF | Rust (llama.cpp) | ✅ Stable |
| Transformers | Python (HuggingFace) | ✅ Stable |
| ONNX | Python (onnxruntime) | ⏳ Migrating |
| LiteRT | Python (mediapipe) | ⏳ Migrating |

### Migration Flags (native_host.py)
```python
class Config:
    ONNX_USE_RUST = False    # Set True when Rust ONNX ready
    LITERT_USE_RUST = False  # Set True when Rust LiteRT ready
```

### Target State (Future)
| Backend | Implementation | Performance Gain |
|---------|---------------|-----------------|
| BitNet | Rust (bitnet.dll) | Current |
| GGUF | Rust (llama.cpp) | Current |
| Transformers | Python (HuggingFace) | Stays in Python |
| ONNX | Rust (onnxruntime-rs) | 2-3x faster |
| LiteRT | Rust (mediapipe-rs) | 2-3x faster |

**Why migrate ONNX/LiteRT but not Transformers?**
- ONNX/LiteRT: C++ libraries → natural fit for Rust FFI
- Transformers: Deep Python ecosystem → better in Python

