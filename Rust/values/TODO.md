# TabAgent Values - TODO

## ✅ Completed

- [x] Core value system architecture
- [x] Type marker traits for compile-time safety
- [x] Request value types (Chat, Generate, Embeddings, etc.)
- [x] Response value types (ChatResponse, GenerateResponse, etc.)
- [x] Model data value types (Tensor, Embedding)
- [x] Runtime type information (ValueType enum)
- [x] Error types with thiserror
- [x] Borrowed value variants (ValueRef)
- [x] Basic serialization support
- [x] Downcast support for dynamic to concrete types

## 🚧 In Progress

- [ ] Comprehensive test suite
  - [ ] Request creation tests
  - [ ] Response creation tests
  - [ ] Serialization/deserialization tests
  - [ ] Downcast tests
  - [ ] Property-based tests with proptest
  - [ ] Edge case tests

## 📋 TODO

### High Priority

- [ ] **Mutable borrowed variants** - Add `ValueRefMut` for mutable zero-copy access
- [ ] **Streaming support** - Add support for streaming responses (SSE/WebSocket)
- [ ] **Validation** - Add validation for value constraints
  - [ ] Temperature bounds (0.0-2.0)
  - [ ] Token limits
  - [ ] Model ID format validation
- [ ] **Builder pattern** - Add builder for complex request construction
- [ ] **Conversion traits** - Implement From/TryFrom for common types

### Medium Priority

- [ ] **Value pools** - Memory pools for frequently created values (performance)
- [ ] **Compression** - Optional compression for large values
- [ ] **Binary serialization** - Add bincode support for faster serialization
- [ ] **Async validation** - Async validation against model manifests
- [ ] **Metrics** - Add instrumentation for value creation/lifetime

### Low Priority

- [ ] **Python bindings** - PyO3 bindings for Python interop
- [ ] **C FFI** - C bindings for maximum compatibility
- [ ] **WASM support** - Compile to WebAssembly
- [ ] **Benchmarks** - Performance benchmarks vs alternatives
- [ ] **Documentation examples** - More real-world examples

## 🔍 Future Enhancements

### Advanced Type System

- [ ] **Type-level validation** - Const generics for compile-time validation
- [ ] **State machine types** - Request/response state transitions
- [ ] **Plugin system** - Extensible value types for custom models

### Performance

- [ ] **Zero-copy deserialization** - Deserialize directly from network buffers
- [ ] **SIMD optimizations** - SIMD for tensor operations
- [ ] **Lock-free structures** - Lock-free pools and caches

### Integration

- [ ] **gRPC support** - Protocol buffer integration
- [ ] **OpenAPI schema** - Generate OpenAPI schemas from types
- [ ] **JSON schema** - Generate JSON schema for validation

## 🐛 Known Issues

None currently.

## 📝 Notes

### Design Decisions

- **Why enums for ValueData?** - Type safety, prevents invalid combinations
- **Why marker traits?** - Compile-time type checking without runtime overhead
- **Why Box for inner data?** - Keeps Value small, single allocation for all data
- **Why separate tests directory?** - Keeps source files clean, follows Rust conventions

### Testing Strategy

1. **Unit tests** - Each function has at least one test
2. **Integration tests** - Full request/response cycles
3. **Property tests** - Proptest for invariants
4. **Edge cases** - Boundary conditions, empty values, large values
5. **Real data** - No mocks, test with actual model data

### Migration Path for Other Crates

1. **Phase 1**: `tabagent-values` standalone (current)
2. **Phase 2**: Integrate with `server` crate
3. **Phase 3**: Integrate with `onnx-loader` and `gguf-loader`
4. **Phase 4**: Integrate with `storage` and `query`
5. **Phase 5**: Full system using unified value types

## 🎯 Success Criteria

- [ ] Zero compiler warnings
- [ ] 100% test coverage for public API
- [ ] All tests passing
- [ ] Documentation complete with examples
- [ ] Used in at least one other crate (server)
- [ ] Performance benchmarks showing <5% overhead vs direct types

