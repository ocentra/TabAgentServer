# Weaver - Knowledge Weaver (Autonomous Enrichment Engine)

**Phase 5 Complete** | Autonomous knowledge graph enrichment through event-driven processing.

## Purpose

The Weaver is an asynchronous, event-driven engine that automatically enriches the knowledge graph by:
- **Generating embeddings** for semantic search
- **Extracting entities** and creating relationships  
- **Creating associative links** between similar content
- **Summarizing conversations** when threshold reached

## Architecture

```
Storage Layer emits events → Weaver Event Queue → Worker Pool
                                                      ├─▶ semantic_indexer
                                                      ├─▶ entity_linker
                                                      ├─▶ associative_linker
                                                      └─▶ summarizer
                                                            │
                                                            ▼ via MlBridge trait
                                                        Python ML (PyO3)
```

## Key Components

### 1. Event System (`events.rs`)
```rust
pub enum WeaverEvent {
    NodeCreated { node_id, node_type },
    NodeUpdated { node_id, node_type },
    ChatUpdated { chat_id, messages_since_summary },
    BatchMessagesAdded { chat_id, message_ids },
    EdgeCreated { edge_id, edge_type },
}
```

### 2. MlBridge Trait (`ml_bridge.rs`)
```rust
#[async_trait]
pub trait MlBridge: Send + Sync {
    async fn generate_embedding(&self, text: &str) -> DbResult<Vec<f32>>;
    async fn extract_entities(&self, text: &str) -> DbResult<Vec<Entity>>;
    async fn summarize(&self, messages: &[String]) -> DbResult<String>;
    async fn health_check(&self) -> DbResult<bool>;
}
```

### 3. Enrichment Modules (`modules/`)
- **semantic_indexer**: Generates vector embeddings for nodes
- **entity_linker**: Extracts named entities, creates MENTIONS edges
- **associative_linker**: Finds similar nodes, creates similarity edges
- **summarizer**: Generates conversation summaries

### 4. Core Engine (`lib.rs`)
- Tokio async runtime
- MPSC unbounded channel for events
- Dispatcher spawns tasks per event
- Automatic concurrency (joins semantic + entity + associative tasks)

## Usage

```rust
use weaver::{Weaver, WeaverContext, WeaverEvent};
use std::sync::Arc;

// Create context
let context = WeaverContext::new(
    Arc::new(storage),
    Arc::new(indexing),
    Arc::new(ml_bridge),  // PyMlBridge or MockMlBridge
);

// Start weaver
let weaver = Weaver::new(context).await?;

// Submit events (non-blocking)
weaver.submit_event(WeaverEvent::NodeCreated {
    node_id: "msg_123".to_string(),
    node_type: "Message".to_string(),
}).await?;

// Shutdown gracefully
weaver.shutdown().await?;
```

## Event Processing Flow

1. **Storage** inserts a Message node
2. **Storage** emits `WeaverEvent::NodeCreated`
3. **Weaver** dispatcher receives event
4. **Weaver** spawns 3 concurrent tasks:
   - **semantic_indexer**: Calls `ml_bridge.generate_embedding()` → stores Embedding
   - **entity_linker**: Calls `ml_bridge.extract_entities()` → creates Entity nodes + MENTIONS edges
   - **associative_linker**: Searches vector index → creates similarity edges
5. All enrichments complete asynchronously

## Integration with TaskScheduler

The Weaver respects activity levels:
- **HighActivity**: Processes urgent events only
- **LowActivity**: Full processing
- **SleepMode**: Batch processing

(Integration pending)

## Testing

```bash
cargo test -p weaver
```

✅ 10 tests passing:
- 2 core tests (initialization, event submission)
- 8 module tests (entity extraction, similarity linking, etc.)

## Performance

- Event submission: O(1) - non-blocking queue push
- Processing: Async, concurrent per event
- ML calls: Offloaded to Python via PyO3
- Typical enrichment time: ~100-300ms per message

## Dependencies

- `common`: Shared types
- `storage`: Database operations
- `indexing`: Vector search
- `query`: Graph traversal
- `task-scheduler`: Activity awareness (planned)
- `tokio`: Async runtime
- `async-trait`: Async trait support

## Next Steps

See [TODO.md](./TODO.md) for pending work including:
- Batch processing optimization
- Integration with task-scheduler
- Deduplication logic for entities
- Performance benchmarking

## References

- ML Bridge: [../ml-bridge/README.md](../ml-bridge/README.md)
- Architecture: [../ARCHITECTURE.md](../ARCHITECTURE.md)
- Python Integration: [../PYTHON_INTEGRATION.md](../PYTHON_INTEGRATION.md)

