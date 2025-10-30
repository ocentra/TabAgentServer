# Rust - Core Inference Infrastructure

**High-performance AI orchestration with gRPC microservices**

Rust workspace providing WebRTC signaling, HTTP API, database layer, and ML orchestration. Spawns and manages Python ML subprocess for vision/language models.

---

## Architecture

```
Rust Server (Orchestrator)
├── HTTP API (REST + OpenAPI)
├── WebRTC Signaling + Data Channels
├── Native Messaging (Chrome Extension)
├── Database Layer (7 MIA databases)
├── Model Management (Download + Cache)
└── Python ML gRPC Client
     ↓ (spawns subprocess)
    Python ML Service (MediaPipe + Transformers + LiteRT)
```

**Design**:
- Rust = Brain (orchestration, state, networking)
- Python = Slave (ML inference only)
- gRPC = Communication (language-agnostic, location-transparent)

---

## Workspace Structure

### Core Server (`server/`)
**Main binary** - Orchestrates all components

**Modes**:
- `native` - Native messaging only
- `http` - HTTP API only  
- `webrtc` - WebRTC signaling only
- `web` - HTTP + WebRTC (no native)
- `all` - Everything (default)

**Auto-starts**: Python ML subprocess on startup

**See**: [server/README.md](server/README.md)

---

### API Layer (`api/`)
**HTTP REST API** with OpenAPI documentation

**Features**:
- Auto-generated routes from traits
- Swagger/ReDoc/RapiDoc UI
- Static file serving
- CORS support

**Endpoints**:
- `/v1/health` - Health check
- `/v1/models/*` - Model management
- `/v1/chat/*` - Chat completion
- `/v1/webrtc/*` - WebRTC signaling
- `/api-doc/*` - API documentation

**See**: [api/README.md](api/README.md)

---

### Application State (`appstate/`)
**Centralized state** for all components

**Components**:
- `ModelOrchestrator` - Model lifecycle (download → load → inference)
- Database clients
- ML clients (gRPC to Python)
- Hardware info
- HuggingFace auth

**Pattern**: Provides `AppStateProvider` trait for dependency injection

**See**: [appstate/README.md](appstate/README.md)

---

### Database Layer (`storage/`)
**MIA cognitive architecture** - 7-database memory system

**Databases**:
- `conversations/` - Chat history
- `knowledge/` - Facts, entities
- `embeddings/` - Vector search
- `tool-results/` - Function call results
- `experience/` - Episodic memory
- `summaries/` - Compressed history
- `meta/` - System metadata

**Features**:
- gRPC service for remote access
- Direct in-process access
- Atomic transactions
- Backup/restore

**See**: [storage/README.md](storage/README.md)

---

### Common Types (`common/`)
**Shared types, clients, utilities**

**Key Components**:
- `MlClient` - gRPC client for Python ML
- `PythonProcessManager` - Subprocess lifecycle
- `AppStateProvider` - State trait
- Platform-specific paths
- Error types
- gRPC generated code

**See**: [common/README.md](common/README.md)

---

### Model Management (`model-cache/`)
**Model download, storage, serving**

**Features**:
- HuggingFace Hub integration
- Parallel chunk downloads
- Resume interrupted downloads
- Local file serving (for Python gRPC)
- Default model library

**File Structure**:
```
models/
├── microsoft--Florence-2-base/
│   ├── config.json
│   ├── model.safetensors
│   └── ...
└── meta-llama--Llama-2-7b-hf/
    └── ...
```

**See**: [model-cache/README.md](model-cache/README.md)

---

### WebRTC (`webrtc/`)
**WebRTC signaling + data channels**

**Features**:
- SDP offer/answer exchange
- ICE candidate gathering
- Data channel creation
- Video stream handling
- Session management

**Use Cases**:
- Browser → Rust real-time video
- Rust → Python streaming inference
- Peer-to-peer connections

**See**: [webrtc/README.md](webrtc/README.md)

---

### Native Messaging (`native-messaging/`)
**Chrome extension protocol**

**Features**:
- stdin/stdout message framing
- JSON message parsing
- Bi-directional communication
- Extension manifest generation

**Protocol**:
```
[4-byte length][JSON message]
```

**See**: [native-messaging/README.md](native-messaging/README.md)

---

### Hardware Detection (`hardware/`)
**Auto-detect CPU, GPU, NPU**

**Detects**:
- CPUs (cores, threads, model)
- NVIDIA GPUs (VRAM via nvidia-smi)
- AMD GPUs (discrete/integrated)
- Intel GPUs (Arc, integrated)
- Acceleration (CUDA, ROCm, Vulkan, DirectML)
- NPUs (AMD Ryzen AI, Intel Core Ultra)

**Platforms**: Windows (complete), Linux/macOS (planned)

**See**: [hardware/README.md](hardware/README.md)

---

### Model Loaders

#### ONNX (`onnx-loader/`)
**ONNX Runtime integration**

**Features**:
- Load `.onnx` models
- Execution providers (CPU, CUDA, DirectML, etc.)
- Session management
- Tensor handling

**See**: [onnx-loader/README.md](onnx-loader/README.md)

---

#### GGUF (`gguf-loader/`)
**llama.cpp integration**

**Features**:
- Load `.gguf` models
- Context management
- Sampling parameters
- Streaming generation

**See**: [gguf-loader/README.md](gguf-loader/README.md)

---

### Execution Providers (`execution-providers/`)
**ONNX Runtime provider definitions**

**Providers**: 25+ execution providers
- CUDA, TensorRT, ROCm (GPU)
- DirectML, CoreML, Metal (Platform-specific)
- XNNPACK, QNN, NNAPI (Edge)
- OpenVINO, Vitis, MIGraphX (Specialized)

**See**: [execution-providers/README.md](execution-providers/README.md)

---

### Pipeline (`pipeline/`)
**Inference pipeline orchestration**

**Features**:
- Multi-stage pipelines
- Preprocessing/postprocessing
- Model chaining
- Error handling

**See**: [pipeline/README.md](pipeline/README.md)

---

### Database Components

#### Indexing (`indexing/`)
**Vector indexing and search**

**Features**:
- HNSW implementation
- Cosine/Euclidean distance
- Lock-free concurrent access
- Incremental updates

**See**: [indexing/README.md](indexing/README.md)

---

#### Query Engine (`query/`)
**Semantic search and retrieval**

**Features**:
- Hybrid search (vector + keyword)
- Filtering
- Ranking
- Result fusion

**See**: [query/README.md](query/README.md)

---

#### Knowledge Weaver (`weaver/`)
**Graph-based knowledge management**

**Features**:
- Entity extraction
- Relationship mapping
- Graph traversal
- Knowledge consolidation

**See**: [weaver/README.md](weaver/README.md)

---

### Utilities

#### Task Scheduler (`task-scheduler/`)
**Async task management**

**Features**:
- Priority queues
- Deadline scheduling
- Resource limits
- Cancellation

**See**: [task-scheduler/README.md](task-scheduler/README.md)

---

#### Tokenization (`tokenization/`)
**Text tokenization utilities**

**Features**:
- HuggingFace tokenizers
- Byte-pair encoding
- Token counting

**See**: [tokenization/README.md](tokenization/README.md)

---

#### Values (`values/`)
**Typed value system**

**Features**:
- Dynamic typing
- Serialization
- Type conversions
- Validation

**See**: [values/README.md](values/README.md)

---

#### Native Handler (`native-handler/`)
**Native messaging request handler**

**Features**:
- Route requests to appstate
- Error handling
- Response formatting

**See**: [native-handler/README.md](native-handler/README.md)

---

## Build & Run

### Prerequisites
```bash
# Rust 1.75+
rustup update

# Protocol Buffers compiler
# Windows: choco install protoc
# macOS: brew install protobuf
# Linux: apt install protobuf-compiler
```

### Build
```bash
cd Rust

# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Build specific crate
cargo build -p tabagent-server
```

### Run
```bash
# Default (all modes)
cargo run --bin tabagent-server

# Specific mode
cargo run --bin tabagent-server -- --mode web --port 3000

# Release mode
cargo run --release --bin tabagent-server

# With script (auto-kills old processes)
./run-server.ps1  # Windows PowerShell
./run-server.bat  # Windows CMD
```

### Test
```bash
# All tests
cargo test --workspace

# Specific crate
cargo test -p storage

# Integration tests
cargo test --test '*'

# With output
cargo test -- --nocapture
```

---

## gRPC Communication

### Proto Definitions
Located in `protos/`:
- `database.proto` - Database service
- `ml_inference.proto` - ML services (ModelManagement, Transformers, MediaPipe)

### Generate Code
```bash
# Rust (automatic via build.rs)
cargo build -p common

# Python
cd ../PythonML
./generate_protos.bat
```

### Architecture
**Rust → Python**:
- `MlClient` in `common/src/ml_client.rs`
- Connects to `localhost:50051`
- Calls: LoadModel, GenerateText, StreamFaceDetection, etc.

**Database**:
- `DatabaseClient` in `storage/src/database_client.rs`
- Can run in-process or remote (gRPC)
- Transparent switching

**See**: [GRPC_ARCHITECTURE.md](GRPC_ARCHITECTURE.md)

---

## Configuration

### Database Path
```bash
# Default (platform-specific AppData)
# Windows: %APPDATA%/TabAgent/tabagent_db/
# Linux: ~/.local/share/TabAgent/tabagent_db/
# macOS: ~/Library/Application Support/TabAgent/tabagent_db/

# Custom
cargo run --bin tabagent-server -- --db-path ./my_db
```

### Model Cache Path
```bash
# Default
# Windows: %APPDATA%/TabAgent/models/
# Linux: ~/.local/share/TabAgent/models/
# macOS: ~/Library/Application Support/TabAgent/models/

# Custom
cargo run --bin tabagent-server -- --model-cache-path ./my_models
```

---

## Crate Dependencies

```
server
├── api (HTTP routes)
├── appstate (shared state)
├── webrtc (signaling)
├── native-messaging (extension)
└── common (types, clients)

appstate
├── storage (database)
├── model-cache (downloads)
├── hardware (detection)
├── common (gRPC clients)
└── ... (all other crates)

storage
├── indexing (vector search)
├── query (semantic search)
├── weaver (knowledge graph)
└── common (types)

common
├── tonic, prost (gRPC)
└── (no other workspace dependencies)
```

---

## Performance

| Component | Metric | Value |
|-----------|--------|-------|
| HTTP API | Latency | <5ms (p99) |
| WebRTC Signaling | Setup Time | <100ms |
| Database Writes | Throughput | 10k/s |
| Database Reads | Throughput | 50k/s |
| Model Download | Speed | 100MB/s (parallel chunks) |
| gRPC (Rust↔Python) | Latency | <1ms (localhost) |

*i9-12900K + NVMe SSD*

---

## Roadmap

### High Priority
- [ ] Linux/macOS hardware detection
- [ ] GPU memory management (VRAM tracking)
- [ ] Model quantization (GPTQ, AWQ)
- [ ] WebRTC video encoding/decoding

### Medium Priority
- [ ] Distributed database (multi-node)
- [ ] Model fine-tuning API
- [ ] Ray tracing for attention visualization
- [ ] Plugin system

### Low Priority
- [ ] Web UI (dashboard)
- [ ] Monitoring/metrics (Prometheus)
- [ ] Clustering support
- [ ] Docker deployment

**See**: Individual crate TODO.md files for details

---

## Documentation

- **[GRPC_ARCHITECTURE.md](GRPC_ARCHITECTURE.md)** - gRPC design patterns
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - System architecture
- **[Rust-Architecture-Guidelines.md](Rust-Architecture-Guidelines.md)** - Code standards
- **[docs/](docs/)** - Database architecture, MIA memory

---

## Contributing

1. Read architecture docs
2. Check crate-specific README.md + TODO.md
3. Write tests
4. Run `cargo fmt` and `cargo clippy`
5. No compilation warnings

---

## See Also

- **[PythonML/README.md](../PythonML/README.md)** - Python ML services
- **[Root README.md](../README.md)** - Project overview

