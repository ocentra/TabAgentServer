# AppState TODO

## Phase 1: Core Infrastructure ✅
- [x] Create crate structure
- [x] Add to workspace
- [x] Define `AppState` struct
- [x] Implement `AppStateProvider` trait
- [x] Add HF auth manager
- [x] Add model registries (ONNX, GGUF)

## Phase 2: Business Logic Methods (NEXT)
- [ ] Move logic from `server/src/handler.rs` to `AppState` methods
- [ ] Implement `load_model()` method
- [ ] Implement `unload_model()` method
- [ ] Implement `generate()` method (inference dispatch)
- [ ] Implement `get_embeddings()` method
- [ ] Implement `save_message()` method
- [ ] Implement `get_history()` method
- [ ] Implement HF token operations (set/get/clear)
- [ ] Implement hardware info queries
- [ ] Implement model feasibility checks

## Phase 3: Wire to Transport Crates
- [ ] Update `api` to depend on `appstate`
- [ ] Update `native-messaging` to depend on `appstate`
- [ ] Update `webrtc` to depend on `appstate`
- [ ] Remove `tabagent-server` dependency from transport crates
- [ ] Update transport tests to use real `AppState`

## Phase 4: Server Refactor
- [ ] Update `server` to depend on `appstate`
- [ ] Remove `server/src/handler.rs`
- [ ] Remove `server/src/state.rs` (moved to `appstate`)
- [ ] Remove `server/src/hf_auth.rs` (moved to `appstate`)
- [ ] Simplify `server/src/main.rs` to just orchestration
- [ ] Server creates `AppState` and passes to transports

## Phase 5: Integration Testing
- [ ] Add real integration tests in `appstate/tests/`
- [ ] Test model loading/unloading
- [ ] Test inference dispatch
- [ ] Test HF auth operations
- [ ] Test concurrent access
- [ ] Add end-to-end tests in `server/tests/`

## Phase 6: Python Bridge Integration
- [ ] Add Python ML bridge to `AppState`
- [ ] Implement Python model loading
- [ ] Implement Python inference dispatch
- [ ] Test Transformers integration
- [ ] Test MediaPipe integration

## Architecture Validation
- [ ] Verify no circular dependencies
- [ ] Verify transport crates don't depend on `server`
- [ ] Verify infrastructure crates don't depend on `appstate`
- [ ] Run `cargo check` on all crates
- [ ] Run `cargo test` on all crates
- [ ] Check for dead code/unused imports

## Documentation
- [x] README.md with architecture diagram
- [x] TODO.md (this file)
- [ ] Add inline docs for all public methods
- [ ] Add examples in doc comments
- [ ] Document error handling strategy

## Known Issues
- Python bridge integration pending (depends on Phase 6)
- Generation cancellation needs proper implementation
- Vector index manager is placeholder
- Need to handle model download auth errors properly

