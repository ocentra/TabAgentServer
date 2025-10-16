# ONNX Runtime Backend

**Multi-provider ONNX inference with NPU support**

---

## Features

- ✅ **CPU Inference** - CPUExecutionProvider
- ✅ **CUDA** - CUDAExecutionProvider (NVIDIA)
- ✅ **DirectML** - DmlExecutionProvider (Windows GPU/NPU)
- ✅ **NPU** - VitisAIExecutionProvider (AMD Ryzen AI)
- ✅ **ROCm** - ROCmExecutionProvider (AMD GPU - Linux)
- ✅ **Multiple Providers** - Automatic fallback

---

## Supported Hardware

| Hardware | Provider | Platform |
|----------|----------|----------|
| **NVIDIA GPU** | CUDA | All |
| **AMD GPU** | ROCm | Linux |
| **AMD GPU** | DirectML | Windows |
| **AMD Ryzen AI NPU** | VitisAI | Windows |
| **AMD Ryzen AI NPU** | DirectML | Windows |
| **Intel Arc GPU** | DirectML | Windows |
| **Intel iGPU** | DirectML | Windows |
| **CPU** | CPU | All |

---

## Usage

```python
from backends.onnxrt import ONNXRuntimeManager
from core import AccelerationBackend

manager = ONNXRuntimeManager()

# Load model with NPU
manager.load_model(
    model_path="model.onnx",
    acceleration=AccelerationBackend.NPU
)

# Generate
output = manager.generate(messages, settings)

# Unload
manager.unload_model()
```

---

## Provider Selection

Providers are tried in priority order with automatic fallback:

### NPU Configuration
```python
providers = [
    "VitisAIExecutionProvider",  # AMD Ryzen AI NPU (preferred)
    "DmlExecutionProvider",       # Fallback to DirectML
    "CPUExecutionProvider"        # Final fallback
]
```

### CUDA Configuration
```python
providers = [
    "CUDAExecutionProvider",  # NVIDIA GPU
    "CPUExecutionProvider"    # Fallback
]
```

### DirectML Configuration  
```python
providers = [
    "DmlExecutionProvider",  # Windows GPU/NPU
    "CPUExecutionProvider"   # Fallback
]
```

---

## Requirements

```bash
# CPU only
pip install onnxruntime>=1.16.0

# CUDA support
pip install onnxruntime-gpu>=1.16.0

# DirectML support (Windows)
pip install onnxruntime-directml>=1.16.0
```

---

## Model Format

Supports ONNX models with optional optimizations:
- **Standard ONNX** (.onnx)
- **Optimized ONNX** (.onnx with graph optimizations)
- **Quantized ONNX** (INT8, UINT8)

---

## Performance

| Provider | Hardware | Performance |
|----------|----------|-------------|
| **VitisAI** | AMD Ryzen AI | Excellent power efficiency |
| **CUDA** | NVIDIA GPU | Best throughput |
| **DirectML** | AMD/Intel GPU | Good compatibility |
| **CPU** | Any CPU | Baseline |

---

## Future Enhancements

- [ ] TensorRT provider support
- [ ] Model caching
- [ ] Quantization on-the-fly
- [ ] Multi-model batching
- [ ] Custom operators

---

**Full-featured ONNX Runtime integration for maximum hardware utilization!** 🚀

