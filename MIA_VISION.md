# MIA: Multi-modal Intelligent Assistant

**Vision Document - What We're Building**

---

## What is MIA?

MIA is not just another LLM wrapper. It's a **cognitive AI platform** that sees, hears, remembers, learns, and acts—like a digital brain.

**Think of it as**:
- 🧠 A brain with **7 specialized memory systems** (not just a vector database)
- 👁️ **Eyes** via MediaPipe (face/hand/pose tracking, gaze estimation)
- 👂 **Ears** via Whisper (real-time speech understanding)
- 💭 **Reasoning** via Transformers + BitNet (1.58-bit quantization!)
- 📚 **Memory** that learns from experience and gets smarter over time

---

## The Problem We're Solving

**Current AI assistants are blind, deaf, and forgetful:**
- ❌ Text-only (can't see your screen or gestures)
- ❌ No memory (forget everything after the chat)
- ❌ Don't learn (repeat the same mistakes)
- ❌ Slow (require powerful GPUs)

**MIA is different:**
- ✅ **Multi-modal**: Vision + Audio + Text
- ✅ **Remembers**: 7-database cognitive architecture
- ✅ **Learns**: Experience database tracks what works
- ✅ **Fast**: BitNet runs at 50 tok/s on CPU (no GPU needed!)

---

## Primary Use Case: TabAgent Extension

MIA powers the **[TabAgent browser extension](https://github.com/yourusername/TabAgent)** - your AI copilot that:

**Sees Your Screen**:
- Tracks your gaze (knows what you're looking at)
- Detects gestures (control with hand movements)
- Understands UI elements (knows what's clickable)

**Hears You**:
- Voice commands (no typing needed)
- Real-time transcription (multilingual)
- Speaker identification (knows who's talking)

**Remembers Context**:
- Conversation history (episodic memory)
- Knowledge graph (semantic memory)
- Past actions (experience memory)
- Search results cache (tool memory)

**Learns From You**:
- "That search wasn't helpful" → learns to refine queries
- "Perfect!" → remembers successful strategies
- User corrections → adapts behavior

---

## Beyond Browser: Future Vision

MIA is a **unified platform** not limited to browsers:

### Desktop AI Agents (Future)
- Screen automation with vision
- Voice-controlled workflows
- Intelligent file management
- Context-aware assistance

### Voice Assistants (Future)
- Natural conversation
- Multi-modal understanding
- Proactive suggestions
- Learning your preferences

### Vision-Based Automation (Future)
- Gesture-controlled interfaces
- Accessibility features
- Video analysis
- Real-time object tracking

### Developer Platform (Future)
- Any app can integrate MIA
- Multi-modal API
- Cognitive memory system
- Pre-trained agent behaviors

---

## Technical Highlights

### 1. Multi-modal by Design

**Vision (MediaPipe)**:
```
Real-time (60+ FPS):
- Face detection (6 keypoints)
- Face mesh (468 landmarks, 3D!)
- Hand tracking (21 landmarks + 7 gestures)
- Pose tracking (33 landmarks)
- Holistic (543 landmarks: face + hands + pose)
- Iris tracking (gaze estimation)
- Segmentation (person/background)
```

**Audio (Whisper + Speech)**:
```
Real-time:
- Transcription (multilingual)
- Translation
- Voice commands
- Speaker recognition
```

**Language (Transformers + BitNet)**:
```
Models:
- GPT-style text generation
- Florence2 (vision-language)
- CLIP (image-text)
- Whisper (speech-to-text)
- BitNet 1.58-bit (50 tok/s on CPU!)
```

### 2. Cognitive Architecture (7 Databases)

Not a single vector database—**7 specialized memory systems** like a human brain:

```
1. conversations/     → Episodic Memory
   - What did we discuss?
   - When did this happen?
   - Current chat context
   
2. knowledge/         → Semantic Memory
   - Static facts about the world
   - Entities and relationships
   - Concept understanding
   
3. embeddings/        → Similarity Search
   - Find related messages
   - Semantic search
   - Vector indexing (HNSW)
   
4. tool-results/      → External Knowledge Cache
   - Web search results
   - Scraped pages
   - API responses
   - Avoid redundant searches
   
5. experience/        → Procedural Memory ⭐
   - What actions worked?
   - User feedback
   - Success/failure patterns
   - LEARNS OVER TIME!
   
6. summaries/         → Compressed Memory
   - Daily summaries
   - Weekly summaries
   - Context compression
   
7. meta/              → Meta-Memory
   - What do I know?
   - Where should I look?
   - Query optimization
   - Confidence tracking
```

**Key Innovation**: Database #5 (`experience/`) makes MIA **truly intelligent**:
- Remembers when user says "That's not helpful"
- Learns which search strategies work
- Improves over time from feedback
- Knows when it's uncertain

### 3. Performance: BitNet 1.58-bit

**The Game Changer**: Run 3B parameter models on CPU at 50 tok/s!

```
Traditional LLM (3B, FP16):
- Size: 6GB
- RAM: 8GB needed
- Speed: 15 tok/s (CPU)
- Requires: GPU for decent speed

BitNet 1.58-bit (3B):
- Size: 750MB ✅
- RAM: 2GB needed ✅
- Speed: 50 tok/s (CPU) ✅
- Requires: Nothing! Works on any CPU ✅
```

**Platform Support**:
- ✅ Windows (x86, ARM, DirectML)
- ✅ Linux (x86, ARM, CUDA, ROCm)
- ✅ macOS (x86, Apple Silicon, Metal)

### 4. Real-time Streaming

**Everything streams**:
- Vision: 60 FPS face/hand/pose tracking
- Audio: Real-time transcription
- Text: Token-by-token generation
- WebRTC: Browser → Rust → Python → Hardware

### 5. Hardware-Aware

**Auto-detects and uses best backend**:
- CPUs: x86 (AVX2, AVX512), ARM (NEON)
- GPUs: NVIDIA (CUDA), AMD (ROCm), Intel (OpenCL, oneAPI)
- NPUs: AMD Ryzen AI, Intel Core Ultra
- Platform: DirectML (Windows), Metal (macOS), Vulkan

---

## Example: Agent Learning in Action

**Scenario**: User asks "Find Rust database info"

### First Time (No Experience)
```
User: "Find Rust database info"
Agent: [Searches "Rust database"]
       → Returns generic SQL database results
User: "That's not helpful, I meant embedded databases"

→ Stored in experience/user-feedback/corrections
→ Pattern learned: "database" is ambiguous
```

### Second Time (Learning Applied)
```
User: "Find Rust database info"
Agent: [Checks experience database]
       → Found: "database" was ambiguous before
       → Pattern: Ask for clarification OR check context
Agent: "Did you mean embedded databases or SQL databases?"
User: "Embedded"
Agent: [Searches "Rust embedded database"]
       → Returns sled, redb, RocksDB docs
User: "Perfect!"

→ Stored in experience/user-feedback/approvals
→ Success pattern reinforced
→ Confidence: 0.85
```

### Third Time (Pattern Established)
```
User: "Find Rust database info"
Agent: [Checks experience database]
       → Pattern: "database" → clarify OR infer from context
       → Checks conversation context
       → Found: Recent mentions of "sled", "embedded"
Agent: [Automatically refines to "Rust embedded database"]
       → Returns sled, redb docs (cached from before!)
       → No external API call needed
User: "Thanks!"

→ Pattern confidence increases to 0.92
```

**This is what makes MIA intelligent!**

---

## Architecture Overview

### High-Level System Architecture

```
┌──────────────────────────────────────────────────────────┐
│                   TabAgent Extension                     │
│              (Browser / Desktop / Web UI)                │
└──────┬──────────────────┬────────────────────┬───────────┘
       │                  │                    │
┌──────▼──────┐   ┌───────▼───────┐   ┌───────▼───────┐
│  HTTP API   │   │  Native Msg   │   │    WebRTC     │
│ (REST+WS)   │   │ (stdin/stdout)│   │  (streaming)  │
│ Port 3000   │   │               │   │  Port 8002    │
└──────┬──────┘   └───────┬───────┘   └───────┬───────┘
       │                  │                    │
       └──────────────────┼────────────────────┘
                          │
┌─────────────────────────▼──────────────────────────────┐
│                   RUST SERVER (Orchestrator)           │
│  ┌──────────────────────────────────────────────────┐  │
│  │                AppState (Core State)             │  │
│  │  • Model Orchestrator  • Hardware Detection      │  │
│  │  • HF Auth Manager     • Generation Tokens       │  │
│  └──────────────────────────────────────────────────┘  │
│                                                         │
│  ┌────────────┐  ┌────────────┐  ┌─────────────────┐  │
│  │ API Routes │  │   WebRTC   │  │ Native Handler  │  │
│  │ (api/)     │  │  Signaling │  │ (native-handler)│  │
│  └────────────┘  └────────────┘  └─────────────────┘  │
└──────┬──────────────────┬────────────────┬────────────┘
       │                  │                │
┌──────▼──────┐  ┌────────▼────────┐  ┌───▼──────────┐
│   Storage   │  │  Task Scheduler │  │ Model Cache  │
│  (storage/) │  │ (task-scheduler)│  │(model-cache/)│
│             │  │                 │  │              │
│ Coordinator │  │ Priority Queue  │  │ HF Hub Sync  │
│ 7 DBs(sled) │  │ Activity-Aware  │  │ Chunk DL     │
└──────┬──────┘  └────────┬────────┘  └──────────────┘
       │                  │
       │         ┌────────▼────────┐
       │         │ Knowledge Weaver│
       │         │    (weaver/)    │
       │         │                 │
       │         │ • Entity Extrac │
       │         │ • Semantic Index│
       │         │ • Assoc. Linker │
       │         │ • Summarizer    │
       │         └────────┬────────┘
       │                  │
       └──────────────────┼──────────────┐
                          │              │
              ┌───────────▼──────────┐   │
              │   Indexing Layer     │   │
              │    (indexing/)       │   │
              │                      │   │
              │ • HNSW (vectors)     │   │
              │ • B-tree (struct)    │   │
              │ • Graph (adjacency)  │   │
              └──────────────────────┘   │
                                         │
              ┌────────────────────────┐ │
              │    Query Engine        │ │
              │     (query/)           │ │
              │                        │ │
              │ Converged Query Model: │ │
              │ 1. Structural Filter   │ │
              │ 2. Semantic Search     │ │
              │ 3. Graph Expansion     │ │
              │ 4. Rank & Confidence   │ │
              └────────────────────────┘ │
                                         │
                          ┌──────────────▼───────┐
                          │       gRPC           │
                          │    (Port 50051)      │
                          └──────────┬───────────┘
                                     │
                          ┌──────────▼───────────┐
                          │   Python ML Service  │
                          │    (PythonML/)       │
                          │                      │
                          │ • Model Management   │
                          │ • RustFileProvider   │
                          │ • StreamHandler      │
                          └──────────┬───────────┘
                                     │
              ┌──────────────────────┼──────────────────┐
              │                      │                  │
         ┌────▼────┐          ┌──────▼──────┐    ┌─────▼─────┐
         │MediaPipe│          │Transformers │    │  LiteRT   │
         │(vision/)│          │(pipelines/) │    │ (litert/) │
         │         │          │             │    │           │
         │7 Modules│          │15 Pipelines │    │Gemma/Edge │
         │60+FPS   │          │BitNet 1.58  │    │Quantized  │
         └─────────┘          └─────────────┘    └───────────┘
```

### Communication Channels

- **HTTP API** (Port 3000): REST endpoints, WebSockets, OpenAPI/Swagger docs
- **Native Messaging**: Chrome extension protocol (stdin/stdout JSON)
- **WebRTC** (Port 8002): Real-time video/audio streaming, data channels
- **gRPC** (Port 50051): Rust ↔ Python internal communication

---

## Core Architecture Components

### Rust Infrastructure (Brain & Orchestration)

#### 1. **Storage Layer** ([`storage/`](Rust/docs/StorageLayer.md))
**Purpose**: 7-database MIA memory system

**Key Components**:
- `DatabaseCoordinator`: Manages all 7 database types
- `StorageManager`: Low-level sled operations (CRUD)
- **7 Databases**:
  - `conversations/` - SOURCE (user messages, chats)
  - `knowledge/` - DERIVED (entities, relationships)
  - `embeddings/` - DERIVED (semantic vectors)
  - `tool-results/` - EXTERNAL (cached searches, scraped pages)
  - `experience/` - LEARNING (action outcomes, feedback)
  - `summaries/` - DERIVED (hierarchical compression)
  - `meta/` - INDEXES (query routing, confidence)

**Why 7?** Each database has different:
- Access patterns (hot vs cold)
- Guarantees (ACID vs eventual)
- Recovery strategies (regenerate vs backup)

**Tech**: `sled` (embedded, ACID, fast)

**See**: [Rust/docs/StorageLayer.md](Rust/docs/StorageLayer.md) for implementation details

---

#### 2. **Indexing Layer** ([`indexing/`](Rust/docs/IndexingLayer.md))
**Purpose**: Fast search across all data types

**Key Components**:
- **HNSW Index**: Vector similarity search (embeddings)
- **Structural Indexes**: B-tree for timestamps, IDs, types
- **Graph Indexes**: Adjacency lists for entity relationships

**Performance**:
- Vector search: <1ms (10K vectors), <10ms (100K vectors)
- Structural filter: O(log n)
- Graph traversal: Configurable depth (1-hop, 2-hop, deep)

**Tech**: Lock-free concurrent access, incremental updates

**See**: [Rust/docs/IndexingLayer.md](Rust/docs/IndexingLayer.md) for HNSW implementation

---

#### 3. **Knowledge Weaver** ([`weaver/`](Rust/docs/KnowledgeWeaver.md))
**Purpose**: Autonomous background enrichment

**Key Modules**:
1. **Semantic Indexer**: Generates embeddings for messages
2. **Entity Linker**: Extracts entities (NER via ML)
3. **Associative Linker**: Finds relationships between entities
4. **Summarizer**: Hierarchical compression (daily → weekly → monthly)

**Flow**:
```
Message Inserted
    ↓ (async event)
Task Scheduler
    ↓ (queues tasks)
Weaver Modules
    ↓ (parallel processing)
Updated Databases (knowledge/, embeddings/, summaries/)
```

**Tech**: Event-driven (tokio MPSC), ML bridge to Python

**See**: [Rust/docs/KnowledgeWeaver.md](Rust/docs/KnowledgeWeaver.md) for module specs

---

#### 4. **Query Engine** ([`query/`](Rust/docs/QueryEngine.md))
**Purpose**: Unified query interface across all databases

**The Converged Query Model** (4 stages):
```rust
mia.query(Query {
    semantic: "Rust database design",
    time_scope: TimeScope::Today,
    context: Context::CurrentChat(chat_id),
    use_knowledge_graph: true,
    search_depth: SearchDepth::Level(2),
    temperature: Temperature::Hot,
    limit: 10,
})
```

**Execution Pipeline**:
1. **Stage 0: Meta-Memory** - Decides which DBs to search (routing)
2. **Stage 1: Structural Filter** - Fast candidate set (timestamps, chat IDs)
3. **Stage 2: Semantic Search** - HNSW vector search on candidates
4. **Stage 3: Graph Expansion** - Follow relationships (N-hop traversal)
5. **Stage 4: Rank & Filter** - Confidence scoring, reasoning

**Why This Matters**: 
- 90% of queries hit only `active/` tier (fast!)
- Deep searches use `archive/` (slower but rare)
- Meta-memory learns optimal routing over time

**See**: [Rust/docs/QueryEngine.md](Rust/docs/QueryEngine.md) for pipeline details

---

#### 5. **Task Scheduler** ([`task-scheduler/`](Rust/docs/))
**Purpose**: Activity-aware background processing

**Key Features**:
- **Priority Levels**: Urgent, Normal, Low, Batch
- **Activity Modes**: HighActivity, LowActivity, SleepMode
- **Task Types**: 
  - Embedding generation
  - Entity extraction (NER)
  - Summarization
  - Index updates
  - Database lifecycle (promotion/demotion)

**Smart Scheduling**:
- High Activity: Only urgent tasks (don't lag UI)
- Low Activity: Process normal queue
- Sleep Mode: Batch operations (consolidation, archiving)

**Integration**: 
```
Message Insert → Task Scheduler → Weaver → Updated DBs
```

---

#### 6. **Model Cache** ([`model-cache/`](Rust/docs/))
**Purpose**: HuggingFace Hub integration, local model storage

**Key Features**:
- Parallel chunk downloads (fast!)
- Resume interrupted downloads
- File serving to Python (via gRPC)
- Default model library
- Version management

**Storage**:
```
models/
├── microsoft--Florence-2-base/
│   ├── config.json
│   ├── model.safetensors
│   └── ...
└── meta-llama--Llama-2-7b-hf/
```

**Why This Matters**: Python never downloads directly—Rust manages all model files and serves them via gRPC `GetModelFile` RPC.

---

#### 7. **Hardware Detection** ([`hardware/`](Rust/docs/))
**Purpose**: Auto-detect CPU/GPU/NPU and select optimal backend

**Detects**:
- CPUs: Cores, threads, model, features (AVX2, AVX512, NEON)
- NVIDIA GPUs: Model, VRAM (via nvidia-smi), CUDA version
- AMD GPUs: Discrete/integrated, ROCm support
- Intel GPUs: Arc, integrated, OpenCL/oneAPI
- NPUs: AMD Ryzen AI, Intel Core Ultra

**Backends Selected**:
- CUDA (NVIDIA)
- ROCm (AMD)
- DirectML (Windows)
- Metal (macOS)
- OpenCL/Vulkan (cross-platform)
- XNNPACK (CPU-optimized)

---

### Python ML Services (Execution Layer)

#### 1. **Model Management** ([`services/model_management_service.py`](PythonML/services/README.md))
**Purpose**: Load/unload models, serve files from Rust cache

**RPCs**:
- `LoadModel`: Create pipeline, set file provider, return memory usage
- `UnloadModel`: Free resources
- `GetModelFile`: Stream file chunks (for Rust-managed models)
- `GetLoadedModels`: List all loaded models

**Integration**: Uses `RustFileProvider` to intercept HuggingFace downloads

---

#### 2. **Transformers Service** ([`pipelines/`](PythonML/pipelines/README.md))
**Purpose**: 15 HuggingFace Transformers pipelines

**Pipelines**:
- Text generation, embeddings, chat
- Florence2 (vision-language)
- CLIP (image-text)
- Whisper (speech-to-text)
- Translation, code completion, etc.

**Tech**: BitNet 1.58-bit, quantization (4-bit, 8-bit)

---

#### 3. **MediaPipe Service** ([`mediapipe/`](PythonML/mediapipe/README.md))
**Purpose**: Real-time vision AI (7 specialized modules)

**Modules**:
- Face detection, face mesh (468 landmarks)
- Hand tracking (21 landmarks + gestures)
- Pose tracking (33 landmarks)
- Holistic (543 landmarks!)
- Iris tracking (gaze)
- Segmentation

**Performance**: 60+ FPS real-time streaming

---

#### 4. **LiteRT Service** ([`litert/`](PythonML/litert/README.md))
**Purpose**: Quantized edge models (Gemma LiteRT)

**Tech**: TensorFlow Lite, XNNPACK, GPU delegates

---

## Data Flow Examples

### 1. Message Insert → Full Enrichment
```
1. User sends message
2. Storage: Insert to conversations/active
3. Task Scheduler: Queue tasks (embed, NER, summarize)
4. Weaver Semantic Indexer: Generate embedding
5. Storage: Insert to embeddings/active
6. Indexing: Update HNSW index
7. Weaver Entity Linker: Extract entities via Python ML
8. Storage: Insert entities to knowledge/active
9. Indexing: Update graph index
10. Done (all async, user sees message immediately!)
```

### 2. Query Execution (Fast Path)
```
1. User: "Rust database design"
2. Query Engine Stage 0: Meta-memory → search active/ only
3. Stage 1: Filter by time_scope=Today → 500 candidates
4. Stage 2: HNSW search → top 20 semantic matches
5. Stage 3: Skip (no graph traversal requested)
6. Stage 4: Rank by confidence → return top 10
7. Total time: <1ms
```

### 3. Model Inference (Python)
```
1. Rust: ml_client.load_model("microsoft/Florence-2-base")
2. Python: Creates Florence2Pipeline
3. Python: Needs config.json
4. RustFileProvider: Fetches from Rust model-cache via gRPC
5. Python: Loads model, returns RAM usage
6. Rust: Tracks loaded model in orchestrator
7. Inference: ml_client.generate_text(...) → streams tokens
```

---

## Current Status

### ✅ Production Ready
- MediaPipe (all 7 modules)
- BitNet (all platforms)
- gRPC Rust ↔ Python
- 7-database architecture
- WebRTC streaming
- Native messaging
- Hardware detection

### ⚙️ In Progress
- All 15 Transformers pipelines
- Experience learning system
- Tool results caching
- LiteRT integration
- Query optimization

### 📋 Planned
- Desktop agent SDK
- Voice assistant mode
- Vision-based automation
- Plugin system

---

## Technical Documentation

**Want the deep dive?**

- **[Rust/docs/mia_memory.md](Rust/docs/mia_memory.md)** - Complete cognitive architecture (1800 lines!)
- **[PythonML/README.md](PythonML/README.md)** - Python ML services
- **[Rust/README.md](Rust/README.md)** - Rust infrastructure
- **[README.md](README.md)** - Quick start guide

---

## What Makes MIA Special?

### 1. True Multi-modal (Not Bolted On)
- Vision, audio, text from day one
- Designed for streaming
- Real-time performance

### 2. Cognitive Architecture (Not Just a Database)
- 7 specialized memory systems
- Human brain-inspired design
- Meta-memory (knows what it knows)

### 3. Learning by Design (Not Static)
- Experience database tracks outcomes
- User feedback shapes behavior
- Pattern recognition improves over time

### 4. Performance First (Not GPU-Dependent)
- BitNet 1.58-bit (50 tok/s on CPU)
- Hardware-aware backend selection
- Streaming everything

### 5. Production-Ready (Not a Prototype)
- Real implementations (no stubs)
- Comprehensive tests
- Enterprise-grade code quality

---

## The Vision: Agents That See, Hear, and Learn

**Current AI**: "Tell me what you want"  
**MIA**: "I can see you're on a docs page, hear you asking a question, remember we discussed this yesterday, and I learned that you prefer concise answers with code examples"

**That's the difference.**

---

## Get Started

```bash
git clone https://github.com/yourusername/TabAgent
cd TabAgent/TabAgentServer

# Install Python ML dependencies
cd PythonML
pip install -r requirements.txt
./generate_protos.bat

# Build and run Rust server
cd ../Rust
cargo run --bin tabagent-server

# Server starts on:
# - HTTP API: http://localhost:3000
# - WebRTC: http://localhost:8002
# - Python ML: localhost:50051 (internal)
```

**Test it**:
```bash
# Test MediaPipe vision
cd PythonML
pytest tests/test_mediapipe.py -v

# Test face detection with your camera
python -c "
from mediapipe import FaceDetector
import cv2

detector = FaceDetector()
cap = cv2.VideoCapture(0)

while True:
    ret, frame = cap.read()
    rgb = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)
    faces = detector.detect_single(rgb)
    print(f'Detected {len(faces)} faces')
    
    if cv2.waitKey(1) & 0xFF == ord('q'):
        break

cap.release()
"
```

---

## Join the Vision

MIA is building the future of **agentic AI that truly understands the world**.

Not just text. Not just vision. **Everything.**

**Contributing**: See [README.md](README.md) for contribution guidelines.

**Questions**: Open an issue or start a discussion.

---

**MIA: Because AI should see, hear, remember, learn, and act—not just chat.**

