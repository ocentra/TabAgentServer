# TabAgent Server

**Multi-modal Intelligent Assistant (MIA) - Agentic AI Platform**

Unified inference infrastructure powering the [TabAgent browser extension](https://github.com/ocentra/TabAgent) and future agentic systems. Combines Rust performance with Python ML for vision, language, and audio understanding.

**What is MIA?** A cognitive architecture that remembers, learns, and acts—not just a model server. Think of it as a brain with multiple memory systems (7 databases), learning from experience, and making intelligent decisions using eyes (vision), ears (audio), and reasoning (LLMs).

**Learn More**:
- 🎯 [MIA_VISION.md](MIA_VISION.md) - **What we're building** (vision document, "show and tell")
- 🧠 [Rust/docs/mia_memory.md](Rust/docs/mia_memory.md) - **How it works** (complete technical architecture)

---

## Architecture

### Desktop Application (Tauri-based)

```
TabAgent Desktop (.exe/.app/deb)
│
├── src-tauri/        → Tauri Rust backend
│   ├── Embedded web server (localhost:3000)
│   │   ├── / → Dashboard (React)
│   │   ├── /workflows → Agent Builder (Vue 3)
│   │   └── /api/* → REST API
│   │
│   └── Native messaging → Chrome Extension
│
├── dashboard/        → React UI (system monitoring & management)
├── agent-builder/    → Vue 3 UI (visual workflow editor)
│
├── Rust/            → Core inference (WebRTC, gRPC, Database, API)
├── PythonML/        → ML services (MediaPipe, Transformers, LiteRT)
├── External/        → Third-party integrations (BitNet, MediaPipe)
└── Scripts/         → Build automation
```

**User Experience**: Double-click `.exe` → Dashboard opens → No Docker, no terminals, no setup!

---

## Key Capabilities

## MIA Vision: Multi-modal Agentic AI

**Beyond Text-Only LLMs** - MIA agents will see, hear, and understand the world:

🎯 **Vision Agents** (MediaPipe + Computer Vision)
- Real-time face/hand/pose tracking
- Gesture recognition for UI control
- Scene understanding & object detection
- Gaze estimation for attention tracking
- **Agents decide with eyes, not just text**

🗣️ **Audio Agents** (Whisper + Speech)
- Real-time transcription & translation
- Voice commands & speaker recognition
- Audio scene analysis
- **Agents listen and respond naturally**

🤖 **Language Agents** (Transformers + LiteRT)
- Multi-turn reasoning & chat
- Code generation & analysis
- Multi-modal understanding (Florence2, CLIP)
- 1.58-bit BitNet (50 tok/s on CPU!)
- **Agents think and communicate**

💾 **Cognitive Memory** (7 Databases)
- Conversations (episodic memory)
- Knowledge graph (semantic memory)
- Tool results (external knowledge cache)
- **Experience** (learning from feedback)
- Embeddings (similarity search)
- Meta-memory (knows what it knows)
- **Agents remember and learn**

🔧 **Tool Use & Learning**
- Web search, scraping, APIs
- Action outcome tracking
- User feedback integration
- Success/failure pattern recognition
- **Agents improve from experience**

⚡ **Performance**
- Hardware-aware (CPU/GPU/NPU auto-detection)
- BitNet 1.58-bit (all platforms)
- Streaming inference
- VRAM-aware offloading

---

## Quick Start

### Prerequisites
- **Rust**: 1.75+ (`rustup`)
- **Python**: 3.10+ with pip
- **Node.js**: 18+ with npm 9+
- **GPU**: NVIDIA/AMD/Intel (optional, auto-detected)

### Setup

```bash
# 1. Clone repository with submodules
git clone --recurse-submodules https://github.com/ocentra/TabAgent
cd TabAgent/TabAgentServer

# If you already cloned, init submodules:
git submodule update --init --recursive

# 2. Install dependencies
npm install

# 3. Install Python dependencies
cd PythonML
pip install -r requirements.txt
python -m grpc_tools.protoc -I../Rust/protos --python_out=generated --grpc_python_out=generated ../Rust/protos/*.proto
cd ..

# 4. Run development environment (auto-starts everything!)
npm run dev
```

This starts:
- Rust backend (port 3000)
- Dashboard dev server (port 5173)
- Agent Builder dev server (port 5175)
- Python ML service (gRPC port 50051, auto-spawned)

Server starts on:
- **HTTP API**: http://localhost:3000
- **WebRTC**: http://localhost:8002
- **Python ML gRPC**: localhost:50051 (internal)

### Test MediaPipe

```bash
cd PythonML
pytest tests/test_mediapipe.py -v
```

---

## Desktop App (Dashboard + Agent Builder)

### For End Users
1. Download `TabAgent.exe` (Windows) / `TabAgent.app` (macOS) / `tabagent-desktop.deb` (Linux)
2. Double-click to run
3. Desktop app opens showing Dashboard at `localhost:3000`
4. Navigate to `/workflows` for Agent Builder

**No installation, no setup, no Docker - just works!**

### For Developers

#### Install Dependencies
```bash
# Root + Tauri
npm install

# Dashboard (React)
cd dashboard && npm install && cd ..

# Agent Builder (Vue 3)
cd agent-builder && npm install && cd ..
```

#### Development Mode (Hot Reload)
```bash
npm run dev
```

**What Happens:**
1. Smart port allocation (kills stale processes, finds available ports)
2. Starts Rust backend (default: port 3000, fallback: 3001-3003)
3. Starts Dashboard dev server (default: port 5173, fallback: 5174-5176)
4. Starts Agent Builder dev server (default: port 5175, fallback: 5177-5179)
5. All components auto-connect via dynamic proxies

**Features:**
- ✅ Single instance enforcement (can't run twice)
- ✅ Auto-kills stale TabAgent processes
- ✅ Smart fallback if ports busy
- ✅ Friendly error if external app conflicts
- ✅ Hot reload on all frontends

#### Build Production Binary
```bash
npm run build
```
Creates:
- **Windows**: `src-tauri/target/release/bundle/msi/TabAgent Desktop.msi`
- **macOS**: `src-tauri/target/release/bundle/dmg/TabAgent Desktop.dmg`
- **Linux**: `src-tauri/target/release/bundle/deb/tabagent-desktop.deb`

#### Platform Requirements
- **Windows**: Visual Studio C++ Build Tools
- **macOS**: Xcode Command Line Tools
- **Linux**: webkit2gtk, libappindicator

---

## Project Structure

### `src-tauri/` - Desktop App (Tauri)
Rust-based desktop application wrapper.

**Entry Point**: `src/main.rs` - Tauri app + embedded web server
**Serves**: Dashboard (/) and Agent Builder (/workflows) on port 3000
**Output**: `.exe` (Windows), `.app` (macOS), `.deb` (Linux)

---

### `dashboard/` - System Dashboard (React + TypeScript)
Modern React dashboard for system monitoring and management.

**Features**:
- Model management (install, configure, monitor)
- Database explorer with knowledge graph visualization
- Real-time system metrics and resource monitoring
- API testing interface
- WebRTC demos

**Routes**: `/`, `/models`, `/database`, `/knowledge`, `/settings`
**Dev**: `npm run dev` → port 5173
**Build**: `npm run build` → `dist/`

---

### `agent-builder/` - Workflow Editor (Vue 3 + TypeScript)
n8n-inspired visual workflow editor for building AI agent workflows.

**Features**:
- Drag & drop node-based editor with Vue Flow
- Resizable/collapsible panels
- Dark theme by default
- D-shaped trigger nodes, status indicators, smart edge routing
- Node library with categories
- Properties panel for node configuration

**Routes**: `/workflows`, `/workflows/new`, `/workflows/:id`
**Dev**: `npm run dev` → port 5175
**Build**: `npm run build` → `dist/`

---

### [`PythonML/`](PythonML/README.md) - ML Services
Python ML stack running as gRPC subprocess managed by Rust.

**Modules**:
- `services/` - gRPC service implementations
- `mediapipe/` - Vision/pose tracking (7 specialized modules)
- `pipelines/` - HuggingFace Transformers (15 pipeline types)
- `litert/` - Quantized edge models
- `core/` - File provider, stream handling

**Communication**: Rust spawns Python, communicates via gRPC (port 50051)

---

### [`Rust/`](Rust/README.md) - Core Infrastructure
High-performance inference orchestration and system integration.

**Key Crates**:
- `server` - Main server binary (HTTP + WebRTC + Native)
- `api` - REST API routes with OpenAPI
- `appstate` - Application state + model orchestrator
- `storage` - Database layer (MIA memory system)
- `common` - Shared types, gRPC clients, platform utils
- `model-cache` - Model download & management
- `webrtc` - WebRTC signaling & data channels
- `native-messaging` - Chrome extension protocol
- `hardware` - Auto-detection (CPU/GPU/NPU)
- `onnx-loader`, `gguf-loader` - Model loaders
- `pipeline` - Inference orchestration

**See**: [Rust/README.md](Rust/README.md) for all crates

---

## Communication Flow

```
User
  ↓ (double-clicks .exe)
Tauri Desktop App
  ├─→ Dashboard (React) @ localhost:3000/
  └─→ Agent Builder (Vue 3) @ localhost:3000/workflows
      ↓
  Embedded Rust Server (port 3000)
      ├─→ HTTP API (/api/*)
      ├─→ WebSocket (/ws)
      └─→ Native Messaging → Chrome Extension
          ↓
      (gRPC - localhost:50051)
          ↓
  Python ML Service
      ↓
  Hardware (CPU/GPU/NPU)
```

**Key Points**:
- **Tauri** wraps everything in native desktop app
- **Rust** is the orchestrator and "brain"
- **Python** is stateless ML service (gRPC slave)
- **UIs** are decoupled (can swap independently, unlike n8n!)
- **Single port** (3000) for simplicity

---

## Features

### Vision AI (MediaPipe)
- ✅ Face detection (6 keypoints)
- ✅ Face mesh (468 landmarks, 3D)
- ✅ Hand tracking (21 landmarks + 7 gestures)
- ✅ Pose tracking (33 landmarks + angles)
- ✅ Holistic tracking (543 landmarks combined)
- ✅ Iris tracking (gaze estimation)
- ✅ Segmentation (person/background + effects)

### Language Models (Transformers)
- ✅ Text generation (streaming)
- ✅ Embeddings (sentence-transformers)
- ✅ Chat completion
- ✅ Multi-modal (Florence2, CLIP, Whisper)
- ⚙️ All 15 pipelines (in progress)

### Edge Models (LiteRT + BitNet)
- ✅ BitNet 1.58-bit (CPU-optimized, all platforms)
- ⚙️ Quantized Gemma models (LiteRT)
- ⚙️ XNNPACK/GPU acceleration

### Database (Storage)
- ✅ 7-database MIA architecture
- ✅ gRPC service for remote access
- ✅ Vector embeddings
- ✅ Graph queries

### WebRTC
- ✅ Signaling server
- ✅ Data channels
- ✅ Video stream processing
- ✅ Browser demos

---

## Development

### HuggingFace Token Setup (For Gated Models)

**For Development (Local Testing):**

1. Copy environment template:
   ```bash
   cp ENV_TEMPLATE.txt .env
   ```

2. Edit `.env` and add your token:
   ```bash
   HUGGINGFACE_TOKEN=hf_xxxxx
   ```
   Get token from: https://huggingface.co/settings/tokens

**For Production (UI Flow):**

Users enter token via UI → stored securely in OS keyring:
- Windows: Credential Manager
- macOS: Keychain
- Linux: Secret Service

**API Endpoints:**
```bash
# Store token
POST /v1/hf/token
{"token": "hf_xxxxx"}

# Check status
GET /v1/hf/token/status

# Clear token
DELETE /v1/hf/token
```

**How It Works:**
1. Extension/Dashboard requests gated model
2. If no token → UI prompts for HF token
3. Token stored securely via API
4. Rust downloads model using token
5. Python accesses via Rust cache (no direct HF access)

---

### Running Tests

```bash
# Python tests
cd PythonML
pytest -v

# Rust tests
cd Rust
cargo test --workspace

# Integration tests
cd Rust
cargo test --test '*' -- --test-threads=1
```

### Building

```bash
# Development
cd Rust
cargo build

# Release (optimized)
cargo build --release

# Specific mode
cargo run --bin tabagent-server -- --mode web --port 3000
```

### Server Modes

- `native` - Native messaging only (for extension)
- `http` - HTTP API only
- `webrtc` - WebRTC signaling only
- `web` - HTTP + WebRTC (no native messaging)
- `all` - Everything (default)

---

## Documentation

### Vision & Architecture
- **[MIA_VISION.md](MIA_VISION.md)** - 🎯 **What we're building** (vision document, accessible overview)
- **[Rust/docs/mia_memory.md](Rust/docs/mia_memory.md)** - 🧠 **MIA Cognitive Architecture** (complete technical design, 7 databases)

### Component Documentation
- **[PythonML/README.md](PythonML/README.md)** - Python ML services architecture
- **[Rust/README.md](Rust/README.md)** - Rust infrastructure overview
- **[Rust/GRPC_ARCHITECTURE.md](Rust/GRPC_ARCHITECTURE.md)** - gRPC communication design
- **[Rust/docs/](Rust/docs/)** - Database layer specs, query engine, knowledge weaver

### Module Documentation
Each module has:
- `README.md` - Architecture, usage, examples
- `TODO.md` - Current state, planned features

---

## Performance

| Configuration | First Token | Throughput | Memory |
|--------------|-------------|------------|--------|
| MediaPipe (face mesh) | 15ms | 60 FPS | 200MB RAM |
| Transformers (7B Q4) | 80ms | 35 tok/s | 6GB VRAM |
| LiteRT (Gemma 3B) | 50ms | 45 tok/s | 4GB VRAM |
| **BitNet (3B 1.58-bit)** | **40ms** | **50 tok/s** | **2GB RAM** ✅ |

*NVIDIA RTX 4090 + i9-12900K*

---

## Platform Support

| Platform | Status | Notes |
|----------|--------|-------|
| Windows | ✅ Complete | Full hardware detection, DirectML |
| Linux | ✅ Complete | CUDA/ROCm support |
| macOS | ✅ Complete | Metal acceleration |

### BitNet Support
✅ **1.58-bit quantization** across all platforms:
- **CPU**: x86 (SSE, AVX2, AVX512), ARM (NEON)
- **GPU**: NVIDIA (CUDA), AMD (ROCm), Intel (OpenCL)
- **Performance**: 50 tok/s @ 3B model on CPU (no GPU needed!)

---

## License

Apache 2.0 - See [LICENSE](LICENSE)

---

## Project Context

**Primary Purpose**: Powers the [TabAgent browser extension](https://github.com/ocentra/TabAgent) with AI capabilities.

**Vision**: Not limited to browser automation—MIA is a unified multi-modal AI platform for:
- Browser assistants (TabAgent)
- Desktop AI agents (future)
- Voice assistants (future)
- Vision-based automation (future)
- Any application needing cognitive AI

**What Makes MIA Different**:
- **Multi-modal by design**: Vision + Audio + Text from day one
- **Cognitive architecture**: 7-database memory system that learns
- **True agents**: Not just models—agents that see, hear, remember, learn, and act
- **Production-ready**: Real implementations, no stubs, enterprise-grade

## Contributing

See individual module READMEs for contribution guidelines:
- [PythonML/README.md](PythonML/README.md)
- [Rust/README.md](Rust/README.md)

---

## System Requirements

**Minimum**:
- 8GB RAM
- 4-core CPU
- 10GB disk space

**Recommended**:
- 16GB RAM
- NVIDIA/AMD GPU with 8GB+ VRAM
- 50GB disk space (for models)

**Models stored in**:
- Windows: `%APPDATA%/TabAgent/models/`
- Linux: `~/.local/share/TabAgent/models/`
- macOS: `~/Library/Application Support/TabAgent/models/`

### Windows Build Requirements

For building on Windows, you need `libclang.dll`:

```powershell
# Auto-detect and set LIBCLANG_PATH
.\setup_libclang.ps1
```

Or install:
- Visual Studio 2022 with "Desktop development with C++"
- LLVM from https://github.com/llvm/llvm-project/releases
