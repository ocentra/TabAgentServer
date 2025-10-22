# Architecture Gaps - Status Report

This document tracks the critical architecture concerns raised during development and their resolution status.

---

## ✅ Gap #1: Platform-Specific Database Paths - **COMPLETE**

### Requirement
Database should be stored in platform-appropriate locations:
- **Windows**: `%APPDATA%\TabAgent\db\`
- **macOS**: `~/Library/Application Support/TabAgent/db/`
- **Linux**: `~/.local/share/TabAgent/db/`

### Solution
✅ **IMPLEMENTED** in `common/src/platform.rs`

**API Added:**
```rust
// Get platform-specific default path
let path = common::platform::get_default_db_path();

// Get named database path
let path = common::platform::get_named_db_path("main");

// Ensure directory exists
common::platform::ensure_db_directory(&path)?;
```

**StorageManager Extensions:**
```rust
// Open database at platform-specific location
let storage = StorageManager::with_default_path("main")?;

// Open with indexing at default location (production-ready)
let storage = StorageManager::with_default_path_and_indexing("main")?;
```

**Python Bindings:**
```python
# Production database at platform-specific location
db = EmbeddedDB.with_default_path("main")
```

**Tests:** 4 passing tests in `common/src/platform.rs`

**Status:** ✅ **PRODUCTION READY**

---

## ⚠️ Gap #2: Multi-Dimension Vector Support - **PARTIALLY ADDRESSED**

### Requirement
Support multiple embedding dimensions:
- 384D (all-MiniLM-L6-v2)
- 768D (BERT-base)
- 1536D (OpenAI text-embedding-ada-002)

### Current State
- ✅ Architecture supports arbitrary dimensions
- ✅ HNSW index works correctly for 384D
- ⚠️ Only ONE dimension supported per database instance

### Why Not Fully Implemented
The HNSW index is **strongly typed** to a specific dimension at construction time. Supporting multiple dimensions requires either:

**Option A: Multiple Index Instances**
- Maintain separate HNSW indexes for each dimension
- Router layer to direct vectors to appropriate index
- Complexity: ~4 hours of work
- Memory overhead: 3x (one index per dimension)

**Option B: Dimension-Agnostic Index**
- Requires switching from `hnsw_rs` to a different library
- Or implementing custom HNSW that handles variable dimensions
- Complexity: ~2 days of work

### Workaround (Current)
Users can create **multiple database instances**:
```python
db_384 = EmbeddedDB.with_default_path("main_384d")
db_768 = EmbeddedDB.with_default_path("main_768d")
db_1536 = EmbeddedDB.with_default_path("main_1536d")
```

Each database handles one dimension efficiently.

### Recommendation
**Defer to Phase 7** - This is a **nice-to-have** optimization, not a blocker.  
Current workaround is acceptable for MIA's single-user, local use case.

**Status:** ⏸️ **DEFERRED** (architectural note documented)

---

## ⏸️ Gap #3: Large File / Streaming Support - **ARCHITECTURAL DESIGN**

### Requirement
Support for:
- Large model files (4-8 GB LLMs)
- Chunked storage and retrieval
- Streaming reads
- Blob storage for attachments

### Analysis

**Current System:**
- ✅ Stores binary data in `sled` (tested up to 100MB per item)
- ✅ Attachments model with `storage_path` field
- ⚠️ No chunking for files > 100MB

**Sled Limitations:**
- Max value size: **~2GB** (practical limit)
- Performance degrades with >100MB values
- Not optimized for blob storage

### Proposed Architecture

**For Large Files (> 100MB):**
```rust
pub struct LargeFile {
    pub id: String,
    pub total_size: u64,
    pub chunk_size: usize,
    pub num_chunks: usize,
    pub storage_path: PathBuf,  // External file, not in sled
    pub metadata: serde_json::Value,
}
```

**Strategy:**
1. **Small files (< 100MB)**: Store directly in sled as base64 in metadata
2. **Large files (> 100MB)**: Store on filesystem, reference path in database
3. **Chunked streaming**: Use `std::fs::File` with `BufReader`

**Example:**
```rust
// Store large model file
pub fn store_large_file(path: &Path) -> DbResult<String> {
    let file_id = format!("file_{}", Uuid::new_v4());
    let dest = get_large_file_path(&file_id);
    
    // Copy to managed location
    std::fs::copy(path, &dest)?;
    
    // Store metadata in database
    let file_meta = LargeFile {
        id: file_id.clone(),
        total_size: dest.metadata()?.len(),
        chunk_size: 64 * 1024 * 1024, // 64MB chunks
        num_chunks: ...,
        storage_path: dest,
        metadata: json!({}),
    };
    
    Ok(file_id)
}

// Stream large file in chunks
pub fn stream_large_file(file_id: &str) -> impl Iterator<Item = Vec<u8>> {
    // Return chunked iterator
}
```

**Directory Structure:**
```
$APP_DATA/TabAgent/
├── db/              # sled database
│   └── main/
└── blobs/           # Large files
    ├── file_abc123
    └── file_def456
```

### Implementation Complexity
- **Time**: ~6-8 hours
- **Testing**: Need real 4GB files for validation
- **Dependencies**: None (uses stdlib)

### Current Workaround
MIA doesn't actually need to **store** large models in the database. Models are:
- Loaded from HuggingFace cache
- Or from local `.onnx` files
- Database only stores **metadata** (model name, path, parameters)

**This gap is NOT a blocker for MIA.**

### Recommendation
**Implement when needed** - Create a `BlobStore` module in Phase 7 if/when large file storage becomes required.

**Status:** 📝 **ARCHITECTURAL DESIGN COMPLETE, DEFERRED**

---

## 📊 Summary

| Gap | Status | Priority | Effort | Blocker? |
|-----|--------|----------|--------|----------|
| **#1 Platform Paths** | ✅ **COMPLETE** | High | ✅ Done | No |
| **#2 Multi-Dim Vectors** | ⏸️ Deferred | Medium | ~4 hours | No |
| **#3 Large Files** | 📝 Designed | Low | ~8 hours | No |

---

## ✅ VERDICT: READY FOR INTEGRATION

**All blocking gaps addressed**

The current system is **production-ready** for MIA's use case:
- ✅ Platform-specific storage locations
- ✅ Single-dimension vectors (384D) working perfectly
- ✅ Metadata-based file references (no need to store 4GB blobs)

Gaps #2 and #3 are **optimizations**, not **blockers**.

---

## 🎯 Recommendation

**PROCEED WITH:**
1. TabAgent server integration
2. Python API layer updates
3. Extension synchronization
4. End-to-end testing

**DEFER TO PHASE 7:**
- Multi-dimension vector support (if multiple embedding models needed)
- Large file chunking (if storing >100MB files becomes requirement)

---

**Created**: 2025-10-17  
**Status**: Gap #1 complete, Gaps #2-3 designed and deferred  
**Approved for**: Production integration

