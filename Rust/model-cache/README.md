# Model Cache Crate

**Chunked model file storage and HuggingFace download management.**

## Purpose

The `model-cache` crate solves the critical problem of **storing and serving large AI model files** (ONNX, GGUF, tokenizers) from HuggingFace repositories. It mirrors the functionality of `src/DB/idbModel.ts` (extension's IndexedDB model storage) but in high-performance Rust with `libmdbx` as the backend.

### Problem This Solves

**Before:** Extension uses IndexedDB to cache model files in chunks, serve them as blobs to transformers.js. Server had NO equivalent - would require re-downloading models every time or using massive memory.

**After:** Rust-native model caching with:
- Chunked storage (100MB chunks) in libmdbx with zero-copy access
- Progressive downloads with callbacks
- Manifest management (track which models/quants are downloaded)
- Zero-copy serving to both Rust and Python inference engines

## Inspiration

Directly inspired by `src/DB/idbModel.ts` and `src/backgroundModelManager.ts`:
- Extension downloads HuggingFace models in chunks
- Stores in IndexedDB for persistence
- Serves via blob URLs to transformers.js
- Manifest tracks download status per quantization variant

This crate brings that same pattern to the Rust server for native app deployment.

## Responsibilities

### 1. Chunked File Storage
- **100MB chunks**: Large model files split for efficient storage/retrieval
- **libmdbx persistence**: Memory-mapped zero-copy storage with `rkyv` serialization
- **Streaming access**: Reassemble chunks on-demand for inference without RAM accumulation

### 2. HuggingFace Integration
- **Repository scanning**: List files via HF API
- **Progressive downloads**: Stream files with progress callbacks
- **Quant detection**: Auto-detect quantization variants (q4, fp16, etc.)
- **Metadata extraction**: Pipeline task, model info

### 3. Manifest Management
- **Per-repo manifests**: Track available/downloading/downloaded/failed status
- **Quant tracking**: Manage multiple quantization variants per model
- **Status updates**: Real-time progress for UI feedback

### 4. Multi-Client Serving
- **Rust model-loader**: Serve files to FFI-based GGUF inference
- **Python ONNX**: Serve files to ONNX Runtime via PyO3 bindings
- **Future WebRTC**: Stream model files over data channels

## Architecture

```
ModelCache
  ├── ChunkStorage (storage.rs)
  │   ├── chunks tree: "file:{repo}:{path}:chunk:{n}" -> bytes
  │   └── metadata tree: "meta:{repo}:{path}" -> FileMetadata
  │
  ├── ModelDownloader (download.rs)
  │   ├── HuggingFace API client
  │   ├── Progressive streaming
  │   └── Progress callbacks
  │
  └── Manifest Manager (manifest.rs)
      ├── manifests tree: "manifest:{repo}" -> ManifestEntry
      └── Per-quant status tracking
```

### Data Flow

```
Python/Rust Request
    │
    ▼
ModelCache::get_file(repo, path)
    │
    ├─ Cache HIT? ✅
    │   └─ ChunkStorage::get_file() -> reconstruct from chunks
    │
    └─ Cache MISS? ❌
        └─ ModelDownloader::download_file()
            │
            ├─ Stream from HuggingFace
            ├─ Report progress (0-100%)
            ├─ ChunkStorage::store_file() (5MB chunks)
            └─ Update manifest status
```

## Usage

### Rust API

```rust
use tabagent_model_cache::{ModelCache, QuantStatus};
use storage::StorageRegistry;
use std::sync::Arc;

// Create storage registry (centralized database management)
let registry = Arc::new(StorageRegistry::new("./data"));

// Create cache (registers model_cache_chunks + model_cache_manifests)
let cache = ModelCache::new(registry, Path::new("./model_cache_db"))?;

// Scan HuggingFace repo
let manifest = cache.scan_repo("onnx-community/Phi-3.5-mini-instruct-onnx-web").await?;
println!("Found {} quantizations", manifest.quants.len());

// Download a specific quant variant
cache.download_quant(
    "onnx-community/Phi-3.5-mini-instruct-onnx-web",
    "q4f16",
    Some(Arc::new(|loaded, total| {
        println!("Progress: {}/{}MB", loaded / 1024 / 1024, total / 1024 / 1024);
    }))
).await?;

// Retrieve cached file
if let Some(data) = cache.get_file("onnx-community/Phi-3.5-mini-instruct-onnx-web", "onnx/model_q4f16.onnx")? {
    println!("File size: {} bytes", data.len());
}

// Check status
let manifest = cache.get_manifest("onnx-community/Phi-3.5-mini-instruct-onnx-web").await?;
if let Some(quant) = manifest.quants.get("q4f16") {
    match quant.status {
        QuantStatus::Downloaded => println!("Ready for inference!"),
        QuantStatus::Downloading => println!("Download in progress..."),
        _ => println!("Not available"),
    }
}
```

## Performance

| Operation | Complexity | Notes |
|-----------|------------|-------|
| Store file (chunked) | O(n/chunk_size) | 100MB chunks, ~100 chunks for 10GB model |
| Get file (cached) | O(chunks) | Zero-copy mmap access from libmdbx |
| Download | Network bound | Streaming with progress callbacks |
| Scan repo | 1 API call | Lists all files at once |

### Storage Efficiency

- **Zero-copy reads**: Direct memory-mapped access, no deserialization overhead
- **MDBX_RESERVE writes**: Guaranteed alignment, no intermediate buffers
- **OS page cache**: Hot data stays in RAM, cold data on disk
- **CRC32C validation**: Hardware-accelerated (SSE4.2) data integrity

## When Code Hits This Crate

### Flow 1: Extension Pattern (Browser)
```
User clicks "Load Model" in UI
    ↓
idbModel.ts: Check IndexedDB manifest
    ↓
Background fetch intercept
    ↓
Chunked download + IndexedDB storage
    ↓
Serve as blob URL to transformers.js
```

### Flow 2: Server Pattern (Native App) **← THIS CRATE**
```
User clicks "Load Model" in UI
    ↓
native_host.py: Check Rust cache via PyO3
    ↓
tabagent_model_cache.scan_repo()
    ↓
Rust downloads + libmdbx storage (100MB chunks, zero-copy)
    ↓
Serve zero-copy streams to:
  • Rust model-loader (GGUF via FFI)
  • Python ONNX Runtime
  • Future: WebRTC streaming
```

### Flow 3: Auto-Download on Inference
```
Python: model_loader.load(repo, quant)
    ↓
Check cache.has_file(repo, "model.onnx")
    ↓
If missing: cache.download_file() with progress
    ↓
If present: cache.get_file() -> bytes
    ↓
Load into inference engine
```

## Dependencies

- `storage` - Core storage layer with libmdbx engine and zero-copy FFI
- `libmdbx` - High-performance embedded transactional key-value store
- `rkyv` - Zero-copy deserialization framework
- `reqwest` - HuggingFace API client
- `tokio` - Async runtime for downloads
- `chrono` - Timestamp management

## See Also

- **Parent**: [Main README](../README.md)
- **Python Bindings**: [model-cache-bindings/README.md](../model-cache-bindings/README.md)
- **Extension Equivalent**: `/src/DB/idbModel.ts`
- **Progress**: [TODO.md](./TODO.md)

