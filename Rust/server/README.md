# TabAgent Server

Unified Rust-native server providing multiple transport layers for TabAgent AI infrastructure.

## Overview

This is the main entry point for the TabAgent server, which orchestrates:
- **Native Messaging**: Chrome extension communication (stdin/stdout)
- **HTTP API**: REST API with OpenAPI documentation
- **WebRTC**: Real-time peer-to-peer communication (planned)

All transports share the same `AppState` backend, ensuring 100% API parity.

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                  TabAgent Server                     │
│                     (main.rs)                        │
└──────┬──────────────┬────────────────┬──────────────┘
       │              │                │
       ▼              ▼                ▼
┌──────────┐  ┌──────────┐    ┌──────────────┐
│ Native   │  │ HTTP API │    │   WebRTC     │
│Messaging │  │  (Axum)  │    │  (Planned)   │
└────┬─────┘  └────┬─────┘    └──────┬───────┘
     │             │                  │
     └─────────────┼──────────────────┘
                   ▼
           ┌───────────────┐
           │   AppState    │
           │   (Shared)    │
           └───────────────┘
                   │
        ┏━━━━━━━━━━┻━━━━━━━━━━┓
        ▼                      ▼
   ┌─────────┐          ┌──────────┐
   │Database │          │ ML Models│
   │ (Sled)  │          │(ONNX/GGUF)│
   └─────────┘          └──────────┘
```

## Usage

### Quick Start (Recommended)

**Use the wrapper scripts for automatic cleanup and restart:**

```powershell
# PowerShell (Windows/Linux/macOS)
./run-server.ps1                    # HTTP mode on port 3000
./run-server.ps1 -Mode webrtc       # WebRTC mode
./run-server.ps1 -Mode all          # All transports

# Windows Batch
run-server.bat                       # HTTP mode on port 3000
```

**Benefits:**
- 🔄 Automatically kills old server instances
- 🛠️ Builds before starting
- ✅ No "access denied" errors
- 🚀 One command to restart

### Manual Usage

If you prefer to run cargo directly:

#### Run in Native Messaging Mode (Chrome Extension)

```bash
cargo run --release --bin tabagent-server -- --mode native
```

### Run HTTP API Server

```bash
cargo run --release --bin tabagent-server -- --mode http --port 8080
```

### Run All Transports (HTTP + Native + WebRTC)

```bash
cargo run --release --bin tabagent-server -- --mode all --port 8080 --webrtc-port 9000
```

## CLI Options

```
--mode <MODE>          Server mode: native, http, webrtc, both, all [default: http]
--port <PORT>          HTTP server port [default: 8080]
--webrtc-port <PORT>   WebRTC signaling port [default: 9000]
--db-path <PATH>       Database path [default: ./data/db]
--model-cache <PATH>   Model cache directory [default: ./models]
```

## Server Modes

| Mode       | Transports                        | Use Case                          |
|------------|-----------------------------------|-----------------------------------|
| `native`   | Native Messaging only             | Chrome extension only             |
| `http`     | HTTP API only                     | Web apps, testing                 |
| `webrtc`   | WebRTC only (planned)             | P2P real-time apps                |
| `both`     | HTTP + Native Messaging           | Extension + web dashboard         |
| `all`      | HTTP + Native + WebRTC            | Full-stack deployment             |

## Features

- ✅ **Zero-Copy Architecture**: Shared `AppState` across all transports
- ✅ **Type-Safe Requests**: `tabagent-values` provides compile-time safety
- ✅ **Async/Await**: Full tokio async runtime
- ✅ **Structured Logging**: tracing with configurable levels
- ✅ **Graceful Shutdown**: Ctrl+C handling
- ✅ **Production Ready**: Error handling, logging, configuration

## Development

### Run Tests

```bash
cargo test --package tabagent-server
```

### Run with Debug Logging

```bash
RUST_LOG=debug cargo run --bin tabagent-server -- --mode http
```

### Build Release Binary

```bash
cargo build --release --bin tabagent-server
```

## Configuration

The server uses:
- **Database**: Sled (embedded KV store) at `./data/db`
- **Model Cache**: Downloaded models at `./models`
- **Logs**: stdout with tracing (configure via `RUST_LOG` env var)

## API Parity

All transport layers expose identical APIs:
- 36+ routes covering chat, generation, embeddings, RAG, etc.
- Same request/response types via `tabagent-values`
- Shared backend logic in `appstate` crate

## TODO

- [ ] Complete WebRTC data channel integration
- [ ] Add configuration file support (YAML/TOML)
- [ ] Add metrics/monitoring endpoints
- [ ] Add rate limiting per transport
- [ ] Add authentication/authorization
- [ ] Add HTTPS/TLS support for HTTP mode
- [ ] Add connection pooling for database
- [ ] Add graceful shutdown with cleanup

## Dependencies

- `appstate`: Shared application state and business logic
- `tabagent-api`: HTTP API layer (Axum)
- `tabagent-native-messaging`: Chrome native messaging protocol
- `tabagent-webrtc`: WebRTC signaling and data channels (in progress)
- `tabagent-values`: Type-safe request/response handling

## License

See project root LICENSE file.

