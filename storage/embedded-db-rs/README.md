# TabAgent Embedded Database

**Rust-based embedded multi-model database for MIA (My Intelligent Assistant).**

## Overview

A high-performance, embedded database designed for personal AI assistants. Combines document, graph, and vector models in a single, integrated system optimized for AI-first workloads.

### Key Features

- 🚀 **Fast**: O(log n) queries via sled + HNSW
- 🔒 **Safe**: Rust memory safety + ACID transactions
- 🧠 **AI-First**: Native support for embeddings, entities, knowledge graphs
- 📦 **Embedded**: No external database server required
- 🎯 **Multi-Modal**: Document + Graph + Vector in one system
- 🤖 **Activity-Aware**: Background processing adapts to user activity

## Architecture

```
┌─────────────────────────────────────────┐
│         Application Layer                │
│  (Python via PyO3 - Future Phase 4)     │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│         Storage Layer                    │
│  ┌────────────────────────────────────┐ │
│  │ StorageManager (CRUD operations)   │ │
│  │  • Nodes (12 types)                │ │
│  │  • Edges (relationships)           │ │
│  │  • Embeddings (vectors)            │ │
│  │  • Optional auto-indexing          │ │
│  └────────────────────────────────────┘ │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│         Indexing Layer                   │
│  ┌─────────────┬─────────────┬────────┐ │
│  │ Structural  │ Graph       │ Vector │ │
│  │ (B-tree)    │ (Adj Lists) │ (HNSW) │ │
│  │ O(log n)    │ O(1)        │O(log n)│ │
│  └─────────────┴─────────────┴────────┘ │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│      Task Scheduler (Background)         │
│  Activity-aware async task execution     │
│  • High Activity: Urgent only            │
│  • Low Activity: Normal processing       │
│  • Sleep Mode: Batch processing          │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│         Foundation Layer                 │
│  Common types, models, error handling    │
│  (Zero dependencies on other crates)     │
└──────────────────────────────────────────┘
```

## Project Structure

### Crates

| Crate | Purpose | Status | README |
|-------|---------|--------|--------|
| **common** | Shared types, models, errors | ✅ Stable | [README](./common/README.md) |
| **storage** | CRUD operations, indexing integration | ✅ Stable | [README](./storage/README.md) |
| **indexing** | Three-tier indexing (structural, graph, vector) | ✅ Stable | [README](./indexing/README.md) |
| **query** | Converged query pipeline (Phase 3) | ✅ Complete | [README](./query/README.md) |
| **weaver** | Knowledge Weaver (Phase 5) - Autonomous enrichment | ✅ **NEW** | [README](./weaver/README.md) |
| **ml-bridge** | Python ML inference bridge (PyO3) | ✅ **NEW** | [README](./ml-bridge/README.md) |
| **task-scheduler** | Activity-aware background processing | ✅ Foundation Ready | [README](./task-scheduler/README.md) |

### Progress Tracking

For detailed progress, blockers, and pending work, see each crate's TODO:
- [common/TODO.md](./common/TODO.md)
- [storage/TODO.md](./storage/TODO.md)
- [indexing/TODO.md](./indexing/TODO.md)
- [query/TODO.md](./query/TODO.md)
- [weaver/TODO.md](./weaver/TODO.md)
- [ml-bridge/TODO.md](./ml-bridge/TODO.md)
- [task-scheduler/TODO.md](./task-scheduler/TODO.md)

## Quick Start

### Installation

```bash
cd Server/storage/embedded-db-rs
cargo build --workspace --release
```

### Basic Usage

```rust
use storage::{StorageManager, Node, Chat};
use serde_json::json;

// Create database
let storage = StorageManager::new("my_database")?;

// Insert a chat
let chat = Node::Chat(Chat {
    id: "chat_001".to_string(),
    title: "Project Discussion".to_string(),
    topic: "Rust Database".to_string(),
    created_at: 1697500000000,
    updated_at: 1697500000000,
    message_ids: vec![],
    summary_ids: vec![],
    embedding_id: None,
    metadata: json!({}),
});

storage.insert_node(&chat)?;

// Retrieve
let retrieved = storage.get_node("chat_001")?;
```

### With Automatic Indexing

```rust
// Enable automatic indexing
let storage = StorageManager::with_indexing("my_database")?;

// Insert (indexes update automatically)
storage.insert_node(&chat)?;
storage.insert_edge(&edge)?;
storage.insert_embedding(&embedding)?;

// Query indexes
if let Some(idx) = storage.index_manager() {
    // Property-based query
    let messages = idx.get_nodes_by_property("chat_id", "chat_123")?;
    
    // Graph traversal
    let outgoing = idx.get_outgoing_edges("chat_123")?;
    
    // Semantic search
    let query_vector = vec![0.1; 384];
    let similar = idx.search_vectors(&query_vector, 10)?;
}
```

## Testing

```bash
# Run all tests
cargo test --workspace --release

# Run specific crate tests
cargo test -p storage --release
cargo test -p indexing --release

# Run with output
cargo test --workspace -- --nocapture

# Check code quality
cargo clippy --workspace --release
```

### Test Results

✅ **96 tests passing** across all crates
- common: 2 tests
- storage: 36 tests (18 unit + 18 integration)
- indexing: 22 tests (9 lib + 4 modules + 9 integration)
- query: 7 tests (5 lib + 2 doc)
- task-scheduler: 16 tests (9 lib + 4 modules + 2 doc + 1 integration)
- weaver: 10 tests (2 lib + 8 modules)
- ml-bridge: 3 tests (1 lib + 2 integration)

## Performance

| Operation | Complexity | Notes |
|-----------|------------|-------|
| Insert/Get/Delete | O(log n) | sled B-tree |
| Property Query | O(log n) | Structural index |
| Graph Neighbor | O(1) | Adjacency lists |
| Vector Search | O(log n) | HNSW ANN |
| Index Update | O(log n) | Automatic |

## Data Models

### Node Types (12 total)

**Communication:**
- Chat, Message, Summary

**Knowledge:**
- Entity (people, places, concepts)
- WebSearch, ScrapedPage, Bookmark

**Media:**
- Attachment, ImageMetadata, AudioTranscript

**System:**
- ModelInfo (AI models)

See [common/README.md](./common/README.md) for detailed model documentation.

## Development Roadmap

### ✅ Completed Phases

- **Phase 1**: Core storage layer (CRUD, transactions)
- **Phase 1.5**: MIA data models (12 node types)
- **Phase 2**: Three-tier indexing system

### 🔄 Current Phase

- **Phase 3**: Query Engine (converged query pipeline)

### 📋 Future Phases

- **Phase 4**: Python bindings (PyO3)
- **Phase 5**: Knowledge Weaver (autonomous enrichment)
- **Phase 6**: Production deployment (encryption, backup)

## Dependencies

### Core
- `sled` - Embedded database engine
- `serde` + `bincode` - Serialization
- `thiserror` - Error handling

### Indexing
- `hnsw_rs` - Vector similarity search

### Async
- `tokio` - Async runtime for task scheduler

## Design Principles

1. **Rust-First**: Everything that can be in Rust, is in Rust
2. **Zero Circular Dependencies**: Clean crate hierarchy
3. **Optional Features**: Core works without indexing/scheduling
4. **AI-Optimized**: Native support for embeddings, knowledge graphs
5. **Single-User Scale**: Optimized for personal AI assistant, not enterprise
6. **Privacy-First**: Local storage, future encryption support

## Contributing

1. Read the relevant crate's README to understand its purpose
2. Check the TODO.md for pending work and blockers
3. Follow [Rust-Architecture-Guidelines.md](./storage/Rust-Architecture-Guidelines.md)
4. Run tests before committing
5. Update TODO.md as you complete work

## License

[Your License Here]

---

**Built with ❤️ for MIA (My Intelligent Assistant)**
