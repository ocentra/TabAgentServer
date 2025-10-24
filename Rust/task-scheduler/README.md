# Task Scheduler - Activity-Aware Background Processing

## Purpose

The `task-scheduler` crate provides an **intelligent background task execution system** inspired by human brain behavior. It automatically adjusts task processing based on user activity levels, ensuring that:

- 🧠 **During active use**: Only critical tasks run (like indexing for instant recall)
- 😴 **During idle time**: Normal background work happens (embeddings, entity extraction)
- 💤 **During sleep**: Aggressive batch processing catches up on everything

This is a **core foundation** for MIA's cognitive engine, built in Phase 1.5/2 so all future systems can plug into it.

---

## Architecture

```
User Activity → Activity Detector → Task Scheduler → Task Queue
                                           ↓
                                   Executor (tokio tasks)
```

### Activity Levels

| Level | User State | Task Behavior |
|-------|------------|---------------|
| **HighActivity** | Actively chatting, typing | Only URGENT tasks (e.g., index new message) |
| **LowActivity** | Idle for 5+ minutes | Normal processing (embeddings, NER) |
| **SleepMode** | Inactive for 30+ minutes | Batch processing (summaries, associations) |

### Task Priorities

| Priority | Description | When It Runs |
|----------|-------------|--------------|
| **Urgent** | Critical for instant recall | Always, even during HighActivity |
| **Normal** | Standard background work | LowActivity and SleepMode |
| **Low** | Nice-to-have enrichment | LowActivity and SleepMode |
| **Batch** | Can wait for downtime | SleepMode only |

---

## Usage

### Basic Example

```rust
use task_scheduler::{TaskScheduler, Task, TaskPriority, ActivityLevel};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let scheduler = TaskScheduler::new();
    
    // User starts chatting - switch to high activity
    scheduler.set_activity(ActivityLevel::HighActivity).await;
    
    // Queue tasks
    scheduler.submit(Task::IndexNode {
        node_id: "msg_123".to_string(),
        priority: TaskPriority::Urgent,  // Will run immediately
    }).await?;
    
    scheduler.submit(Task::GenerateEmbedding {
        node_id: "msg_123".to_string(),
        text: "Hello world".to_string(),
        priority: TaskPriority::Normal,  // Will queue until idle
    }).await?;
    
    // User goes idle - tasks start processing
    scheduler.set_activity(ActivityLevel::LowActivity).await;
    
    // Check queue status
    let stats = scheduler.queue_stats().await;
    println!("Pending tasks: {:?}", stats);
    
    Ok(())
}
```

### Integration with Storage

```rust
// In storage layer:
impl StorageManager {
    pub async fn insert_message_with_tasks(
        &self, 
        message: Message,
        scheduler: &TaskScheduler
    ) -> DbResult<()> {
        // 1. Store the message
        self.insert_node(&Node::Message(message.clone()))?;
        
        // 2. Queue URGENT index update (instant recall)
        scheduler.submit(Task::IndexNode {
            node_id: message.id.clone(),
            priority: TaskPriority::Urgent,
        }).await?;
        
        // 3. Queue NORMAL embedding generation (background)
        scheduler.submit(Task::GenerateEmbedding {
            node_id: message.id.clone(),
            text: message.text_content.clone(),
            priority: TaskPriority::Normal,
        }).await?;
        
        // 4. Queue LOW entity extraction (enrichment)
        scheduler.submit(Task::ExtractEntities {
            node_id: message.id,
            text: message.text_content,
            priority: TaskPriority::Low,
        }).await?;
        
        Ok(())
    }
}
```

### UI Integration

```python
# Python wrapper (via PyO3)
class MIA:
    def __init__(self):
        self.scheduler = TaskScheduler()
        self.activity_detector = ActivityDetector()
        
    def on_user_input(self, event):
        """Called on keypress, mouse click, etc."""
        new_level = self.activity_detector.record_activity()
        if new_level:
            self.scheduler.set_activity(new_level)
    
    def on_idle_timer(self):
        """Called every second"""
        new_level = self.activity_detector.update()
        if new_level:
            self.scheduler.set_activity(new_level)
    
    def on_window_blur(self):
        """Called when TabAgent loses focus"""
        self.scheduler.set_activity(ActivityLevel.SleepMode)
```

---

## Task Types (Currently Defined)

### Implemented in Later Phases

1. **GenerateEmbedding** - Create vector embeddings for semantic search
2. **ExtractEntities** - NER to identify people, places, concepts
3. **LinkEntities** - Create MENTIONS edges in knowledge graph
4. **GenerateSummary** - LLM-based conversation summarization
5. **CreateAssociativeLinks** - Find semantic similarities
6. **IndexNode** - Update structural/graph indexes
7. **UpdateVectorIndex** - Update HNSW vector index
8. **ProcessAttachment** - Handle document/PDF processing
9. **ChunkDocument** - Split large documents into chunks
10. **ExtractAttachmentText** - OCR/transcription for attachments
11. **GenerateAttachmentEmbeddings** - Create embeddings for document chunks
12. **RotateMemoryLayers** - Hot → warm → cold memory management
13. **BackupData** - Encrypted backups during sleep

**Note**: Task execution is currently stubbed. Actual implementations will be added as corresponding systems (embeddings, NER, etc.) are built in Phases 2-5.

---

## Testing

```bash
# Run all tests
cargo test -p task-scheduler

# Run with output
cargo test -p task-scheduler -- --nocapture
```

### Test Coverage

- ✅ Activity level detection and transitions
- ✅ Priority-based task queuing
- ✅ Task execution during different activity levels
- ✅ Queue statistics
- ✅ Manual activity level setting

---

## Design Decisions

### Why Activity-Aware?

Traditional task queues process at a constant rate. This causes problems:
- ❌ Background tasks compete with UI responsiveness
- ❌ Battery drain during active use
- ❌ Wasted idle time

Activity-aware scheduling solves this:
- ✅ UI stays responsive (only urgent tasks during chat)
- ✅ Efficient use of idle time
- ✅ No user-visible slowdown

### Why Tokio?

- **Async execution**: Tasks don't block each other
- **Efficient**: Minimal overhead for background work
- **Standard**: Well-tested, production-ready runtime

### Why Priority Levels?

Different tasks have different urgency:
- **Index updates**: Must be instant (user expects immediate recall)
- **Embeddings**: Can wait a few seconds (search works without them temporarily)
- **Summarization**: Can wait hours (deep enrichment)

---

## Future Enhancements (Post-Phase 2)

1. **Task Dependencies**: "Don't extract entities until embedding is done"
2. **Rate Limiting**: "Max 10 LLM calls per minute"
3. **Task Cancellation**: "User deleted the message, cancel pending tasks"
4. **Persistence**: "Resume pending tasks after restart"
5. **Metrics**: "Track task execution times, success rates"

---

## Integration Timeline

| Phase | Integration |
|-------|-------------|
| **Phase 1.5** | ✅ Task scheduler foundation (complete) |
| **Phase 2** | → Integrate with indexing layer |
| **Phase 3** | → Integrate with query engine |
| **Phase 4** | → Expose to Python via PyO3 |
| **Phase 5** | → Knowledge Weaver uses for all enrichment |

---

## Dependencies

- `tokio` - Async runtime
- `common` - Shared types (NodeId, etc.)
- `serde` - Task serialization (future: persistence)
- `thiserror` - Error handling

---

**Status**: ✅ Foundation complete and tested  
**Next**: Integrate with storage and indexing layers

