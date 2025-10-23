# TabAgent Model Cache - Python Bindings

Python bindings for the TabAgent model cache, enabling HuggingFace model management from Python.

## ✨ Features

- 🚀 **Native Performance**: Rust-powered chunked storage and downloads
- 📦 **HuggingFace Integration**: Automatic repo scanning and file listing
- 📊 **Progress Tracking**: Real-time download progress callbacks
- 💾 **Persistent Cache**: Sled-based storage with deduplication
- 🔄 **Async Support**: Built on Tokio for concurrent operations
- 🎯 **Manifest Management**: Track multiple quantization variants per model

## Purpose

This crate exposes the `model-cache` Rust functionality to Python via PyO3, enabling `native_host.py` to:
1. Cache large model files efficiently (like extension's IndexedDB)
2. Download from HuggingFace with progress reporting
3. Serve cached files to both Rust (GGUF) and Python (ONNX) inference engines

## 📦 Installation

### From Wheel (Recommended)

```bash
# Build the wheel
cd Server/Rust/model-cache-bindings
maturin build --release

# Install
pip install ../target/wheels/tabagent_model_cache-0.1.0-cp39-abi3-win_amd64.whl
```

### From Source

```bash
pip install maturin
cd Server/Rust/model-cache-bindings
maturin develop --release
```

## 🚀 Quick Start

```python
import tabagent_model_cache

# Create cache
cache = tabagent_model_cache.ModelCache("./model_cache_db")

# Scan a HuggingFace repository
manifest = cache.scan_repo("onnx-community/Phi-3.5-mini-instruct-onnx-web")
print(f"Found {len(manifest['quants'])} quantizations")

# Download a specific quant with progress
def progress_callback(loaded, total):
    percent = (loaded / total * 100) if total > 0 else 0
    print(f"Progress: {percent:.1f}%", end='\r')

cache.download_quant(
    "onnx-community/Phi-3.5-mini-instruct-onnx-web",
    "q4f16",
    progress_callback
)

# Check if files are cached
has_model = cache.has_file(
    "onnx-community/Phi-3.5-mini-instruct-onnx-web",
    "onnx/model_q4f16.onnx"
)

# Retrieve cached file
if has_model:
    file_data = cache.get_file(
        "onnx-community/Phi-3.5-mini-instruct-onnx-web",
        "onnx/model_q4f16.onnx"
    )
    print(f"File size: {len(file_data)} bytes")

# Get cache statistics
stats = cache.get_stats()
print(f"Total repos: {stats['total_repos']}")
print(f"Total size: {stats['total_size'] / 1024 / 1024:.1f} MB")
```

## 📚 API Reference

### ModelCache

Main cache management class.

#### `__init__(db_path: str)`

Create a new model cache instance.

**Parameters:**
- `db_path` (str): Path to sled database directory

**Example:**
```python
cache = tabagent_model_cache.ModelCache("./cache_db")
```

---

#### `scan_repo(repo_id: str) -> dict`

Scan a HuggingFace repository and update manifest.

**Parameters:**
- `repo_id` (str): HuggingFace repo ID (e.g., "username/model-name")

**Returns:**
- `dict`: Manifest with structure:
  ```python
  {
      "repo_id": str,
      "task": str | None,  # e.g., "text-generation"
      "created_at": int,
      "updated_at": int,
      "quants": {
          "q4f16": {
              "status": "available" | "downloading" | "downloaded" | "failed",
              "files": list[str],
              "total_size": int | None,
              "downloaded_size": int | None,
              "last_updated": int
          },
          ...
      }
  }
  ```

**Example:**
```python
manifest = cache.scan_repo("onnx-community/Phi-3.5-mini-instruct-onnx-web")
for quant, info in manifest['quants'].items():
    print(f"{quant}: {info['status']}")
```

---

#### `get_manifest(repo_id: str) -> dict | None`

Get cached manifest for a repository.

**Parameters:**
- `repo_id` (str): HuggingFace repo ID

**Returns:**
- `dict | None`: Manifest dict or None if not scanned yet

---

#### `download_file(repo_id: str, file_path: str, progress_callback: callable = None)`

Download a specific file from HuggingFace.

**Parameters:**
- `repo_id` (str): HuggingFace repo ID
- `file_path` (str): File path within repo (e.g., "onnx/model.onnx")
- `progress_callback` (callable, optional): `callback(loaded: int, total: int)`

**Example:**
```python
def on_progress(loaded, total):
    print(f"{loaded}/{total} bytes")

cache.download_file(
    "onnx-community/Phi-3.5-mini-instruct-onnx-web",
    "onnx/model_q4f16.onnx",
    on_progress
)
```

---

#### `download_quant(repo_id: str, quant_key: str, progress_callback: callable = None)`

Download all files for a quantization variant.

**Parameters:**
- `repo_id` (str): HuggingFace repo ID
- `quant_key` (str): Quantization key (e.g., "q4f16", "fp16")
- `progress_callback` (callable, optional): Progress callback

**Example:**
```python
cache.download_quant("onnx-community/Phi-3.5-mini-instruct-onnx-web", "q4f16")
```

---

#### `get_file(repo_id: str, file_path: str) -> bytes | None`

Retrieve a cached file.

**Parameters:**
- `repo_id` (str): HuggingFace repo ID
- `file_path` (str): File path within repo

**Returns:**
- `bytes | None`: File contents or None if not cached

---

#### `has_file(repo_id: str, file_path: str) -> bool`

Check if a file is cached.

**Returns:**
- `bool`: True if file is cached

---

#### `delete_model(repo_id: str)`

Delete a model and all its cached files.

**Parameters:**
- `repo_id` (str): HuggingFace repo ID

---

#### `get_stats() -> dict`

Get cache statistics.

**Returns:**
- `dict`: `{"total_repos": int, "total_size": int}`

---

## 🧪 Testing

Run the test script:

```bash
python Server/test_model_cache_bindings.py
```

Expected output:
```
🧪 Testing TabAgent Model Cache Bindings
============================================================
✅ Successfully imported tabagent_model_cache
1️⃣ Creating model cache...
   ✅ Cache created
2️⃣ Getting cache statistics...
   📊 Total repos: 0
   📊 Total size: 0 bytes
...
✅ ALL BASIC TESTS PASSED!
```

## 🏗️ Architecture

```
Python Application (native_host.py)
        ↓
   PyO3 Bindings (this crate)
        ↓
   ┌─────────────────────────────┐
   │  model-cache (Rust)         │
   ├─────────────────────────────┤
   │  • ChunkStorage (sled)      │
   │  • ModelDownloader (reqwest)│
   │  • Manifest Manager         │
   └─────────────────────────────┘
        ↓
   Persistent Storage (sled)
        ↓
   Serve to:
     • Rust model-loader (GGUF/BitNet)
     • Python ONNX Runtime
     • Future: WebRTC streaming
```

## 🔄 Integration with Native Host

### native_host.py Usage

```python
from tabagent_model_cache import ModelCache

# Initialize in native_host
model_cache = ModelCache("./TabAgent/model_cache")

def handle_load_model_request(repo_id, quant):
    # Check manifest
    manifest = model_cache.get_manifest(repo_id)
    if not manifest:
        manifest = model_cache.scan_repo(repo_id)
    
    # Check if downloaded
    quant_info = manifest['quants'].get(quant)
    if quant_info['status'] != 'downloaded':
        # Download with progress
        def progress(loaded, total):
            send_progress_to_ui(loaded, total)
        
        model_cache.download_quant(repo_id, quant, progress)
    
    # Get model file
    model_data = model_cache.get_file(repo_id, f"onnx/model_{quant}.onnx")
    
    # Load into inference engine
    load_model_from_bytes(model_data)
```

## 🔮 Future Enhancements

- [ ] Async Python API (asyncio support)
- [ ] Streaming file access (avoid loading full file in memory)
- [ ] Cache management CLI (list, clean, verify)
- [ ] Type hints / stubs for better IDE support
- [ ] Context managers for transactions

## 📄 License

Same as parent project (see root LICENSE)

## 🤝 Contributing

This is part of the TabAgent project. See main README for contribution guidelines.

