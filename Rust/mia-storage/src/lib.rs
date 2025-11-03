//! MIA Storage - Domain-aware storage layer for TabAgent's cognitive architecture
//!
//! This crate implements MIA's 7-database cognitive architecture on top of the
//! generic `storage` crate infrastructure.
//!
//! # Architecture
//!
//! MIA's storage is organized into 7 specialized databases:
//! 1. **Conversations** - Chat messages and dialogue
//! 2. **Knowledge** - Long-term knowledge graph (nodes + edges)
//! 3. **Embeddings** - Vector embeddings for semantic search
//! 4. **Experiences** - Episodic memories and experiences
//! 5. **Summaries** - Compressed knowledge summaries
//! 6. **ToolResults** - Tool execution results and web searches
//! 7. **TimeQueries** - Time-based query patterns
//!
//! Each database supports temperature-based tiering:
//! - **Active**: Hot data, frequently accessed
//! - **Recent**: Warm data, occasionally accessed
//! - **Archive**: Cold data, rarely accessed
//!
//! # Domain-Specific Managers
//!
//! - `DatabaseCoordinator` - Manages all 7 databases and their tiers
//! - `ConversationManager` - Handles conversation CRUD operations
//! - `KnowledgeManager` - Manages knowledge graph nodes and edges
//! - `EmbeddingManager` - Manages vector embeddings
//! - `ExperienceManager` - Manages episodic memories
//!
//! # Indexing Integration
//!
//! This crate integrates with the `indexing` crate to automatically update
//! search indexes (structural, graph, vector) when data changes.

// Re-export EVERYTHING from storage for now
// This makes mia-storage a convenient facade while we keep implementation in storage
pub use storage::{
    // Core infrastructure
    StorageRegistry, DbConfig, RegistryError, StorageManager,
    StorageEngine, MdbxEngine,
    
    // Domain types
    DatabaseType, TemperatureTier, TimeScope,
    
    // Managers
    DatabaseCoordinator,
    DatabaseClient,
    
    // Zero-copy accessors
    ArchivedNodeRef, ArchivedEdgeRef, ArchivedEmbeddingRef,
    
    // Re-export for convenience
    DefaultStorageManager,
};

// Re-export common types for convenience
pub use common::{
    models::{
        Attachment, AudioTranscript, Bookmark, Chat, Edge, Embedding, Entity, ImageMetadata,
        Message, ModelInfo, Node, ScrapedPage, Summary, WebSearch,
    },
    DbError, EdgeId, EmbeddingId, NodeId,
};

