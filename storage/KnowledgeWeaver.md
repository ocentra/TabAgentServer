```markdown
# Specification: Knowledge Weaver (`weaver` crate)

## 1. Objective & Core Principles

The `weaver` crate is the **autonomous cognitive engine** of the database. It transforms the database from a passive store of information into an active, self-organizing knowledge graph. Its purpose is to run in the background, observing changes to the data and asynchronously enriching it by creating new insights, summaries, and relationships. This is the "subconscious" of the "humane brain."

This component is governed by the following core principles:

*   **Asynchronous & Event-Driven:** The Weaver operates entirely in the background and does not block incoming API requests. It is driven by an internal event queue, reacting to data changes (`NodeCreated`, `EdgeDeleted`, etc.) as they happen.
*   **Autonomous Enrichment:** The Weaver is responsible for the tasks that a human brain performs subconsciously: making connections, identifying patterns, and consolidating memories. This includes semantic indexing, entity linking, summarization, and discovering associative links.
*   **Decoupled & Resilient:** The Weaver is a separate system from the core query path. If a Weaver task fails, it must not impact the database's ability to serve read/write queries. Failed tasks should be logged and potentially retried.
*   **Configurable:** The behavior of the Weaver modules (e.g., summarization frequency, entity extraction models) should be configurable to adapt to different workloads and performance requirements.

## 2. Architectural Analysis & Rationale (The "Why")

The decision to create a dedicated, asynchronous Weaver engine is the final and most critical step in realizing the "humane brain" vision. A database that only stores what it's explicitly told is just a filing cabinet. A database that *thinks* about the data it holds is a true knowledge engine.

### 2.1. Findings from Current System (IndexedDB)

*   **Critical Limitations to Solve:** The current system has **no autonomous enrichment capabilities**.
    *   **Summarization:** While the `idbSummary.ts` class exists, the creation of a summary is a manual process that must be triggered explicitly by the application.
    *   **Entity Linking:** The `KnowledgeGraphEdge` class allows for the creation of `MENTIONS` edges, but there is no automated system to perform NER and create these links. An agent would have to manually parse a message, identify entities, and create the edges itself.
    *   **Associative Connections:** There is absolutely no mechanism for discovering and creating links between semantically similar but structurally disconnected pieces of information across different chats.

Solving these limitations is the entire purpose of the Weaver. It offloads this complex, computationally expensive cognitive work from the application/agent layer into a highly optimized, persistent, background process.

### 2.2. Findings from Reference Systems

This architectural pattern is common in large-scale data processing and AI systems, though not always within the database itself.

*   **Data Pipelines (e.g., Kafka, Flink):** The event-driven model is the cornerstone of modern data streaming. A new piece of data (an "event") is published to a topic, and multiple independent "consumers" can react to it. Our internal event queue and Weaver modules are a microcosm of this powerful paradigm.
*   **AI/ML Systems:** In production RAG systems, the process of "ingestion"—chunking documents, generating embeddings, extracting metadata—is an asynchronous pipeline that runs separately from the "retrieval" process. The Weaver formalizes this ingestion and enrichment pipeline as a core, perpetual feature of the database itself.

### 2.3. Synthesis & Final Architectural Decision

The **Asynchronous, Event-Driven "Knowledge Weaver"** is the definitive architecture for this crate.

This design is a direct solution to the static, passive nature of the current database. By making data enrichment an autonomous, built-in feature, we dramatically increase the intelligence and connectivity of the knowledge graph without burdening the client application.

This architecture is what enables the high-level, "human-like" queries defined in the `query` crate. The `QueryManager` can ask "what is the topic of this chat?" or "find me conversations related to this one" *because* the Weaver has already done the background work of determining the topic and creating the associative links.

## 3. Detailed Rust Implementation Blueprint (The "What")

### 3.1. Core Architecture: Event Queue & Worker Pool

The Weaver will be built on an asynchronous runtime like `tokio`.

File: `weaver/src/lib.rs`
```rust
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use storage::models::NodeId;
use std::sync::Arc;
// ... other imports for DB managers

/// The central hub for the Knowledge Weaver.
/// It spawns worker tasks and manages the event queue.
pub struct Weaver {
    event_sender: UnboundedSender<WeaverEvent>,
}

/// A context struct holding shared, thread-safe handles to all necessary
/// database and model managers.
#[derive(Clone)]
pub struct WeaverContext {
    // Arc allows safe sharing of managers across async tasks
    storage: Arc<StorageManager>,
    indexing: Arc<IndexManager>,
    query: Arc<QueryManager>,
    // A trait object for the model interface
    // model_manager: Arc<dyn ModelManager + Send + Sync>,
}

impl Weaver {
    /// Initializes the Weaver, spawning a pool of worker tasks.
    pub fn new(context: WeaverContext) -> Self {
        let (event_sender, mut event_receiver) = unbounded_channel();

        // Spawn a pool of worker tasks
        for i in 0..num_cpus::get() { // Example: one worker per CPU core
            let worker_context = context.clone();
            tokio::spawn(async move {
                log::info!("Weaver worker {} started", i);
                while let Some(event) = event_receiver.recv().await {
                    Self::process_event(event, &worker_context).await;
                }
            });
        }

        Self { event_sender }
    }

    /// The public API for submitting an event to the Weaver.
    /// This is a non-blocking operation.
    pub fn submit_event(&self, event: WeaverEvent) {
        if let Err(e) = self.event_sender.send(event) {
            log::error!("Failed to send event to Weaver queue: {}", e);
        }
    }

    /// The main event dispatch logic.
    async fn process_event(event: WeaverEvent, context: &WeaverContext) {
        let result = match event {
            WeaverEvent::NodeCreated(node_id) => {
                // Trigger multiple modules for a new node
                // These can run in parallel
                tokio::try_join!(
                    semantic_indexer::on_node_created(context.clone(), node_id.clone()),
                    entity_linker::on_node_created(context.clone(), node_id.clone()),
                    associative_linker::on_node_created(context.clone(), node_id.clone())
                )
            }
            WeaverEvent::ChatUpdated(chat_id) => {
                // Trigger topic modeling and summarization
                tokio::try_join!(
                    topic_modeler::on_chat_updated(context.clone(), chat_id.clone()),
                    summarizer::on_chat_updated(context.clone(), chat_id.clone())
                )
            }
        };

        if let Err(e) = result {
            log::error!("Error processing Weaver event: {}", e);
        }
    }
}

/// Defines the types of events the Weaver can react to.
#[derive(Debug, Clone)]
pub enum WeaverEvent {
    NodeCreated(NodeId),
    ChatUpdated(NodeId), // e.g., after a certain number of new messages
}
```

### 3.2. Weaver Modules

Each module will be a separate Rust file responsible for a specific cognitive task.

#### 3.2.1. Semantic Indexer

File: `weaver/src/semantic_indexer.rs`
*   **Trigger:** `WeaverEvent::NodeCreated`
*   **Logic:**
    1.  Load the `Node` from the `storage` crate.
    2.  Check if the node type is one that requires an embedding (e.g., `Message`, `Summary`).
    3.  Check if it already has an `embedding_id`. If so, do nothing.
    4.  Extract the text content from the node.
    5.  **Call an external or compiled-in model (e.g., via ONNX, Llama.cpp bindings) to generate the vector embedding.**
    6.  Create a new `Embedding` object.
    7.  Start a transaction to:
        *   Save the new `Embedding` via the `storage` crate.
        *   Update the original `Node` to include the new `embedding_id`.
        *   Update the HNSW index via the `indexing` crate.
    8.  Commit the transaction.

#### 3.2.2. Entity Linker

File: `weaver/src/entity_linker.rs`
*   **Trigger:** `WeaverEvent::NodeCreated`
*   **Logic:**
    1.  Load the `Node` and check if it has text content.
    2.  **Call an NER model to extract entities (e.g., "Project Phoenix [PROJECT]").**
    3.  For each extracted entity:
        *   Query the `indexing` crate for an `Entity` node with the same label and type.
        *   If it doesn't exist, create a new `Entity` node via the `storage` crate.
        *   Create a new `Edge` (`from_node`: message ID, `to_node`: entity ID, `edge_type`: "MENTIONS").
        *   Save the new `Edge` via the `storage` crate (within a transaction that also updates the graph indexes).

#### 3.2.3. Associative Linker

File: `weaver/src/associative_linker.rs`
*   **Trigger:** `WeaverEvent::NodeCreated` (after the semantic indexer has run).
*   **Logic:**
    1.  Load the newly created `Node` and its `Embedding`.
    2.  Perform a broad, efficient vector search via the `indexing` crate for the top ~5 most similar nodes created in the last N days (e.g., 30 days).
    3.  For each result that exceeds a high similarity threshold (e.g., > 0.9) and is not already directly connected:
        *   Create a new `Edge` (`from_node`: new node ID, `to_node`: similar node ID, `edge_type`: "IS_SEMANTICALLY_SIMILAR_TO").
        *   Save the new `Edge`.

#### 3.2.4. Summarizer

File: `weaver/src/summarizer.rs`
*   **Trigger:** `WeaverEvent::ChatUpdated`
*   **Logic:**
    1.  Load the `Chat` node.
    2.  Check if the number of messages since the last `Summary` exceeds a configurable threshold (e.g., 20).
    3.  If it does, fetch the content of the latest messages.
    4.  **Call an LLM with a summarization prompt.**
    5.  Create a new `Summary` node via the `storage` crate, linking it to the chat and the messages it covers.

## 4. Implementation Plan & Checklist

*   [ ] **Project Setup:**
    *   [ ] Create the `weaver` crate.
    *   [ ] Add dependencies on `tokio`, `storage`, `indexing`, and `query` crates.
*   [ ] **Core Engine:**
    *   [ ] Implement the `WeaverEvent` enum.
    *   [ ] Implement the `Weaver` struct with the `tokio` MPSC channel and worker pool.
    *   [ ] Implement the `submit_event` public API.
    *   [ ] Modify the `storage` crate's write methods to call `submit_event` upon successful transaction commit.
*   [ ] **Model Interface:**
    *   [ ] Design a `ModelManager` trait that abstracts the calls to embedding, NER, and LLM models. This is a critical prerequisite for implementing the modules.
*   [ ] **Module Implementation:**
    *   [ ] Implement the `semantic_indexer` module.
    *   [ ] Implement the `entity_linker` module.
    *   [ ] Implement the `associative_linker` module.
    *   [ ] Implement the `summarizer` module.
*   [ ] **Configuration:**
    *   [ ] Create a configuration struct to manage Weaver settings (e.g., summarization threshold, similarity thresholds, worker count).
*   [ ] **Integration Tests:**
    *   [ ] Write a test that creates a new `Message` and then polls the database until the message's `embedding_id` is populated, verifying the Semantic Indexer worked.
    *   [ ] Write a test that creates a `Message` containing a unique entity name, and then verifies that a new `Entity` node and a `MENTIONS` edge were created in the background.

## 5. Open Questions & Decisions Needed

*   **Model Integration Strategy:** The single biggest dependency for this crate is the interface to the AI models (embedding, NER, LLM). A clean, trait-based interface must be designed to abstract away the specific model implementation (e.g., ONNX, Llama.cpp). This is a critical design task that must be completed before module implementation.
*   **Resource Management:** The number of worker threads and the rate of event processing need to be carefully managed to avoid overwhelming the system's CPU and memory, especially during bulk data imports. The worker pool size should be configurable.
*   **Error Handling & Retries:** A robust strategy for handling failed Weaver tasks is required. A simple "log and drop" strategy is sufficient for MVP, but a more advanced system with a dead-letter queue and retry logic should be considered for future versions.
*   **Event Granularity:** The initial `WeaverEvent` enum is simple. More granular events (e.g., `NodePropertyUpdated`, `EdgeCreated`) may be needed in the future to trigger more specific Weaver modules, but `NodeCreated` and `ChatUpdated` are sufficient for the initial set of tasks.
```