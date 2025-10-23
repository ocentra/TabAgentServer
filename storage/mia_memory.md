# MIA Memory Architecture: Cognitive Database Design

## Table of Contents
1. [Related Documents](#related-documents)
2. [The Problem](#the-problem)
3. [The Journey](#the-journey)
4. [Core Principles](#core-principles)
5. [Memory Systems](#memory-systems)
6. [Knowledge vs Experience](#knowledge-vs-experience)
7. [Query Interface](#query-interface)
8. [Implementation Status](#implementation-status)
9. [Design Rationale](#design-rationale)

---

## Related Documents

This document is the **cognitive architecture overview**. For detailed implementation specifications, see:

### 📚 Complete Architecture Suite

**Genesis & Planning:**
- **`RustEmbeddedDatabaseplan.md`** - The original study plan that started everything
  - *When to read:* Understanding WHY we chose Rust + embedded DB approach
  - *What it covers:* ArangoDB study, Qdrant/petgraph research, IndexedDB analysis
  - *Key value:* Design decisions rationale, trade-offs analyzed

- **`MasterPlan.md`** - The browser extension's original database design
  - *When to read:* Understanding the IndexedDB foundation we're evolving from
  - *What it covers:* Browser-first architecture, converged query model, single-DB approach
  - *Key insight:* Why this works for browser (10K messages) but needs evolution for server (1M+ messages)

**Core Implementation Specs (Crate-Level Detail):**

1. **`StorageLayer.md`** - Specification for `storage/` crate
   - *When to read:* Implementing conversations/, embeddings/, or any database CRUD
   - *What it covers:* 
     - Hybrid Schema Model (typed core + flexible metadata)
     - `sled` on-disk layout (key formats, serialization)
     - `StorageManager` API (get/insert/delete operations)
     - Transactional guarantees
   - *Status:* ✅ IMPLEMENTED (Phase 1 complete)
   - *Referenced in:* [Implementation Status](#implementation-status), [Conversations Database](#1-conversations-database-source---critical)

2. **`IndexingLayer.md`** - Specification for `indexing/` crate
   - *When to read:* Implementing HNSW vector index or structural indexes
   - *What it covers:*
     - Three index types: Structural (B-tree), Graph (adjacency), Vector (HNSW)
     - Index update/query API
     - Transactional index maintenance
     - Performance targets (O(log n) lookups)
   - *Status:* ✅ IMPLEMENTED (integrated with storage)
   - *Referenced in:* [Embeddings](#2-embeddings-derived---regeneratable), [Implementation Status](#implementation-status)

3. **`QueryEngine.md`** - Specification for `query/` crate
   - *When to read:* Implementing the unified `mia.query()` interface
   - *What it covers:*
     - Converged Query Model (3-stage pipeline)
     - `ConvergedQuery` struct and execution
     - Stage 1: Structural filter (candidate set)
     - Stage 2: Semantic re-ranking (HNSW search)
     - Stage 3: Result ranking and confidence scoring
   - *Status:* ⚠️ EXISTS (needs audit for multi-DB support)
   - *Referenced in:* [Query Interface](#query-interface), [Query Execution Pipeline](#query-execution-pipeline-multi-stage)

4. **`KnowledgeWeaver.md`** - Specification for `weaver/` crate
   - *When to read:* Understanding autonomous background enrichment
   - *What it covers:*
     - Event-driven architecture (tokio + MPSC)
     - Four modules: Semantic Indexer, Entity Linker, Associative Linker, Summarizer
     - ML integration via `MLBridge` trait
     - Asynchronous, non-blocking enrichment
   - *Status:* ✅ IMPLEMENTED (4 modules, 10 tests passing)
   - *Referenced in:* [Knowledge Graph](#3-knowledge-graph-derived---regeneratable), [Integration with Existing Systems](#phase-4-integration-with-existing-systems)

5. **`APIBindings.md`** - Specification for Python bindings (`db-bindings/`)
   - *When to read:* Exposing new Rust functionality to Python
   - *What it covers:*
     - Two-layer API: Stateless Rust core + Stateful Python facade
     - PyO3 bindings (`PyEmbeddedDB` class)
     - Python wrapper classes (Node, Edge, Chat with lazy loading)
     - Error handling (Rust → Python exceptions)
   - *Status:* ✅ IMPLEMENTED (Python can import and use Rust DB)
   - *Referenced in:* [Python API](#python-api-same-interface), [Implementation Status](#implementation-status)

**Strategic Planning:**

- **`DATABASE_FOUNDATION_PLAN.md`** - Phase 1 implementation roadmap
  - *When to read:* Starting Phase 1 multi-database work NOW
  - *What it covers:* Concrete tasks, success criteria, week-by-week breakdown
  - *Status:* 🔜 CURRENT FOCUS
  - *Referenced in:* [Next Steps](#next-steps-immediate), [Implementation Status](#-current-goal-phase-1)

**Document Hierarchy:**
```
mia_memory.md (THIS FILE)          ← Cognitive architecture (WHY + WHAT)
    ↓ references
    ├── MasterPlan.md              ← Browser extension genesis (WHERE WE CAME FROM)
    ├── RustEmbeddedDatabaseplan.md ← Study & research phase (DECISIONS MADE)
    │
    ├── StorageLayer.md            ← storage/ crate (HOW: CRUD)
    ├── IndexingLayer.md           ← indexing/ crate (HOW: Search)
    ├── QueryEngine.md             ← query/ crate (HOW: Unified query)
    ├── KnowledgeWeaver.md         ← weaver/ crate (HOW: Enrichment)
    ├── APIBindings.md             ← db-bindings/ crate (HOW: Python)
    │
    └── DATABASE_FOUNDATION_PLAN.md ← Phase 1 tasks (WHAT TO BUILD NOW)
```

**Reading Path for Different Goals:**

- **"I want to understand MIA's memory architecture"**  
  → Start here (`mia_memory.md`)

- **"I want to implement a new database type"**  
  → `mia_memory.md` (architecture) → `StorageLayer.md` (implementation details)

- **"I want to add a new query feature"**  
  → `mia_memory.md` (query interface vision) → `QueryEngine.md` (pipeline details)

- **"I want to understand the enrichment flow"**  
  → `mia_memory.md` (weaver integration) → `KnowledgeWeaver.md` (module specs)

- **"I want to expose new functionality to Python"**  
  → `APIBindings.md` (PyO3 patterns) → `mia_memory.md` (Python API examples)

- **"I'm starting Phase 1 multi-DB work"**  
  → `DATABASE_FOUNDATION_PLAN.md` (tasks) → `mia_memory.md` (architecture constraints) → `StorageLayer.md` (implementation)

---

## The Problem

### Initial Question (October 23, 2025)
**User**: "Chat have separate, models have separate knowledge graph would have separate embedding etc will have it own db or is all in one thing?"

**Context**: The user was concerned that as the database grows larger, search performance degrades. Should we split:
- Chats → one database?
- Embeddings → separate database?
- Knowledge graph → separate database?

### Initial Wrong Answer
I initially suggested ONE database with multiple trees (like the MasterPlan.md from browser extension):
```
main/
├── nodes         (ALL entities)
├── edges         (ALL relationships)  
├── embeddings    (ALL vectors)
└── HNSW_index    (one big vector index)
```

**Rationale**: Converged Query Model needs all data together for Stage 1 (structural filter) + Stage 2 (semantic search).

### The Breakthrough Insight
**User**: "ya i knw when i made this plan vs now i am having doubts, like say as bigger the db slower the search no? think hard for this please"

**KEY REALIZATION**: The MasterPlan was written for the browser (IndexedDB with 1K-10K messages). The server is different:
- Persistent, accumulating data (years of history)
- Could be 100K-1M+ messages over time
- Vector index grows with EVERY message → search slows down!

---

## The Journey

### Phase 1: Hot/Cold Separation Discovery

**User's Key Insight**: "why not put recent | mid | longterm ... only retrieves as needed same for others no?"

**The Solution**: Don't separate by ENTITY TYPE (Chat vs Message vs Entity).  
**Separate by ACCESS PATTERN** (Hot vs Cold data)!

**Real-world usage patterns**:
- **90% of queries**: Recent data (last 30-90 days)
  - "Summarize today's chat"
  - "Find similar to this recent message"
  - "What did we discuss this week?"
- **10% of queries**: Full history (rare, can be slower)
  - "Find all mentions of topic X ever"
  - "Historical analysis"

### Phase 2: Cognitive Architecture Revelation

**User**: "no lazy work please, we think we do as much future proof right now cause as this grows it become major debt to maintain and refactor test everything."

Then the **critical explanation**:

> "this server is like human brain, there is multiple level, here, to think. atm its not implemented but soon will be actual agentic stuffs.
>
> when i say want to do task → have i done this before → i need to search my memory → now here we have semantic → this then tells me yes no → yes when → which chat → is this the chat i am in → how far should i deep search? does this question need this far or is it just relevant for this current chat → i need to think → i need memory retrieval.
>
> everything is about how well can we remember and how fast and how relevant memory gets to me quickly, my human brain is not fast it cant remember a damn single page forget pages of context of million token, yet i have smart system, i have meta, i decide when to look or think harder"

**The Epiphany**: This isn't a database problem - it's a **COGNITIVE ARCHITECTURE** problem!

MIA needs:
1. **Working Memory** - current context, super fast
2. **Episodic Memory** - autobiographical events ("when did I discuss X?")
3. **Semantic Memory** - persistent knowledge ("What IS X?")
4. **Procedural Memory** - system experience ("what query strategy worked?")
5. **Meta-Memory** - knowing WHAT you know and WHERE to look

### Phase 3: Atomic Operations & Fault Isolation

**User**: "everything should be atomic, collapse of one then does not mean going to coma... yes this means proper management"

**The Key Principle**: Source vs Derived

**SOURCE OF TRUTH** = Cannot lose (critical user data)  
**DERIVED DATA** = Regeneratable (can rebuild from source)  
**INDEXES** = Rebuildable (can reconstruct)

**Example**:
- ❌ **Embeddings corrupted?** → Re-run task-scheduler on conversations/
- ❌ **Knowledge corrupted?** → Re-run weaver on conversations/
- 🔴 **Conversations corrupted?** → DISASTER (but backups exist!)

**No cascading failures!**

### Phase 4: Integration with Existing Systems

**User shared**: `@weaver/` and `@task-scheduler/` folders

**Discovery**: The user has ALREADY built:
1. **`task-scheduler/`** - Activity-aware background processing
   - Queues tasks by priority (Urgent/Normal/Low/Batch)
   - Adjusts based on user activity (HighActivity/LowActivity/SleepMode)
2. **`weaver/`** - Event-driven enrichment engine
   - Entity extraction via ML
   - Relationship creation
   - Semantic indexing

**The Integration Point**:
```
Message inserted → task-scheduler → weaver → enrichment
```

This means the database architecture must:
- Support asynchronous enrichment
- Allow background tasks to write to separate DBs
- Enable recovery if enrichment fails

### Phase 5: The Unified Query Interface

**User**: "let results = conversations_active.search(today, chat ref current, look in knowledge graph yes, search level 2)?"

**The Vision**: A single query API that controls EVERYTHING:
- **WHAT** to search for (semantic query)
- **TIME** scope (today, last week, all time)
- **CONTEXT** scope (current chat, all chats, related chats)
- **DEPTH** (1-hop, 2-hop, deep graph traversal)
- **TEMPERATURE** (hot/warm/cold - which tier DBs to search)

```rust
let results = mia.query(Query {
    semantic: "Rust database design",
    time_scope: TimeScope::Today,
    context: Context::CurrentChat(chat_ref),
    use_knowledge_graph: true,
    search_depth: SearchDepth::Level(2),
    temperature: Temperature::Hot,
    limit: 10,
})?;
```

**This mirrors human cognition**: "Let me think about today's discussion in this chat, following 2-hop relationships, using recent memory only."

---

## Core Principles

### 1. Source vs Derived (CRITICAL!)

**SOURCE OF TRUTH**:
- User's conversations (chats, messages)
- Cannot lose this data
- ACID guarantees, backups on every write
- Lives in: `conversations/` database

**DERIVED DATA**:
- Embeddings (generated from messages)
- Entities (extracted from messages)
- Relationships (inferred from content)
- Summaries (generated from conversations)
- Can regenerate from source if corrupted
- Lives in: `embeddings/`, `knowledge/`, `summaries/` databases

**INDEXES**:
- HNSW vector index
- Structural indexes (chat_id, timestamp)
- Graph indexes (from_node, to_node)
- Can rebuild from data if corrupted

### 2. Fault Isolation (NO CASCADING FAILURES!)

Each database type is independent:
```
conversations/  ← SOURCE (critical)
embeddings/     ← DERIVED (regeneratable)
knowledge/      ← DERIVED (regeneratable)
summaries/      ← DERIVED (regeneratable)
meta/           ← INDEXES (rebuildable)
```

**If `embeddings/` corrupts:**
```python
def recover_embeddings():
    embeddings_db.clear()
    for msg in conversations_db.all_messages():
        task_scheduler.queue(EmbedMessage(msg.id))
```

**Result**: User data is safe, system recovers automatically!

### 3. Temperature Tiers (PERFORMANCE!)

Each database type has HOT/WARM/COLD tiers:

```
conversations/
├── active/     ← 0-30 days (10K messages, <1ms search)
├── recent/     ← 30-90 days (50K messages, <10ms search)
└── archive/    ← 90+ days (1M+ messages, 100ms search, acceptable because rare)
```

**Why?**
- **Performance**: Small active DB = fast queries
- **Scalability**: Archive doesn't slow down active queries
- **Cost**: Hot data in RAM, cold data on disk
- **Access patterns**: 90% queries hit active/, 10% deep search

### 4. Atomic Operations (NO PARTIAL STATES!)

Each database operation is atomic within its scope:
```rust
// ATOMIC: Chat + its messages in ONE transaction
let tx = conversations_db.transaction();
tx.insert_chat(chat)?;
tx.insert_messages(messages)?;
tx.commit()?;  // All or nothing!
```

**Cross-DB operations are NOT atomic** (by design):
```rust
// Step 1: Insert message to conversations (ATOMIC)
conversations_db.insert_message(msg)?;  // ← If this fails, nothing happens

// Step 2: Queue enrichment tasks (ASYNC, can fail independently)
task_scheduler.queue(EmbedMessage(msg.id)).await?;  // ← Can retry if fails
task_scheduler.queue(ExtractEntities(msg.id)).await?;  // ← Can retry if fails
```

**Why not atomic across DBs?**
- Enrichment can fail/retry without affecting source data
- User sees message immediately, enrichment happens in background
- Fault isolation: embedding failure doesn't corrupt conversation

### 5. Lazy Loading (MINIMAL RAM!)

```rust
struct DatabaseManager {
    active: HashMap<String, ConversationsDB>,     // Always in RAM
    recent: Option<ConversationsDB>,              // Lazy load on first access
    archives: HashMap<String, ConversationsDB>,   // Load on demand
}
```

**Why?**
- Don't load all 1M messages into RAM!
- Most queries hit active/ only
- Archive DBs loaded only when explicitly queried

---

## Memory Systems

### Physical Database Structure

```
%APPDATA%\TabAgent\db\
│
├── conversations/              ← SOURCE OF TRUTH (critical!)
│   ├── active/                 (0-30 days, 10K messages)
│   │   ├── chats
│   │   └── messages
│   ├── recent/                 (30-90 days, 50K messages)
│   │   ├── chats
│   │   └── messages
│   └── archive/                (90+ days, 1M+ messages)
│       ├── 2024-Q4/
│       └── 2024-Q3/
│
├── embeddings/                 ← DERIVED (regeneratable)
│   ├── active/                 (vectors for active conversations)
│   │   ├── vectors
│   │   └── HNSW_index          (10K vectors → <1ms search)
│   ├── recent/                 (vectors for recent conversations)
│   │   ├── vectors
│   │   └── HNSW_index          (50K vectors → <10ms search)
│   └── archive/                (old vectors, rarely searched)
│       └── 2024-Q4/
│           └── HNSW_index      (500K vectors → 100ms, acceptable)
│
├── knowledge/                  ← DERIVED (regeneratable)
│   ├── active/                 (recently mentioned entities)
│   │   ├── entities
│   │   └── edges               (MENTIONS, RELATED_TO)
│   ├── stable/                 (well-established concepts)
│   │   ├── entities
│   │   └── edges               (strong relationships)
│   └── inferred/               (weak signals, experimental)
│       ├── entities
│       └── edges               (low-confidence relationships)
│
├── summaries/                  ← DERIVED (regeneratable)
│   ├── session/                (current session, in-memory)
│   ├── daily/                  (last 30 days)
│   ├── weekly/                 (last 6 months)
│   └── monthly/                (all time)
│
├── meta/                       ← INDEXES (rebuildable)
│   ├── query_index/            (which queries hit which DBs)
│   ├── routing_cache/          (query → optimal DB path)
│   ├── performance_stats/      (query execution metrics)
│   └── confidence_map/         (what we're confident about)
│
├── tool-results/               ← EXTERNAL KNOWLEDGE (cached from tools)
│   ├── searches/               (web search results, URLs, snippets)
│   │   ├── query_cache         (query → results mapping)
│   │   └── embeddings          (semantic search over past results)
│   ├── scraped-pages/          (full page content, structured)
│   │   ├── content             (raw + cleaned content)
│   │   └── embeddings          (semantic page vectors)
│   ├── api-responses/          (external API results)
│   │   ├── brave-search        (Brave API responses)
│   │   ├── weather             (weather API results)
│   │   └── [other-apis]        (extensible for future tools)
│   └── url-metadata/           (URL → page metadata, success/fail)
│
├── experience/                 ← AGENT LEARNING (action outcomes)
│   ├── action-outcomes/        (what happened when agent acted)
│   │   ├── tool-calls          (tool → args → result → feedback)
│   │   └── embeddings          (semantic search over past actions)
│   ├── user-feedback/          (user reactions to agent actions)
│   │   ├── corrections         (user corrected agent)
│   │   ├── approvals           (user liked action)
│   │   └── rejections          (user rejected action)
│   ├── error-patterns/         (what went wrong and why)
│   │   ├── tool-errors         (tool failures, causes)
│   │   └── recovery-strategies (how to fix errors)
│   └── success-patterns/       (what worked, should repeat)
│       ├── strategies          (action patterns that succeed)
│       └── confidence-scores   (how confident in each pattern)
│
└── model-cache/                ← MODELS (separate, not user data)
    ├── chunks
    ├── metadata
    └── manifests
```

## Memory Systems (Detailed)

### 1. Conversations Database (SOURCE - Critical!)
```
conversations/
├── active/          ← Last 30 days (HOT - frequently accessed)
│   ├── chats        (chat metadata, settings)
│   └── messages     (raw text, attachments, timestamps)
├── recent/          ← 30-90 days (WARM - occasionally accessed)
│   ├── chats
│   └── messages
└── archive/         ← 90+ days (COLD - rarely accessed)
    ├── 2024-Q4/
    ├── 2024-Q3/
    └── ...
```

**Key Properties:**
- **ATOMIC**: Chat + its messages = ONE transaction
- **CRITICAL**: If corrupted, user loses data
- **BACKUP**: Auto-backup on every write
- **NO EMBEDDINGS**: Just IDs → link to embeddings/
- **NO ENTITIES**: Just IDs → link to knowledge/

**Managed by:** `storage/` crate

### 2. Embeddings (DERIVED - Regeneratable)
```
embeddings/
├── active/          ← Vectors for active conversations
│   ├── vectors      (message_id → vector)
│   └── HNSW_index   (10K vectors → <1ms search)
├── recent/          ← Vectors for recent conversations
│   ├── vectors
│   └── HNSW_index   (50K vectors → <10ms search)
└── archive/         ← Old vectors (rarely searched)
    ├── 2024-Q4/
    └── HNSW_index   (500K vectors → 100ms, acceptable because rare)
```

**Key Properties:**
- **DERIVED**: Generated from conversations/ by `task-scheduler`
- **REGENERATABLE**: If corrupted, re-embed all messages
- **REF**: `message_id` links to conversations/messages
- **LAZY LOADED**: Only load tier when needed

**Managed by:** `indexing/` crate (future)

### 3. Knowledge Graph (DERIVED - Regeneratable)
```
knowledge/
├── active/          ← Recently mentioned entities
│   ├── entities     (extracted in last 30 days)
│   └── edges        (MENTIONS, RELATED_TO)
├── stable/          ← Well-established concepts
│   ├── entities     (mentioned 10+ times, proven important)
│   └── edges        (strong relationships)
└── inferred/        ← Weak signals, experimental
    ├── entities     (extracted but unconfirmed)
    └── edges        (low-confidence relationships)
```

**Key Properties:**
- **DERIVED**: Extracted from conversations/ by `weaver`
- **REGENERATABLE**: If corrupted, re-run NER on all chats
- **REF**: `source_message_id` links to conversations/
- **PROMOTION**: active → stable when proven important
- **PRUNING**: inferred → deleted if not confirmed

**Managed by:** `weaver/` crate

### 4. Summaries (DERIVED - Regeneratable)
```
summaries/
├── session/         ← Current session (in-memory, volatile)
├── daily/           ← Last 30 days (per-day summaries)
├── weekly/          ← Last 6 months (per-week summaries)
└── monthly/         ← All time (per-month summaries)
```

**Key Properties:**
- **DERIVED**: Generated from conversations/ by `weaver`
- **HIERARCHICAL**: Daily → Weekly → Monthly consolidation
- **REGENERATABLE**: Re-summarize if corrupted

**Managed by:** `weaver/` crate (future enhancement)

### 5. Meta-Memory (INDEXES - Rebuildable)
```
meta/
├── query_index/       ← Which queries hit which DBs
├── routing_cache/     ← Query → optimal DB path
├── performance_stats/ ← Query execution metrics
└── confidence_map/    ← What we're confident about
```

**Key Properties:**
- **DERIVED**: Built from query patterns
- **REBUILDABLE**: Re-index if corrupted
- **LEARNING**: Improves over time from usage

**Managed by:** `query/` crate (future)

### 6. Tool Results Database (EXTERNAL KNOWLEDGE - Cached)

```
tool-results/
├── searches/               ← Web search results
│   ├── query_cache         (query → results)
│   └── embeddings          (semantic search over results)
├── scraped-pages/          ← Full page content
│   ├── content
│   └── embeddings
├── api-responses/          ← External API results
│   ├── brave-search
│   ├── weather
│   └── [other-apis]
└── url-metadata/           ← URL success/failure tracking
```

**Key Properties:**
- **EXTERNAL**: Knowledge from outside MIA (web searches, APIs, scraped pages)
- **CACHED**: Avoid re-fetching the same data
- **TIMESTAMPED**: Know when data was fetched (for staleness checks)
- **SEMANTIC**: Can search "have we seen something like this before?"
- **REF**: Links to conversations (which chat triggered this search?)

**Example Flow:**
```rust
// Agent: "Search for Rust database design"
1. Check tool-results/searches: Have we searched this before?
   - Query embedding → semantic search in past searches
   - Found similar: "Rust embedded DB" from 2 days ago
   
2. If fresh enough (< 7 days), reuse cached results
   - Return: [URL1, URL2, URL3] from cache
   
3. If stale or not found, call external API
   - Brave Search API → new results
   - Store in tool-results/searches
   - Store embeddings for future semantic search
   
4. Scrape promising URLs
   - URL2 looks relevant → scrape
   - Store content in tool-results/scraped-pages
   - Generate embedding for semantic search
   
5. Link to conversation
   - Store edge: conversation_msg → search_result
   - Agent can later recall "what search led to this?"
```

**Data Structure:**
```rust
struct SearchResult {
    id: String,
    query: String,                  // Original query
    query_embedding: Vec<f32>,      // For semantic matching
    results: Vec<SearchHit>,        // URLs, titles, snippets
    api_source: String,             // "brave", "google", etc.
    timestamp: i64,                 // When fetched
    triggered_by: String,           // Message ID that triggered search
    used_in: Vec<String>,           // Message IDs that used these results
}

struct ScrapedPage {
    id: String,
    url: String,
    content: String,                // Full text content
    content_embedding: Vec<f32>,    // Semantic vector
    metadata: PageMetadata,         // Title, author, date, etc.
    timestamp: i64,                 // When scraped
    success: bool,                  // Did scraping succeed?
    error: Option<String>,          // Error message if failed
}
```

**Why This Database?**
- **Efficiency**: Don't re-search the same query!
- **Cost**: External APIs cost money (rate limits, quotas)
- **Speed**: Cached results are instant
- **Context**: Remember where results came from

**Managed by:** Tools system (future), `weaver/` for semantic indexing

### 7. Experience Database (AGENT LEARNING - Critical!)

```
experience/
├── action-outcomes/        ← What happened when agent acted
│   ├── tool-calls          (tool → args → result → feedback)
│   └── embeddings          (semantic search over actions)
├── user-feedback/          ← User reactions
│   ├── corrections         (user corrected agent)
│   ├── approvals           (user liked action)
│   └── rejections          (user rejected action)
├── error-patterns/         ← What went wrong
│   ├── tool-errors
│   └── recovery-strategies
└── success-patterns/       ← What worked
    ├── strategies
    └── confidence-scores
```

**Key Properties:**
- **LEARNING**: Agent improves from experience
- **FEEDBACK-DRIVEN**: User corrections shape future behavior
- **PATTERN-BASED**: Recognizes what works, what doesn't
- **CONFIDENCE-SCORED**: Knows how sure it is about each pattern
- **TEMPORAL**: Recent experiences weigh more than old ones

**Example Flow:**
```rust
// Scenario 1: Bad Search (Learning from Failure)
1. Agent searches "Rust DB"
   → Uses Brave API
   → Returns generic database results

2. User feedback: "That's not helpful, I meant embedded databases"
   → Store in experience/user-feedback/corrections
   → Link: query_pattern → correction → better_query

3. Agent learns:
   - "Rust DB" is ambiguous
   - Should ask clarifying questions OR
   - Should check conversation context for clues
   - Store pattern: ambiguous_query → needs_clarification

4. Next time similar query:
   → Agent: "I found 'Rust DB' ambiguous before. Did you mean embedded DBs?"
   → Confidence: 0.8 (based on past correction)

// Scenario 2: Good Action (Reinforcement Learning)
1. Agent searches "Rust embedded database"
   → Returns sled, redb, RocksDB
   → Scrapes sled documentation

2. User: "Perfect! This is exactly what I needed."
   → Store in experience/user-feedback/approvals
   → Increment success_pattern confidence

3. Agent learns:
   - Pattern: "embedded database" → prioritize library docs over tutorials
   - Confidence: 0.9 (multiple successes)

4. Next time similar query:
   → Agent applies same strategy automatically
   → Higher priority for official docs
```

**Data Structure:**
```rust
struct ActionOutcome {
    id: String,
    action_type: String,            // "search", "scrape", "summarize", etc.
    action_args: serde_json::Value, // Tool arguments
    result: ActionResult,           // Success/failure, data returned
    user_feedback: Option<UserFeedback>,
    timestamp: i64,
    conversation_context: String,   // Message ID where action occurred
}

struct UserFeedback {
    feedback_type: FeedbackType,    // Correction, Approval, Rejection
    user_comment: Option<String>,   // "That's not helpful" or "Perfect!"
    correction: Option<String>,     // What user wanted instead
    timestamp: i64,
}

enum FeedbackType {
    Correction,  // User corrected agent's action
    Approval,    // User liked agent's action
    Rejection,   // User rejected agent's action
    Neutral,     // No explicit feedback (infer from follow-up)
}

struct SuccessPattern {
    id: String,
    pattern_type: String,           // "query_refinement", "tool_selection", etc.
    pattern_data: serde_json::Value,// Pattern specifics
    success_count: u32,             // How many times it worked
    failure_count: u32,             // How many times it failed
    confidence: f32,                // success / (success + failure)
    last_used: i64,                 // Temporal decay
    embedding: Vec<f32>,            // For semantic pattern matching
}

struct ErrorPattern {
    id: String,
    error_type: String,             // "api_timeout", "parse_error", etc.
    error_context: serde_json::Value,
    recovery_strategy: Option<String>, // What worked to fix it
    occurrence_count: u32,
    last_seen: i64,
}
```

**Why This Database?**
- **CRITICAL FOR AGENTIC BEHAVIOR**: Without experience, agent repeats mistakes!
- **User-Driven Improvement**: Agent learns what YOU want
- **Self-Correction**: Agent remembers its errors, doesn't repeat them
- **Pattern Recognition**: "This worked before, try it again"
- **Confidence Calibration**: Agent knows when it's uncertain

**Managed by:** Agentic system (future), feedback collection hooks

---

## Knowledge vs Experience

### The Critical Distinction

**KNOWLEDGE** = Static facts about the world
- Entities: "Rust is a programming language"
- Relationships: "Rust is related to systems programming"
- Embeddings: Semantic similarity of concepts

**TOOL RESULTS** = External knowledge cached locally
- Search results: "Query 'Rust DB' returned these URLs"
- Scraped pages: "This URL contained this content"
- API responses: "Weather API said 72°F"

**EXPERIENCE** = Dynamic learning from actions
- Outcomes: "When I searched X, user said Y"
- Feedback: "User corrected my search to Z"
- Patterns: "Queries like X usually need clarification"
- Confidence: "This strategy works 80% of the time"

### Example: Complete Agent Loop

**Scenario**: User asks "Find me information about Rust databases"

```
1. CONVERSATIONS DB (Source)
   → Store user message: "Find me information about Rust databases"
   → Message ID: msg_123

2. KNOWLEDGE GRAPH (What we know)
   → Check: Do we have entity "Rust"? Yes (programming language)
   → Check: Do we have entity "Database"? Yes (data storage)
   → Check: Related concepts? "Embedded DB", "NoSQL", "SQLite"

3. EXPERIENCE DB (What worked before?)
   → Semantic search: "Similar requests in the past?"
   → Found: msg_045 (3 days ago): "Rust embedded database"
   →   User feedback: APPROVAL (user liked sled, redb results)
   → Pattern: "embedded" keyword leads to better results
   → Confidence: 0.85

4. AGENT DECISION (Apply experience)
   → Refine query: "Rust embedded databases" (learned from experience)
   → Tool: Search (learned: check cache first)

5. TOOL-RESULTS DB (Have we searched this?)
   → Semantic search in past searches
   → Found: "Rust embedded DB" from 2 days ago
   → Results still fresh (< 7 days)
   → Return cached: [sled, redb, RocksDB docs]

6. AGENT ACTION
   → Present results to user
   → Wait for feedback

7. USER FEEDBACK
   Option A: "Perfect, thanks!" 
     → Store in experience/user-feedback/approvals
     → Increment success_pattern confidence
     → This strategy works again!
   
   Option B: "No, I meant SQL databases"
     → Store in experience/user-feedback/corrections
     → Learn: "database" ambiguous, need to ask "SQL or NoSQL?"
     → Update pattern: ambiguous_queries → ask_clarification

8. KNOWLEDGE GRAPH (Update)
   → Create entity: "sled" (if new)
   → Create edge: Rust → HAS_LIBRARY → sled
   → This is STATIC KNOWLEDGE (factual)

9. EXPERIENCE GRAPH (Update)
   → Create pattern: query_refinement(database → embedded_database) = SUCCESS
   → Link: msg_123 → action_outcome → user_approval
   → This is DYNAMIC EXPERIENCE (learning)
```

### Flow Diagram

```
User Query
    ↓
┌───────────────────────────────────────────────┐
│ 1. CONVERSATIONS DB (Store user intent)      │
└───────────────┬───────────────────────────────┘
                ↓
┌───────────────────────────────────────────────┐
│ 2. KNOWLEDGE GRAPH (What do we know?)        │
│    - Entities, relationships, concepts        │
└───────────────┬───────────────────────────────┘
                ↓
┌───────────────────────────────────────────────┐
│ 3. EXPERIENCE DB (What worked before?)       │
│    - Past similar queries                     │
│    - User feedback on past actions            │
│    - Success/failure patterns                 │
│    → Output: Refined strategy + confidence    │
└───────────────┬───────────────────────────────┘
                ↓
┌───────────────────────────────────────────────┐
│ 4. TOOL-RESULTS DB (Check cache)             │
│    - Have we searched this before?            │
│    - Are cached results still fresh?          │
│    → Cache hit: Return results                │
│    → Cache miss: Call external tool           │
└───────────────┬───────────────────────────────┘
                ↓
┌───────────────────────────────────────────────┐
│ 5. AGENT ACTION (Use tool, present results)  │
└───────────────┬───────────────────────────────┘
                ↓
┌───────────────────────────────────────────────┐
│ 6. USER FEEDBACK (How did agent do?)         │
│    - Approval, correction, or rejection       │
└───────────────┬───────────────────────────────┘
                ↓
    ┌───────────┴───────────┐
    ↓                       ↓
┌──────────────────┐  ┌──────────────────┐
│ KNOWLEDGE UPDATE │  │ EXPERIENCE UPDATE│
│ (Static facts)   │  │ (Dynamic learning)│
└──────────────────┘  └──────────────────┘
```

### Why Both Are Essential

**Without KNOWLEDGE**: Agent has no understanding of concepts
- Can't relate ideas
- Can't leverage semantic similarity
- No structured facts

**Without TOOL-RESULTS**: Agent wastes time/money
- Re-searches same queries
- Re-scrapes same pages
- No caching efficiency

**Without EXPERIENCE**: Agent never improves
- Repeats same mistakes
- Ignores user corrections
- No pattern recognition
- No confidence calibration

**With ALL THREE**: Agent becomes intelligent
- ✅ Understands concepts (knowledge)
- ✅ Caches efficiently (tool-results)
- ✅ Learns from feedback (experience)
- ✅ Improves over time (experience patterns)
- ✅ Knows when uncertain (confidence scores)

### Database Relationships

```
conversations/  ← User's raw input (SOURCE)
      ↓ triggers
agent_action  
      ↓ uses
knowledge/    ← Static facts ("What IS this?")
      ↓ informs
experience/   ← Learning ("What WORKED before?")
      ↓ decides strategy
tool-results/ ← External data ("Have we seen this?")
      ↓ produces
agent_output
      ↓ gets feedback
experience/   ← Updated with outcome
```

**Key Insight**: Knowledge tells agent WHAT things are. Experience tells agent WHAT TO DO!

---

## Query Interface

### The Vision: Single Entry Point for All Queries

**User Request**: "let results = conversations_active.search(today, chat ref current, look in knowledge graph yes, search level 2)?"

**What the user wanted**: A single query that controls:
- WHAT to search (semantic query text)
- WHEN (time scope)
- WHERE (context scope - current chat, all chats, related chats)
- HOW DEEP (graph traversal depth)
- HOW HOT (temperature - which tier DBs to use)

### Query Structure

```rust
pub struct Query {
    // WHAT: Semantic query
    pub semantic: String,
    
    // WHEN: Time scope (which conversation DBs)
    pub time_scope: TimeScope,
    
    // WHERE: Context scope (current chat or all chats?)
    pub context: Context,
    
    // HOW DEEP: Knowledge graph traversal
    pub use_knowledge_graph: bool,
    pub search_depth: SearchDepth,
    
    // HOW HOT: Database temperature (which tier DBs to search?)
    pub temperature: Temperature,
    
    // RESULT: Preferences
    pub limit: usize,
    pub confidence_threshold: f32,
}
```

### Query Enums (Complete)

```rust
/// WHEN: Time scope
pub enum TimeScope {
    Today,                          // Last 24 hours
    LastWeek,                       // Last 7 days
    LastMonth,                      // Last 30 days
    LastQuarter,                    // Last 90 days
    AllTime,                        // Everything (slowest!)
    Range(DateTime, DateTime),      // Custom range
}

/// WHERE: Context scope
pub enum Context {
    CurrentChat(ChatId),            // Only this chat
    AllChats,                       // All chats (broader)
    RelatedChats(ChatId),           // Chats related via knowledge graph
    ChatsByTopic(Vec<EntityId>),    // Chats about specific topics
}

/// HOW DEEP: Graph traversal depth
pub enum SearchDepth {
    Shallow,                        // No graph traversal
    Level(u32),                     // N-hop traversal (1, 2, 3...)
    Deep,                           // Full graph search (5+ hops, expensive!)
}

/// HOW HOT: Database temperature
pub enum Temperature {
    Hot,                            // Only active/ (fastest)
    Warm,                           // active/ + recent/
    Cold,                           // active/ + recent/ + archive/
    All,                            // Everything (slowest, most thorough)
}
```

### Query Examples (Real-World Usage)

**Example 1: Quick Lookup in Current Chat (FAST)**
```rust
// User: "Where did we put that file?"
let results = mia.query(Query {
    semantic: "where did we put that file?",
    time_scope: TimeScope::Today,
    context: Context::CurrentChat(current_chat),
    use_knowledge_graph: false,  // Don't need graph
    search_depth: SearchDepth::Shallow,
    temperature: Temperature::Hot,  // Only active/
    limit: 5,
    confidence_threshold: 0.7,
})?;

// Hits: conversations/active, embeddings/active
// Skips: recent, archive, knowledge
// Speed: <1ms
```

**Example 2: Medium Search Across Recent Work (MEDIUM)**
```rust
// User: "What were our Rust database design discussions?"
let results = mia.query(Query {
    semantic: "Rust database design discussions",
    time_scope: TimeScope::LastWeek,
    context: Context::AllChats,
    use_knowledge_graph: true,   // Include related concepts
    search_depth: SearchDepth::Level(1),  // 1-hop only
    temperature: Temperature::Warm,  // active + recent
    limit: 10,
    confidence_threshold: 0.6,
})?;

// Hits: conversations/active+recent, embeddings/active+recent, knowledge/active+stable
// Speed: <10ms
```

**Example 3: Deep Historical Search (SLOW but thorough)**
```rust
// User: "Find all discussions about machine learning ever"
let results = mia.query(Query {
    semantic: "all discussions about machine learning",
    time_scope: TimeScope::AllTime,
    context: Context::AllChats,
    use_knowledge_graph: true,
    search_depth: SearchDepth::Level(2),  // 2-hop traversal
    temperature: Temperature::All,  // active + recent + archive
    limit: 50,
    confidence_threshold: 0.5,
})?;

// Hits: ALL databases
// Speed: 100ms-1s (acceptable for deep search)
```

### Query Execution Pipeline (Multi-Stage)

```rust
impl MIA {
    pub fn query(&self, query: Query) -> Result<Vec<ExpandedResult>> {
        // ========================================
        // STAGE 0: META-MEMORY (routing decision)
        // ========================================
        let plan = self.meta.plan_query(&query);
        // plan = {
        //   conversations: ["active"],      (only hot data)
        //   embeddings: ["active"],
        //   knowledge: ["active", "stable"], (active + stable)
        // }
        
        // ========================================
        // STAGE 1: STRUCTURAL FILTER (fast, accurate)
        // ========================================
        let conversation_candidates = match query.context {
            Context::CurrentChat(chat_id) => {
                // Only search THIS chat's messages
                self.conversations[plan.temperature]
                    .filter_by_chat(chat_id)
                    .filter_by_time(query.time_scope)
            }
            Context::AllChats => {
                // Search across all chats (broader)
                self.conversations[plan.temperature]
                    .filter_by_time(query.time_scope)
            }
            Context::RelatedChats(chat_id) => {
                // Find chats related to this one via knowledge graph
                let related_chat_ids = self.knowledge[plan.temperature]
                    .find_related_chats(chat_id, depth=1)?;
                self.conversations[plan.temperature]
                    .filter_by_chats(related_chat_ids)
            }
        };
        
        // ========================================
        // STAGE 2: SEMANTIC SEARCH (on filtered candidates)
        // ========================================
        let semantic_matches = self.embeddings[plan.temperature]
            .vector_search(
                query.semantic,
                candidate_ids=conversation_candidates,
                limit=query.limit * 3  // Get more for next stage
            )?;
        
        // ========================================
        // STAGE 3: KNOWLEDGE GRAPH EXPANSION (if requested)
        // ========================================
        let expanded_results = if query.use_knowledge_graph {
            self.expand_with_knowledge(
                semantic_matches,
                depth=query.search_depth,
                temperature=plan.temperature
            )?
        } else {
            semantic_matches
        };
        
        // ========================================
        // STAGE 4: RANK & FILTER (final results)
        // ========================================
        let final_results = self.rank_and_filter(
            expanded_results,
            query.confidence_threshold,
            query.limit
        )?;
        
        Ok(final_results)
    }
}
```

### Result Structure

```rust
pub struct ExpandedResult {
    // The original message found
    pub original: MessageResult,
    
    // Entities in this message
    pub entities: Vec<Entity>,
    
    // Related entities found via graph traversal
    pub related_entities: Vec<Entity>,
    
    // Messages mentioning related entities
    pub related_messages: Vec<MessageResult>,
    
    // Confidence score (0.0-1.0)
    pub confidence: f32,
    
    // Reasoning: WHY was this result returned?
    pub reasoning: String,
    // Example: "Found via 2-hop graph traversal: message → Rust → database design"
}
```

### Python API (Same Interface)

```python
# Simple query
results = mia.query(
    semantic="Rust database design",
    time_scope="today",
    context={"current_chat": chat_id},
    use_knowledge_graph=True,
    search_depth=2,
    temperature="hot",
    limit=10
)

# Results include reasoning!
for result in results:
    print(f"Message: {result.message.text}")
    print(f"Confidence: {result.confidence}")
    print(f"Reasoning: {result.reasoning}")
    # "Found via 2-hop graph traversal: message → Rust → database design"
    
    print(f"Related entities: {result.entities}")
    print(f"Related messages: {len(result.related_messages)}")
```

### Query Optimizer (Meta-Memory Learning)

```rust
impl MetaMemory {
    fn plan_query(&self, query: &Query) -> QueryPlan {
        // Learn from past queries
        let similar_past_queries = self.procedural
            .find_similar_queries(query)?;
        
        // What worked before?
        let best_strategy = similar_past_queries
            .max_by_key(|q| q.user_satisfaction);
        
        // Adjust plan based on query characteristics
        let mut plan = QueryPlan::default();
        
        // Time scope determines which conversation DBs
        plan.conversations = match query.time_scope {
            TimeScope::Today => vec!["active"],
            TimeScope::LastWeek => vec!["active", "recent"],
            TimeScope::AllTime => vec!["active", "recent", "archives/*"],
            _ => vec!["active"],
        };
        
        // Context determines search breadth
        if matches!(query.context, Context::CurrentChat(_)) {
            // Narrow search - can afford to go deeper
            plan.max_depth_affordable = 3;
        } else {
            // Broad search - limit depth to keep speed up
            plan.max_depth_affordable = 1;
        }
        
        // Knowledge graph usage determines which knowledge DBs
        if query.use_knowledge_graph {
            plan.knowledge = match query.search_depth {
                SearchDepth::Shallow => vec![],
                SearchDepth::Level(1) => vec!["active"],
                SearchDepth::Level(2) => vec!["active", "stable"],
                SearchDepth::Deep => vec!["active", "stable", "inferred"],
            };
        }
        
        // Estimate cost and optimize if needed
        let estimated_cost = self.estimate_cost(&plan);
        if estimated_cost > COST_THRESHOLD {
            // Too expensive - optimize
            plan = self.optimize_plan(plan, query);
        }
        
        plan
    }
}
```

---

## Unified Query Interface

### Single Entry Point (Future - `query/` crate)
```rust
let results = mia.query(Query {
    semantic: "Rust database design",
    time_scope: TimeScope::Today,
    context: Context::CurrentChat(chat_ref),
    use_knowledge_graph: true,
    search_depth: SearchDepth::Level(2),
    temperature: Temperature::Hot,  // Only active/ tier
    limit: 10,
})?;
```

### Query Execution Pipeline
1. **Stage 0**: Meta-memory decides which DBs to use
2. **Stage 1**: Structural filter (conversations/) → candidate set
3. **Stage 2**: Semantic search (embeddings/) → ranked results
4. **Stage 3**: Knowledge graph expansion (knowledge/) → related entities
5. **Stage 4**: Rank & filter → final results with reasoning

## Data Lifecycle Management

### Automatic Promotion/Demotion (via `task-scheduler`)
```rust
// Conversations
active/ → recent/     (after 30 days)
recent/ → archive/    (after 90 days)
archive/ → deleted    (after 2 years, optional)

// Knowledge
active/ → stable/     (after 10+ mentions)
inferred/ → deleted   (if not confirmed in 30 days)
stable/ → active/     (if recently accessed)

// Embeddings
(Follow conversations/ lifecycle automatically)
```

### Recovery Strategy
```python
# If any derived DB corrupts:
def recover_embeddings():
    embeddings_db.clear()
    for msg in conversations_db.all_messages():
        task_scheduler.queue(EmbedMessage(msg.id))

def recover_knowledge():
    knowledge_db.clear()
    for msg in conversations_db.all_messages():
        task_scheduler.queue(ExtractEntities(msg.id))
```

## Fault Isolation Guarantees

✅ **Embeddings corrupted?** → Re-run task-scheduler on conversations/  
✅ **Knowledge corrupted?** → Re-run weaver on conversations/  
✅ **Conversations corrupted?** → DISASTER (but backups exist!)  
✅ **One tier corrupted?** → Other tiers still work  
✅ **HNSW index corrupted?** → Rebuild from vectors/  

**No cascading failures!**

## Implementation Phases

### Phase 1: Foundation (NOW)
- [x] `storage/` crate with basic sled wrapper
- [ ] Multi-tier structure: active/ recent/ archive/
- [ ] Conversations DB with chats + messages
- [ ] Embeddings DB with vectors (no HNSW yet)
- [ ] Knowledge DB with entities + edges
- [ ] Python bindings (`db-bindings`)

### Phase 2: Intelligence (SOON)
- [ ] `indexing/` crate with HNSW
- [ ] `weaver/` entity extraction integration
- [ ] `task-scheduler/` lifecycle management
- [ ] Auto promotion/demotion logic

### Phase 3: Query Engine (FUTURE)
- [ ] `query/` crate with unified interface
- [ ] Multi-stage query pipeline
- [ ] Meta-memory learning
- [ ] Confidence scoring

### Phase 4: Advanced (FAR FUTURE)
- [ ] Cross-modal embeddings (text + image + audio)
- [ ] Reasoning chains
- [ ] Memory consolidation (sleep-like processing)
- [ ] Explainable retrieval

## Inspiration: Human Brain Analogy

| MIA Memory System | Human Brain Region | Function |
|-------------------|-------------------|----------|
| conversations/active | Working Memory (Prefrontal Cortex) | Current context |
| conversations/recent | Short-term Memory (Hippocampus) | Recent events |
| conversations/archive | Long-term Memory (Neocortex) | Distant past |
| knowledge/active | Active Concepts | Currently relevant |
| knowledge/stable | Semantic Memory (Temporal Cortex) | Persistent knowledge |
| embeddings/ | Neural Activation Patterns | Similarity matching |
| tool-results/ | Sensory Cache (Visual/Auditory) | External information cache |
| experience/ | **Procedural Memory (Basal Ganglia)** | **What actions WORKED** |
| meta/ | Executive Function | Decision making |
| task-scheduler/ | Sleep/Consolidation | Memory processing |
| weaver/ | Pattern Recognition | Learning |

## Key Architectural Decisions

### Why Temperature Tiers?
- **Performance**: Small active/ DB = fast queries
- **Scalability**: Archive doesn't slow down active queries
- **Cost**: Hot data in RAM, cold data on disk
- **Access patterns**: 90% queries hit active/, 10% deep search

### Why Separate DB Types?
- **Fault isolation**: Corruption doesn't cascade
- **Recovery**: Know exactly what to regenerate
- **Guarantees**: ACID for source, eventual consistency for derived
- **Scaling**: Each DB grows at different rates

### Why ID-based Linking (Not Embedding)?
- **Atomic operations**: Each DB manages its own data
- **No duplication**: Message text stored once
- **Flexible**: Can change derived data without touching source
- **Efficient**: Only load what's needed

### Why Not ONE Big DB?
- **Performance**: Searching 1M vectors is slow, searching 10K is fast
- **Management**: Clear boundaries for backup/recovery
- **Evolution**: Can change derived schema without migrating source

---

## Implementation Status

### ✅ What Exists NOW (October 2025)

**`Server/tabagent-rs/common/` - Shared Types**
- ✅ `models.rs`: Chat, Message, Entity, Summary, Edge, Embedding
- ✅ Hybrid Schema Model (typed core + flexible metadata)
- ✅ Binary serialization with `bincode`
- ✅ Platform-specific paths (Windows/macOS/Linux)

**`Server/tabagent-rs/storage/` - Storage Layer**
- ✅ `StorageManager`: Basic CRUD for nodes/edges/embeddings
- ✅ Three trees: `nodes`, `edges`, `embeddings`
- ✅ Single database support
- ✅ Optional indexing integration
- ✅ Platform-specific default paths

**`Server/tabagent-rs/indexing/` - Indexing Layer**
- ✅ Structural indexes (type, properties)
- ✅ Graph indexes (from/to adjacency)
- ✅ Vector index (HNSW) integration
- ✅ Automatic index maintenance

**`Server/tabagent-rs/weaver/` - Enrichment Engine**
- ✅ Event-driven architecture
- ✅ ML bridge trait for Python integration
- ✅ Semantic indexer module
- ✅ Entity linker module
- ✅ Associative linker module
- ✅ Summarizer module (stub)
- ✅ 10 tests passing

**`Server/tabagent-rs/task-scheduler/` - Background Processing**
- ✅ Activity-aware task queue (High/Low/Sleep)
- ✅ Priority levels (Urgent/Normal/Low/Batch)
- ✅ Task types defined (embedding, NER, summarization)
- ✅ Tests passing

**`Server/tabagent-rs/query/` - Query Engine**
- ⚠️ Exists but needs audit (implementation status unclear)

### 🔴 What's Missing (Need to Build)

**Phase 1 (Immediate - Foundation):**
1. **Multi-database support** in `storage/`
   - `DatabaseType` enum (Conversations, Knowledge, ToolResults, Experience, ModelCache)
   - `open_typed()` method
   - Platform-specific paths per DB type

2. **DatabaseCoordinator** in `storage/src/coordinator.rs`
   - Manages multiple `StorageManager` instances
   - Cross-DB operations (message + entities + tool results + experience)
   - Integration with task-scheduler

3. **Integration Layer**
   - Storage → task-scheduler wiring
   - Task-scheduler → weaver wiring
   - Background enrichment flow

4. **Tool Results Database**
   - Search result caching
   - Scraped page storage
   - API response caching
   - Semantic search over cached results

5. **Experience Database** (CRITICAL for agentic!)
   - Action outcome tracking
   - User feedback collection
   - Success/failure pattern recognition
   - Confidence scoring
   - Learning from mistakes

**Phase 2 (Next - Temperature Tiers):**
1. **Multi-tier storage** in `storage/src/multi_tier.rs`
   - Active/recent/archive structure
   - Lazy loading for cold tiers
   - Tier-specific `StorageManager` instances

2. **Lifecycle management**
   - Automatic promotion/demotion
   - Age-based queries
   - Bulk operations (bulk_insert, bulk_delete)
   - Background task for lifecycle management

**Phase 3 (Future - Query Engine):**
1. **Unified query API** in `query/`
   - `Query` struct with all parameters
   - Multi-stage pipeline (Stage 0-4)
   - Python bindings

2. **Meta-memory learning**
   - Query routing optimization
   - Performance tracking
   - Strategy adaptation

### 🎯 Current Goal: Phase 1

**Objective**: Multi-database support WITHOUT temperature tiers yet

**Success Criteria**:
- Can open 3 separate databases (conversations, knowledge, model-cache)
- DatabaseCoordinator can insert message → queue tasks
- Cross-DB query works (get message + its entities)
- Tests pass for fault isolation

**Estimated Timeline**: 1-2 weeks

---

## Design Rationale

### Why This Architecture? (Summary of Discussion)

**The Original Problem:**
> "Bigger DB = slower search. Should we split chat/embeddings/knowledge?"

**The Evolution:**
1. **First thought**: One big DB (like browser extension MasterPlan)
   - ❌ Problem: 1M messages = 1M vectors in HNSW = slow search forever!

2. **Second thought**: Split by entity type (chat DB, embedding DB, knowledge DB)
   - ❌ Problem: WHERE DO EDGES GO? (span across types!)
   - ❌ Problem: Can't traverse graph across databases!

3. **Breakthrough**: Split by ACCESS PATTERN (hot/warm/cold) AND DATA TYPE
   - ✅ Solution: conversations/active (10K msgs, fast) + conversations/archive (1M msgs, slow but rare)
   - ✅ Solution: knowledge/active + knowledge/stable (different purposes!)

4. **Final insight**: This is COGNITIVE ARCHITECTURE, not just a database!
   - MIA needs multiple memory systems like a human brain
   - Each system has different access patterns, guarantees, and lifecycles

### Key Design Patterns

**1. Source vs Derived (Fault Isolation)**
```
conversations/  ← SOURCE (cannot lose!)
    ↓ task-scheduler queues tasks
embeddings/     ← DERIVED (can regenerate)
knowledge/      ← DERIVED (can regenerate)
```

**Why?**
- If derived data corrupts, regenerate from source
- Enrichment can fail/retry without affecting user data
- No cascading failures

**2. Temperature Tiers (Performance)**
```
active/    ← 10K messages, <1ms search (HOT)
recent/    ← 50K messages, <10ms search (WARM)
archive/   ← 1M messages, 100ms search (COLD, rare)
```

**Why?**
- 90% of queries hit active/ (fast!)
- Archive doesn't slow down active queries
- Performance stays constant over years

**3. Lazy Loading (Minimal RAM)**
```rust
active: Always in RAM
recent: Lazy load on first access
archive: Load on demand, LRU eviction
```

**Why?**
- Don't load 1M messages into RAM!
- Most queries never need archive

**4. Atomic Operations (Data Integrity)**
```rust
// ATOMIC within scope
conversations_db.insert_message(msg)?;  // All or nothing

// NOT atomic across DBs (by design)
task_scheduler.queue(Embed(msg.id)).await?;  // Can retry
```

**Why?**
- User sees message immediately
- Enrichment happens async in background
- Failures don't block user

**5. ID-based Linking (Flexibility)**
```rust
Message {
    id: "msg_123",
    embedding_id: Some("embed_456"),  // Link, not embed!
}
```

**Why?**
- No duplication (message text stored once)
- Can change embedding without touching message
- Only load what's needed

### Human Brain Inspiration

The user explicitly said:
> "this server is like human brain, there is multiple level... everything is about how well can we remember and how fast and how relevant memory gets to me quickly"

This led to modeling MIA's memory systems after human cognition:

| Human System | MIA System | Purpose |
|--------------|------------|---------|
| **Working Memory** | conversations/active | Current context (fast!) |
| **Short-term Memory** | conversations/recent | Recent events |
| **Long-term Memory** | conversations/archive | Distant past |
| **Semantic Memory** | knowledge/stable | Persistent concepts |
| **Episodic Memory** | conversations/ | "When did I...?" |
| **Procedural Memory** | meta/ | "What strategy works?" |
| **Sleep Consolidation** | task-scheduler | Background processing |
| **Executive Function** | meta-memory | "Where should I look?" |

**Key Insight**: The brain doesn't search ALL memories for EVERY query!
- Quick recall from working memory (conversations/active)
- Deep search only when needed (conversations/archive)
- Meta-cognition decides where to look (meta-memory routing)

### Why NOT Alternatives?

**Alternative 1: ONE big database**
- ❌ Corruption cascades across all data
- ❌ 1M vectors in HNSW = searches slow down over time
- ❌ Can't optimize for different access patterns
- ❌ Backup/restore is all-or-nothing

**Alternative 2: Microservices (separate processes)**
- ❌ Overkill for single-user local app
- ❌ IPC overhead for every query
- ❌ Complexity of distributed transactions
- ❌ Harder to reason about failures

**Alternative 3: Traditional RDBMS (PostgreSQL, etc.)**
- ❌ External dependency (not embedded)
- ❌ Heavyweight for local assistant
- ❌ Harder to distribute with native app
- ❌ Vector search (pgvector) less mature than Rust HNSW

**Our Choice: Multiple embedded Rust DBs**
- ✅ Embedded (ships with app)
- ✅ Fast (native Rust performance)
- ✅ Flexible (each DB optimized for its use case)
- ✅ Fault-tolerant (isolation by design)
- ✅ Scalable (tiers keep performance constant)

---

## Next Steps (Immediate)

**See `/Server/storage/DATABASE_FOUNDATION_PLAN.md` for detailed implementation plan.**

**Phase 1 (Current Focus):**
1. ✅ Document MIA memory architecture (this file)
2. 🔜 Create `DatabaseType` enum in `storage/`
3. 🔜 Implement `DatabaseCoordinator`
4. 🔜 Wire storage → task-scheduler → weaver
5. 🔜 Write integration tests

**Start with**: Multi-DB support (conversations, knowledge) WITHOUT temperature tiers yet.

---

## Summary: The Complete Picture

### What We Learned (Journey Recap)

**October 23, 2025** - The discussion started with a simple question about database separation and evolved into designing a complete cognitive architecture for MIA.

**Key Realizations:**
1. **Performance Problem**: Browser extension's "one DB" approach doesn't scale to server (years of data)
2. **Access Pattern Insight**: 90% queries = recent data; 10% = deep history
3. **Cognitive Architecture**: This isn't a database—it's a brain!
4. **Fault Isolation**: Source (conversations) must be separate from derived (embeddings, knowledge)
5. **Temperature Tiers**: Hot/warm/cold keeps performance constant over years
6. **Unified Query Interface**: Single API controls WHAT/WHEN/WHERE/HOW DEEP/HOW HOT
7. **Knowledge vs Experience**: Static facts ≠ Dynamic learning from actions (CRITICAL distinction!)
8. **Tool Results Cache**: External knowledge cached to avoid redundant API calls
9. **Experience Learning**: Agent learns from user feedback, improves over time

### What We're Building

**Phase 1: Foundation (Weeks 1-2)**
- Multiple database types:
  - conversations/ (SOURCE - user data)
  - knowledge/ (DERIVED - static facts)
  - tool-results/ (EXTERNAL - cached searches/scrapes)
  - experience/ (LEARNING - agent feedback and patterns)
  - model-cache/ (MODELS - already exists)
- DatabaseCoordinator for cross-DB operations
- Integration with existing task-scheduler and weaver
- Tool results caching system
- Experience tracking and learning framework

**Phase 2: Temperature Tiers (Weeks 3-4)**
- Active/recent/archive structure per DB type
- Automatic lifecycle management (promotion/demotion)
- Lazy loading and performance optimization

**Phase 3: Query Engine (Future)**
- Unified `mia.query()` interface
- Multi-stage pipeline (meta → structural → semantic → graph → rank)
- Meta-memory learning from query patterns

### Why This Matters

**For Performance:**
- Queries stay fast (<1ms) even with years of data
- Archive doesn't slow down active queries
- Only loads what's needed (lazy loading)
- Tool results cached (no redundant API calls)

**For Reliability:**
- Fault isolation (no cascading failures)
- Regeneratable derived data
- Automatic recovery from corruption
- Experience persists across sessions

**For Intelligence:**
- Meta-memory learns optimal routing
- Multi-hop graph traversal
- Confidence scoring and reasoning
- **Agent learns from mistakes** (experience DB)
- **Tool results cached** (efficiency + cost savings)

**For Agentic Behavior (NEW!):**
- **Experience learning**: Agent improves from user feedback
- **Pattern recognition**: "This worked before, try again"
- **Confidence calibration**: Agent knows when uncertain
- **Self-correction**: Remembers errors, doesn't repeat them
- **Tool optimization**: Learns which tools work for which tasks

**For Users:**
- Fast instant recall (working memory)
- Deep historical search when needed
- **Agent gets better over time** (learns your preferences)
- **No repeated mistakes** (experience learning)
- Transparent—just works!

### The Foundation

MIA's memory system is modeled after human cognition:
- **Fast** (working memory for current context)
- **Deep** (long-term memory for history)
- **Smart** (meta-memory for routing)
- **Learning** (procedural memory from experience)
- **Fault-tolerant** (no "coma" from partial failure)
- **Scalable** (constant performance over years)

### The Critical Innovation: 7 Databases, Not 5!

**Original plan** (missing piece):
1. conversations/ - SOURCE
2. embeddings/ - DERIVED
3. knowledge/ - DERIVED
4. summaries/ - DERIVED
5. meta/ - INDEXES

**Complete plan** (with learning!):
1. conversations/ - SOURCE (user data)
2. embeddings/ - DERIVED (semantic search)
3. knowledge/ - DERIVED (static facts)
4. summaries/ - DERIVED (hierarchical)
5. meta/ - INDEXES (routing)
6. **tool-results/** - EXTERNAL (cached searches/scrapes) ← NEW!
7. **experience/** - LEARNING (agent feedback/patterns) ← NEW!

**Why These Two Are CRITICAL:**

**Without `tool-results/`**:
- ❌ Agent re-searches the same queries
- ❌ Wastes API quota and money
- ❌ Slow (every search = external API call)
- ❌ Can't remember "we found this before"

**Without `experience/`**:
- ❌ Agent repeats the same mistakes forever
- ❌ Ignores user corrections
- ❌ No improvement over time
- ❌ Not truly "intelligent"—just a search engine

**With BOTH**:
- ✅ Agent caches external knowledge (efficiency)
- ✅ Agent learns from feedback (intelligence)
- ✅ Agent improves over time (adaptation)
- ✅ Agent knows when uncertain (confidence)
- ✅ True agentic behavior!

**Key Insight**: 
> "Knowledge tells agent WHAT things are.  
> Experience tells agent WHAT TO DO!"

This is the difference between a **search tool** and an **intelligent agent**!

---

**This document is the complete architectural reference for MIA's memory system.**  
**No need to re-explain these concepts—everything is here.** 🧠

**Last Updated**: October 23, 2025  
**Status**: Design Complete (7 databases), Implementation In Progress (Phase 1)  
**Next**: See `DATABASE_FOUNDATION_PLAN.md` for build steps

**Critical Addition**: Tool-results and experience databases added for true agentic learning!

