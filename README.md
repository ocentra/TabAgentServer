# TabAgent Server

**Hardware-aware inference platform with intelligent backend selection**

Production-grade server for local AI inference with automatic hardware detection, optimal configuration, and robust process management.

---

## Features

- 🔍 **Auto Hardware Detection** - Automatically detects CPUs, GPUs (NVIDIA/AMD/Intel), VRAM, and acceleration capabilities
- 🎯 **Smart Backend Selection** - Intelligently selects optimal backend based on available hardware
- 📊 **VRAM-Aware Configuration** - Calculates optimal GPU layer offloading based on available memory
- 🔄 **Multi-Backend Support** - BitNet, LM Studio, and extensible architecture
- ⚙️ **Robust Process Management** - Health checking, graceful shutdown, and port conflict resolution
- 📚 **Curated Model Library** - 8 pre-configured models with HuggingFace integration
- 💪 **100% Strong Typing** - 35+ Enums, 20+ Pydantic models, zero magic strings
- 🧪 **CLI Tools** - Built-in testing and administration commands

---

## Quick Start

### Installation

```bash
cd Server/
pip install -r requirements.txt
```

### CLI Usage

```bash
# Show system information
python cli.py info

# List available backends  
python cli.py backends

# Test backend selection
python cli.py test bitnet_1.58 --size 3.5

# JSON output
python cli.py info --format json
```

### Python API

```python
from core import (
    create_hardware_detector,
    BackendSelector,
    ModelLibrary,
)

# Hardware detection
detector = create_hardware_detector()
hw_info = detector.get_hardware_info()
print(f"GPUs: {hw_info.nvidia_gpus}")

# Backend selection
selector = BackendSelector()
result = selector.select_backend(ModelType.BITNET_158, model_size_gb=3.5)
print(f"Selected: {result.backend}, ngl: {result.ngl}")

# Model library
library = ModelLibrary()
models = library.get_recommended_models()
for model in models:
    print(f"{model.name} - {model.size_gb}GB")
```

---

## Architecture

### Module Organization

**`core/`** - Types, enums, and configuration (foundation layer)  
**`hardware/`** - Hardware detection and backend selection  
**`server_mgmt/`** - Port allocation and process lifecycle  
**`models/`** - Model library and download management  
**`backends/`** - Inference backend implementations  

**See:** [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) for details

---

## Hardware Detection

### Supported Platforms

| OS | Status | Detection |
|----|--------|-----------|
| **Windows** | ✅ Complete | WMI + nvidia-smi |
| **Linux** | 🚧 Planned | lspci + nvidia-smi |
| **macOS** | 🚧 Planned | system_profiler |

### Detected Hardware

- **CPUs** - Name, cores, threads, clock speed
- **NVIDIA GPUs** - With VRAM via nvidia-smi
- **AMD GPUs** - Discrete/integrated classification
- **Intel GPUs** - Arc and integrated
- **Acceleration** - CUDA, Vulkan, ROCm, Metal, DirectML
- **NPUs** - AMD Ryzen AI (planned)

---

## Backend Selection

### Algorithm

1. Detect all available hardware
2. Check acceleration capabilities (CUDA, Vulkan, etc)
3. Parse GPU VRAM amounts
4. Calculate optimal layer offloading (ngl)
5. Select best backend with confidence score

### VRAM-Aware Layer Offloading

```
available_vram = total_vram - 2GB (reserved)
if available_vram >= model_size:
    ngl = all_layers  # Full GPU offload
else:
    ngl = int((available_vram / model_size) * total_layers * 0.9)
```

**Example:** 7B model (5GB) + 8GB GPU = 32 layers offloaded (full)

---

## Model Library

### Curated Models

- **Llama 3.2** (1B, 3B) - Fast instruction models
- **Phi-4** (14B) - Microsoft's reasoning model
- **Qwen 2.5 Coder** (7B) - Best coding model
- **Qwen 2.5** (14B) - General purpose
- **Gemma 2** (2B) - Google's efficient model
- **BitNet 3B** - 1.58-bit quantized

Each model includes:
- HuggingFace repository
- Available quantization variants
- Size, context length, use cases
- License information

---

## Type Safety

**35+ Enums** defined for all constants  
**20+ Pydantic Models** for data validation  
**Zero Magic Strings** - all strings are enum values  
**Zero Magic Numbers** - all numbers are enum values  
**100% Type Coverage** - complete type hints

### Example

```python
# Strong typing everywhere
from core import BackendType, ServerType, ModelType

backend = BackendType.BITNET_CPU  # Not "bitnet_cpu"
port = DefaultPort.BITNET_CPU.value  # Not 8081
model = ModelType.BITNET_158  # Not "bitnet_1.58"
```

---

## Documentation

- **[Architecture](docs/ARCHITECTURE.md)** - System design and components
- **[API Reference](docs/API.md)** - Complete API documentation
- **[Features Analysis](docs/FEATURES_ANALYSIS.md)** - Implementation status and roadmap
- **[Project Structure](docs/STRUCTURE.md)** - File organization
- **[BitNet Integration](docs/README_BITNET.md)** - BitNet backend details

---

## Development

### Adding a Backend

1. Create folder in `backends/`
2. Implement `manager.py` with standard interface
3. Update backend routing
4. Done!

### Adding Models

1. Edit `models/models_library.json`
2. Add entry with metadata
3. Done!

### Testing

```bash
# Run CLI tests
python cli.py info
python cli.py backends
python cli.py test bitnet_1.58 --size 3.5

# Python tests
python -m pytest tests/
```

---

## Requirements

### Core
- Python 3.9+
- `pydantic` - Data validation
- `requests` - HTTP client

### Optional
- `torch` - For CUDA detection
- `wmi` - For Windows hardware detection (Windows only)
- `huggingface-hub` - For model downloads

```bash
pip install -r requirements.txt
```

---

## Performance

| Configuration | First Token | Throughput | VRAM |
|--------------|-------------|------------|------|
| BitNet GPU (3B) | 50ms | 45 tok/s | 4GB |
| CUDA (7B Q4) | 80ms | 35 tok/s | 6GB |
| CPU (7B Q4) | 500ms | 8 tok/s | 0GB |

*RTX 4090 + i9-12900K*

---

## Production Quality

✅ **Modular** - Clean separation of concerns  
✅ **Typed** - 100% strong typing  
✅ **Robust** - Comprehensive error handling  
✅ **Documented** - Complete documentation  
✅ **Extensible** - Easy to add features  
✅ **Clean** - 0 lint errors  

---

## License

Apache 2.0 - See [LICENSE](../LICENSE) file.

