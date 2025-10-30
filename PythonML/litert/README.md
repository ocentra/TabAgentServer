# LiteRT - Quantized Edge Model Inference

**Ultra-low latency inference with TensorFlow Lite**

Quantized model support for edge deployment. Runs 4-bit/8-bit models with minimal memory footprint, ideal for mobile and embedded systems.

Formerly known as TensorFlow Lite, now LiteRT (Lite Runtime).

Reference: https://ai.google.dev/edge/litert

---

## Overview

**Purpose**: Run quantized `.tflite` models with maximum efficiency

**Use Cases**:
- Edge devices (phones, IoT)
- Low-latency requirements
- Limited RAM/VRAM
- Battery-powered devices

**Models**:
- Gemma LiteRT (e.g., `google/gemma-3n-E4B-it-litert-lm`)
- Custom quantized models
- TensorFlow Lite model zoo

---

## Architecture

```
.tflite Model File
    ↓
LiteRT Interpreter
    ↓
Delegates (XNNPACK/GPU/NNAPI)
    ↓
Hardware (CPU/GPU/NPU)
```

**Delegates**:
- **XNNPACK**: Optimized CPU inference
- **GPU**: OpenGL/Metal/Vulkan acceleration
- **NNAPI**: Android Neural Networks API
- **Core ML**: iOS acceleration
- **QNN**: Qualcomm Hexagon DSP

---

## LiteRTManager

**`manager.py`** - Core inference manager

### Initialization

```python
from litert import LiteRTManager

manager = LiteRTManager()
```

### Load Model

```python
success = manager.load_model(
    model_path="models/gemma-3b-litert.tflite",
    use_xnnpack=True,  # CPU acceleration
    use_gpu=False,      # GPU acceleration
    num_threads=4       # CPU threads
)
```

### Run Inference

```python
result = manager.generate({
    "input": input_tensor  # numpy array
})

output = result['output']
```

### Unload

```python
manager.unload()
```

---

## Gemma LiteRT Models

**Example**: `google/gemma-3n-E4B-it-litert-lm`

**Quantization**: E4B (4-bit quantization with 8-bit activations)

**Download**:
```python
from huggingface_hub import hf_hub_download

model_path = hf_hub_download(
    repo_id="google/gemma-3n-E4B-it-litert-lm",
    filename="gemma-3n-E4B-it-litert-lm.tflite"
)
```

**Usage**:
```python
manager = LiteRTManager()
manager.load_model(model_path, use_xnnpack=True)

# Text generation (simplified)
result = manager.generate({
    "input": tokenized_input
})
```

---

## Delegates

### XNNPACK (CPU Acceleration)

**Best for**: ARM/x86 CPUs  
**Performance**: 2-4x faster than default  
**Setup**: Automatic (built into TensorFlow Lite)

```python
manager.load_model(model_path, use_xnnpack=True)
```

### GPU Delegate

**Best for**: Mobile GPUs, discrete GPUs  
**Performance**: 5-10x faster than CPU  
**Setup**: Requires GPU support in TensorFlow Lite

```python
manager.load_model(model_path, use_gpu=True)
```

**Platforms**:
- Android: OpenGL ES
- iOS: Metal
- Desktop: OpenCL/Vulkan

### NNAPI (Android Neural Networks)

**Best for**: Android devices with NPU  
**Performance**: Hardware-dependent  
**Setup**: Android 8.1+

```python
# TODO: Implement NNAPI delegate
```

### Core ML (iOS)

**Best for**: iOS devices (iPhone, iPad)  
**Performance**: Hardware-dependent (Neural Engine)  
**Setup**: iOS 11+

```python
# TODO: Implement Core ML delegate
```

---

## Quantization

**Supported Formats**:
- **INT8**: 8-bit integer quantization
- **FLOAT16**: Half-precision floating point
- **E4B**: 4-bit weights + 8-bit activations (Gemma LiteRT)

**Memory Savings**:
- FP32 → INT8: 4x smaller
- FP32 → FLOAT16: 2x smaller
- FP32 → E4B: ~8x smaller

**Accuracy Trade-off**:
- INT8: Minimal accuracy loss (<1%)
- FLOAT16: Negligible loss
- E4B: Slightly higher loss, acceptable for most tasks

---

## Performance

| Model | Quantization | Size | Latency | Memory |
|-------|--------------|------|---------|--------|
| Gemma 2B FP32 | None | 8GB | 150ms | 8GB RAM |
| Gemma 2B INT8 | 8-bit | 2GB | 80ms | 2GB RAM |
| Gemma 3B E4B | 4-bit | 1GB | 50ms | 1.5GB RAM |

*Pixel 7 Pro (Tensor G2)*

**Desktop Performance**:
- Gemma 3B LiteRT: 45 tok/s @ 4 threads (i9-12900K)
- XNNPACK enabled: 2.5x faster vs. default

---

## Integration with Services

**Future**: LiteRT Service (gRPC)

```python
# services/litert_service.py
class LiteRTServiceImpl(ml_inference_pb2_grpc.LiteRTServiceServicer):
    def LoadModel(self, request, context):
        manager = LiteRTManager()
        manager.load_model(request.model_path)
        return LoadModelResponse(success=True)
    
    def Generate(self, request, context):
        result = manager.generate({"input": request.input})
        return GenerateResponse(output=result['output'])
```

---

## Examples

### Basic Text Generation

```python
from litert import LiteRTManager
import numpy as np

manager = LiteRTManager()

# Load model
manager.load_model(
    "gemma-3b-litert.tflite",
    use_xnnpack=True,
    num_threads=4
)

# Prepare input (tokenize first)
input_ids = np.array([[1, 2, 3, 4]], dtype=np.int32)

# Generate
result = manager.generate({"input": input_ids})

# Process output (detokenize)
output_ids = result['output']
```

### Benchmark Inference

```python
import time

manager = LiteRTManager()
manager.load_model("model.tflite", use_xnnpack=True)

# Warmup
for _ in range(10):
    manager.generate({"input": dummy_input})

# Benchmark
times = []
for _ in range(100):
    start = time.time()
    manager.generate({"input": input_data})
    times.append(time.time() - start)

avg_latency = np.mean(times) * 1000  # ms
print(f"Average latency: {avg_latency:.2f}ms")
```

---

## Testing

```bash
# Unit tests
pytest tests/test_litert.py -v

# Benchmark
python -m litert.benchmark --model gemma-3b-litert.tflite
```

---

## Limitations

**Current**:
- No streaming support (batch only)
- Limited model coverage (not all HF models have .tflite)
- Manual tokenization required
- Delegates not fully implemented

**Not Supported**:
- GGUF models (use gguf-loader instead)
- ONNX models (use onnx-loader instead)
- FP32 models (convert to quantized first)

---

## Converting Models

### From PyTorch to LiteRT

```python
import tensorflow as tf
from transformers import AutoModel

# 1. Export to TensorFlow
model = AutoModel.from_pretrained("model-name")
# ... conversion code ...

# 2. Quantize
converter = tf.lite.TFLiteConverter.from_saved_model("saved_model")
converter.optimizations = [tf.lite.Optimize.DEFAULT]
tflite_model = converter.convert()

# 3. Save
with open("model.tflite", "wb") as f:
    f.write(tflite_model)
```

**Note**: Many models need custom conversion scripts. Check model documentation.

---

## See Also

- **[LiteRT Docs](https://ai.google.dev/edge/litert)** - Official documentation
- **[Gemma LiteRT Models](https://huggingface.co/collections/google/gemma-litert-models)** - Pre-quantized models
- **[TensorFlow Lite](https://www.tensorflow.org/lite)** - Framework documentation
- **[Services](../services/README.md)** - Integration with gRPC services

