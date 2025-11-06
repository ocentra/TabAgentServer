# Orchestration Flow: Weaver + TaskScheduler + Embedding

## The Complete Flow (Message to Embedding)

```
┌──────────────────────────────────────────────────────────────────────┐
│ 1. MESSAGE ARRIVES                                                   │
└──────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌──────────────────────────────────────────────────────────────────────┐
│ 2. STORAGE (Database Owner)                                          │
│    - Saves message to conversations.mdbx                             │
│    - Emits WeaverEvent::NodeCreated { node_id, node_type }          │
└──────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌──────────────────────────────────────────────────────────────────────┐
│ 3. WEAVER (Decides WHAT to enrich)                                   │
│    - Receives event in dispatcher                                    │
│    - Spawns concurrent enrichment tasks:                             │
│      • semantic_indexer::on_node_created()                           │
│      • entity_linker::on_node_created()                              │
│      • associative_linker::on_node_created()                         │
│                                                                       │
│    ┌─────────────────────────────────────────────────────┐          │
│    │ semantic_indexer (enrichment module)                 │          │
│    │  1. Fetch message text from storage                 │          │
│    │  2. Create TWO embedding tasks:                      │          │
│    │     a) Fast (0.6B) - URGENT priority                │          │
│    │     b) Accurate (8B) - NORMAL priority              │          │
│    │  3. Submit BOTH to TaskScheduler                    │          │
│    └─────────────────────────────────────────────────────┘          │
└──────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌──────────────────────────────────────────────────────────────────────┐
│ 4. TASK-SCHEDULER (Decides WHEN to run)                              │
│    - Receives tasks:                                                  │
│      Task::GenerateEmbedding { model: Fast, priority: Urgent }      │
│      Task::GenerateEmbedding { model: Accurate, priority: Normal }  │
│                                                                       │
│    - Checks current activity level:                                  │
│      • HighActivity: Only Urgent tasks → Fast embedding runs NOW    │
│      • LowActivity: All tasks → Both embeddings run                 │
│      • SleepMode: Batch mode → All embeddings run                   │
│                                                                       │
│    - Queues tasks appropriately                                      │
└──────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌──────────────────────────────────────────────────────────────────────┐
│ 5A. IMMEDIATE: Fast Embedding (Urgent)                               │
│     - User is typing → HighActivity                                  │
│     - TaskScheduler executes Task::GenerateEmbedding(Fast)          │
│     - Task calls: embedding_service.embed_fast(...)                 │
│     - Takes storage's DB env + dbi pointers                          │
│     - Generates 384D vector (~50ms)                                  │
│     - Stores to embeddings.mdbx/0.6b_vectors                        │
│     - ✅ Fast search now available!                                  │
└──────────────────────────────────────────────────────────────────────┘
                              │
                              │ User stops typing (5 min)
                              ▼
┌──────────────────────────────────────────────────────────────────────┐
│ 5B. DELAYED: Accurate Embedding (Normal priority)                    │
│     - User idle → LowActivity                                        │
│     - TaskScheduler now executes Task::GenerateEmbedding(Accurate)  │
│     - Task calls: embedding_service.embed_accurate(...)             │
│     - Takes storage's DB env + dbi pointers                          │
│     - Generates 1536D vector (~2s)                                   │
│     - Stores to embeddings.mdbx/8b_vectors                          │
│     - ✅ Accurate reranking now available!                           │
└──────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌──────────────────────────────────────────────────────────────────────┐
│ 6. INDEXING (Builds search indexes)                                  │
│    - Receives MDBX_env + MDBX_dbi from storage                       │
│    - Reads vectors from embeddings.mdbx/0.6b_vectors                │
│    - Builds HNSW graph for ANN search                                │
│    - Stores graph to embeddings.mdbx/hnsw_graph                     │
│    - ✅ Vector search fully operational!                             │
└──────────────────────────────────────────────────────────────────────┘
```

---

## Who Does What?

### **Storage** (Database Owner)
- **Owns**: All MDBX databases (conversations, knowledge, embeddings, etc.)
- **Creates**: Database environments, tables (DBIs)
- **Emits**: Events when data changes
- **Provides**: `MDBX_env` and `MDBX_dbi` pointers to other services

### **Weaver** (Orchestration - WHAT to enrich)
- **Listens**: For storage events via event queue
- **Decides**: What enrichment is needed (embeddings, entities, summaries)
- **Creates**: Tasks for each enrichment operation
- **Submits**: Tasks to TaskScheduler with appropriate priority
- **Does NOT**: Own databases, execute tasks directly, or wait for completion

### **TaskScheduler** (Orchestration - WHEN to run)
- **Receives**: Tasks from Weaver (and potentially other sources)
- **Monitors**: User activity level (HighActivity/LowActivity/SleepMode)
- **Decides**: When to execute tasks based on priority + activity
- **Executes**: Tasks at appropriate times
- **Does NOT**: Know about embeddings/entities/etc - just executes generic tasks

### **Embedding Service** (Execution)
- **Receives**: Database env+dbi pointers from TaskScheduler (which got them from Weaver, which got them from Storage)
- **Generates**: Vectors using ML models (0.6B fast, 8B accurate)
- **Stores**: Vectors directly to storage's embeddings database
- **Does NOT**: Own databases, create tables, or decide when to run

### **Indexing Service** (Index Building)
- **Receives**: Database env+dbi pointers from storage
- **Reads**: Vectors from storage's embeddings database
- **Builds**: HNSW graphs, B-tree indexes, etc.
- **Stores**: Indexes directly to storage's databases
- **Does NOT**: Own databases, create tables, or generate embeddings

---

## Activity-Aware Execution Timeline

```
Time    User State       Activity Level    Tasks Executed
────────────────────────────────────────────────────────────────
00:00   Typing message   HighActivity      Fast embedding (Urgent)
00:01   Still typing     HighActivity      (Accurate embedding queued)
00:02   Stops typing     HighActivity      (Still queued)
...
05:00   Idle 5 min       → LowActivity     ✅ Accurate embedding runs!
05:02   Idle             LowActivity       Entity extraction runs
05:05   Idle             LowActivity       Associative linking runs
...
30:00   Idle 30 min      → SleepMode       Batch: Summaries, cleanup, etc.
```

---

## Code Integration Example

### 1. Weaver's semantic_indexer module
```rust
// weaver/src/modules/semantic_indexer.rs
pub async fn on_node_created(
    context: &WeaverContext,
    node_id: &str,
    node_type: &str,
) -> WeaverResult<()> {
    // 1. Fetch text from storage
    let text = context.conversations_db.get_node_text(node_id)?;
    
    // 2. Get storage's database pointers
    let embeddings_env = context.conversations_db.get_embeddings_env();
    let fast_dbi = context.conversations_db.get_dbi("0.6b_vectors");
    let accurate_dbi = context.conversations_db.get_dbi("8b_vectors");
    
    // 3. Create tasks for TaskScheduler
    let fast_task = Task::GenerateEmbedding {
        node_id: node_id.to_string(),
        text: text.clone(),
        model: EmbeddingModel::Fast06B,
        env: embeddings_env,
        dbi: fast_dbi,
        priority: TaskPriority::Urgent,  // IMMEDIATE
    };
    
    let accurate_task = Task::GenerateEmbedding {
        node_id: node_id.to_string(),
        text: text.clone(),
        model: EmbeddingModel::Accurate8B,
        env: embeddings_env,
        dbi: accurate_dbi,
        priority: TaskPriority::Normal,  // BACKGROUND
    };
    
    // 4. Submit to TaskScheduler (non-blocking)
    context.task_scheduler.submit(fast_task).await?;
    context.task_scheduler.submit(accurate_task).await?;
    
    Ok(())
}
```

### 2. TaskScheduler's Task execution
```rust
// task-scheduler/src/tasks.rs
impl Task {
    pub async fn execute(&self) -> TaskResult<()> {
        match self {
            Task::GenerateEmbedding { node_id, text, model, env, dbi, .. } => {
                match model {
                    EmbeddingModel::Fast06B => {
                        println!("⚡ Generating fast embedding for {}", node_id);
                        EMBEDDING_SERVICE.embed_fast(*env, *dbi, text, node_id).await?;
                    }
                    EmbeddingModel::Accurate8B => {
                        println!("🎯 Generating accurate embedding for {}", node_id);
                        EMBEDDING_SERVICE.embed_accurate(*env, *dbi, text, node_id).await?;
                    }
                }
                Ok(())
            }
            // ... other task types
        }
    }
}
```

### 3. Embedding Service execution
```rust
// embedding/src/lib.rs
impl EmbeddingService {
    pub async fn embed_fast(
        &self,
        db_env: *mut MDBX_env,     // From storage!
        vectors_dbi: MDBX_dbi,      // From storage!
        text: &str,
        id: &str,
    ) -> DbResult<Vec<f32>> {
        // 1. Generate embedding with 0.6B model
        let embedding = self.model_0_6b.encode(text)?;
        
        // 2. Store to storage's database
        self.store_embedding(db_env, vectors_dbi, id, &embedding)?;
        
        Ok(embedding)
    }
}
```

---

## Key Design Principles

1. **Separation of Concerns**
   - Weaver: WHAT to enrich (orchestration logic)
   - TaskScheduler: WHEN to run (activity-aware execution)
   - EmbeddingService: HOW to generate vectors (ML execution)
   - Storage: WHERE to store data (database ownership)

2. **Database Ownership**
   - **Storage** owns ALL databases
   - **Embedding** receives pointers, never creates DBs
   - **Indexing** receives pointers, never creates DBs
   - **Weaver** orchestrates but doesn't touch storage directly

3. **Non-Blocking Orchestration**
   - Weaver submits tasks and returns immediately
   - TaskScheduler queues and executes asynchronously
   - No service waits for another service to complete

4. **Activity-Aware Priorities**
   - Fast embeddings: URGENT (always run, even during HighActivity)
   - Accurate embeddings: NORMAL (wait for LowActivity)
   - Summaries: BATCH (wait for SleepMode)

---

## Current Status

✅ **Implemented:**
- Weaver event system
- TaskScheduler with activity levels
- Embedding service stub

⏳ **TODO:**
- Integrate Weaver with TaskScheduler (add `task_scheduler` field to `WeaverContext`)
- Update enrichment modules to submit tasks instead of executing directly
- Add `EmbeddingModel` enum to distinguish Fast/Accurate
- Update `Task` enum to include database pointers
- Wire up actual ML models in EmbeddingService

---

**Next Step**: Should I integrate Weaver with TaskScheduler?

