# Python Version Compatibility

**TabAgent Server Python version requirements**

---

## Supported Python Versions

| Python | Core | ONNX RT | MediaPipe | Recommendation |
|--------|------|---------|-----------|----------------|
| **3.13** | ✅ Yes | ✅ Yes | ❌ No | ⚠️ Limited |
| **3.12** | ✅ Yes | ✅ Yes | ✅ Yes | ✅ **BEST** |
| **3.11** | ✅ Yes | ✅ Yes | ✅ Yes | ✅ **BEST** |
| **3.10** | ✅ Yes | ✅ Yes | ✅ Yes | ✅ Good |
| **3.9** | ✅ Yes | ✅ Yes | ✅ Yes | ✅ Good |

---

## Feature Matrix by Python Version

### Python 3.13 (Current)
**Status:** ⚠️ Partially Supported

**Works:**
- ✅ Hardware detection (all platforms)
- ✅ Backend selection
- ✅ ONNX Runtime (DirectML, CUDA, CPU)
- ✅ llama.cpp backend
- ✅ BitNet backend
- ✅ Server management
- ✅ Model library

**Doesn't Work:**
- ❌ MediaPipe backend (not yet released for 3.13)
- ❌ On-device Gemma models

**Workaround:** Use Python 3.12 for full feature set

---

### Python 3.12 or 3.11 ✅ RECOMMENDED
**Status:** ✅ Fully Supported

**Everything works:**
- ✅ All hardware detection
- ✅ All backends (BitNet, ONNX, llama.cpp, MediaPipe)
- ✅ All acceleration (CUDA, DirectML, NPU, Vulkan, ROCm, Metal)
- ✅ MediaPipe on-device inference
- ✅ Gemma models

---

## Installation Instructions

### Option 1: Use Python 3.12 (Recommended)

```bash
# Install Python 3.12
# Windows: Download from python.org
# Linux: sudo apt install python3.12
# macOS: brew install python@3.12

# Create virtual environment
python3.12 -m venv venv

# Activate
# Windows:
venv\Scripts\activate
# Linux/macOS:
source venv/bin/activate

# Install dependencies
pip install -r requirements.txt

# Verify
python cli.py backends
```

---

### Option 2: Continue with Python 3.13 (Limited)

```bash
# Install dependencies (MediaPipe will be skipped)
pip install -r requirements.txt

# Verify (no MediaPipe)
python cli.py backends
```

**Available backends on 3.13:**
- BitNet (CPU/GPU)
- ONNX Runtime (CPU/CUDA/DirectML/NPU)
- llama.cpp (CPU/CUDA/Vulkan/ROCm/Metal)
- ~~MediaPipe~~ (not available)

---

## ONNX Runtime Builds

### Windows
**Best:** `onnxruntime-directml`
- Supports GPU (NVIDIA/AMD/Intel)
- Supports NPU (AMD Ryzen AI)
- Supports DirectML acceleration

```bash
pip install onnxruntime-directml
```

### Linux with NVIDIA GPU
**Best:** `onnxruntime-gpu`
- CUDA acceleration

```bash
pip install onnxruntime-gpu
```

### Linux with AMD GPU
**Best:** `onnxruntime` (CPU) + ROCm via llama.cpp
- Use llama.cpp with ROCm for AMD

```bash
pip install onnxruntime
```

### macOS
**Best:** `onnxruntime` (CPU) + Metal via llama.cpp
- Use llama.cpp with Metal for Apple GPU

```bash
pip install onnxruntime
```

---

## Checking Your Setup

```bash
# Check Python version
python --version

# Check ONNX Runtime providers
python -c "import onnxruntime as ort; print(ort.get_available_providers())"

# Check MediaPipe (3.12 or earlier only)
python -c "import mediapipe as mp; print(f'MediaPipe {mp.__version__}')"

# Check all backends
python cli.py backends
```

---

## What Works Where

### All Python Versions (3.9-3.13)
- Hardware detection (CPU, GPU, NPU)
- Backend selection with ngl calculation
- BitNet inference
- ONNX Runtime inference
- llama.cpp inference
- Server management
- Port management
- Model library

### Python 3.9-3.12 Only
- MediaPipe LLM inference
- On-device Gemma models (.task bundles)

---

## Recommendation

**For development/production:** Use **Python 3.12**
- Full compatibility
- MediaPipe support
- Stable, mature
- All features work

**Current setup (3.13):** Works but limited
- MediaPipe unavailable
- Can still use ONNX, llama.cpp, BitNet
- Can switch to 3.12 anytime

---

**Your choice: Continue with 3.13 (14/16 backends) or switch to 3.12 (16/16 backends)?** 🚀

