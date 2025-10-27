# TabAgent Values

**Type-safe, zero-copy value system for TabAgent request/response/data handling.**

Inspired by ONNX Runtime's value abstraction, this crate provides a unified type system for all TabAgent data types with compile-time safety guarantees and runtime type information.

## What is this?

A foundational crate that defines type-safe wrappers for:
- **Request values**: Chat, Generate, Embeddings, Load Model, etc.
- **Response values**: ChatResponse, GenerateResponse, Error, etc.
- **Model data**: Tensors, Embeddings, Parameters, Tokenizer data

## Why use this?

### Type Safety
```rust
// Compile-time type checking
fn handle_request(req: Value<RequestValue>) {
    // Guaranteed to be a request, not a response
}

fn send_response(resp: Value<ResponseValue>) {
    // Guaranteed to be a response, not a request
}
```

### Zero-Copy Borrows
```rust
// No cloning needed for read-only access
fn inspect_request(req: ValueRef<'_, RequestValue>) {
    // Borrowed access, no overhead
    println!("Type: {:?}", req.value_type());
}
```

### Runtime Type Information
```rust
// Dynamic type checking when needed
let value: Value<DynValueTypeMarker> = get_value();
match value.value_type() {
    ValueType::ChatRequest { model } => { /* handle chat */ }
    ValueType::GenerateRequest { model } => { /* handle generate */ }
    _ => { /* fallback */ }
}
```

### Downcasting Support
```rust
// Safe downcasting from dynamic to concrete types
let dyn_value: Value<DynValueTypeMarker> = get_dynamic_value();
let request: Value<RequestValue> = dyn_value.downcast()?;
```

## When to use this?

### Use tabagent-values when:
- ✅ Building API handlers that need type-safe request/response handling
- ✅ Implementing message passing between components
- ✅ Creating serialization/deserialization layers
- ✅ Building protocol adapters (HTTP, WebRTC, native messaging)
- ✅ You need both compile-time and runtime type safety

### Don't use tabagent-values when:
- ❌ Simple data structures are sufficient
- ❌ You only need serialization (use serde directly)
- ❌ You're building internal-only code with no type ambiguity

## How to use

### Basic Request Creation

```rust
use tabagent_values::{RequestValue, EmbeddingInput};

// Create a chat request
let request = RequestValue::chat(
    "gpt-3.5-turbo",
    vec![("user", "Hello, world!")],
    Some(0.7),  // temperature
);

// Create an embeddings request
let request = RequestValue::embeddings(
    "sentence-transformers/all-MiniLM-L6-v2",
    EmbeddingInput::Single("Embed this text".to_string()),
);

// Create a load model request
let request = RequestValue::load_model(
    "Qwen/Qwen2.5-1.5B-Instruct-GGUF",
    Some("q4_0"),  // variant
);
```

### Basic Response Creation

```rust
use tabagent_values::{ResponseValue, TokenUsage};

// Create a chat response
let response = ResponseValue::chat(
    "req-123",
    "gpt-3.5-turbo",
    "Hello! How can I help you today?",
    TokenUsage::new(10, 15),
);

// Create an error response
let response = ResponseValue::error(
    "model_not_found",
    "The requested model could not be found",
);
```

### JSON Serialization

```rust
use tabagent_values::RequestValue;

// Parse from JSON
let json = r#"{"action":"chat","model":"gpt-4","messages":[{"role":"user","content":"Hi"}]}"#;
let request = RequestValue::from_json(json)?;

// Serialize to JSON
let json = response.to_json()?;
```

### Type-Safe Pattern Matching

```rust
use tabagent_values::{RequestValue, RequestType};

fn handle_request(request: RequestValue) -> Result<ResponseValue, Error> {
    match request.request_type() {
        RequestType::Chat { model, messages, temperature, .. } => {
            // Handle chat request
            generate_chat_response(model, messages, *temperature)
        }
        RequestType::Embeddings { model, input } => {
            // Handle embeddings request
            generate_embeddings(model, input)
        }
        RequestType::LoadModel { model_id, variant, .. } => {
            // Handle load model request
            load_model(model_id, variant.as_deref())
        }
        _ => {
            Err(Error::UnsupportedRequest)
        }
    }
}
```

### Borrowed Access (Zero-Copy)

```rust
use tabagent_values::ValueRef;

// No cloning, just borrowing
fn log_request(req: ValueRef<'_, RequestValue>) {
    tracing::info!("Received request: {:?}", req.value_type());
}

let request = RequestValue::chat("model", vec![("user", "test")], None);
log_request(ValueRef::new(&request));  // Zero-copy
```

### Downcasting

```rust
use tabagent_values::{Value, DynValueTypeMarker, RequestValue};

// Start with dynamic value
let dyn_value: Value<DynValueTypeMarker> = get_from_network();

// Downcast to specific type
match dyn_value.downcast::<RequestValue>() {
    Ok(request) => {
        // It's a request, process it
        handle_request(request);
    }
    Err(_) => {
        // Not a request, try response
        match dyn_value.downcast::<ResponseValue>() {
            Ok(response) => handle_response(response),
            Err(_) => handle_unknown(),
        }
    }
}
```

## Architecture

```
Value<Type>                    <- Owned value (like Box<T>)
  ├─ ValueInner
  │    ├─ ValueData            <- Actual data (enum)
  │    └─ ValueType            <- Runtime type info
  └─ PhantomData<Type>         <- Compile-time type marker

ValueRef<'v, Type>             <- Borrowed value (like &T)
  ├─ &ValueInner
  └─ PhantomData<Type>

Type Markers (Zero-sized):
  ├─ DynValueTypeMarker        <- Any value
  ├─ RequestValueMarker        <- Any request
  │    ├─ ChatRequestMarker    <- Specific request types
  │    ├─ GenerateRequestMarker
  │    └─ ...
  ├─ ResponseValueMarker       <- Any response
  └─ ModelDataValueMarker      <- Model data (tensors, etc.)
```

## Integration with Other Crates

### Server (HTTP/WebRTC/Native Messaging)
```rust
use axum::{Json, extract::State};
use tabagent_values::RequestValue;

async fn handle_request(
    State(state): State<AppState>,
    Json(request): Json<RequestValue>,
) -> Result<Json<ResponseValue>, ServerError> {
    // Type-safe request handling
    let response = process_request(&state, request).await?;
    Ok(Json(response))
}
```

### ONNX Loader
```rust
use tabagent_values::{ModelValue, TensorData, TensorDataType};

// Create tensor for ONNX input
let input = ModelValue::tensor(
    TensorData::F32(vec![1.0, 2.0, 3.0, 4.0]),
    TensorDataType::Float32,
    vec![2, 2],
);
```

### Database Storage
```rust
use tabagent_values::RequestValue;

// Serialize request for storage
let json = request.to_json()?;
db.store("requests", id, &json).await?;

// Deserialize from storage
let json = db.fetch("requests", id).await?;
let request = RequestValue::from_json(&json)?;
```

## Features

- **Default**: Core value system
- **ndarray** (optional): Integration with ndarray for tensor operations

## RAG Compliance

This crate follows the Rust Architecture Guidelines:

- ✅ **No `.unwrap()` in production code** - All errors are properly handled
- ✅ **Type safety** - Enums instead of strings, newtype pattern for IDs
- ✅ **Zero-cost abstractions** - Borrowed variants for zero-copy access
- ✅ **Proper error handling** - thiserror for rich error types
- ✅ **Comprehensive tests** - Unit tests for all functionality
- ✅ **Documentation** - All public APIs documented with examples

## Performance

- **Zero allocation** for borrowed access via `ValueRef`
- **Minimal overhead** for owned values (single heap allocation)
- **Compile-time dispatch** for type-safe operations
- **Efficient serialization** via serde

## Examples

See `tests/` directory for comprehensive usage examples:
- `tests/request_tests.rs` - Request value creation and serialization
- `tests/response_tests.rs` - Response value creation and serialization
- `tests/downcast_tests.rs` - Type downcasting examples
- `tests/integration_tests.rs` - End-to-end usage patterns

## License

MIT

