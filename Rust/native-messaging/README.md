# TabAgent Native Messaging Crate

**Purpose**: Chrome extension communication layer for TabAgent Server with 100% API parity.

## ✅ TIER 0 Migration Status - COMPLETED!

**This crate is now fully migrated:**
- ✅ Uses `common::backend::AppStateProvider` (unified backend trait)
- ✅ Wired to real `tabagent-server::AppState` (no more mocks in main.rs!)
- ✅ Type-safe routing with compile-time enforcement
- ✅ All 42 unit tests passing
- ✅ Fixed all 4 failing tests

**Next:** After all 3 entry points migrate, delete `src/routes/` folder (2000+ lines) and replace with 50-line dispatcher.

## Overview

The `tabagent-native-messaging` crate provides a production-grade native messaging host for Chrome extensions. It exposes all TabAgent functionality through Chrome's native messaging protocol with identical request/response schemas as the HTTP API and WebRTC implementations.

## Architecture

### Core Design Principles
1. **100% API Parity**: Identical functionality to HTTP API and WebRTC (36+ routes)
2. **Chrome Protocol Compliance**: Follows Chrome's native messaging specification exactly
3. **RAG Compliance**: Follows all Rust Architecture Guidelines
4. **Trait-Based Integration**: Uses same `AppStateProvider` trait as API/WebRTC crates
5. **Compile-Time Enforcement**: Prevents "random crappy routes" via trait system

### Technology Stack
- **Chrome Native Messaging**: Length-prefixed JSON over stdin/stdout
- **Tokio**: Async runtime for concurrent request handling
- **serde/serde_json**: Type-safe JSON serialization
- **tabagent-values**: Unified value system with typed error propagation
- **Trait System**: Compile-time enforcement of documentation, tests, validation

### Communication Contract
All requests/responses use the unified `tabagent-values` system:
- **Input**: `RequestValue` (type-safe request envelope)
- **Output**: `ResponseValue` (type-safe response envelope)
- **Errors**: Chrome-compatible error responses with structured details

## Native Messaging Protocol

### Message Format
Chrome's native messaging uses length-prefixed JSON:

```
[4-byte length][JSON payload]
```

### Request Format
```json
{
  "route": "chat",
  "request_id": "client-generated-uuid",
  "payload": {
    "model": "gpt-3.5-turbo",
    "messages": [{"role": "user", "content": "Hello"}]
  }
}
```

### Response Format
```json
{
  "request_id": "client-generated-uuid",
  "success": true,
  "data": {
    "id": "chat-123",
    "object": "chat.completion",
    "choices": [{"message": {"role": "assistant", "content": "Hi there!"}}]
  }
}
```

### Error Response Format
```json
{
  "request_id": "client-generated-uuid",
  "success": false,
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "temperature must be between 0.0 and 2.0, got 3.0",
    "details": {
      "field": "temperature",
      "validation_message": "must be between 0.0 and 2.0"
    }
  }
}
```

## Supported Routes

### Core ML Operations
| Route ID | Description | OpenAI Compatible |
|----------|-------------|-------------------|
| `chat` | Chat completion | ✅ |
| `responses` | Alternative chat format | ❌ |
| `generate` | Text generation | ✅ |
| `embeddings` | Generate embeddings | ✅ |
| `rerank` | Rerank documents | ❌ |

### Model Management
| Route ID | Description | OpenAI Compatible |
|----------|-------------|-------------------|
| `models` | List loaded models | ✅ |
| `load-model` | Load a model | ❌ |
| `unload-model` | Unload a model | ❌ |
| `model-info` | Get model details | ❌ |
| `registered-models` | List available models | ❌ |
| `select-model` | Select active model | ❌ |

### RAG & Search
| Route ID | Description | OpenAI Compatible |
|----------|-------------|-------------------|
| `rag-query` | RAG with context | ❌ |
| `semantic-search` | Semantic search | ❌ |
| `similarity` | Calculate similarity | ❌ |
| `evaluate-embeddings` | Evaluate embeddings | ❌ |
| `cluster` | Cluster documents | ❌ |
| `recommend` | Content recommendations | ❌ |

### System & Health
| Route ID | Description | OpenAI Compatible |
|----------|-------------|-------------------|
| `health` | Health check | ❌ |
| `system` | System information | ❌ |
| `stats` | Performance stats | ❌ |
| `resources` | Resource monitoring | ❌ |
| `compatibility` | Model compatibility | ❌ |

### Session Management
| Route ID | Description | OpenAI Compatible |
|----------|-------------|-------------------|
| `get-history` | Get chat history | ❌ |
| `save-message` | Save message | ❌ |
| `get-params` | Get parameters | ❌ |
| `set-params` | Set parameters | ❌ |

### Generation Control
| Route ID | Description | OpenAI Compatible |
|----------|-------------|-------------------|
| `stop-generation` | Stop generation | ❌ |
| `halt-status` | Get halt status | ❌ |

## Integration with Server

The native messaging crate is designed to be used by the `tabagent-server` binary:

```rust
// Server implements the trait (same as API crate)
impl tabagent_native_messaging::AppStateProvider for AppState {
    async fn handle_request(&self, request: RequestValue) -> Result<ResponseValue> {
        handler::handle_request(self, request).await
    }
}

// Server starts the native messaging host
let state = Arc::new(AppState::new().await?);
let config = NativeMessagingConfig::default();
tabagent_native_messaging::run_host(state, config).await?;
```

### State Provider Trait

```rust
pub trait AppStateProvider: Send + Sync + 'static {
    /// Handle a request and return a response (identical to API crate)
    async fn handle_request(&self, request: RequestValue) -> Result<ResponseValue>;
}
```

## Route Handler System

### Compile-Time Enforcement

Every route MUST implement the `NativeMessagingRoute` trait:

```rust
#[async_trait]
impl NativeMessagingRoute for ChatRoute {
    type Request = ChatCompletionRequest;
    type Response = ChatCompletionResponse;
    
    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "chat",
            tags: &["AI", "Chat"],
            description: "Chat completion via native messaging",
            openai_compatible: true,
            requires_auth: false,
            rate_limit_tier: Some("inference"),
            // ...
        }
    }
    
    async fn validate_request(req: &Self::Request) -> NativeMessagingResult<()> {
        // Mandatory validation
    }
    
    async fn handle<S>(req: Self::Request, state: &S) -> NativeMessagingResult<Self::Response>
    where S: AppStateProvider + Send + Sync
    {
        // Handler implementation with tracing
    }
    
    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        // Mandatory test cases
    }
}

// Compile-time enforcement
enforce_native_messaging_route!(ChatRoute);
```

### Route Standards

The `enforce_native_messaging_route!` macro ensures every route has:
- ✅ **Documentation**: Non-empty description and tags
- ✅ **Validation**: Real validation logic (not just `Ok(())`)
- ✅ **Testing**: At least one test case defined
- ✅ **Tracing**: Request ID logging for all operations
- ✅ **Error Handling**: Proper `NativeMessagingError` usage

## Error Handling

All errors follow a consistent hierarchy:

```rust
pub enum NativeMessagingError {
    Protocol(String),           // Chrome protocol errors
    ValidationError { field, message },  // Request validation
    RouteNotFound { route },    // Unknown route
    BadRequest(String),         // Client errors
    Internal(String),           // Server errors
    Backend(anyhow::Error),     // Backend service errors
    // ...
}
```

### Error Response Chain

1. **Backend** returns `anyhow::Error` or typed errors
2. **Native Messaging** converts to `NativeMessagingError`
3. **Chrome Extension** receives structured error response

## Configuration

```rust
pub struct NativeMessagingConfig {
    /// Maximum message size (Chrome limit: 1MB)
    pub max_message_size: usize,
    
    /// Enable request/response logging
    pub enable_logging: bool,
    
    /// Rate limiting configuration
    pub rate_limiting: RateLimitConfig,
    
    /// Security configuration
    pub security: SecurityConfig,
}
```

## Security Features

1. **Origin Validation**: Verify Chrome extension origins
2. **Rate Limiting**: Per-extension request limits
3. **Authentication**: Support for protected routes
4. **Audit Logging**: All requests logged for security monitoring
5. **Input Validation**: Comprehensive request validation

## Chrome Extension Integration

### Manifest Configuration
```json
{
  "name": "TabAgent Extension",
  "permissions": ["nativeMessaging"],
  "host_permissions": ["<all_urls>"],
  "native_messaging_hosts": {
    "com.tabagent.native_messaging": {
      "path": "path/to/tabagent-native-messaging-host.exe",
      "type": "stdio"
    }
  }
}
```

### Extension Code Example
```javascript
// Connect to native messaging host
const port = chrome.runtime.connectNative('com.tabagent.native_messaging');

// Send chat request
port.postMessage({
  route: 'chat',
  request_id: crypto.randomUUID(),
  payload: {
    model: 'gpt-3.5-turbo',
    messages: [{role: 'user', content: 'Hello!'}]
  }
});

// Handle response
port.onMessage.addListener((response) => {
  if (response.success) {
    console.log('Chat response:', response.data);
  } else {
    console.error('Error:', response.error);
  }
});
```

## Testing Strategy

### Unit Tests
- Route handler validation and logic
- Protocol message parsing/formatting
- Error conversion and formatting
- Configuration validation

### Integration Tests
- Full message processing workflow
- Route registration and dispatch
- Concurrent request handling
- Error recovery scenarios

### End-to-End Tests
- Real Chrome extension communication
- All routes tested with actual payloads
- Performance and stability testing

## RAG Compliance Checklist

- [x] **Rule 1.1**: No `unwrap()` - all errors handled properly
- [x] **Rule 1.2**: Explicit error propagation with `?` or `map_err`
- [x] **Rule 2.1**: Async with Tokio runtime
- [x] **Rule 3.1**: Concurrent state handling where needed
- [x] **Rule 4.1**: Type-safe JSON with `serde`
- [x] **Rule 5.1**: Comprehensive error types with `thiserror`
- [x] **Rule 6.1**: `tracing` for structured logging
- [x] **Rule 7.1**: Unit tests for all routes (compile-time enforced)
- [x] **Rule 7.2**: Integration tests with mock state
- [x] **Rule 8.1**: Complete documentation for all public items

## Performance Characteristics

- **Simple Requests**: < 50ms (health, system info)
- **AI Requests**: Depends on model and backend
- **Concurrent Handling**: Supports multiple Chrome extension instances
- **Memory Usage**: Minimal overhead over backend services
- **Protocol Overhead**: ~8 bytes per message (4-byte length header)

## Future Enhancements

- [ ] Streaming response support for long-running operations
- [ ] Binary data support for media streams
- [ ] WebRTC signaling integration
- [ ] Extension authentication tokens
- [ ] Request batching for efficiency
- [ ] Metrics endpoint for monitoring

## Dependencies

```toml
[dependencies]
tabagent-values = { path = "../values" }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
async-trait = "0.1"
anyhow = "1"
thiserror = "2"
tracing = "0.1"
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
```

## Usage Example

```rust
use tabagent_native_messaging::{run_host, AppStateProvider, NativeMessagingConfig};
use tabagent_values::{RequestValue, ResponseValue};
use std::sync::Arc;

struct MyState {
    // Your state here
}

#[async_trait::async_trait]
impl AppStateProvider for MyState {
    async fn handle_request(&self, req: RequestValue) -> anyhow::Result<ResponseValue> {
        // Your request handling logic (same as API crate)
        Ok(ResponseValue::health("ok"))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = Arc::new(MyState { /* ... */ });
    let config = NativeMessagingConfig::default();
    tabagent_native_messaging::run_host(state, config).await
}
```

## License

Part of the TabAgent project. See root LICENSE file.