# Database Foundation Plan: Multi-Tier Memory for MIA

## Current State (What Exists)

### ✅ Built and Working:
1. **`common/` crate**: Models for Chat, Message, Entity, Summary, Edge, Embedding
2. **`storage/` crate**: Basic StorageManager with ONE database (nodes/edges/embeddings trees)
3. **`indexing/` crate**: Structural, graph, and vector indexes (auto-maintained)
4. **`weaver/` crate**: Event-driven enrichment (entity extraction, embeddings generation)
5. **`task-scheduler/` crate**: Activity-aware background processing
6. **`query/` crate**: (exists, need to check implementation status)

### 🔴 Missing (What We Need):
1. **Multi-tier database structure** (active/recent/archive)
2. **Database coordinator** to manage multiple `StorageManager` instances
3. **Lifecycle management** (promotion/demotion logic)
4. **Integration** between storage → task-scheduler → weaver

---

## Phase 1: Multi-Tier Storage Foundation (Build NOW)

### Goal: Support 3-tier memory WITHOUT changing existing code

### Step 1: Create `MultiTierStorage` Coordinator

**New File**: `Server/tabagent-rs/storage/src/multi_tier.rs`

```rust
pub struct MultiTierStorage {
    // Active tier (hot data: 0-30 days)
    active: Arc<StorageManager>,
    
    // Recent tier (warm data: 30-90 days)
    recent: Arc<RwLock<Option<StorageManager>>>,  // Lazy load
    
    // Archives (cold data: 90+ days)
    archives: Arc<RwLock<HashMap<String, StorageManager>>>,  // Lazy load by quarter
    
    // Config
    config: TierConfig,
}

impl MultiTierStorage {
    /// Open with platform-specific paths
    pub fn new() -> DbResult<Self> {
        // Opens:
        // - %APPDATA%/TabAgent/db/conversations/active/
        // - %APPDATA%/TabAgent/db/embeddings/active/
        // - %APPDATA%/TabAgent/db/knowledge/active/
    }
    
    /// Query with automatic tier selection
    pub fn get_node(&self, id: &str, hint: TierHint) -> DbResult<Option<Node>> {
        match hint {
            TierHint::Active => self.active.get_node(id),
            TierHint::Any => {
                // Try active first, then recent, then archives
                if let Some(node) = self.active.get_node(id)? {
                    return Ok(Some(node));
                }
                
                // Lazy load recent if needed
                if let Some(recent) = self.get_or_load_recent()? {
                    if let Some(node) = recent.get_node(id)? {
                        return Ok(Some(node));
                    }
                }
                
                // Search archives if needed
                // ...
                
                Ok(None)
            }
        }
    }
}
```

### Step 2: Create Database Types (Not Temperature Yet!)

For **NOW**, create 3 database **TYPES** (not tiers):

```
%APPDATA%/TabAgent/db/
├── conversations/          ← SOURCE OF TRUTH
│   ├── nodes
│   ├── edges
│   └── embeddings
│
├── knowledge/              ← DERIVED (entities, relationships)
│   ├── nodes
│   ├── edges
│   └── embeddings
│
└── model-cache/            ← Already exists (models)
```

**Why only 3 for now?**
- Start simple: SOURCE (conversations) + DERIVED (knowledge) + MODELS (cache)
- Add temperature tiers (active/recent/archive) in Phase 2
- Get fault isolation working first

### Step 3: Update `StorageManager` for Named Databases

**File**: `Server/tabagent-rs/storage/src/lib.rs`

Add a `db_type` field:

```rust
pub struct StorageManager {
    db: sled::Db,
    nodes: sled::Tree,
    edges: sled::Tree,
    embeddings: sled::Tree,
    index_manager: Option<Arc<indexing::IndexManager>>,
    
    db_type: DatabaseType,  // NEW
}

pub enum DatabaseType {
    Conversations,  // SOURCE
    Knowledge,      // DERIVED
    ModelCache,     // SEPARATE
}

impl StorageManager {
    /// Open a named database of a specific type
    pub fn open_typed(db_type: DatabaseType) -> DbResult<Self> {
        let path = match db_type {
            DatabaseType::Conversations => {
                get_named_db_path("conversations")
            }
            DatabaseType::Knowledge => {
                get_named_db_path("knowledge")
            }
            DatabaseType::ModelCache => {
                get_named_db_path("model-cache")
            }
        };
        
        Self::new(path.to_str().unwrap())
            .map(|mut storage| {
                storage.db_type = db_type;
                storage
            })
    }
}
```

### Step 4: Create `DatabaseCoordinator` (High-Level API)

**New File**: `Server/tabagent-rs/storage/src/coordinator.rs`

```rust
/// High-level coordinator for all database operations
pub struct DatabaseCoordinator {
    conversations: Arc<StorageManager>,  // SOURCE
    knowledge: Arc<StorageManager>,      // DERIVED
    
    task_scheduler: Option<Arc<TaskScheduler>>,
    weaver: Option<Arc<Weaver>>,
}

impl DatabaseCoordinator {
    pub fn new() -> DbResult<Self> {
        Ok(Self {
            conversations: Arc::new(
                StorageManager::open_typed(DatabaseType::Conversations)?
            ),
            knowledge: Arc::new(
                StorageManager::open_typed(DatabaseType::Knowledge)?
            ),
            task_scheduler: None,
            weaver: None,
        })
    }
    
    /// Insert a message and queue background tasks
    pub async fn insert_message(&self, message: Message) -> DbResult<()> {
        // 1. Insert to conversations DB (SOURCE)
        self.conversations.insert_node(&Node::Message(message.clone()))?;
        
        // 2. Queue background tasks if scheduler is available
        if let Some(scheduler) = &self.task_scheduler {
            // URGENT: Index for instant recall
            scheduler.submit(Task::IndexNode {
                node_id: message.id.clone(),
                priority: TaskPriority::Urgent,
            }).await?;
            
            // NORMAL: Generate embedding (background)
            scheduler.submit(Task::GenerateEmbedding {
                node_id: message.id.clone(),
                text: message.text_content.clone(),
                priority: TaskPriority::Normal,
            }).await?;
            
            // LOW: Extract entities (enrichment)
            scheduler.submit(Task::ExtractEntities {
                node_id: message.id,
                text: message.text_content,
                priority: TaskPriority::Low,
            }).await?;
        }
        
        Ok(())
    }
    
    /// Query messages with automatic cross-DB resolution
    pub fn get_message(&self, id: &str) -> DbResult<Option<Message>> {
        // Try conversations DB
        if let Some(Node::Message(msg)) = self.conversations.get_node(id)? {
            return Ok(Some(msg));
        }
        
        Ok(None)
    }
    
    /// Get entities for a message (crosses databases!)
    pub fn get_message_entities(&self, message_id: &str) -> DbResult<Vec<Entity>> {
        // 1. Get edges from knowledge DB (MENTIONS relationships)
        let edges = if let Some(idx) = self.knowledge.index_manager() {
            idx.get_outgoing_edges(message_id)?
        } else {
            vec![]
        };
        
        // 2. Filter for MENTIONS edges
        let entity_ids: Vec<_> = edges.iter()
            .filter(|e| e.edge_type == "MENTIONS")
            .map(|e| e.to_node.as_str())
            .collect();
        
        // 3. Load entities from knowledge DB
        let mut entities = Vec::new();
        for entity_id in entity_ids {
            if let Some(Node::Entity(entity)) = self.knowledge.get_node(entity_id)? {
                entities.push(entity);
            }
        }
        
        Ok(entities)
    }
}
```

---

## Phase 2: Lifecycle Management (Build NEXT)

### Add Temperature Tiers to Each Database Type

```
%APPDATA%/TabAgent/db/
├── conversations/
│   ├── active/             ← 0-30 days (HOT)
│   ├── recent/             ← 30-90 days (WARM)
│   └── archive/
│       ├── 2024-Q4/        ← 90+ days (COLD)
│       └── 2024-Q3/
│
├── knowledge/
│   ├── active/             ← Recently mentioned
│   └── stable/             ← Well-established
│
└── model-cache/            ← (no tiers needed)
```

### Add Promotion/Demotion Logic

```rust
impl DatabaseCoordinator {
    /// Background task: Promote/demote data based on age and access patterns
    pub async fn manage_lifecycle(&self) -> DbResult<()> {
        // 1. Demote: conversations/active → conversations/recent (after 30 days)
        let old_active = self.conversations.active.query_by_age(30_days)?;
        self.conversations.recent.bulk_insert(old_active)?;
        self.conversations.active.bulk_delete(&old_active)?;
        
        // 2. Demote: conversations/recent → conversations/archive (after 90 days)
        let old_recent = self.conversations.recent.query_by_age(90_days)?;
        let quarter = get_quarter(old_recent.timestamp);
        self.conversations.archive[quarter].bulk_insert(old_recent)?;
        self.conversations.recent.bulk_delete(&old_recent)?;
        
        // 3. Promote: knowledge/active → knowledge/stable (after 10+ mentions)
        let stable_entities = self.knowledge.active.query_by_mention_count(10)?;
        self.knowledge.stable.bulk_insert(stable_entities)?;
        self.knowledge.active.bulk_delete(&stable_entities)?;
        
        Ok(())
    }
}
```

---

## Phase 3: Python Bindings (Build AFTER Phase 1/2)

### Expose `DatabaseCoordinator` to Python

```rust
// In db-bindings/src/lib.rs
#[pyclass]
pub struct PyDatabaseCoordinator {
    inner: DatabaseCoordinator,
}

#[pymethods]
impl PyDatabaseCoordinator {
    #[new]
    fn new() -> PyResult<Self> {
        let inner = DatabaseCoordinator::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(Self { inner })
    }
    
    fn insert_message(&mut self, message_dict: &PyDict) -> PyResult<()> {
        // Convert Python dict → Rust Message → insert
        // ...
    }
    
    fn get_message(&self, id: &str) -> PyResult<Option<PyObject>> {
        // ...
    }
}
```

---

## Implementation Order (What to Build First)

### Week 1: Database Types (3 DBs)
- [ ] Create `DatabaseType` enum
- [ ] Update `StorageManager::open_typed()`
- [ ] Test: Can open conversations/ and knowledge/ separately
- [ ] Test: Insert to conversations/, verify isolation from knowledge/

### Week 2: DatabaseCoordinator (Cross-DB Operations)
- [ ] Create `DatabaseCoordinator` struct
- [ ] Implement `insert_message()` with task queueing
- [ ] Implement `get_message_entities()` (cross-DB query)
- [ ] Test: Insert message → task-scheduler queues tasks
- [ ] Test: Query entities linked to message

### Week 3: Lifecycle Manager (Preparation)
- [ ] Add timestamp queries to `StorageManager`
- [ ] Add bulk operations (bulk_insert, bulk_delete)
- [ ] Test: Query messages older than 30 days
- [ ] Test: Bulk move 1000 messages between DBs

### Week 4: Temperature Tiers (Active/Recent/Archive)
- [ ] Extend `DatabaseCoordinator` with tier support
- [ ] Implement lazy loading for recent/archive
- [ ] Implement `manage_lifecycle()` background task
- [ ] Test: Automatic demotion after 30 days
- [ ] Test: Query spans active → recent → archive

### Week 5: Python Bindings
- [ ] Create `PyDatabaseCoordinator` class
- [ ] Expose insert/query methods to Python
- [ ] Test: Python can insert message → Rust handles it
- [ ] Test: Python can query cross-DB (message + entities)

---

## Testing Strategy

### Unit Tests (Per Crate)
- ✅ `storage/`: Already has tests for StorageManager
- 🔜 `storage/`: Add tests for MultiTierStorage
- 🔜 `storage/`: Add tests for DatabaseCoordinator

### Integration Tests (Cross-Crate)
```rust
#[tokio::test]
async fn test_full_message_lifecycle() {
    // 1. Create coordinator with task-scheduler
    let coordinator = DatabaseCoordinator::new()?;
    let scheduler = TaskScheduler::new();
    coordinator.set_scheduler(scheduler.clone());
    
    // 2. Insert message
    let message = Message { /* ... */ };
    coordinator.insert_message(message).await?;
    
    // 3. Verify tasks were queued
    let stats = scheduler.queue_stats().await;
    assert_eq!(stats.pending_urgent, 1);  // Index task
    assert_eq!(stats.pending_normal, 1);  // Embedding task
    assert_eq!(stats.pending_low, 1);     // Entity task
    
    // 4. Wait for tasks to complete
    scheduler.set_activity(ActivityLevel::LowActivity).await;
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    // 5. Verify enrichments were created
    let entities = coordinator.get_message_entities(&message.id)?;
    assert!(!entities.is_empty(), "Entities should be extracted");
    
    let embedding = coordinator.conversations.get_embedding_by_node(&message.id)?;
    assert!(embedding.is_some(), "Embedding should be generated");
}
```

---

## Success Criteria

### Phase 1 Complete When:
- ✅ Can open 3 separate databases (conversations, knowledge, model-cache)
- ✅ DatabaseCoordinator can insert message → queues tasks
- ✅ Cross-DB query works (get message + its entities)
- ✅ Tests pass for fault isolation (knowledge corrupted → conversations still work)

### Phase 2 Complete When:
- ✅ Active/recent/archive tiers work for conversations
- ✅ Lifecycle management demotes old data automatically
- ✅ Queries span multiple tiers transparently
- ✅ Performance: Active queries stay fast (<1ms) even with 1M+ archive messages

### Phase 3 Complete When:
- ✅ Python can call all coordinator methods
- ✅ `native_host.py` uses Rust coordinator (not direct storage)
- ✅ End-to-end test: Chrome → Python → Rust → multi-DB → back to Chrome

---

## What NOT to Build Yet

❌ **Full query engine** (Stage 0-4 pipeline) - Wait for Phase 3  
❌ **Meta-memory learning** - Wait for Phase 4  
❌ **Unified query API** (`mia.query(...)`) - Wait for Phase 5  
❌ **Summarization** - Weaver already has stubs, implement when needed  
❌ **Active/stable promotion logic for knowledge** - After Phase 2  

---

## Next Immediate Actions

1. **Create `storage/src/multi_tier.rs`** (stub for now)
2. **Add `DatabaseType` enum to `storage/src/lib.rs`**
3. **Create `storage/src/coordinator.rs`** with basic structure
4. **Update `storage/Cargo.toml`** to depend on `task-scheduler`
5. **Write integration test** (message insert → task queue → enrichment)

Should I start building? 🚀

