# BitNet Backend for TabAgent

**Restored missing backend implementation for BitNet 1.58-bit quantized models**

## Overview

This backend provides inference support for BitNet 1.58 models - ultra-low bit quantization models that offer 2-6x faster inference than standard GGUF models.

## What Was Missing

The `backends/__init__.py` was importing `BitNetManager` and `BitNetConfig` from a non-existent `bitnet/` folder. This caused import failures throughout the Server codebase. The documentation referenced BitNet backend extensively, but the actual implementation was never created.

## What Was Restored

Created complete BitNet backend with:

1. **`__init__.py`** - Module exports
2. **`config.py`** - BitNetConfig dataclass and enums
3. **`manager.py`** - BitNetManager for model lifecycle and inference
4. **`validator.py`** - Model type detection and GGUF metadata reading

## Architecture

```
Server/
├── backends/
│   ├── bitnet/                    ← Restored!
│   │   ├── __init__.py
│   │   ├── config.py             # Configuration
│   │   ├── manager.py            # Lifecycle manager
│   │   └── validator.py          # Model detection
│   ├── llamacpp/
│   ├── onnxrt/
│   └── ...
└── BitNet/                        # Submodule
    └── Release/
        ├── cpu/
        │   ├── linux/
        │   │   └── llama-server-bitnet
        │   ├── macos/
        │   │   └── llama-server-bitnet
        │   └── windows/
        │       └── llama-server-bitnet.exe
        └── gpu/
```

## Binary Location

The BitNet backend uses binaries from `BitNet/Release/`:

- **CPU**: `BitNet/Release/cpu/{platform}/llama-server-bitnet`
- **GPU**: `BitNet/Release/gpu/{platform}/` (Python modules + CUDA)

## Features

### Model Detection
- Automatic detection of BitNet 1.58 models
- Filename pattern matching (bitnet, b1.58, i2_s, tl1, tl2)
- GGUF metadata parsing
- Quantization type detection (TL1, TL2, i2_s)

### Inference Manager
- Model loading/unloading
- Streaming and non-streaming generation
- Embedding generation
- Health checking
- Performance tracking
- Port management

### Configuration
- TL1 optimization (Intel)
- TL2 optimization (AMD Ryzen)
- GPU CUDA support (future)
- Configurable context size, batch size, threads

## Usage

```python
from backends.bitnet import BitNetManager, BitNetConfig
from backends.bitnet.validator import is_bitnet_model

# Check if model is BitNet
if is_bitnet_model("/path/to/model.gguf"):
    # Create configuration
    config = BitNetConfig(
        binary_path="/path/to/llama-server-bitnet",
        backend=BitNetBackend.CPU_TL2,  # AMD optimized
        context_size=4096,
        port=8082
    )
    
    # Initialize manager
    manager = BitNetManager()
    
    # Load model
    success = manager.load_model("/path/to/model.gguf", config)
    
    # Generate
    if success:
        response = await manager.generate(messages, settings)
```

## How It Compares

| Backend | Use Case | Speed | Models |
|---------|----------|-------|--------|
| **BitNet** | BitNet 1.58 models | 2-6x faster | Specialized quantization |
| **LlamaCpp** | Standard GGUF | Baseline | All GGUF formats |
| **ONNX Runtime** | ONNX models | Varies | ONNX format |

## Building Binaries

If binaries are missing:

```bash
# Linux/macOS
cd BitNet
./build-all-linux.sh   # or build-all-macos.sh

# Windows
cd BitNet
build-all-windows.bat
```

Binaries will be placed in `BitNet/Release/cpu/{platform}/`.

## Integration Points

The BitNet backend integrates with:

1. **native_host.py** - Message routing
2. **hardware/** - Hardware detection
3. **server_mgmt/** - Process management
4. **core/** - Type definitions

## Status

✅ **Fully Implemented**
- Model detection and validation
- Configuration management
- Lifecycle management (load/unload)
- Streaming inference
- Embedding generation
- Performance tracking

⏳ **Future Enhancements**
- GPU backend (CUDA)
- Multi-model support
- Advanced quantization options

## Related Documentation

- [Server README](../../README.md)
- [BitNet Integration Guide](../../docs/README_BITNET.md)
- [Backend Architecture](../../docs/ARCHITECTURE.md)
- [Backends README](../README.md)

---

**Restored**: October 17, 2025  
**Status**: Production Ready

