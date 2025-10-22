# 🦀 Rust GGUF Handler Migration Plan

**Goal:** Add Rust handlers for GGUF models (BitNet, llama.cpp) while keeping Python for other backends

**Status:** Planning phase - DO NOT DELETE until all phases complete

---

## 📋 Current State Assessment

### ✅ What We Have

**Transport Layer:**
- ✅ `native_host.py` - Chrome Native Messaging (stdin/stdout)
- ✅ `api/main.py` - FastAPI HTTP server
- ✅ Both use shared `core/inference_service.py` (DRY principle working!)

**Backend Layer:**
- ✅ `backends/bitnet/` - BitNet manager (subprocess approach)
- ✅ `backends/llamacpp/` - llama.cpp manager (subprocess approach)
- ✅ `backends/onnxrt/` - ONNX Runtime (subprocess)
- ✅ `backends/mediapipe/` - MediaPipe (subprocess)
- ✅ `backends/lmstudio/` - LM Studio (HTTP proxy)

**Issues:**
- ❌ Paths wrong: Looking for `BitNet/Release/` but it's `BitNet/BitnetRelease/{variant}/`
- ❌ No hardware detection (CPU variant selection)
- ❌ Subprocess overhead (spawn process + HTTP to localhost)
- ❌ No Rust handlers yet

### 🎯 What We Want

**Transport Layer:** (NO CHANGE)
- ✅ Keep `native_host.py`
- ✅ Keep `api/main.py`
- ✅ Keep shared `core/inference_service.py`

**Backend Layer:** (ADD RUST)
- ✅ Rust library for GGUF (BitNet + llama.cpp) via PyO3
- ✅ Python for others (ONNX, MediaPipe, LM Studio)
- ✅ Dual handler pattern: both see messages, self-select

**Result:**
- Single process (no subprocess for GGUF)
- Direct FFI to llama.dll/libllama.so (per DEVELOPER_GUIDE)
- Still use Python when needed (AI ecosystem)

---

## 🏗️ Phase Breakdown

### Phase 0: Preparation & Audit ⏳

**Goal:** Understand what we have, verify sharing works

**Tasks:**
1. ✅ Audit `core/inference_service.py` - verify both transports use it
2. ✅ Check if backends are truly shared
3. ✅ Document current message flow
4. ⏳ List all files that import backends
5. ⏳ Verify BitNet/BitnetRelease structure exists

**Output:** 
- Clear map of current architecture
- List of files to modify
- Confirmation of what NOT to change

**Blockers:**
- QUESTION: Does extension currently work with native_host.py?
- QUESTION: Does FastAPI currently work?
- QUESTION: Which platform are you developing on? (affects variant selection)

---

### Phase 1: Fix Python Backends (Quick Win) 🔧

**Goal:** Make current system work properly before adding Rust

**Tasks:**
1. Fix paths in `backends/bitnet/manager.py`:
   - Change: `BitNet/Release/` → `BitNet/BitnetRelease/`
   - Add variant selection logic (portable, zen2, zen3, alderlake, etc.)

2. Fix paths in `backends/llamacpp/manager.py`:
   - Same path fixes
   - Add variant selection

3. Add hardware detection:
   - Create `hardware/cpu_detector.py` (or enhance existing?)
   - Detect: AMD Zen2/3/4/5, Intel Alderlake/Skylake/etc.
   - Return optimal variant name

4. Test with existing transports:
   - Test via native_host.py (if working)
   - Test via FastAPI HTTP (curl/Postman)

**Files to modify:**
- `backends/bitnet/manager.py`
- `backends/llamacpp/manager.py`
- `hardware/` (check if detector exists, else create)

**Files NOT to touch:**
- `native_host.py` (works as-is)
- `api/main.py` (works as-is)
- `core/inference_service.py` (works as-is)

**Output:**
- Working GGUF inference via subprocess (current approach)
- Correct paths to BitnetRelease variants
- Hardware detection working
- **Baseline for comparison when we add Rust**

**Success Criteria:**
- [ ] Can load a GGUF model via native_host.py
- [ ] Can load a GGUF model via FastAPI
- [ ] Correct variant selected for your CPU
- [ ] No crashes, proper error messages

---

### Phase 2: Create Rust Library Structure 🦀

**Goal:** Build Rust library that Python can import

**Tasks:**
1. Create Rust workspace:
   ```
   Server/
   ├── tabagent-rs/          ← NEW
   │   ├── Cargo.toml        (workspace)
   │   ├── gguf-handler/     (library crate)
   │   │   ├── Cargo.toml
   │   │   ├── src/
   │   │   │   ├── lib.rs
   │   │   │   ├── bitnet.rs
   │   │   │   └── llamacpp.rs
   │   │   └── build.rs      (PyO3 config)
   ```

2. Implement BitNet FFI (following DEVELOPER_GUIDE):
   - Load `llama.dll`/`libllama.so` via libloading
   - Implement: load_model, tokenize, decode, sample, detokenize
   - Handle hardware detection (call Python's detector? or port to Rust?)

3. PyO3 bindings:
   - Expose: `load_model()`, `generate()`, `unload_model()`
   - Make it importable: `from tabagent_rs import GGUFHandler`

4. Build system:
   - Use `maturin` for building
   - Output: `tabagent_rs.pyd` (Windows) or `.so` (Linux)

**Files to create:**
- `Server/tabagent-rs/` (entire directory)

**Dependencies:**
- libloading (Rust crate for dynamic library loading)
- PyO3 (Rust-Python bindings)
- maturin (build tool)

**Output:**
- Python-importable Rust library
- Can call from Python: `handler = GGUFHandler(variant_path, model_path)`

**Success Criteria:**
- [ ] `maturin build` succeeds
- [ ] Can import in Python: `from tabagent_rs import GGUFHandler`
- [ ] Can load a model via Rust
- [ ] Can generate text via Rust
- [ ] Response matches subprocess approach (validates correctness)

**Blockers/Questions:**
- QUESTION: Should hardware detection be in Rust or call Python's?
- QUESTION: Which CPU variant should we test with first?

---

### Phase 3: Integrate Dual Handler Pattern 🔀

**Goal:** Wire Rust handlers into existing inference flow

**Tasks:**
1. Create `core/handler_router.py`:
   ```python
   def route_message(message: Dict) -> Optional[Response]:
       # Try Rust handler
       rust_result = try_rust_handler(message)
       if rust_result:
           return rust_result
       
       # Try Python handlers
       python_result = try_python_handler(message)
       return python_result
   ```

2. Modify `core/inference_service.py`:
   - Import: `from tabagent_rs import GGUFHandler`
   - Add: `self.rust_handler: Optional[GGUFHandler] = None`
   - In `load_model()`: check if GGUF, route to Rust or Python

3. Update backend selection logic:
   - If `model_path.endswith('.gguf')` or `is_bitnet_model()`:
     → Use Rust handler
   - Else:
     → Use existing Python backends

**Files to modify:**
- NEW: `core/handler_router.py`
- MODIFY: `core/inference_service.py` (add Rust handler)
- NO CHANGE: `native_host.py` (still just calls InferenceService)
- NO CHANGE: `api/main.py` (still just calls InferenceService)

**Output:**
- Both native_host and FastAPI automatically use Rust for GGUF
- Python backends still work for others
- Single codebase, dual handlers

**Success Criteria:**
- [ ] GGUF model → Rust handler (verify via logging)
- [ ] ONNX model → Python handler (verify via logging)
- [ ] Works via native_host.py
- [ ] Works via FastAPI
- [ ] Faster than subprocess (measure!)

---

### Phase 4: Remove Subprocess Approach 🗑️

**Goal:** Clean up old code, use direct FFI only

**Tasks:**
1. Update `backends/bitnet/manager.py`:
   - Remove subprocess/WrappedServer code
   - Become thin wrapper around Rust handler
   - Or deprecate entirely?

2. Update `backends/llamacpp/manager.py`:
   - Same as above

3. Update `core/inference_service.py`:
   - Remove subprocess initialization
   - Only use Rust handler for GGUF

4. Clean up unused imports

**Files to modify:**
- `backends/bitnet/manager.py` (simplify or deprecate)
- `backends/llamacpp/manager.py` (simplify or deprecate)
- `core/inference_service.py` (remove subprocess code)

**Files to potentially archive:**
- `server_mgmt/` (if only used for subprocess management)

**Output:**
- Cleaner codebase
- No subprocess overhead
- Direct FFI path for all GGUF

**Success Criteria:**
- [ ] No subprocess spawning for GGUF models
- [ ] Tests still pass
- [ ] Both transports still work

---

### Phase 5: Testing & Validation ✅

**Goal:** Ensure everything works, measure improvements

**Tasks:**
1. Functional testing:
   - Test each backend (BitNet, llama.cpp, ONNX, MediaPipe)
   - Test via native_host.py
   - Test via FastAPI
   - Test error cases (model not found, wrong format, etc.)

2. Performance testing:
   - Measure old subprocess approach (from Phase 1)
   - Measure new Rust approach
   - Compare: startup time, inference speed, memory usage

3. Documentation:
   - Update README with new architecture
   - Document how to build Rust library
   - Document hardware detection

**Output:**
- Test suite passing
- Performance metrics
- Updated docs

**Success Criteria:**
- [ ] All tests pass
- [ ] Performance improved over baseline
- [ ] Documentation complete
- [ ] Ready for production use

---

## 📊 Progress Tracking

### Current Phase: Phase 0 (Preparation)

**Completed:**
- [x] Created this plan
- [x] Identified shared code (inference_service.py)
- [x] Identified backends structure

**In Progress:**
- [ ] Verify BitnetRelease structure
- [ ] Check current working state
- [ ] Answer blocker questions

**Blocked On:**
- Extension working status?
- Development platform?
- Test models available?

---

## 🚧 Blockers & Questions

**Must answer before proceeding:**

1. **Does extension work with current native_host.py?**
   - If yes: We have baseline to test against
   - If no: Need to fix that first

2. **Which platform are you developing on?**
   - Windows / Linux / macOS?
   - Affects which variant to target first

3. **Do you have test GGUF models?**
   - Need small model for testing (1-3GB)
   - BitNet model and regular GGUF

4. **Is FastAPI currently working?**
   - Can we test via HTTP now?
   - Or is it also in development?

---

## 🎯 Success Metrics

**Phase 1 Success:**
- Subprocess approach works with correct paths
- Can load and infer with GGUF models
- Baseline performance measured

**Phase 2 Success:**
- Rust library builds
- Can import from Python
- Direct FFI works

**Phase 3 Success:**
- Dual handler pattern working
- Both transports use it
- Transparent to clients

**Final Success:**
- ✅ GGUF models use Rust (fast, direct FFI)
- ✅ Other models use Python (rich ecosystem)
- ✅ Single server, single codebase
- ✅ Both native_host and FastAPI work
- ✅ Performance improved
- ✅ Ready to add WebRTC later (just another transport wrapper)

---

## 📝 Notes

- This plan assumes we keep native_host.py as Python (easier migration)
- WebRTC is deferred to after this is working
- Database integration (embedded-db-rs) is separate effort
- Can parallelize: someone on Rust library, someone on Python path fixes

---

**WHEN TO DELETE THIS FILE:**
- After Phase 5 complete
- After production deployment
- After WebRTC added (if desired)
- Create permanent docs before deleting

**CURRENT STATUS:** Planning - waiting on blocker questions

