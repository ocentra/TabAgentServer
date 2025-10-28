//! Core storage manager for the TabAgent embedded database.
//!
//! This module provides the `StorageManager` which handles all direct interactions
//! with the sled database for CRUD operations.

use crate::database_type::{DatabaseType, TemperatureTier};
use common::DbResult;
use std::sync::Arc;

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
#[derive(Clone)]
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
    pub fn open_typed(db_type: DatabaseType, tier: Option<TemperatureTier>) -> DbResult<Self> {
        let path = db_type.get_path(tier);

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            common::platform::ensure_db_directory(parent)?;
        }

        let path_str = path.to_str().ok_or_else(|| {
            common::DbError::InvalidOperation("Invalid UTF-8 in database path".to_string())
        })?;

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
    pub fn with_default_path(name: &str) -> DbResult<Self> {
        let db_path = common::platform::get_named_db_path(name);

        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            common::platform::ensure_db_directory(parent)?;
        }

        let path_str = db_path.to_str().ok_or_else(|| {
            common::DbError::InvalidOperation("Invalid UTF-8 in database path".to_string())
        })?;

        Self::new(path_str)
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
    pub fn open_typed_with_indexing(
        db_type: DatabaseType,
        tier: Option<TemperatureTier>,
    ) -> DbResult<Self> {
        let path = db_type.get_path(tier);

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            common::platform::ensure_db_directory(parent)?;
        }

        let path_str = path.to_str().ok_or_else(|| {
            common::DbError::InvalidOperation("Invalid UTF-8 in database path".to_string())
        })?;

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
    pub fn get_node(&self, id: &str) -> DbResult<Option<common::models::Node>> {
        // Input validation (RAG Rule 5.1)
        if id.is_empty() {
            return Err(common::DbError::InvalidOperation(
                "Node ID cannot be empty".to_string(),
            ));
        }

        match self.nodes.get(id.as_bytes())? {
            Some(bytes) => {
                let (node, _): (common::models::Node, usize) = bincode::serde::decode_from_slice(&bytes, bincode::config::standard())
                    .map_err(|e| common::DbError::Serialization(e.to_string()))?;
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
    pub fn insert_node(&self, node: &common::models::Node) -> DbResult<()> {
        let id = node.id();

        // Input validation (RAG Rule 5.1)
        if id.as_str().is_empty() {
            return Err(common::DbError::InvalidOperation(
                "Node ID cannot be empty".to_string(),
            ));
        }

        let bytes = bincode::serde::encode_to_vec(node, bincode::config::standard())
            .map_err(|e| common::DbError::Serialization(e.to_string()))?;
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
    pub fn delete_node(&self, id: &str) -> DbResult<Option<common::models::Node>> {
        // Input validation (RAG Rule 5.1)
        if id.is_empty() {
            return Err(common::DbError::InvalidOperation(
                "Node ID cannot be empty".to_string(),
            ));
        }

        match self.nodes.remove(id.as_bytes())? {
            Some(bytes) => {
                let (node, _): (common::models::Node, usize) = bincode::serde::decode_from_slice(&bytes, bincode::config::standard())
                    .map_err(|e| common::DbError::Serialization(e.to_string()))?;

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
    pub fn get_edge(&self, id: &str) -> DbResult<Option<common::models::Edge>> {
        // Input validation (RAG Rule 5.1)
        if id.is_empty() {
            return Err(common::DbError::InvalidOperation(
                "Edge ID cannot be empty".to_string(),
            ));
        }

        match self.edges.get(id.as_bytes())? {
            Some(bytes) => {
                let (edge, _): (common::models::Edge, usize) = bincode::serde::decode_from_slice(&bytes, bincode::config::standard())
                    .map_err(|e| common::DbError::Serialization(e.to_string()))?;
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
    pub fn insert_edge(&self, edge: &common::models::Edge) -> DbResult<()> {
        // Input validation (RAG Rule 5.1)
        if edge.id.as_str().is_empty() {
            return Err(common::DbError::InvalidOperation(
                "Edge ID cannot be empty".to_string(),
            ));
        }

        let bytes = bincode::serde::encode_to_vec(edge, bincode::config::standard())
            .map_err(|e| common::DbError::Serialization(e.to_string()))?;
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
    pub fn delete_edge(&self, id: &str) -> DbResult<Option<common::models::Edge>> {
        // Input validation (RAG Rule 5.1)
        if id.is_empty() {
            return Err(common::DbError::InvalidOperation(
                "Edge ID cannot be empty".to_string(),
            ));
        }

        match self.edges.remove(id.as_bytes())? {
            Some(bytes) => {
                let (edge, _): (common::models::Edge, usize) = bincode::serde::decode_from_slice(&bytes, bincode::config::standard())
                    .map_err(|e| common::DbError::Serialization(e.to_string()))?;

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
    pub fn get_embedding(&self, id: &str) -> DbResult<Option<common::models::Embedding>> {
        // Input validation (RAG Rule 5.1)
        if id.is_empty() {
            return Err(common::DbError::InvalidOperation(
                "Embedding ID cannot be empty".to_string(),
            ));
        }

        match self.embeddings.get(id.as_bytes())? {
            Some(bytes) => {
                let (embedding, _): (common::models::Embedding, usize) = bincode::decode_from_slice(&bytes, bincode::config::standard())
                    .map_err(|e| common::DbError::Serialization(e.to_string()))?;
                Ok(Some(embedding))
            }
            None => Ok(None),
        }
    }

    /// Retrieves an embedding for a given node.
    ///
    /// Loads the node, gets its embedding_id, then loads the embedding.
    pub fn get_embedding_by_node(
        &self,
        node_id: &str,
    ) -> DbResult<Option<common::models::Embedding>> {
        // Load the node
        let node = match self.get_node(node_id)? {
            Some(n) => n,
            None => return Ok(None),
        };

        // Get embedding_id from the node
        let embedding_id = match node {
            common::models::Node::Message(m) => m.embedding_id,
            common::models::Node::Summary(s) => s.embedding_id,
            common::models::Node::Entity(e) => e.embedding_id,
            common::models::Node::ScrapedPage(p) => p.embedding_id,
            common::models::Node::WebSearch(w) => w.embedding_id,
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
    pub fn insert_embedding(&self, embedding: &common::models::Embedding) -> DbResult<()> {
        // Input validation (RAG Rule 5.1)
        if embedding.id.as_str().is_empty() {
            return Err(common::DbError::InvalidOperation(
                "Embedding ID cannot be empty".to_string(),
            ));
        }

        let bytes = bincode::encode_to_vec(embedding, bincode::config::standard())
            .map_err(|e| common::DbError::Serialization(e.to_string()))?;
        self.embeddings
            .insert(embedding.id.as_str().as_bytes(), bytes)?;

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
    pub fn delete_embedding(&self, id: &str) -> DbResult<Option<common::models::Embedding>> {
        // Input validation (RAG Rule 5.1)
        if id.is_empty() {
            return Err(common::DbError::InvalidOperation(
                "Embedding ID cannot be empty".to_string(),
            ));
        }

        match self.embeddings.remove(id.as_bytes())? {
            Some(bytes) => {
                let (embedding, _): (common::models::Embedding, usize) = bincode::decode_from_slice(&bytes, bincode::config::standard())
                    .map_err(|e| common::DbError::Serialization(e.to_string()))?;

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
    #[inline]
    pub fn db(&self) -> &sled::Db {
        &self.db
    }

    /// Provides access to the index manager, if indexing is enabled.
    ///
    /// Returns `Some(&IndexManager)` if the database was created with `with_indexing()`,
    /// otherwise returns `None`.
    #[inline]
    pub fn index_manager(&self) -> Option<&indexing::IndexManager> {
        self.index_manager.as_ref().map(|arc| arc.as_ref())
    }
}
