# Native Handler - TODO

## ✅ Phase 1: Structure (COMPLETE)

- [x] Crate created with PyO3
- [x] `handle_message` function signature (NO Option - always returns response)
- [x] Message parsing (action, modelPath)
- [x] Action routing (match Python's actions)
- [x] Skeleton handlers for all actions (LOAD, GENERATE, UNLOAD, etc.)
- [x] README and TODO documentation
- [x] No fallback logic - Rust MUST handle GGUF/BitNet

## 🔄 Phase 2: Model Cache Integration (IN PROGRESS)

### Download & Storage
- [ ] Integrate `model-cache` crate
- [ ] Implement `handle_pull_model`:
  - [ ] Parse repo_id and quant variant
  - [ ] Call `ModelCache::scan_repo`
  - [ ] Call `ModelCache::download_quant`
  - [ ] Progress callbacks to Python
  - [ ] Return success with file info
- [ ] Implement `handle_delete_model`:
  - [ ] Call `ModelCache::delete_model`
  - [ ] Return success

### File Serving
- [ ] Check if model file exists in cache
- [ ] Assemble chunks into file path
- [ ] Handle file-not-found errors

## 📋 Phase 3: Model Loading (PENDING - Blocked on model-loader)

### Load Model
- [ ] Implement `handle_load_model`:
  - [ ] Check model-cache for file
  - [ ] Get file path or download
  - [ ] Call `model_loader::Model::load`
  - [ ] Store model handle
  - [ ] Return model metadata
- [ ] Implement `handle_unload_model`:
  - [ ] Drop model from memory
  - [ ] Free resources
- [ ] Implement `handle_get_model_state`:
  - [ ] Check if model loaded
  - [ ] Return model info

### Generate (Blocked - needs model-loader Phase 3)
- [ ] Implement `handle_generate`:
  - [ ] Parse messages array
  - [ ] Call model tokenize
  - [ ] Call model generate
  - [ ] Stream tokens via progress callback
  - [ ] Return final text

## 🚀 Phase 4: Database Integration (PENDING)

### Chat History
- [ ] Save generated messages to storage
- [ ] Use `storage::StorageManager`
- [ ] Create Message nodes
- [ ] Link to Chat nodes
- [ ] Update conversation history

### Embeddings
- [ ] Generate embeddings for messages
- [ ] Store in indexing layer
- [ ] Enable semantic search

## 📋 Phase 5: Python Integration (PENDING)

### native_host.py Updates
- [ ] Add model type detection helpers (is_gguf_or_bitnet, is_onnx, etc.)
- [ ] Import `tabagent_native_handler.handle_message`
- [ ] Route GGUF/BitNet → Rust (NO fallback)
- [ ] Route ONNX/MediaPipe → Python
- [ ] Remove/deprecate Python GGUF backends (backends/bitnet/, backends/llamacpp/)
- [ ] Error handling for unknown model types

### Testing
- [ ] Unit tests for message parsing
- [ ] Integration test with model-cache
- [ ] Integration test with model-loader (when ready)
- [ ] End-to-end test via native_host.py

## 🐛 Known Issues

- ⚠️ All handlers return skeleton responses (not functional)
- ⚠️ No model-loader integration yet (Phase 3 blocked)
- ⚠️ No database integration yet
- ⚠️ No streaming support for generation
- ⚠️ No progress callbacks implemented

## 📊 Progress

- **Phase 1 (Structure)**: ✅ 100% Complete
- **Phase 2 (Cache)**: 🔴 0% (not started)
- **Phase 3 (Loading)**: 🔴 0% (blocked on model-loader)
- **Phase 4 (Database)**: 🔴 0% (not started)
- **Phase 5 (Integration)**: 🔴 0% (not started)
- **Overall**: **SKELETON** - Structure ready, implementation needed

## 🔗 Dependencies

### Upstream (Rust):
- `model-cache` - Must have download/serve APIs
- `model-loader` - Must have Phase 3 (inference)
- `storage` - Ready for use
- `hardware` - Ready for use

### Downstream (Python):
- `native_host.py` - Primary consumer
- Will replace Python GGUF logic over time

## 🎯 Next Steps

1. **Immediate:** Implement model-cache integration (Phase 2)
2. **Blocked:** Wait for model-loader Phase 3 for generation
3. **Then:** Integrate with native_host.py
4. **Finally:** Test end-to-end flow

## 🧪 Testing Strategy

```bash
# 1. Build wheel
maturin build --release

# 2. Install in Python
pip install target/wheels/tabagent_native_handler-*.whl

# 3. Test from Python
python -c "
from tabagent_native_handler import try_handle_message
import json

msg = json.dumps({'action': 'LOAD_MODEL', 'modelPath': 'model.gguf'})
result = try_handle_message(msg)
print(result)
"

# 4. Integrate with native_host.py
# 5. Test via Chrome extension
```

## 📝 Notes

- This crate is a MESSAGE HANDLER for GGUF/BitNet (NOT a router)
- Python does the routing ONCE, then calls this
- NO fallback - Rust MUST handle GGUF/BitNet messages
- Uses existing Rust crates (DRY principle)
- Matches Python's message format exactly
- Will REPLACE backends/bitnet/ and backends/llamacpp/ entirely
- Future: Will handle ALL model types as we migrate to Rust

## 🚀 Migration Path

```
Current State:
├─ Python: bitnet, llamacpp, onnx, mediapipe
└─ Rust: None (this is new!)

After Phase 5:
├─ Python: onnx, mediapipe
└─ Rust: GGUF, BitNet

Future State:
├─ Python: transformers/safetensors only (if needed)
└─ Rust: ALL model types
```

