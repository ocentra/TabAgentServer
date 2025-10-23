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
//! # Examples
//!
//! ```
//! use storage::{StorageManager, Node, Chat, NodeId};
//! use serde_json::json;
//!
//! # fn main() -> Result<(), storage::DbError> {
//! // Create a new database
//! let storage = StorageManager::new("test_db")?;
//!
//! // Create and insert a chat node
//! let chat = Chat {
//!     id: NodeId::new("chat_123"),
//!     title: "My Chat".to_string(),
//!     topic: "Discussion".to_string(),
//!     created_at: 1697500000000,
//!     updated_at: 1697500000000,
//!     message_ids: vec![],
//!     summary_ids: vec![],
//!     embedding_id: None,
//!     metadata: json!({}),
//! };
//!
//! storage.insert_node(&Node::Chat(chat))?;
//!
//! // Retrieve the node
//! let retrieved = storage.get_node("chat_123")?;
//! assert!(retrieved.is_some());
//! # Ok(())
//! # }
//! ```

mod database_type;
pub mod coordinator;

use common::{models, DbResult};
use std::sync::Arc;

pub use database_type::{DatabaseType, TemperatureTier};
pub use coordinator::DatabaseCoordinator;

/// Manages all direct interactions with the sled database for CRUD operations.
///
/// `StorageManager` provides a safe, ergonomic interface to the underlying
/// `sled` key-value store. It manages three primary trees:
/// - `nodes`: Stores all node types (Chat, Message, Entity, etc.)
/// - `edges`: Stores all relationships between nodes
/// - `embeddings`: Stores vector embeddings for semantic search
///
/// All operations are atomic at the single-entity level. Multi-entity
/// transactions are supported through the exposed `db()` method.
pub struct StorageManager {
    db: sled::Db,
    nodes: sled::Tree,
    edges: sled::Tree,
    embeddings: sled::Tree,
    index_manager: Option<Arc<indexing::IndexManager>>,
    
    // Database type and tier (for multi-tier architecture)
    db_type: DatabaseType,
    tier: Option<TemperatureTier>,
}

impl StorageManager {
    /// Opens or creates a database at the specified path.
    ///
    /// This method will create the database directory if it doesn't exist.
    /// Three trees are initialized: `nodes`, `edges`, and `embeddings`.
    ///
    /// # Arguments
    ///
    /// * `path` - File system path where the database will be stored
    ///
    /// # Errors
    ///
    /// Returns `DbError::Sled` if:
    /// - The database path is invalid or inaccessible
    /// - The database is corrupted
    /// - Insufficient permissions to access the path
    ///
    /// # Examples
    ///
    /// ```
    /// use storage::StorageManager;
    ///
    /// # fn main() -> Result<(), storage::DbError> {
    /// let storage = StorageManager::new("my_database")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(path: &str) -> DbResult<Self> {
        let db = sled::open(path)?;
        let nodes = db.open_tree("nodes")?;
        let edges = db.open_tree("edges")?;
        let embeddings = db.open_tree("embeddings")?;

        Ok(Self {
            db,
            nodes,
            edges,
            embeddings,
            index_manager: None,
            db_type: DatabaseType::Conversations, // Default for backward compatibility
            tier: None,
        })
    }
    
    /// Opens a typed database at a specific temperature tier
    ///
    /// This is the recommended method for MIA's multi-tier architecture.
    ///
    /// # Arguments
    ///
    /// * `db_type` - Type of database (Conversations, Knowledge, etc.)
    /// * `tier` - Optional temperature tier (Active, Recent, Archive, etc.)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use storage::{StorageManager, DatabaseType, TemperatureTier};
    ///
    /// # fn main() -> Result<(), storage::DbError> {
    /// // Open conversations/active (HOT tier)
    /// let conv_active = StorageManager::open_typed(
    ///     DatabaseType::Conversations,
    ///     Some(TemperatureTier::Active)
    /// )?;
    ///
    /// // Open knowledge/stable
    /// let knowledge_stable = StorageManager::open_typed(
    ///     DatabaseType::Knowledge,
    ///     Some(TemperatureTier::Stable)
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn open_typed(
        db_type: DatabaseType,
        tier: Option<TemperatureTier>,
    ) -> DbResult<Self> {
        let path = db_type.get_path(tier);
        
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            common::platform::ensure_db_directory(parent)?;
        }
        
        let path_str = path
            .to_str()
            .ok_or_else(|| common::DbError::InvalidOperation(
                "Invalid UTF-8 in database path".to_string()
            ))?;
        
        let mut storage = Self::new(path_str)?;
        storage.db_type = db_type;
        storage.tier = tier;
        
        Ok(storage)
    }
    
    /// Get the database type of this storage manager
    pub fn db_type(&self) -> DatabaseType {
        self.db_type
    }
    
    /// Get the temperature tier of this storage manager
    pub fn tier(&self) -> Option<TemperatureTier> {
        self.tier
    }

    /// Opens or creates a database at the platform-specific default location.
    ///
    /// This uses platform-appropriate paths:
    /// - **Windows**: `%APPDATA%\TabAgent\db\{name}`
    /// - **macOS**: `~/Library/Application Support/TabAgent/db/{name}`
    /// - **Linux**: `~/.local/share/TabAgent/db/{name}`
    ///
    /// The directory will be created if it doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the database (e.g., "main", "cache")
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The default directory cannot be created
    /// - The database cannot be opened
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use storage::StorageManager;
    ///
    /// # fn main() -> Result<(), storage::DbError> {
    /// // Opens database at platform-specific location
    /// let storage = StorageManager::with_default_path("main")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_default_path(name: &str) -> DbResult<Self> {
        let db_path = common::platform::get_named_db_path(name);
        
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            common::platform::ensure_db_directory(parent)?;
        }
        
        let path_str = db_path.to_str()
            .ok_or_else(|| common::DbError::InvalidOperation(
                "Invalid UTF-8 in database path".to_string()
            ))?;
        
        Self::new(path_str)
    }

    /// Opens or creates a database at the default location with automatic indexing.
    ///
    /// Combines `with_default_path()` and `with_indexing()` for convenience.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the database (e.g., "main", "cache")
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use storage::StorageManager;
    ///
    /// # fn main() -> Result<(), storage::DbError> {
    /// // Production-ready database at platform-specific location with indexing
    /// let storage = StorageManager::with_default_path_and_indexing("main")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_default_path_and_indexing(name: &str) -> DbResult<Self> {
        let db_path = common::platform::get_named_db_path(name);
        
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            common::platform::ensure_db_directory(parent)?;
        }
        
        let path_str = db_path.to_str()
            .ok_or_else(|| common::DbError::InvalidOperation(
                "Invalid UTF-8 in database path".to_string()
            ))?;
        
        Self::with_indexing(path_str)
    }

    /// Opens or creates a database with automatic indexing enabled.
    ///
    /// This method creates a `StorageManager` with an integrated `IndexManager`
    /// that automatically maintains structural, graph, and vector indexes.
    ///
    /// # Arguments
    ///
    /// * `path` - File system path where the database will be stored
    ///
    /// # Errors
    ///
    /// Returns `DbError` if database or index initialization fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use storage::StorageManager;
    ///
    /// let storage = StorageManager::with_indexing("my_db")?;
    /// // Indexes are now automatically maintained!
    /// # Ok::<(), storage::DbError>(())
    /// ```
    pub fn with_indexing(path: &str) -> DbResult<Self> {
        let db = sled::open(path)?;
        let nodes = db.open_tree("nodes")?;
        let edges = db.open_tree("edges")?;
        let embeddings = db.open_tree("embeddings")?;
        
        // Initialize the index manager
        let index_manager = indexing::IndexManager::new(&db)?;
        
        Ok(Self {
            db,
            nodes,
            edges,
            embeddings,
            index_manager: Some(Arc::new(index_manager)),
            db_type: DatabaseType::Conversations, // Default for backward compatibility
            tier: None,
        })
    }
    
    /// Opens a typed database with indexing at a specific temperature tier
    ///
    /// Combines `open_typed()` and indexing setup.
    ///
    /// # Arguments
    ///
    /// * `db_type` - Type of database
    /// * `tier` - Optional temperature tier
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use storage::{StorageManager, DatabaseType, TemperatureTier};
    ///
    /// # fn main() -> Result<(), storage::DbError> {
    /// let storage = StorageManager::open_typed_with_indexing(
    ///     DatabaseType::Conversations,
    ///     Some(TemperatureTier::Active)
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn open_typed_with_indexing(
        db_type: DatabaseType,
        tier: Option<TemperatureTier>,
    ) -> DbResult<Self> {
        let path = db_type.get_path(tier);
        
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            common::platform::ensure_db_directory(parent)?;
        }
        
        let path_str = path
            .to_str()
            .ok_or_else(|| common::DbError::InvalidOperation(
                "Invalid UTF-8 in database path".to_string()
            ))?;
        
        let mut storage = Self::with_indexing(path_str)?;
        storage.db_type = db_type;
        storage.tier = tier;
        
        Ok(storage)
    }

    // --- Node Operations ---

    /// Retrieves a node by its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the node to retrieve
    ///
    /// # Returns
    ///
    /// - `Ok(Some(Node))` if the node exists
    /// - `Ok(None)` if the node does not exist
    ///
    /// # Errors
    ///
    /// Returns `DbError` if:
    /// - `DbError::Sled`: Database I/O error
    /// - `DbError::Serialization`: Data corruption or version mismatch
    ///
    /// # Examples
    ///
    /// ```
    /// use storage::StorageManager;
    ///
    /// # fn main() -> Result<(), storage::DbError> {
    /// let storage = StorageManager::new("test_db")?;
    /// let node = storage.get_node("node_123")?;
    ///
    /// if let Some(n) = node {
    ///     println!("Found node: {}", n.id());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_node(&self, id: &str) -> DbResult<Option<models::Node>> {
        // Input validation (RAG Rule 5.1)
        if id.is_empty() {
            return Err(common::DbError::InvalidOperation(
                "Node ID cannot be empty".to_string()
            ));
        }
        
        match self.nodes.get(id.as_bytes())? {
            Some(bytes) => {
                let node: models::Node = bincode::deserialize(&bytes)?;
                Ok(Some(node))
            }
            None => Ok(None),
        }
    }

    /// Inserts or updates a node in the database.
    ///
    /// This performs an "upsert" operation - if a node with the same ID
    /// already exists, it will be replaced.
    ///
    /// # Arguments
    ///
    /// * `node` - Reference to the node to insert or update
    ///
    /// # Errors
    ///
    /// Returns `DbError` if:
    /// - `DbError::Sled`: Database I/O error
    /// - `DbError::Serialization`: Failed to serialize the node
    ///
    /// # Examples
    ///
    /// ```
    /// use storage::{StorageManager, Node, Entity, NodeId};
    /// use serde_json::json;
    ///
    /// # fn main() -> Result<(), storage::DbError> {
    /// let storage = StorageManager::new("test_db")?;
    ///
    /// let entity = Entity {
    ///     id: NodeId::new("entity_456"),
    ///     label: "Rust".to_string(),
    ///     entity_type: "LANGUAGE".to_string(),
    ///     embedding_id: None,
    ///     metadata: json!({}),
    /// };
    ///
    /// storage.insert_node(&Node::Entity(entity))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn insert_node(&self, node: &models::Node) -> DbResult<()> {
        let id = node.id();
        
        // Input validation (RAG Rule 5.1)
        if id.as_str().is_empty() {
            return Err(common::DbError::InvalidOperation(
                "Node ID cannot be empty".to_string()
            ));
        }
        
        let bytes = bincode::serialize(node)?;
        self.nodes.insert(id.as_str().as_bytes(), bytes)?;
        
        // Update indexes if indexing is enabled
        if let Some(ref idx) = self.index_manager {
            idx.index_node(node)?;
        }
        
        Ok(())
    }

    /// Deletes a node from the database.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the node to delete
    ///
    /// # Returns
    ///
    /// - `Ok(Some(Node))` with the deleted node if it existed
    /// - `Ok(None)` if no node with the given ID was found
    ///
    /// # Errors
    ///
    /// Returns `DbError` if:
    /// - `DbError::Sled`: Database I/O error
    /// - `DbError::Serialization`: Data corruption in deleted node
    ///
    /// # Examples
    ///
    /// ```
    /// use storage::StorageManager;
    ///
    /// # fn main() -> Result<(), storage::DbError> {
    /// let storage = StorageManager::new("test_db")?;
    /// let deleted = storage.delete_node("node_123")?;
    ///
    /// if let Some(node) = deleted {
    ///     println!("Deleted node: {}", node.id());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn delete_node(&self, id: &str) -> DbResult<Option<models::Node>> {
        // Input validation (RAG Rule 5.1)
        if id.is_empty() {
            return Err(common::DbError::InvalidOperation(
                "Node ID cannot be empty".to_string()
            ));
        }
        
        match self.nodes.remove(id.as_bytes())? {
            Some(bytes) => {
                let node: models::Node = bincode::deserialize(&bytes)?;
                
                // Update indexes if indexing is enabled
                if let Some(ref idx) = self.index_manager {
                    idx.unindex_node(&node)?;
                }
                
                Ok(Some(node))
            }
            None => Ok(None),
        }
    }

    // --- Edge Operations ---

    /// Retrieves an edge by its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the edge to retrieve
    ///
    /// # Returns
    ///
    /// - `Ok(Some(Edge))` if the edge exists
    /// - `Ok(None)` if the edge does not exist
    ///
    /// # Errors
    ///
    /// Returns `DbError` if:
    /// - `DbError::Sled`: Database I/O error
    /// - `DbError::Serialization`: Data corruption or version mismatch
    pub fn get_edge(&self, id: &str) -> DbResult<Option<models::Edge>> {
        // Input validation (RAG Rule 5.1)
        if id.is_empty() {
            return Err(common::DbError::InvalidOperation(
                "Edge ID cannot be empty".to_string()
            ));
        }
        
        match self.edges.get(id.as_bytes())? {
            Some(bytes) => {
                let edge: models::Edge = bincode::deserialize(&bytes)?;
                Ok(Some(edge))
            }
            None => Ok(None),
        }
    }

    /// Inserts or updates an edge in the database.
    ///
    /// # Arguments
    ///
    /// * `edge` - Reference to the edge to insert or update
    ///
    /// # Errors
    ///
    /// Returns `DbError` if:
    /// - `DbError::Sled`: Database I/O error
    /// - `DbError::Serialization`: Failed to serialize the edge
    pub fn insert_edge(&self, edge: &models::Edge) -> DbResult<()> {
        // Input validation (RAG Rule 5.1)
        if edge.id.as_str().is_empty() {
            return Err(common::DbError::InvalidOperation(
                "Edge ID cannot be empty".to_string()
            ));
        }
        
        let bytes = bincode::serialize(edge)?;
        self.edges.insert(edge.id.as_str().as_bytes(), bytes)?;
        
        // Update indexes if indexing is enabled
        if let Some(ref idx) = self.index_manager {
            idx.index_edge(edge)?;
        }
        
        Ok(())
    }

    /// Deletes an edge from the database.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the edge to delete
    ///
    /// # Returns
    ///
    /// - `Ok(Some(Edge))` with the deleted edge if it existed
    /// - `Ok(None)` if no edge with the given ID was found
    ///
    /// # Errors
    ///
    /// Returns `DbError` if:
    /// - `DbError::Sled`: Database I/O error
    /// - `DbError::Serialization`: Data corruption in deleted edge
    pub fn delete_edge(&self, id: &str) -> DbResult<Option<models::Edge>> {
        // Input validation (RAG Rule 5.1)
        if id.is_empty() {
            return Err(common::DbError::InvalidOperation(
                "Edge ID cannot be empty".to_string()
            ));
        }
        
        match self.edges.remove(id.as_bytes())? {
            Some(bytes) => {
                let edge: models::Edge = bincode::deserialize(&bytes)?;
                
                // Update indexes if indexing is enabled
                if let Some(ref idx) = self.index_manager {
                    idx.unindex_edge(&edge)?;
                }
                
                Ok(Some(edge))
            }
            None => Ok(None),
        }
    }

    // --- Embedding Operations ---

    /// Retrieves an embedding by its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the embedding to retrieve
    ///
    /// # Returns
    ///
    /// - `Ok(Some(Embedding))` if the embedding exists
    /// - `Ok(None)` if the embedding does not exist
    ///
    /// # Errors
    ///
    /// Returns `DbError` if:
    /// - `DbError::Sled`: Database I/O error
    /// - `DbError::Serialization`: Data corruption or version mismatch
    pub fn get_embedding(&self, id: &str) -> DbResult<Option<models::Embedding>> {
        // Input validation (RAG Rule 5.1)
        if id.is_empty() {
            return Err(common::DbError::InvalidOperation(
                "Embedding ID cannot be empty".to_string()
            ));
        }
        
        match self.embeddings.get(id.as_bytes())? {
            Some(bytes) => {
                let embedding: models::Embedding = bincode::deserialize(&bytes)?;
                Ok(Some(embedding))
            }
            None => Ok(None),
        }
    }

    /// Retrieves an embedding for a given node.
    ///
    /// Loads the node, gets its embedding_id, then loads the embedding.
    pub fn get_embedding_by_node(&self, node_id: &str) -> DbResult<Option<models::Embedding>> {
        // Load the node
        let node = match self.get_node(node_id)? {
            Some(n) => n,
            None => return Ok(None),
        };
        
        // Get embedding_id from the node
        let embedding_id = match node {
            models::Node::Message(m) => m.embedding_id,
            models::Node::Summary(s) => s.embedding_id,
            models::Node::Entity(e) => e.embedding_id,
            models::Node::ScrapedPage(p) => p.embedding_id,
            models::Node::WebSearch(w) => w.embedding_id,
            _ => None,
        };
        
        // Load the embedding if ID exists
        match embedding_id {
            Some(id) => self.get_embedding(id.as_str()),
            None => Ok(None),
        }
    }

    /// Inserts or updates an embedding in the database.
    ///
    /// # Arguments
    ///
    /// * `embedding` - Reference to the embedding to insert or update
    ///
    /// # Errors
    ///
    /// Returns `DbError` if:
    /// - `DbError::Sled`: Database I/O error
    /// - `DbError::Serialization`: Failed to serialize the embedding
    ///
    /// # Examples
    ///
    /// ```
    /// use storage::{StorageManager, Embedding, EmbeddingId};
    ///
    /// # fn main() -> Result<(), storage::DbError> {
    /// let storage = StorageManager::new("test_db")?;
    ///
    /// let embedding = Embedding {
    ///     id: EmbeddingId::new("embed_001"),
    ///     vector: vec![0.1; 384], // 384-dimensional vector
    ///     model: "all-MiniLM-L6-v2".to_string(),
    /// };
    ///
    /// storage.insert_embedding(&embedding)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn insert_embedding(&self, embedding: &models::Embedding) -> DbResult<()> {
        // Input validation (RAG Rule 5.1)
        if embedding.id.as_str().is_empty() {
            return Err(common::DbError::InvalidOperation(
                "Embedding ID cannot be empty".to_string()
            ));
        }
        
        let bytes = bincode::serialize(embedding)?;
        self.embeddings.insert(embedding.id.as_str().as_bytes(), bytes)?;
        
        // Update indexes if indexing is enabled
        if let Some(ref idx) = self.index_manager {
            idx.index_embedding(embedding)?;
        }
        
        Ok(())
    }

    /// Deletes an embedding from the database.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the embedding to delete
    ///
    /// # Returns
    ///
    /// - `Ok(Some(Embedding))` with the deleted embedding if it existed
    /// - `Ok(None)` if no embedding with the given ID was found
    ///
    /// # Errors
    ///
    /// Returns `DbError` if:
    /// - `DbError::Sled`: Database I/O error
    /// - `DbError::Serialization`: Data corruption in deleted embedding
    pub fn delete_embedding(&self, id: &str) -> DbResult<Option<models::Embedding>> {
        // Input validation (RAG Rule 5.1)
        if id.is_empty() {
            return Err(common::DbError::InvalidOperation(
                "Embedding ID cannot be empty".to_string()
            ));
        }
        
        match self.embeddings.remove(id.as_bytes())? {
            Some(bytes) => {
                let embedding: models::Embedding = bincode::deserialize(&bytes)?;
                
                // Update indexes if indexing is enabled
                if let Some(ref idx) = self.index_manager {
                    idx.unindex_embedding(embedding.id.as_str())?;
                }
                
                Ok(Some(embedding))
            }
            None => Ok(None),
        }
    }

    // --- Utility Methods ---

    /// Provides direct access to the underlying sled database.
    ///
    /// This method is provided for higher-level crates (like `indexing` or `weaver`)
    /// to orchestrate multi-tree transactions and complex operations.
    ///
    /// # Examples
    ///
    /// ```
    /// use storage::StorageManager;
    ///
    /// # fn main() -> Result<(), storage::DbError> {
    /// let storage = StorageManager::new("test_db")?;
    /// let db = storage.db();
    /// // Use db for advanced operations like transactions
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn db(&self) -> &sled::Db {
        &self.db
    }

    /// Provides access to the index manager, if indexing is enabled.
    ///
    /// Returns `Some(&IndexManager)` if the database was created with `with_indexing()`,
    /// otherwise returns `None`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use storage::StorageManager;
    ///
    /// # fn main() -> Result<(), storage::DbError> {
    /// let storage = StorageManager::with_indexing("my_db")?;
    ///
    /// if let Some(idx) = storage.index_manager() {
    ///     // Query indexes
    ///     let chat_nodes = idx.get_nodes_by_property("node_type", "Chat")?;
    ///     println!("Found {} chats", chat_nodes.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn index_manager(&self) -> Option<&indexing::IndexManager> {
        self.index_manager.as_ref().map(|arc| arc.as_ref())
    }
}

// Re-export commonly used types for convenience
pub use common::{
    models::{
        Attachment, AudioTranscript, Bookmark, Chat, Edge, Embedding, Entity, ImageMetadata,
        Message, ModelInfo, Node, ScrapedPage, Summary, WebSearch,
    },
    DbError, EdgeId, EmbeddingId, NodeId,
};

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    /// Helper to create a temporary test database that auto-cleans on drop
    fn create_temp_db() -> (StorageManager, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let storage = StorageManager::new(db_path.to_str().unwrap())
            .expect("Failed to create storage");
        (storage, temp_dir)
    }

    fn create_test_chat(id: &str) -> Node {
        Node::Chat(Chat {
            id: NodeId::new(id),
            title: "Test Chat".to_string(),
            topic: "Testing".to_string(),
            created_at: 1697500000000,
            updated_at: 1697500000000,
            message_ids: vec![],
            summary_ids: vec![],
            embedding_id: None,
            metadata: json!({}),
        })
    }

    #[test]
    fn test_node_crud() -> DbResult<()> {
        let (storage, _temp) = create_temp_db();

        // Create and insert
        let node = create_test_chat("chat_001");
        storage.insert_node(&node)?;

        // Read
        let retrieved = storage.get_node("chat_001")?;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id().as_str(), "chat_001");

        // Update (re-insert with modified data)
        let mut updated_chat = create_test_chat("chat_001");
        if let Node::Chat(ref mut chat) = updated_chat {
            chat.title = "Updated Title".to_string();
        }
        storage.insert_node(&updated_chat)?;

        let retrieved = storage.get_node("chat_001")?;
        if let Some(Node::Chat(chat)) = retrieved {
            assert_eq!(chat.title, "Updated Title");
        } else {
            panic!("Expected Chat node");
        }

        // Delete
        let deleted = storage.delete_node("chat_001")?;
        assert!(deleted.is_some());

        let should_be_none = storage.get_node("chat_001")?;
        assert!(should_be_none.is_none());

        Ok(())
    }

    #[test]
    fn test_edge_operations() -> DbResult<()> {
        let (storage, _temp) = create_temp_db();

        let edge = Edge {
            id: EdgeId::new("edge_001"),
            from_node: NodeId::new("node_a"),
            to_node: NodeId::new("node_b"),
            edge_type: "CONTAINS".to_string(),
            created_at: 1697500000000,
            metadata: json!({}),
        };

        storage.insert_edge(&edge)?;

        let retrieved = storage.get_edge("edge_001")?;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().edge_type, "CONTAINS");

        let deleted = storage.delete_edge("edge_001")?;
        assert!(deleted.is_some());

        Ok(())
    }

    #[test]
    fn test_embedding_operations() -> DbResult<()> {
        let (storage, _temp) = create_temp_db();

        let embedding = Embedding {
            id: EmbeddingId::new("embed_001"),
            vector: vec![0.1, 0.2, 0.3],
            model: "test-model".to_string(),
        };

        storage.insert_embedding(&embedding)?;

        let retrieved = storage.get_embedding("embed_001")?;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().vector.len(), 3);

        let deleted = storage.delete_embedding("embed_001")?;
        assert!(deleted.is_some());

        Ok(())
    }

    #[test]
    fn test_nonexistent_entity() -> DbResult<()> {
        let (storage, _temp) = create_temp_db();

        let result = storage.get_node("does_not_exist")?;
        assert!(result.is_none());

        let result = storage.delete_node("does_not_exist")?;
        assert!(result.is_none());

        Ok(())
    }
    
    #[test]
    fn test_empty_id_validation() {
        let (storage, _temp) = create_temp_db();
        
        // Test empty node ID (RAG Rule 5.1)
        let result = storage.get_node("");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
        
        let result = storage.delete_node("");
        assert!(result.is_err());
        
        // Test empty edge ID
        let result = storage.get_edge("");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
        
        // Test empty embedding ID
        let result = storage.get_embedding("");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }
    
    #[test]
    fn test_insert_with_empty_id() -> DbResult<()> {
        let (storage, _temp) = create_temp_db();
        
        // Create node with empty ID (RAG Rule 5.1)
        let bad_chat = Node::Chat(Chat {
            id: NodeId::new(""),
            title: "Test".to_string(),
            topic: "Test".to_string(),
            created_at: 1697500000000,
            updated_at: 1697500000000,
            message_ids: vec![],
            summary_ids: vec![],
            embedding_id: None,
            metadata: json!({}),
        });
        
        let result = storage.insert_node(&bad_chat);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
        
        Ok(())
    }
}


