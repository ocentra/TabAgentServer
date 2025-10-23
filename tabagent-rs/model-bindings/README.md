# TabAgent Model Bindings - Python Bindings

Python bindings for TabAgent model loading and hardware detection via PyO3.

## ✨ Features

- 🚀 **Native Performance**: Direct Rust-to-Python FFI
- 🔍 **Hardware Detection**: CPU architecture and optimal binary selection
- 📦 **Model Loading**: Load GGUF models via llama.cpp FFI
- 🎯 **Zero Overhead**: Minimal Python wrapper around Rust

## Purpose

This crate bridges `model-loader` and `hardware` Rust crates to Python, enabling `native_host.py` to:
1. Detect system hardware (CPU/GPU)
2. Select optimal llama.dll binary
3. Load GGUF models for inference

Unlike `bindings/` (database) and `model-cache-bindings/` (file storage), this crate focuses on **model inference** via FFI.

## 📦 Installation

### From Wheel (Recommended)

```bash
# Build the wheel
cd Server/tabagent-rs/model-bindings
maturin build --release

# Install
pip install ../target/wheels/tabagent_model-0.1.0-cp39-abi3-win_amd64.whl
```

### From Source

```bash
pip install maturin
cd Server/tabagent-rs/model-bindings
maturin develop --release
```

## 🚀 Quick Start

```python
import tabagent_model

# 1. Detect hardware
cpu_variant = tabagent_model.get_cpu_variant()
print(f"CPU: {cpu_variant}")  # e.g., "bitnet-amd-zen2"

# 2. Get optimal binary path
binary_path = tabagent_model.get_optimal_binary("llama.dll")
print(f"Binary: {binary_path}")
# Output: "BitNet/Release/cpu/Windows/bitnet-amd-zen2/llama.dll"

# 3. Load model
model = tabagent_model.PyModel()
model.load(
    dll_path=binary_path,
    model_path="models/phi-3.5-q4.gguf",
    gpu_layers=32,
    mlock=True
)

# 4. Get model info
print(f"Vocab size: {model.vocab_size()}")
print(f"Context size: {model.context_train_size()}")
print(f"Embedding dim: {model.embedding_dim()}")
print(f"BOS token: {model.token_bos()}")
print(f"EOS token: {model.token_eos()}")
```

## 📚 API Reference

### Hardware Detection Functions

#### `get_cpu_variant() -> str`

Detect CPU architecture and return variant string.

**Returns:**
- `str`: CPU variant (e.g., "bitnet-amd-zen2", "bitnet-intel-alderlake")

**Example:**
```python
cpu = tabagent_model.get_cpu_variant()
if "zen" in cpu:
    print("AMD Ryzen detected")
elif "alderlake" in cpu:
    print("Intel 12th gen detected")
```

---

#### `get_optimal_binary(binary_name: str) -> str`

Get optimal binary path for current CPU.

**Parameters:**
- `binary_name` (str): Binary filename (e.g., "llama.dll", "llama-server.exe")

**Returns:**
- `str`: Full path relative to project root

**Example:**
```python
dll_path = tabagent_model.get_optimal_binary("llama.dll")
# Returns: "BitNet/Release/cpu/Windows/bitnet-amd-zen2/llama.dll"
```

---

### PyModel Class

Main model loading class.

#### `PyModel()`

Create a new model instance.

**Example:**
```python
model = tabagent_model.PyModel()
```

---

#### `load(dll_path: str, model_path: str, gpu_layers: int = 0, mlock: bool = False)`

Load a GGUF model via llama.cpp FFI.

**Parameters:**
- `dll_path` (str): Path to llama.dll (from `get_optimal_binary()`)
- `model_path` (str): Path to GGUF model file
- `gpu_layers` (int, optional): Number of layers to offload to GPU (default: 0)
- `mlock` (bool, optional): Lock model in RAM to prevent swapping (default: False)

**Raises:**
- `RuntimeError`: If model loading fails

**Example:**
```python
model = tabagent_model.PyModel()
try:
    model.load(
        dll_path="BitNet/Release/cpu/Windows/generic/llama.dll",
        model_path="models/phi-3.5-mini-q4.gguf",
        gpu_layers=32,
        mlock=True
    )
    print("Model loaded successfully!")
except RuntimeError as e:
    print(f"Failed to load: {e}")
```

---

#### `vocab_size() -> int`

Get vocabulary size.

**Returns:**
- `int`: Number of tokens in vocabulary

---

#### `context_train_size() -> int`

Get maximum context size model was trained with.

**Returns:**
- `int`: Context size in tokens

---

#### `embedding_dim() -> int`

Get embedding dimension.

**Returns:**
- `int`: Embedding vector size

---

#### `token_bos() -> int`

Get beginning-of-sequence token ID.

**Returns:**
- `int`: BOS token ID

---

#### `token_eos() -> int`

Get end-of-sequence token ID.

**Returns:**
- `int`: EOS token ID

---

#### `token_nl() -> int`

Get newline token ID.

**Returns:**
- `int`: Newline token ID

---

## 🧪 Testing

Run the test script:

```bash
python Server/test_rust_bindings.py
```

Expected output:
```
🧪 Testing TabAgent Rust Model Bindings
============================================================
✅ Successfully imported tabagent_model

1️⃣ Testing hardware detection...
   CPU Variant: bitnet-amd-zen2
   Binary Path: BitNet/Release/cpu/Windows/bitnet-amd-zen2/llama.dll

2️⃣ Testing PyModel class...
   ✅ PyModel class available

============================================================
✅ ALL TESTS PASSED!
```

## 🏗️ Architecture

```
Python Application (native_host.py)
        ↓
   PyO3 Bindings (this crate)
        ↓
   ┌─────────────────────────────┐
   │  Rust Core                  │
   ├─────────────────────────────┤
   │  • hardware (detection)     │
   │  • model-loader (FFI)       │
   └─────────────────────────────┘
        ↓
   llama.dll (llama.cpp)
        ↓
   GGUF Model Inference
```

## 🔄 Integration with Native Host

### native_host.py Usage

```python
import tabagent_model
import tabagent_model_cache  # For model downloads

# 1. Initialize on startup
cpu_variant = tabagent_model.get_cpu_variant()
logger.info(f"Detected CPU: {cpu_variant}")

# 2. Handle model load request
def handle_load_model(repo_id, quant):
    # Download if needed (via model-cache)
    cache = tabagent_model_cache.ModelCache("./cache")
    if not cache.has_file(repo_id, f"{quant}.gguf"):
        cache.download_quant(repo_id, quant)
    
    # Get model file
    model_data = cache.get_file(repo_id, f"{quant}.gguf")
    
    # Write to temp file (llama.cpp needs file path)
    with tempfile.NamedTemporaryFile(delete=False, suffix=".gguf") as f:
        f.write(model_data)
        temp_path = f.name
    
    # Load model
    dll_path = tabagent_model.get_optimal_binary("llama.dll")
    model = tabagent_model.PyModel()
    model.load(
        dll_path=dll_path,
        model_path=temp_path,
        gpu_layers=32,
        mlock=True
    )
    
    return model

# 3. Generate text (when implemented)
def generate(model, prompt, max_tokens=100):
    # Will be implemented in model-loader Phase 3
    output = model.generate(prompt, max_tokens=max_tokens)
    return output
```

## 🔮 Future Enhancements

- [ ] GPU detection functions
- [ ] Model generation/inference methods
- [ ] Streaming generation with callbacks
- [ ] Async Python API
- [ ] Type hints / stubs for IDE support
- [ ] Context manager for model lifecycle

## 📄 License

Same as parent project (see root LICENSE)

## 🤝 Contributing

This is part of the TabAgent project. See main README for contribution guidelines.

