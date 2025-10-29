# AppState

Central application state for TabAgent. This crate provides the shared `AppState` struct that holds all resources and implements business logic.

## Purpose

`appstate` solves the circular dependency problem by providing a single concrete implementation of the `AppStateProvider` trait that all transport layers can depend on without creating cycles.

## Architecture

```text
┌─────────────────────────────────────────────────┐
│  Transport Layer                                │
│  (api, native-messaging, webrtc)                │
└───────────────────┬─────────────────────────────┘
                    │ depends on
                    ▼
┌─────────────────────────────────────────────────┐
│  appstate (this crate)                          │
│  - AppState struct                              │
│  - Business logic methods                       │
│  - Implements AppStateProvider                  │
└───────────────────┬─────────────────────────────┘
                    │ depends on
                    ▼
┌─────────────────────────────────────────────────┐
│  Infrastructure Layer                           │
│  (model-cache, storage, hardware, loaders)      │
└─────────────────────────────────────────────────┘
```

## Dependency Flow

**One-way, no cycles:**

```
server binary
  └─> appstate
       └─> infrastructure crates
            └─> common (types/traits)
                 └─> values (data types)

api, native-messaging, webrtc
  └─> appstate (for concrete state)
  └─> common (for trait)
  └─> values (for request/response types)
```

## What It Holds

- **Database**: `storage::DatabaseCoordinator`
- **Model Cache**: `tabagent_model_cache::ModelCache`
- **Hardware Info**: `tabagent_hardware::SystemInfo`
- **Loaded Models**: ONNX and GGUF model registries
- **HF Auth**: Secure HuggingFace token management
- **Generation Control**: Cancellation tokens for active requests

## What It Does NOT Do

- ❌ HTTP routing (that's `api`)
- ❌ Native messaging protocol (that's `native-messaging`)
- ❌ WebRTC signaling (that's `webrtc`)
- ❌ Model inference (delegates to `onnx-loader`, `gguf-loader`)
- ❌ Database operations (delegates to `storage`)
- ❌ Downloads (delegates to `model-cache`)

## Usage

```rust
use appstate::{AppState, AppStateConfig};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppStateConfig {
        db_path: PathBuf::from("./data"),
        model_cache_path: PathBuf::from("./models"),
    };

    let state = AppState::new(config).await?;
    
    // Pass to transport layers
    api::run_server(state.clone()).await?;
    native_messaging::run(state.clone()).await?;
    webrtc::run(state).await?;
    
    Ok(())
}
```

## Testing

Transport crates can now test with the real `AppState`:

```rust
use appstate::{AppState, AppStateConfig};

#[tokio::test]
async fn test_with_real_state() {
    let config = AppStateConfig::default();
    let state = AppState::new(config).await.unwrap();
    
    // Test routes with real state
    // No mocks needed, no circular dependency!
}
```

## Design Principles

1. **Single Responsibility**: Only manages shared state, delegates all operations
2. **Dependency Inversion**: Depends on abstractions (traits), not implementations
3. **No Cycles**: One-way dependency flow from top to bottom
4. **Thread-Safe**: All fields are Arc-wrapped for cheap cloning
5. **Async-First**: All operations are async for non-blocking I/O

