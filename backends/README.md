# TabAgent Backends

**Inference backend implementations using BitNet/Release/ binaries**

---

## Structure

```
backends/
├── bitnet/           # BitNet 1.58 backend
│   ├── manager.py    # Uses BitNet/Release/cpu & gpu
│   └── validator.py  # Model type detection
│
├── llamacpp/         # General GGUF backend
│   ├── manager.py    # Uses BitNet/Release/cpu (standard binary)
│   └── config.py
│
├── onnxrt/           # ONNX Runtime backend
│   ├── manager.py    # DirectML, CUDA, NPU support
│   └── config.py
│
├── mediapipe/        # MediaPipe backend
│   └── manager.py
│
└── lmstudio/         # LM Studio proxy
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

