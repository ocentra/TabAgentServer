# Common Crate TODO

## ✅ Completed

### Architecture Unification (Tier 0)
- [x] Create unified backend trait (`AppStateProvider`)
  - Single trait definition in `src/backend.rs`
  - Blanket implementations for `Arc`, `Box`, wrapper types
  - Comprehensive test coverage
  - Documentation with examples
- [x] Create unified routing trait system (`RouteHandler<M>`)
  - Generic trait parameterized by transport metadata
  - Transport-specific metadata types (`HttpMetadata`, `NativeMessagingMetadata`, `WebRtcMetadata`)
  - Test case support (`TestCase<Req, Resp>`)
  - Compile-time enforcement of route standards
  - Comprehensive tests
- [x] Update crate documentation
  - README with migration guide
  - Lib.rs with architecture overview
  - Examples for both traits
- [x] Add dependencies
  - `async-trait` for async trait support
  - `tabagent-values` for request/response types
  - `anyhow` for error handling
  - `tokio` for async tests

## 🔄 Next Steps (For Other Crates)

### API Crate Migration
- [ ] Replace `api/src/traits.rs::AppStateProvider` with `common::AppStateProvider`
- [ ] Replace `api/src/route_trait.rs::RouteHandler` with `common::RouteHandler<HttpMetadata>`
- [ ] Update all route implementations to use new trait
- [ ] Remove old trait definitions
- [ ] Update tests

### Native Messaging Crate Migration
- [ ] Replace `native-messaging/src/traits.rs::AppStateProvider` with `common::AppStateProvider`
- [ ] Replace `native-messaging/src/route_trait.rs::NativeMessagingRoute` with `common::RouteHandler<NativeMessagingMetadata>`
- [ ] Update all route implementations to use new trait
- [ ] Remove old trait definitions
- [ ] Update tests

### WebRTC Crate Migration
- [ ] Replace `webrtc/src/traits.rs::RequestHandler` with `common::AppStateProvider`
- [ ] Replace `webrtc/src/route_trait.rs::DataChannelRoute` with `common::RouteHandler<WebRtcMetadata>`
- [ ] **CREATE** routing dispatcher system (currently missing)
- [ ] Update all route implementations to use new trait
- [ ] Remove old trait definitions
- [ ] Update tests

### Server Crate Wiring
- [ ] Replace `MockAppState` in `api/src/main.rs` with real backend
- [ ] Replace `MockAppState` in `native-messaging/src/main.rs` with real backend
- [ ] Ensure `server/src/handler.rs` implements `common::AppStateProvider`
- [ ] Wire all three entry points to same backend implementation

## ⚠️ IMPORTANT - Replace Mock Tests with Real Backend (After Migration Complete)

**CRITICAL**: Once all three entry points (API, Native Messaging, WebRTC) have migrated to use the unified traits, we MUST:

- [ ] **Replace MockBackend with real backend from `server/handler.rs`**
  - Location: `tests/backend_integration_tests.rs`
  - Location: `tests/routing_integration_tests.rs`
  - Current: Using `MockBackend` for trait contract testing
  - Target: Use actual GGUF/ONNX/Python backend implementations
  
- [ ] **Add end-to-end integration tests**
  - Test real model loading (GGUF, ONNX)
  - Test real inference with actual models
  - Test real database operations
  - Verify all three transports hit the SAME backend code
  - Test error handling with real backend errors
  
- [ ] **Add cross-transport parity tests**
  - Same request to HTTP, Native Messaging, WebRTC should produce identical responses
  - Verify no transport-specific behavior differences
  - Test that backend is truly transport-agnostic

**Why wait?**
- Right now we're testing the trait system itself (contracts, wrappers, bounds)
- Real backend tests require all three entry points to be wired up
- Avoids circular dependencies during migration

**When to do this?**
- After `tier0-migrate-api` is complete
- After `tier0-migrate-native-messaging` is complete
- After `tier0-migrate-webrtc` is complete
- After `tier0-wire-api-backend` is complete
- After `tier0-wire-native-backend` is complete

## 📋 Future Enhancements (Low Priority)

- [ ] Add middleware trait for composable request processing
- [ ] Add authentication/authorization traits
- [ ] Add metrics collection traits
- [ ] Add rate limiting traits
- [ ] Add streaming support traits
- [ ] Add more transport metadata fields as needed

## 🔍 Potential Issues to Watch

1. **Trait Object Safety**: Ensure all trait methods remain object-safe if dynamic dispatch is needed
2. **Async Trait Overhead**: Monitor performance impact of `async-trait` macro
3. **Metadata Bloat**: Watch for excessive metadata fields that should be in route-specific config
4. **Test Coverage**: Ensure all routes have meaningful test cases, not just stubs

## 📝 Notes

- The unified traits are designed to be **stable** - changes should be rare and well-considered
- All three transport crates should converge on identical patterns after migration
- The `AppStateProvider` trait is intentionally simple to allow maximum flexibility in backend implementation
- The `RouteHandler<M>` trait enforces standards without being overly prescriptive about implementation details
