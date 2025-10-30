# gRPC Architecture

## Overview

TabAgent uses gRPC for service-to-service communication, enabling clean separation of concerns and location transparency.

## Services

### 1. Database Service (Rust)

**Location:** `storage/src/grpc_server.rs`

**Port:** 50052 (configurable)

**Endpoints:**
- `GetConversations(ConversationRequest) -> ConversationResponse`
- `StoreConversation(StoreConversationRequest) -> StatusResponse`
- `GetKnowledge(KnowledgeRequest) -> KnowledgeResponse`
- `StoreKnowledge(StoreKnowledgeRequest) -> StatusResponse`
- `GetEmbeddings(EmbeddingRequest) -> EmbeddingResponse`
- `StoreEmbedding(StoreEmbeddingRequest) -> StatusResponse`
- `GetToolResults(ToolResultRequest) -> ToolResultResponse`
- `StoreToolResult(StoreToolResultRequest) -> StatusResponse`

**Modes:**
- **In-Process** (default): No network, direct function calls
- **Remote**: TCP/HTTP2 communication for separate service

### 2. ML Inference Service (Python)

**Location:** `PythonML/ml_server.py`

**Port:** 50051 (default)

**Services:**

#### TransformersService
- `GenerateText(TextRequest) -> stream TextResponse` - Streaming text generation
- `GetEmbeddings(EmbeddingRequest) -> EmbeddingResponse` - Text embeddings
- `ChatCompletion(ChatRequest) -> stream ChatResponse` - Chat completion

#### MediapipeService
- `DetectFaces(ImageRequest) -> FaceDetectionResponse`
- `DetectHands(ImageRequest) -> HandDetectionResponse`
- `DetectPose(ImageRequest) -> PoseDetectionResponse`

## Configuration

### Environment Variables

```bash
# Database service
DATABASE_ENDPOINT=http://localhost:50052  # Optional, uses in-process if not set

# ML service
ML_ENDPOINT=http://localhost:50051  # Default: localhost:50051
```

### Server Modes

#### Development (Default)
```
┌─────────────────────────────┐
│  Rust Server Process        │
│  ├── HTTP API (port 3000)   │
│  ├── Storage (in-process)   │
│  └── WebRTC (port 8002)     │
└─────────────────────────────┘
         ↓ gRPC
┌─────────────────────────────┐
│  Python ML (port 50051)     │
└─────────────────────────────┘
```

#### Microservices Mode
```
┌──────────────┐    gRPC    ┌──────────────┐
│  API Server  │ ←────────→ │  Storage     │
│  (port 3000) │            │  (port 50052)│
└──────┬───────┘            └──────────────┘
       │ gRPC
       ↓
┌──────────────┐
│  Python ML   │
│  (port 50051)│
└──────────────┘
```

## Protocol Buffers

### Location

`Rust/protos/`
- `database.proto` - Database service definitions
- `ml_inference.proto` - ML service definitions

### Code Generation

**Rust (automatic via build.rs):**
```bash
cargo build  # Generates code in common/src/generated/
```

**Python:**
```bash
cd PythonML
bash generate_protos.sh
```

## Usage Examples

### Rust Client (Database)

```rust
use server::grpc_config::{GrpcConfig, DatabaseClient};

// In-process (default)
let db_client = DatabaseClient::new_in_process().await?;

// Remote
let db_client = DatabaseClient::new_remote("http://localhost:50052").await?;

// From config (reads env vars)
let config = GrpcConfig::from_env();
let db_client = DatabaseClient::from_config(&config).await?;
```

### Rust Client (ML)

```rust
use server::grpc_config::{GrpcConfig, MlClients};

let config = GrpcConfig::from_env();
let ml_clients = MlClients::from_config(&config).await?;

if let Some(mut client) = ml_clients.transformers {
    let request = TextRequest {
        prompt: "Hello world".to_string(),
        model: "gpt2".to_string(),
        max_length: 50,
        temperature: 0.7,
        top_p: 0.95,
    };
    
    let mut stream = client.generate_text(request).await?.into_inner();
    
    while let Some(response) = stream.message().await? {
        print!("{}", response.text);
    }
}
```

### Python Server

```bash
cd PythonML
pip install -r requirements.txt
python ml_server.py
```

## Benefits

### 1. **Location Transparency**
- Same code works for local and remote services
- Switch with environment variable

### 2. **Language Agnostic**
- Rust ↔ Python via gRPC
- Can add Go, Node.js, etc.

### 3. **Clean Separation**
- Storage can be extracted to separate project
- ML models in Python where ecosystem is best
- API/business logic in Rust for performance

### 4. **Future-Proof**
- Easy to dockerize
- Easy to scale horizontally
- Easy to deploy to cloud

## Migration Path

### Phase 1: Current (In-Process)
- All Rust in one process
- Python ML as separate service

### Phase 2: Extract Storage
- Move storage to separate service
- One environment variable change

### Phase 3: Microservices
- Each service in own container
- Load balancer + service discovery

### Phase 4: Cloud Native
- Kubernetes deployment
- Auto-scaling
- Multi-region

## Performance

### In-Process
- **Latency:** ~nanoseconds (function call)
- **Overhead:** Zero

### Local gRPC
- **Latency:** ~1-2ms (localhost TCP)
- **Overhead:** Minimal (HTTP/2 framing)

### Remote gRPC
- **Latency:** Network dependent
- **Overhead:** Same as any HTTP/2

## Original Problem Solved

**Before:**
```rust
❌ server/config.rs     → get_default_db_path().join("tabagent_db")
❌ server/main.rs       → get_default_db_path().join("tabagent_db")
❌ appstate/state.rs    → get_default_db_path().join("tabagent_db")
```

**After:**
```rust
✅ storage/grpc_server.rs  → Handles ALL database paths internally
✅ server/main.rs          → Just connects to storage service
✅ appstate                → No knowledge of storage implementation
```

**Result:** Clean separation, single source of truth, location transparent!

