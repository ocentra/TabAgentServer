//! Knowledge Weaver - Autonomous knowledge graph enrichment engine.
//!
//! The Knowledge Weaver is an event-driven system that listens for changes to the database
//! and automatically enriches the knowledge graph by:
//!
//! - Generating vector embeddings for semantic search
//! - Extracting and linking entities across conversations
//! - Creating associative links between similar content
//! - Summarizing conversations
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │      Storage Layer (CRUD Events)        │
//! └──────────────┬──────────────────────────┘
//!                │ emit WeaverEvent
//!                ▼
//! ┌─────────────────────────────────────────┐
//! │   Weaver (Event Queue + Worker Pool)    │
//! │  ┌─────────────────────────────────┐   │
//! │  │  async-channel MPSC Queue       │   │
//! │  └─────────────────────────────────┘   │
//! │  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐  │
//! │  │Worker│ │Worker│ │Worker│ │Worker│  │
//! │  └──┬───┘ └──┬───┘ └──┬───┘ └──┬───┘  │
//! └─────┼────────┼────────┼────────┼───────┘
//!       │        │        │        │
//!       ▼        ▼        ▼        ▼
//! ┌─────────────────────────────────────────┐
//! │      Enrichment Modules                  │
//! │  • SemanticIndexer                       │
//! │  • EntityLinker                          │
//! │  • AssociativeLinker                     │
//! │  • Summarizer                            │
//! └──────────────┬──────────────────────────┘
//!                │ DB writes
//!                ▼
//! ┌─────────────────────────────────────────┐
//! │    Storage + Indexing Layers            │
//! └─────────────────────────────────────────┘
//! ```
//!
//! # Examples
//!
//! ```
//! use weaver::{Weaver, WeaverContext, WeaverEvent};
//! use weaver::ml_bridge::MockMlBridge;
//! use std::sync::Arc;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Set up database layers (simplified)
//! # use storage::StorageManager;
//! # use indexing::IndexManager;
//! # let storage = StorageManager::new("test_db")?;
//! # let indexing = IndexManager::new(storage.db())?;
//!
//! // Create weaver context
//! let context = WeaverContext::new(
//!     Arc::new(storage),
//!     Arc::new(indexing),
//!     Arc::new(MockMlBridge), // Use real ML bridge in production
//! );
//!
//! // Start the weaver
//! let weaver = Weaver::new(context).await?;
//!
//! // Submit events
//! weaver.submit_event(WeaverEvent::NodeCreated {
//!     node_id: "msg_123".to_string(),
//!     node_type: "Message".to_string(),
//! }).await?;
//! # Ok(())
//! # }
//! ```

pub mod events;
pub mod ml_bridge;
mod modules;

pub use events::WeaverEvent;
pub use ml_bridge::{MlBridge, MockMlBridge};

use common::DbError;
use storage::{StorageManager, DatabaseCoordinator};
use std::sync::Arc;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;
use anyhow::Result;

/// Error type for Weaver operations.
#[derive(Debug, thiserror::Error)]
pub enum WeaverError {
    /// Database error
    #[error("Database error: {0}")]
    Database(#[from] DbError),
    
    /// ML bridge error
    #[error("ML inference error: {0}")]
    MlInference(String),
    
    /// Event processing error
    #[error("Event processing error: {0}")]
    EventProcessing(String),
    
    /// Weaver is shutting down
    #[error("Weaver is shutting down")]
    ShuttingDown,
    
    /// Other errors
    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

/// Result type for Weaver operations.
pub type WeaverResult<T> = Result<T, WeaverError>;

/// Shared context for all weaver workers.
///
/// This contains thread-safe handles to all necessary components.
#[derive(Clone)]
pub struct WeaverContext {
    /// Conversations database for chat-related data
    pub conversations_db: Arc<StorageManager>,
    
    /// Knowledge database for graph-related data
    pub knowledge_db: Arc<StorageManager>,
    
    /// Database coordinator for multi-tier operations
    pub coordinator: Arc<DatabaseCoordinator>,
    
    /// Conversations index manager for search operations
    pub conversations_index: Arc<indexing::IndexManager>,
    
    /// Knowledge index manager for search operations
    pub knowledge_index: Arc<indexing::IndexManager>,
    
    /// ML bridge for model inference
    pub ml_bridge: Arc<dyn MlBridge>,
}

impl WeaverContext {
    /// Creates a new weaver context.
    pub fn new(
        conversations_db: Arc<StorageManager>,
        knowledge_db: Arc<StorageManager>,
        coordinator: Arc<DatabaseCoordinator>,
        conversations_index: Arc<indexing::IndexManager>,
        knowledge_index: Arc<indexing::IndexManager>,
        ml_bridge: Arc<dyn MlBridge>,
    ) -> WeaverResult<Self> {
        Ok(Self {
            conversations_db,
            knowledge_db,
            coordinator,
            conversations_index,
            knowledge_index,
            ml_bridge,
        })
    }
    
}

/// The Knowledge Weaver engine.
///
/// Manages an event queue and worker pool for autonomous knowledge enrichment.
pub struct Weaver {
    /// Sender for submitting events to the queue
    event_sender: UnboundedSender<WeaverEvent>,
    
    /// Worker task handles
    worker_handles: Vec<JoinHandle<()>>,
    
    /// Shared context (kept for potential future direct access)
    _context: WeaverContext,
}

impl Weaver {
    /// Creates and starts a new Knowledge Weaver.
    ///
    /// This spawns a pool of worker tasks that listen for events on the queue.
    ///
    /// # Arguments
    ///
    /// * `context` - Shared context with database and ML components
    ///
    /// # Examples
    ///
    /// ```
    /// # use weaver::{Weaver, WeaverContext, ml_bridge::MockMlBridge};
    /// # use storage::{StorageManager, DatabaseCoordinator};
    /// # use indexing::IndexManager;
    /// # use std::sync::Arc;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let conversations_db = Arc::new(StorageManager::new("test_conversations")?);
    /// # let knowledge_db = Arc::new(StorageManager::new("test_knowledge")?);
    /// # let coordinator = Arc::new(DatabaseCoordinator::new()?);
    /// # let conversations_index = Arc::new(IndexManager::new("test_conversations")?);
    /// # let knowledge_index = Arc::new(IndexManager::new("test_knowledge")?);
    /// # let ml_bridge = Arc::new(MockMlBridge);
    /// let context = WeaverContext::new(
    ///     conversations_db,
    ///     knowledge_db,
    ///     coordinator,
    ///     conversations_index,
    ///     knowledge_index,
    ///     ml_bridge,
    /// )?;
    ///
    /// let weaver = Weaver::new(context).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(context: WeaverContext) -> WeaverResult<Self> {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        // Determine worker pool size (use number of CPUs, min 2, max 8)
        let num_workers = num_cpus::get().clamp(2, 8);
        log::info!("Starting Knowledge Weaver with {} workers", num_workers);
        
        let worker_handles = Self::spawn_workers(num_workers, event_receiver, context.clone());
        
        Ok(Self {
            event_sender,
            worker_handles,
            _context: context,
        })
    }

    /// Spawns worker tasks that process events from the queue.
    fn spawn_workers(
        count: usize,
        mut event_receiver: UnboundedReceiver<WeaverEvent>,
        context: WeaverContext,
    ) -> Vec<JoinHandle<()>> {
        let mut handles = Vec::new();
        
        // Spawn a single dispatcher task that receives events and spawns processing tasks
        let dispatcher_handle = tokio::spawn(async move {
            log::info!("Weaver dispatcher started with {} worker capacity", count);
            
            while let Some(event) = event_receiver.recv().await {
                log::debug!("Dispatcher received: {}", event.description());
                
                let worker_context = context.clone();
                // Spawn a task to process this event
                tokio::spawn(async move {
                    if let Err(e) = Self::process_event(&event, &worker_context).await {
                        log::error!(
                            "Failed to process event: {}. Error: {}",
                            event.description(),
                            e
                        );
                    }
                });
            }
            
            log::info!("Weaver dispatcher stopped");
        });
        
        handles.push(dispatcher_handle);
        handles
    }

    /// Submits an event to the Weaver for processing.
    ///
    /// This is a non-blocking operation that adds the event to the queue.
    ///
    /// # Arguments
    ///
    /// * `event` - The event to process
    ///
    /// # Errors
    ///
    /// Returns an error if the Weaver is shutting down and can no longer accept events.
    pub async fn submit_event(&self, event: WeaverEvent) -> WeaverResult<()> {
        self.event_sender
            .send(event)
            .map_err(|_| WeaverError::ShuttingDown)?;
        Ok(())
    }

    /// Processes a single event by dispatching to appropriate modules.
    async fn process_event(event: &WeaverEvent, context: &WeaverContext) -> WeaverResult<()> {
        use modules::*;
        
        match event {
            WeaverEvent::NodeCreated { node_id, node_type } => {
                // Run multiple enrichment tasks concurrently
                let (sem_result, ent_result, assoc_result) = tokio::join!(
                    semantic_indexer::on_node_created(context, node_id.as_str(), node_type.as_str()),
                    entity_linker::on_node_created(context, node_id.as_str(), node_type.as_str()),
                    associative_linker::on_node_created(context, node_id.as_str(), node_type.as_str()),
                );
                
                // Log any errors but don't fail the entire event
                if let Err(e) = sem_result {
                    log::warn!("Semantic indexer failed for {}: {}", node_id, e);
                }
                if let Err(e) = ent_result {
                    log::warn!("Entity linker failed for {}: {}", node_id, e);
                }
                if let Err(e) = assoc_result {
                    log::warn!("Associative linker failed for {}: {}", node_id, e);
                }
            }
            
            WeaverEvent::NodeUpdated { node_id, node_type } => {
                // Re-index if content changed
                semantic_indexer::on_node_updated(context, node_id.as_str(), node_type.as_str()).await?;
            }
            
            WeaverEvent::ChatUpdated { chat_id, messages_since_summary } => {
                // Trigger summarization if threshold reached
                if *messages_since_summary >= 20 {
                    summarizer::on_chat_updated(context, chat_id.as_str()).await?;
                }
            }
            
            WeaverEvent::BatchMessagesAdded { chat_id, message_ids } => {
                // Process messages in batch for efficiency
                log::info!("Batch processing {} messages for chat {}", message_ids.len(), chat_id);
                // TODO: Implement batch processing optimization
            }
            
            WeaverEvent::EdgeCreated { .. } => {
                // Currently no processing needed for edge creation
                // Future: Could trigger graph analysis updates
            }
        }
        
        Ok(())
    }

    /// Gracefully shuts down the Weaver.
    ///
    /// Waits for all workers to finish processing current events.
    pub async fn shutdown(self) -> WeaverResult<()> {
        log::info!("Shutting down Knowledge Weaver...");
        
        // Drop the sender to signal workers to stop
        drop(self.event_sender);
        
        // Wait for all workers to finish
        for handle in self.worker_handles {
            handle.await.map_err(|e| {
                WeaverError::EventProcessing(format!("Worker panic: {}", e))
            })?;
        }
        
        log::info!("Knowledge Weaver shutdown complete");
        Ok(())
    }

    /// Returns statistics about the Weaver's operation.
    pub fn stats(&self) -> WeaverStats {
        WeaverStats {
            active_workers: self.worker_handles.len(),
            queue_size: 0, // TODO: Track queue size
        }
    }
}

/// Statistics about Weaver operation.
#[derive(Debug, Clone)]
pub struct WeaverStats {
    /// Number of active worker tasks
    pub active_workers: usize,
    
    /// Current size of the event queue
    pub queue_size: usize,
}

// Note: We need num_cpus for worker pool sizing
use num_cpus;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use storage::{StorageManager, DatabaseCoordinator};
    use indexing;

    async fn create_test_context() -> (TempDir, Arc<StorageManager>, WeaverContext) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let storage = Arc::new(
            StorageManager::with_indexing(db_path.to_str().unwrap())
                .expect("Failed to create storage")
        );
        
        // Create a mock DatabaseCoordinator for testing
        let coordinator = Arc::new(
            DatabaseCoordinator::new()
                .expect("Failed to create DatabaseCoordinator")
        );
        
        // Extract index manager from storage
        let index = Arc::new(
            indexing::IndexManager::new(db_path.to_str().unwrap())
                .expect("Failed to create IndexManager")
        );
        
        // Create a mock ML bridge
        let ml_bridge = Arc::new(MockMlBridge);
        
        // Create the context with all required parameters
        let context = WeaverContext::new(
            Arc::clone(&storage),
            Arc::clone(&storage),
            coordinator,
            Arc::clone(&index),
            Arc::clone(&index),
            ml_bridge,
        ).expect("Failed to create WeaverContext");
        
        (temp_dir, storage, context)
    }

    #[tokio::test]
    async fn test_weaver_initialization() {
        let (_temp, _storage, context) = create_test_context().await;
        let weaver = Weaver::new(context).await.unwrap();
        
        let stats = weaver.stats();
        assert!(stats.active_workers >= 1);
        
        weaver.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_submit_event() {
        let (_temp, _storage, context) = create_test_context().await;
        let weaver = Weaver::new(context).await.unwrap();
        
        let event = WeaverEvent::NodeCreated {
            node_id: "test_node".to_string().into(),
            node_type: "Message".to_string(),
        };
        
        weaver.submit_event(event).await.unwrap();
        
        // Give workers time to process
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        weaver.shutdown().await.unwrap();
    }
}
