//! Core storage layer for the TabAgent system.
//!
//! This crate provides a multi-database orchestration system built on `libmdbx`.
//!
//! # Architecture
//!
//! The storage system is split into two layers:
//!
//! ## Generic Storage Infrastructure (this crate)
//! - **StorageRegistry**: Multi-database orchestrator that manages named databases
//! - **StorageManager**: Generic CRUD operations over a single database
//! - **MdbxEngine**: Low-level libmdbx interface with zero-copy capabilities
//! - **DbConfig**: Configuration for database instances
//!
//! ## Domain-Specific Logic (mia-storage crate)
//! - DatabaseCoordinator: Manages MIA's 7-database cognitive architecture
//! - Domain managers: ConversationManager, KnowledgeManager, etc.
//! - Indexing integration: Updates search indexes when data changes
//!
//! # Using StorageRegistry
//!
//! ```no_run
//! use storage::{StorageRegistry, DbConfig};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let registry = StorageRegistry::new("/data");
//!
//! // Register a database
//! let config = DbConfig::new("knowledge_graph.mdbx")
//!     .with_collection("nodes")
//!     .with_collection("edges");
//! registry.add_storage("knowledge_graph", config)?;
//!
//! // Perform CRUD operations
//! registry.insert("knowledge_graph", "nodes", b"key1", b"value1")?;
//! let value = registry.get("knowledge_graph", "nodes", b"key1")?;
//! # Ok(())
//! # }
//! ```
//!
//! # Concurrency Safety
//!
//! `StorageRegistry` and `StorageManager` are thread-safe and can be safely shared
//! across multiple threads. The underlying libmdbx database provides MVCC concurrency,
//! so multiple threads can perform operations concurrently without additional
//! synchronization.

// Core infrastructure modules
pub mod config;
pub mod registry;
pub mod engine;
pub mod zero_copy_ffi;
mod storage_manager;

// Domain-specific modules (will be moved to mia-storage crate)
mod archived_node;
pub mod conversations;
pub mod coordinator;
mod database_type;
pub mod embeddings;
pub mod experience;
pub mod knowledge;
pub mod summaries;
pub mod grpc_server;
pub mod database_client;
pub mod time_queries;
pub mod time_scope;
pub mod tool_results;
pub mod traits;

// Core infrastructure exports
pub use config::DbConfig;
pub use registry::{StorageRegistry, RegistryError};
pub use engine::{StorageEngine, StorageTransaction, MdbxEngine, MdbxEngineError};
pub use storage_manager::{StorageManager, DefaultStorageManager};

// Domain-specific exports (will be moved to mia-storage)
pub use archived_node::{ArchivedNodeRef, ArchivedEdgeRef, ArchivedEmbeddingRef};
pub use database_client::DatabaseClient;
pub use coordinator::DatabaseCoordinator;
pub use database_type::{DatabaseType, TemperatureTier};
pub use time_scope::TimeScope;

// Re-export commonly used types for convenience
pub use common::{
    models::{
        Attachment, AudioTranscript, Bookmark, Chat, Edge, Embedding, Entity, ImageMetadata,
        Message, ModelInfo, Node, ScrapedPage, Summary, WebSearch,
    },
    DbError, EdgeId, EmbeddingId, NodeId,
};
