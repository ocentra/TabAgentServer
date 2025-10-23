# Model Cache Crate

**Chunked model file storage and HuggingFace download management.**

## Purpose

The `model-cache` crate solves the critical problem of **storing and serving large AI model files** (ONNX, GGUF, tokenizers) from HuggingFace repositories. It mirrors the functionality of `src/DB/idbModel.ts` (extension's IndexedDB model storage) but in high-performance Rust with `sled` as the backend.

### Problem This Solves

**Before:** Extension uses IndexedDB to cache model files in chunks, serve them as blobs to transformers.js. Server had NO equivalent - would require re-downloading models every time or using massive memory.

**After:** Rust-native model caching with:
- Chunked storage (5MB chunks) in sled
- Progressive downloads with callbacks
- Manifest management (track which models/quants are downloaded)
- Blob serving to both Rust and Python inference engines

## Inspiration

Directly inspired by `src/DB/idbModel.ts` and `src/backgroundModelManager.ts`:
- Extension downloads HuggingFace models in chunks
- Stores in IndexedDB for persistence
- Serves via blob URLs to transformers.js
- Manifest tracks download status per quantization variant

This crate brings that same pattern to the Rust server for native app deployment.

## Responsibilities

### 1. Chunked File Storage
- **5MB chunks**: Large model files split for efficient storage/retrieval
- **sled persistence**: Binary storage with `bincode` serialization
- **Blob reconstruction**: Reassemble chunks on-demand for inference

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

// Create cache
let cache = ModelCache::new("./model_cache_db")?;

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
| Store file (chunked) | O(n/chunk_size) | 5MB chunks, ~200 chunks for 1GB model |
| Get file (cached) | O(chunks) | Fast reconstruction from sled B-tree |
| Download | Network bound | Streaming with progress callbacks |
| Scan repo | 1 API call | Lists all files at once |

### Storage Efficiency

- **Deduplication**: Common files (config.json, tokenizer) shared across quants
- **Chunk reuse**: Identical chunks (rare) stored once
- **Compression**: sled's built-in compression for chunks

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
Rust downloads + sled storage (5MB chunks)
    ↓
Serve bytes to:
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

- `sled` - Embedded key-value store for chunks
- `reqwest` - HuggingFace API client
- `tokio` - Async runtime for downloads
- `bincode` - Fast binary serialization
- `chrono` - Timestamp management

## See Also

- **Parent**: [Main README](../README.md)
- **Python Bindings**: [model-cache-bindings/README.md](../model-cache-bindings/README.md)
- **Extension Equivalent**: `/src/DB/idbModel.ts`
- **Progress**: [TODO.md](./TODO.md)

