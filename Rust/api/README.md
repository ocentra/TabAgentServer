# TabAgent API Crate

**Purpose**: Atomic, self-contained HTTP API layer for TabAgent Server using Axum.

## Overview

The `tabagent-api` crate provides a production-grade REST API for the TabAgent system. It exposes all functionality through HTTP endpoints with OpenAPI/Swagger documentation, CORS support, and enterprise-grade middleware.

## Architecture

### Core Design Principles
1. **Atomic & Self-Contained**: Complete API implementation with no external module dependencies
2. **Framework**: Built on [Axum](https://github.com/tokio-rs/axum) for type-safe, performant HTTP
3. **RAG Compliance**: Follows all Rust Architecture Guidelines
4. **Trait-Based Integration**: Uses traits for state provider, enabling clean separation

### Technology Stack
- **Axum 0.8**: Web framework (built on Tokio/Tower) - *Note: serve compatibility pending*
- **Tower/Tower-HTTP**: Middleware stack (CORS, compression, tracing, rate limiting)
- **utoipa**: OpenAPI 3.0 code generation
- **utoipa-swagger-ui**: Interactive API documentation
- **serde/serde_json**: Type-safe JSON serialization
- **tabagent-values**: Unified value system with typed error propagation

### API Contract
All requests/responses use the unified `tabagent-values` system:
- **Input**: `RequestValue` (type-safe request envelope)
- **Output**: `ResponseValue` (type-safe response envelope)
- **Errors**: Proper HTTP status codes with structured error responses

## API Routes

### Core ML Operations
| Method | Route | Description | Request Type |
|--------|-------|-------------|--------------|
| `POST` | `/v1/chat/completions` | Chat completion (OpenAI-compatible) | `Chat` |
| `POST` | `/v1/completions` | Text generation | `Generate` |
| `POST` | `/v1/embeddings` | Generate embeddings | `Embeddings` |
| `POST` | `/v1/rerank` | Rerank documents | `Rerank` |

### Model Management
| Method | Route | Description | Request Type |
|--------|-------|-------------|--------------|
| `POST` | `/v1/models/load` | Load a model | `LoadModel` |
| `POST` | `/v1/models/unload` | Unload a model | `UnloadModel` |
| `GET` | `/v1/models` | List loaded models | `ListModels` |
| `GET` | `/v1/models/{model_id}` | Get model info | `ModelInfo` |

### Chat History & RAG
| Method | Route | Description | Request Type |
|--------|-------|-------------|--------------|
| `GET` | `/v1/sessions/{session_id}/history` | Get chat history | `ChatHistory` |
| `POST` | `/v1/sessions/{session_id}/messages` | Save message | `SaveMessage` |
| `POST` | `/v1/rag/query` | RAG query with context | `RagQuery` |

### System & Health
| Method | Route | Description | Request Type |
|--------|-------|-------------|--------------|
| `GET` | `/health` | Health check | `Health` |
| `GET` | `/v1/system/info` | System information | `SystemInfo` |
| `POST` | `/v1/generation/stop` | Stop active generation | `StopGeneration` |

## Middleware Stack

Applied in order (outer → inner):

1. **Tracing**: Request/response logging with trace IDs
2. **CORS**: Configurable cross-origin resource sharing
3. **Compression**: Gzip/Brotli response compression
4. **Rate Limiting**: Token bucket per IP/API key
5. **Timeout**: Configurable request timeout (default: 300s)
6. **Error Handling**: Unified error response format

## Integration with Server

The API crate is designed to be used by the `tabagent-server` binary:

```rust
// Server implements the trait
impl tabagent_api::AppStateProvider for AppState {
    async fn handle_request(&self, request: RequestValue) -> Result<ResponseValue> {
        handler::handle_request(self, request).await
    }
}

// Server starts the API
let state = Arc::new(AppState::new().await?);
tabagent_api::run_server(state, 8080).await?;
```

### State Provider Trait

```rust
pub trait AppStateProvider: Send + Sync + 'static {
    /// Handle a request and return a response
    async fn handle_request(&self, request: RequestValue) -> Result<ResponseValue>;
}
```

The server provides the `AppState` which contains:
- Database coordinator
- Model cache
- Loaded ONNX/GGUF models
- Hardware information
- Python ML bridge

## OpenAPI Documentation

The API provides **three different documentation UIs** - choose your favorite!

### 📘 Swagger UI (Classic)
Interactive API playground with "Try it out" buttons:
- **URL**: `http://localhost:8080/swagger-ui/`
- **Best for**: Testing APIs, trying requests
- **Features**: Request/response examples, auth, schemas

### 📗 RapiDoc (Modern)
Modern, customizable UI with multiple themes:
- **URL**: `http://localhost:8080/rapidoc/`
- **Best for**: Mobile-friendly docs, modern look
- **Features**: Dark mode, three themes, fast rendering

### 📕 Redoc (Beautiful)
Beautiful three-panel documentation:
- **URL**: `http://localhost:8080/redoc/`
- **Best for**: Public documentation, complex APIs
- **Features**: Search, three-panel layout, responsive

### 📄 OpenAPI Spec
Machine-readable OpenAPI 3.0 specification:
- **URL**: `http://localhost:8080/api-doc/openapi.json`
- **Use for**: Code generation, validation, tools

## Error Handling

All errors follow **RFC 7807 Problem Details** with actionable guidance.

### Downstream Error Propagation

The API implements a complete error chain from backend → API → client:

1. **Backend** returns typed `BackendError` (14 specific variants)
2. **API** maps to appropriate `ApiError` with helpful messages
3. **Client** receives RFC 7807 response with "what to do next" guidance

### Example Error Response

```json
{
  "type": "https://tabagent.dev/errors/service-unavailable",
  "title": "Service Unavailable",
  "status": 503,
  "detail": "Model 'llama-2-7b' is not currently loaded. Load it with: POST /v1/models/load {\"model_id\": \"llama-2-7b\"}",
  "request_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

### Backend Error Types

The `tabagent-values` crate defines 14 typed error variants:

| Backend Error | HTTP Status | Client Guidance |
|---------------|-------------|-----------------|
| `ModelNotLoaded` | 503 | Load instructions + endpoint |
| `ModelNotFound` | 404 | View available models endpoint |
| `OutOfMemory` | 503 | Unload other models suggestion |
| `GenerationTimeout` | 408 | Stop generation endpoint |
| `InvalidInput` | 400 | Field-specific validation error |
| `CudaError` | 500 | CUDA troubleshooting hints |
| `ModelCorrupted` | 422 | Re-download instructions |
| `ResourceLimitExceeded` | 429 | Retry guidance |
| `SessionNotFound` | 404 | Create session instructions |
| `EmbeddingModelNotAvailable` | 503 | Load embedding model steps |
| `VectorStoreError` | 500 | Database rebuild hint |
| `InternalError` | 500 | Context details |
| `ConfigurationError` | 500 | Setting name + reason |
| `NotImplemented` | 500 | Feature name + docs link |

### HTTP Status Codes
- `200 OK`: Success
- `400 Bad Request`: Invalid request format or validation error
- `404 Not Found`: Resource not found (model, session)
- `408 Request Timeout`: Generation timeout
- `422 Unprocessable Entity`: Valid format, invalid semantics (corrupted model)
- `429 Too Many Requests`: Rate limit or resource limit exceeded
- `500 Internal Server Error`: Server-side error
- `503 Service Unavailable`: Model loading, memory issues

### For Backend Developers

Use typed errors for better client experience:

```rust
// ❌ Generic error (still works, but less helpful):
Err(anyhow!("Model not loaded"))

// ✅ Typed error (provides actionable guidance):
Err(BackendError::ModelNotLoaded { 
    model: "llama-2-7b".to_string() 
})
```

The API automatically converts `BackendError` to RFC 7807 with helpful instructions.

## Configuration

```rust
pub struct ApiConfig {
    /// Port to bind to (default: 8080)
    pub port: u16,
    
    /// Enable CORS (default: true)
    pub enable_cors: bool,
    
    /// Allowed origins for CORS (default: ["*"])
    pub cors_origins: Vec<String>,
    
    /// Request timeout in seconds (default: 300)
    pub timeout_secs: u64,
    
    /// Enable Swagger UI (default: true)
    pub enable_swagger: bool,
    
    /// Rate limit: requests per minute (default: 60)
    pub rate_limit_rpm: u32,
}
```

## RAG Compliance Checklist

- [x] **Rule 1.1**: No `unwrap()` - all errors handled properly
- [x] **Rule 1.2**: Explicit error propagation with `?` or `map_err`
- [x] **Rule 2.1**: Async with Tokio runtime
- [x] **Rule 3.1**: DashMap for concurrent state when needed
- [x] **Rule 4.1**: Type-safe JSON with `serde`
- [x] **Rule 5.1**: Comprehensive error types with `thiserror`
- [x] **Rule 6.1**: `tracing` for structured logging
- [x] **Rule 7.1**: Unit tests for all routes
- [x] **Rule 7.2**: Integration tests with mock state
- [x] **Rule 8.1**: OpenAPI documentation for all routes

## Testing Strategy

### Unit Tests
- Route handler functions
- Middleware behavior
- Error conversion
- Request/response serialization

### Integration Tests
- Full HTTP request/response cycle
- Middleware stack integration
- OpenAPI spec validation
- CORS preflight handling

### Performance Tests
- Concurrent request handling
- Response time benchmarks
- Memory usage under load

## Security Considerations

1. **CORS**: Configurable allowed origins
2. **Rate Limiting**: Per-IP token bucket
3. **Input Validation**: JSON schema validation
4. **Error Messages**: No sensitive information leakage
5. **Timeouts**: Prevent resource exhaustion

## Future Enhancements

- [ ] API key authentication
- [ ] Request signing
- [ ] Streaming responses (SSE/WebSocket)
- [ ] GraphQL endpoint
- [ ] Metrics endpoint (Prometheus)
- [ ] Health check with dependency status

## Dependencies

```toml
[dependencies]
axum = "0.7"
tower = { version = "0.4", features = ["util", "timeout", "limit"] }
tower-http = { version = "0.5", features = ["cors", "trace", "compression-full"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
utoipa = { version = "4", features = ["axum_extras"] }
utoipa-swagger-ui = "6"
tracing = "0.1"
anyhow = "1"
```

## Usage Example

```rust
use tabagent_api::{run_server, AppStateProvider};
use tabagent_values::{RequestValue, ResponseValue, HealthStatus};
use std::sync::Arc;

struct MyState {
    // Your state here
}

#[async_trait::async_trait]
impl AppStateProvider for MyState {
    async fn handle_request(&self, req: RequestValue) -> anyhow::Result<ResponseValue> {
        // Your request handling logic
        Ok(ResponseValue::health(HealthStatus::Healthy))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = Arc::new(MyState { /* ... */ });
    tabagent_api::run_server(state, 8080).await
}
```

## License

Part of the TabAgent project. See root LICENSE file.

