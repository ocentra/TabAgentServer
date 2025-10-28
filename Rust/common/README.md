# TabAgent Common

**Foundational types, traits, and utilities shared across all TabAgent crates.**

## What's New (Architecture Unification)

### ✅ Unified Backend Trait

**Location**: `src/backend.rs`

- **Single trait** (`AppStateProvider`) used by ALL entry points (API, Native Messaging, WebRTC)
- **DRY principle**: No more duplicated traits with different names
- **Transport agnostic**: Same backend handles HTTP, stdin, WebRTC data channels
- **Type-safe**: Uses `tabagent-values::RequestValue` and `ResponseValue`

```rust
use common::AppStateProvider;
use tabagent_values::{RequestValue, ResponseValue};
use async_trait::async_trait;

struct MyBackend;

#[async_trait]
impl AppStateProvider for MyBackend {
    async fn handle_request(&self, request: RequestValue) 
        -> anyhow::Result<ResponseValue> 
    {
        // Single implementation handles all transports
        Ok(ResponseValue::health("ok"))
    }
}
```

### ✅ Unified Routing Trait

**Location**: `src/routing.rs`

- **Single trait** (`RouteHandler<M>`) used by ALL route implementations
- **Transport-parameterized**: `HttpMetadata`, `NativeMessagingMetadata`, `WebRtcMetadata`
- **Compile-time enforcement**: All routes MUST validate, test, document
- **Consistent API**: Same validation, handling, testing across all transports
- **Axum 0.8 Compatible**: Includes `RegisterableRoute<M>` for transport-specific registration
  - HTTP: Requires concrete `async fn` handlers (no closures)
  - Native Messaging/WebRTC: Can use type-erased handlers

```rust
use common::routing::{RouteHandler, HttpMetadata, TestCase};
use async_trait::async_trait;

struct ChatRoute;

#[async_trait]
impl RouteHandler<HttpMetadata> for ChatRoute {
    type Request = ChatRequest;
    type Response = ChatResponse;
    
    fn metadata() -> HttpMetadata {
        HttpMetadata {
            path: "/v1/chat/completions",
            method: http::Method::POST,
            tags: &["Chat"],
            description: "Chat completions",
            // ... other fields
        }
    }
    
    async fn validate_request(req: &Self::Request) -> anyhow::Result<()> {
        // Validation logic
    }
    
    async fn handle<S>(req: Self::Request, state: &S) -> anyhow::Result<Self::Response>
    where
        S: common::AppStateProvider + Send + Sync,
    {
        // Handler logic
    }
    
    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        // Test cases
    }
}
```

## Existing Features

### Database Types

- **`NodeId`**: Type-safe ID for knowledge graph nodes
- **`EdgeId`**: Type-safe ID for knowledge graph edges
- **`EmbeddingId`**: Type-safe ID for vector embeddings
- **`DbError`**: Common error type for database operations
- **`json_metadata`**: Serialization helpers for JSON metadata

### Platform Types

- **Models**: Model definitions and metadata
- **Actions**: Action types for routing
- **Platform**: Platform detection and system info
- **Inference Settings**: Configuration for inference
- **Hardware Constants**: Hardware capability constants

## Architecture

```text
┌─────────────────────────────────────────────┐
│           tabagent-common                    │
│  (Bottom of dependency hierarchy)            │
│                                              │
│  ┌──────────────┐  ┌──────────────────────┐ │
│  │   Backend    │  │      Routing         │ │
│  │              │  │                      │ │
│  │ AppState     │  │ RouteHandler<M>     │ │
│  │ Provider     │  │ - HttpMetadata      │ │
│  │              │  │ - NativeMessaging   │ │
│  │              │  │ - WebRtcMetadata    │ │
│  └──────────────┘  └──────────────────────┘ │
│                                              │
│  ┌──────────────┐  ┌──────────────────────┐ │
│  │  Database    │  │     Platform         │ │
│  │              │  │                      │ │
│  │ NodeId       │  │ Models               │ │
│  │ EdgeId       │  │ Actions              │ │
│  │ EmbeddingId  │  │ InferenceSettings   │ │
│  │ DbError      │  │ HardwareConstants   │ │
│  └──────────────┘  └──────────────────────┘ │
└─────────────────────────────────────────────┘
           │                  │
           ▼                  ▼
    ┌────────────┐    ┌─────────────┐
    │ tabagent-  │    │  tabagent-  │
    │   api      │    │   native-   │
    │            │    │  messaging  │
    └────────────┘    └─────────────┘
           │                  │
           └──────┬───────────┘
                  │
                  ▼
         ┌────────────────┐
         │  tabagent-     │
         │   webrtc       │
         └────────────────┘
```

## Dependencies

**Runtime:**
- `tabagent-values`: Type-safe value system (RequestValue/ResponseValue)
- `async-trait`: Async trait support for `AppStateProvider` and `RouteHandler`
- `serde`, `serde_json`: Serialization for request/response/metadata
- `bincode`: Binary serialization for database operations
- `sled`: Embedded database backend
- `thiserror`: Error handling for `DbError`
- `anyhow`: Error composition for handler results

**Dev (tests only):**
- `tokio`: Async runtime for integration tests
- `tempfile`: Temporary directories for platform tests

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
common = { path = "../common" }
```

Import the unified traits:

```rust
use common::{AppStateProvider, RouteHandler, HttpMetadata};
```

## Testing

### Test Suite (43 tests)

**Unit Tests** (in source files):
- `src/backend.rs`: 5 tests for `AppStateProvider` trait and wrappers
- `src/routing.rs`: 3 tests for `RouteHandler<M>` trait and metadata
- Existing: `platform.rs`, `errors.rs` unit tests

**Integration Tests** (in `tests/`):
- `backend_integration_tests.rs`: 11 tests for backend trait patterns
  - Direct implementation, Arc wrapper, Box wrapper
  - AppStateWrapper (Axum 0.8 compatibility)
  - Trait objects, error handling, Send+Sync bounds
- `routing_integration_tests.rs`: 15 tests for routing trait patterns
  - HTTP, Native Messaging, WebRTC route handlers
  - Metadata, validation, test cases
  - All three transport types verified
- `common_tests.rs`: 3 tests for newtype wrappers and errors
- `platform_tests.rs`: 6 tests for platform-specific paths

**Run all tests:**
```bash
cargo test -p common
```

**Current Status**: ✅ All tests use `MockBackend` for trait contract testing. 
Real backend integration will be added after all three entry points are migrated (see TODO.md).

## Migration Guide

### For Backend Implementations

**Before** (duplicated across crates):
```rust
// In api/src/traits.rs
trait AppStateProvider { ... }

// In native-messaging/src/traits.rs
trait AppStateProvider { ... }  // Duplicate!

// In webrtc/src/traits.rs
trait RequestHandler { ... }  // Different name!
```

**After** (unified in common):
```rust
use common::AppStateProvider;

// Single trait, used everywhere
impl AppStateProvider for MyBackend { ... }
```

### For Route Handlers

**Before** (three different traits):
```rust
// api/src/route_trait.rs
trait RouteHandler { ... }

// native-messaging/src/route_trait.rs
trait NativeMessagingRoute { ... }

// webrtc/src/route_trait.rs
trait DataChannelRoute { ... }
```

**After** (unified with transport parameter):
```rust
use common::routing::{RouteHandler, HttpMetadata};

impl RouteHandler<HttpMetadata> for MyRoute { ... }
```

### Axum 0.8 Compatibility Notes

When implementing HTTP routes, you MUST follow these patterns:

**✅ CORRECT** (Axum 0.8 compatible):
```rust
// Use concrete async fn handler (not closure!)
async fn handler(
    State(state): State<AppStateWrapper>,  // Concrete type
    Json(req): Json<Request>,
) -> Result<Json<Response>, ApiError> {
    // ...
}

// Register with named function
router.route("/path", post(handler))
```

**❌ WRONG** (breaks Axum 0.8):
```rust
// DON'T use closures for handlers
router.route("/path", post(|state, req| async move { ... }))
```

**Why?** Axum 0.8 requires `Clone` state and concrete handler types. The `AppStateWrapper` 
provides this via `Arc<dyn AppStateProvider>`.

See `api/src/route_trait.rs::impl_registerable_route!` macro for the working pattern.

## Status

✅ **COMPLETE** - Ready for use by all three entry points (API, Native Messaging, WebRTC)

**Next Steps** (for other crates):
1. Migrate `tabagent-api` to use unified traits
2. Migrate `tabagent-native-messaging` to use unified traits  
3. Migrate `tabagent-webrtc` to use unified traits + add dispatcher
4. Wire all three to real backend (remove MockAppState)

See `TODO.md` for detailed migration checklist.

## License

MIT
