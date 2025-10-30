//! Core storage layer for the TabAgent embedded database.
//!
//! This crate provides a safe, transactional interface for CRUD operations on
//! nodes, edges, and embeddings using the `sled` embedded database engine.
//!
//! # Architecture
//!
//! The storage layer implements the Hybrid Schema Model:
//! - **Strongly-typed core fields** enable high-performance indexing
//! - **Flexible metadata fields** provide extensibility
//! - **Binary serialization** with `bincode` for performance
//!
//! # Concurrency Safety
//!
//! The `StorageManager` is designed to be thread-safe and can be safely shared
//! across multiple threads. The underlying sled database is thread-safe, so
//! multiple threads can perform operations concurrently without additional
//! synchronization.
//!
//! For sharing across threads, wrap the StorageManager in an `Arc`:
//!
//! ```no_run
//! use storage::StorageManager;
//! use std::sync::Arc;
//! use std::thread;
//!
//! # fn main() -> Result<(), storage::DbError> {
//! let storage = Arc::new(StorageManager::new("my_database")?);
//!
//! // Share across threads
//! let storage_clone = Arc::clone(&storage);
//! let handle = thread::spawn(move || {
//!     // Safe to use storage_clone here
//!     # Ok::<(), storage::DbError>(())
//! });
//!
//! // Safe to use storage here as well
//! handle.join().unwrap()?;
//! # Ok(())
//! # }
//! ```

pub mod conversations;
pub mod coordinator;
mod database_type;
pub mod embeddings;
pub mod experience;
pub mod knowledge;
mod storage_manager;
pub mod summaries;
pub mod grpc_server;
pub mod database_client;

// Re-export for convenience
pub use database_client::DatabaseClient;
pub mod time_queries;
pub mod time_scope;
pub mod tool_results;
pub mod traits;

pub use coordinator::DatabaseCoordinator;
pub use database_type::{DatabaseType, TemperatureTier};
pub use storage_manager::StorageManager;
pub use time_scope::TimeScope;

// Re-export commonly used types for convenience
pub use common::{
    models::{
        Attachment, AudioTranscript, Bookmark, Chat, Edge, Embedding, Entity, ImageMetadata,
        Message, ModelInfo, Node, ScrapedPage, Summary, WebSearch,
    },
    DbError, EdgeId, EmbeddingId, NodeId,
};
