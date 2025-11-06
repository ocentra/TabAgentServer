//! Core storage manager for the TabAgent embedded database.
//!
//! This module provides the `StorageManager` which handles all direct interactions
//! with storage engines for CRUD operations using rkyv serialization.

use crate::{
    archived_node::{ArchivedNodeRef, ArchivedEdgeRef, ArchivedEmbeddingRef},
    database_type::{DatabaseType, TemperatureTier},
    engine::{StorageEngine, ReadGuard},
};
use common::DbResult;

/// Manages all direct interactions with storage engines for CRUD operations.
///
/// `StorageManager` provides a safe, ergonomic interface to the underlying
/// storage engine. It manages three primary trees:
/// - `nodes`: Stores all node types (Chat, Message, Entity, etc.)
/// - `edges`: Stores all relationships between nodes
/// - `embeddings`: Stores vector embeddings for semantic search
///
/// All operations are atomic at the single-entity level. Multi-entity
/// transactions are supported through the engine's transaction support.
#[derive(Clone)]
pub struct StorageManager<E: StorageEngine = crate::engine::MdbxEngine> {
    engine: E,

    // Database type and tier (for multi-tier architecture)
    db_type: DatabaseType,
    tier: Option<TemperatureTier>,
}

impl<E: StorageEngine> StorageManager<E> {
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
    /// Returns `DbError` if:
    /// - The database path is invalid or inaccessible
    /// - The database is corrupted
    /// - Insufficient permissions to access the path
    pub fn new(path: &str) -> DbResult<Self> {
        let engine = E::open(path)?;
        
        engine.open_tree("nodes")
            .map_err(|e| common::DbError::InvalidOperation(format!("Failed to open nodes tree: {}", e)))?;
        engine.open_tree("edges")
            .map_err(|e| common::DbError::InvalidOperation(format!("Failed to open edges tree: {}", e)))?;
        engine.open_tree("embeddings")
            .map_err(|e| common::DbError::InvalidOperation(format!("Failed to open embeddings tree: {}", e)))?;

        Ok(Self {
            engine,
            db_type: DatabaseType::Conversations,
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
    pub fn with_default_path(name: &str) -> DbResult<Self> {
        let db_path = common::platform::get_named_db_path(name);

        if let Some(parent) = db_path.parent() {
            common::platform::ensure_db_directory(parent)?;
        }

        let path_str = db_path.to_str().ok_or_else(|| {
            common::DbError::InvalidOperation("Invalid UTF-8 in database path".to_string())
        })?;

        Self::new(path_str)
    }

    /// Opens a database with automatic indexing enabled
    // NOTE: Indexing is now handled externally by the indexing crate.
    // Storage provides database access, indexing receives DB pointers.
    // See: indexing/ARCHITECTURE.md for details.

    // --- Node Operations ---
    
    /// Get a reference to archived node data.
    pub fn get_node_ref(&self, id: &str) -> DbResult<Option<ArchivedNodeRef<E::ReadGuard>>> {
        if id.is_empty() {
            return Err(common::DbError::InvalidOperation(
                "Node ID cannot be empty".to_string(),
            ));
        }

        match self.engine.get("nodes", id.as_bytes())
            .map_err(|e| common::DbError::InvalidOperation(format!("Engine error: {}", e)))? 
        {
            Some(guard) => Ok(Some(ArchivedNodeRef::new(guard)?)),
            None => Ok(None),
        }
    }

    /// Get a node guard for advanced use cases.
    pub fn get_node_guard(&self, id: &str) -> DbResult<Option<E::ReadGuard>> {
        if id.is_empty() {
            return Err(common::DbError::InvalidOperation(
                "Node ID cannot be empty".to_string(),
            ));
        }

        self.engine.get("nodes", id.as_bytes())
            .map_err(|e| common::DbError::InvalidOperation(format!("Engine error: {}", e)))
    }
    

    /// Get a node as owned type.
    ///
    /// # Arguments
    ///
    /// * `id` - The node ID to retrieve
    ///
    /// # Returns
    ///
    /// Returns `Some(Node)` if found, `None` if not found, or an error on serialization failure.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use storage::StorageManager;
    /// # let storage = StorageManager::new("test_db")?;
    /// if let Some(node) = storage.get_node("chat_001")? {
    ///     println!("Found node: {:?}", node);
    /// }
    /// # Ok::<(), storage::DbError>(())
    /// ```
    pub fn get_node(&self, id: &str) -> DbResult<Option<common::models::Node>> {
        match self.get_node_ref(id)? {
            Some(node_ref) => Ok(Some(node_ref.deserialize()?)),
            None => Ok(None),
        }
    }

    /// Insert or update a node in the database.
    ///
    /// If a node with the same ID already exists, it will be replaced.
    /// Automatic index updates are performed if indexing is enabled.
    ///
    /// # Arguments
    ///
    /// * `node` - The node to insert or update
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if serialization or insertion fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use storage::StorageManager;
    /// # use common::models::{Node, Chat};
    /// # let storage = StorageManager::new("test_db")?;
    /// let chat = Node::Chat(Chat {
    ///     id: common::NodeId::new("chat_001"),
    ///     title: "My Chat".to_string(),
    ///     metadata: "{}".to_string(),
    /// });
    /// storage.insert_node(&chat)?;
    /// # Ok::<(), storage::DbError>(())
    /// ```
    pub fn insert_node(&self, node: &common::models::Node) -> DbResult<()> {
        let id = node.id();

        if id.as_str().is_empty() {
            return Err(common::DbError::InvalidOperation(
                "Node ID cannot be empty".to_string(),
            ));
        }

        // Serialize with rkyv
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(node)
            .map_err(|e| common::DbError::Serialization(e.to_string()))?;
        
        self.engine.insert("nodes", id.as_str().as_bytes(), bytes.as_slice().to_vec())
            .map_err(|e| common::DbError::InvalidOperation(format!("Engine error: {}", e)))?;

        // Update indexes if enabled
        // Indexing handled externally - see indexing/ARCHITECTURE.md

        Ok(())
    }

    /// Delete a node from the database.
    ///
    /// If indexing is enabled, the node is automatically removed from all indexes.
    ///
    /// # Arguments
    ///
    /// * `id` - The node ID to delete
    ///
    /// # Returns
    ///
    /// Returns `Some(Node)` if the node was found and deleted, `None` if not found,
    /// or an error on serialization or deletion failure.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use storage::StorageManager;
    /// # let storage = StorageManager::new("test_db")?;
    /// if let Some(deleted) = storage.delete_node("chat_001")? {
    ///     println!("Deleted node: {:?}", deleted);
    /// }
    /// # Ok::<(), storage::DbError>(())
    /// ```
    pub fn delete_node(&self, id: &str) -> DbResult<Option<common::models::Node>> {
        if id.is_empty() {
            return Err(common::DbError::InvalidOperation(
                "Node ID cannot be empty".to_string(),
            ));
        }

        // Get node first for zero-copy deserialization
        let node = if let Some(node_ref) = self.get_node_ref(id)? {
            node_ref.deserialize()?
        } else {
            return Ok(None);
        };

        // Now remove it
        self.engine.remove("nodes", id.as_bytes())
            .map_err(|e| common::DbError::InvalidOperation(format!("Engine error: {}", e)))?;

        // Indexing handled externally - see indexing/ARCHITECTURE.md

        Ok(Some(node))
    }

    // --- Edge Operations ---
    
    /// Get a reference to archived edge data.
    pub fn get_edge_ref(&self, id: &str) -> DbResult<Option<ArchivedEdgeRef<E::ReadGuard>>> {
        if id.is_empty() {
            return Err(common::DbError::InvalidOperation(
                "Edge ID cannot be empty".to_string(),
            ));
        }

        match self.engine.get("edges", id.as_bytes())
            .map_err(|e| common::DbError::InvalidOperation(format!("Engine error: {}", e)))? 
        {
            Some(guard) => Ok(Some(ArchivedEdgeRef::new(guard)?)),
            None => Ok(None),
        }
    }

    /// Get an edge guard for advanced use cases.
    pub fn get_edge_guard(&self, id: &str) -> DbResult<Option<E::ReadGuard>> {
        if id.is_empty() {
            return Err(common::DbError::InvalidOperation(
                "Edge ID cannot be empty".to_string(),
            ));
        }

        self.engine.get("edges", id.as_bytes())
            .map_err(|e| common::DbError::InvalidOperation(format!("Engine error: {}", e)))
    }
    

    /// Get an edge as owned type.
    ///
    /// # Arguments
    ///
    /// * `id` - The edge ID to retrieve
    ///
    /// # Returns
    ///
    /// Returns `Some(Edge)` if found, `None` if not found, or an error on serialization failure.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use storage::StorageManager;
    /// # let storage = StorageManager::new("test_db")?;
    /// if let Some(edge) = storage.get_edge("edge_001")? {
    ///     println!("Found edge: {:?}", edge);
    /// }
    /// # Ok::<(), storage::DbError>(())
    /// ```
    pub fn get_edge(&self, id: &str) -> DbResult<Option<common::models::Edge>> {
        match self.get_edge_ref(id)? {
            Some(edge_ref) => Ok(Some(edge_ref.deserialize()?)),
            None => Ok(None),
        }
    }

    /// Insert or update an edge in the database.
    ///
    /// If an edge with the same ID already exists, it will be replaced.
    /// Automatic index updates are performed if indexing is enabled.
    ///
    /// # Arguments
    ///
    /// * `edge` - The edge to insert or update
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if serialization or insertion fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use storage::StorageManager;
    /// # use common::models::Edge;
    /// # let storage = StorageManager::new("test_db")?;
    /// let edge = Edge {
    ///     id: common::EdgeId::new("edge_001"),
    ///     from: common::NodeId::new("node_1"),
    ///     to: common::NodeId::new("node_2"),
    ///     relationship: "CONTAINS".to_string(),
    ///     metadata: "{}".to_string(),
    /// };
    /// storage.insert_edge(&edge)?;
    /// # Ok::<(), storage::DbError>(())
    /// ```
    pub fn insert_edge(&self, edge: &common::models::Edge) -> DbResult<()> {
        if edge.id.as_str().is_empty() {
            return Err(common::DbError::InvalidOperation(
                "Edge ID cannot be empty".to_string(),
            ));
        }

        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(edge)
            .map_err(|e| common::DbError::Serialization(e.to_string()))?;
        
        self.engine.insert("edges", edge.id.as_str().as_bytes(), bytes.as_slice().to_vec())
            .map_err(|e| common::DbError::InvalidOperation(format!("Engine error: {}", e)))?;

        // Indexing handled externally - see indexing/ARCHITECTURE.md

        Ok(())
    }

    /// Delete an edge from the database.
    ///
    /// If indexing is enabled, the edge is automatically removed from all indexes.
    ///
    /// # Arguments
    ///
    /// * `id` - The edge ID to delete
    ///
    /// # Returns
    ///
    /// Returns `Some(Edge)` if the edge was found and deleted, `None` if not found,
    /// or an error on serialization or deletion failure.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use storage::StorageManager;
    /// # let storage = StorageManager::new("test_db")?;
    /// if let Some(deleted) = storage.delete_edge("edge_001")? {
    ///     println!("Deleted edge: {:?}", deleted);
    /// }
    /// # Ok::<(), storage::DbError>(())
    /// ```
    pub fn delete_edge(&self, id: &str) -> DbResult<Option<common::models::Edge>> {
        if id.is_empty() {
            return Err(common::DbError::InvalidOperation(
                "Edge ID cannot be empty".to_string(),
            ));
        }

        // Get edge first for zero-copy deserialization
        let edge = if let Some(edge_ref) = self.get_edge_ref(id)? {
            edge_ref.deserialize()?
        } else {
            return Ok(None);
        };

        // Now remove it
        self.engine.remove("edges", id.as_bytes())
            .map_err(|e| common::DbError::InvalidOperation(format!("Engine error: {}", e)))?;

        // Update indexes
        // Indexing handled externally - see indexing/ARCHITECTURE.md

        Ok(Some(edge))
    }

    // --- Embedding Operations ---
    
    /// Get a reference to archived embedding data.
    pub fn get_embedding_ref(&self, id: &str) -> DbResult<Option<ArchivedEmbeddingRef<E::ReadGuard>>> {
        if id.is_empty() {
            return Err(common::DbError::InvalidOperation(
                "Embedding ID cannot be empty".to_string(),
            ));
        }

        match self.engine.get("embeddings", id.as_bytes())
            .map_err(|e| common::DbError::InvalidOperation(format!("Engine error: {}", e)))? 
        {
            Some(guard) => Ok(Some(ArchivedEmbeddingRef::new(guard)?)),
            None => Ok(None),
        }
    }

    /// Get an embedding guard for advanced use cases.
    pub fn get_embedding_guard(&self, id: &str) -> DbResult<Option<E::ReadGuard>> {
        if id.is_empty() {
            return Err(common::DbError::InvalidOperation(
                "Embedding ID cannot be empty".to_string(),
            ));
        }

        self.engine.get("embeddings", id.as_bytes())
            .map_err(|e| common::DbError::InvalidOperation(format!("Engine error: {}", e)))
    }
    

    /// Get an embedding as owned type.
    ///
    /// # Arguments
    ///
    /// * `id` - The embedding ID to retrieve
    ///
    /// # Returns
    ///
    /// Returns `Some(Embedding)` if found, `None` if not found, or an error on serialization failure.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use storage::StorageManager;
    /// # let storage = StorageManager::new("test_db")?;
    /// if let Some(embedding) = storage.get_embedding("emb_001")? {
    ///     println!("Found embedding with dimension: {}", embedding.vector.len());
    /// }
    /// # Ok::<(), storage::DbError>(())
    /// ```
    pub fn get_embedding(&self, id: &str) -> DbResult<Option<common::models::Embedding>> {
        match self.get_embedding_ref(id)? {
            Some(emb_ref) => Ok(Some(emb_ref.deserialize()?)),
            None => Ok(None),
        }
    }

    /// Get an embedding associated with a specific node.
    ///
    /// This method first retrieves the node, extracts its `embedding_id` field,
    /// then retrieves the embedding using that ID.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The node ID whose embedding to retrieve
    ///
    /// # Returns
    ///
    /// Returns `Some(Embedding)` if the node exists and has an associated embedding,
    /// `None` if the node doesn't exist or has no embedding, or an error on failure.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use storage::StorageManager;
    /// # let storage = StorageManager::new("test_db")?;
    /// if let Some(embedding) = storage.get_embedding_by_node("msg_001")? {
    ///     println!("Retrieved embedding for message");
    /// }
    /// # Ok::<(), storage::DbError>(())
    /// ```
    pub fn get_embedding_by_node(
        &self,
        node_id: &str,
    ) -> DbResult<Option<common::models::Embedding>> {
        if let Some(node_ref) = self.get_node_ref(node_id)? {
            // TRUE ZERO-COPY: Access archived fields directly without deserializing!
            let archived = node_ref.archived();

            let embedding_id_str = match archived {
                rkyv::Archived::<common::models::Node>::Message(m) => {
                    m.embedding_id.as_ref().map(|id| id.0.as_str())
                },
                rkyv::Archived::<common::models::Node>::Summary(s) => {
                    s.embedding_id.as_ref().map(|id| id.0.as_str())
                },
                rkyv::Archived::<common::models::Node>::Entity(e) => {
                    e.embedding_id.as_ref().map(|id| id.0.as_str())
                },
                rkyv::Archived::<common::models::Node>::ScrapedPage(p) => {
                    p.embedding_id.as_ref().map(|id| id.0.as_str())
                },
                rkyv::Archived::<common::models::Node>::WebSearch(w) => {
                    w.embedding_id.as_ref().map(|id| id.0.as_str())
                },
                _ => None,
            };

            match embedding_id_str {
                Some(id) => self.get_embedding(id),
                None => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    /// Insert or update an embedding in the database.
    ///
    /// If an embedding with the same ID already exists, it will be replaced.
    /// Automatic index updates are performed if indexing is enabled.
    ///
    /// # Arguments
    ///
    /// * `embedding` - The embedding to insert or update
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if serialization or insertion fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use storage::StorageManager;
    /// # use common::models::Embedding;
    /// # let storage = StorageManager::new("test_db")?;
    /// let embedding = Embedding {
    ///     id: common::EmbeddingId::new("emb_001"),
    ///     vector: vec![0.1, 0.2, 0.3],
    ///     metadata: "{}".to_string(),
    /// };
    /// storage.insert_embedding(&embedding)?;
    /// # Ok::<(), storage::DbError>(())
    /// ```
    pub fn insert_embedding(&self, embedding: &common::models::Embedding) -> DbResult<()> {
        if embedding.id.as_str().is_empty() {
            return Err(common::DbError::InvalidOperation(
                "Embedding ID cannot be empty".to_string(),
            ));
        }

        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(embedding)
            .map_err(|e| common::DbError::Serialization(e.to_string()))?;
        
        self.engine.insert("embeddings", embedding.id.as_str().as_bytes(), bytes.as_slice().to_vec())
            .map_err(|e| common::DbError::InvalidOperation(format!("Engine error: {}", e)))?;

        // Indexing handled externally - see indexing/ARCHITECTURE.md

        Ok(())
    }

    /// Delete an embedding from the database.
    ///
    /// If indexing is enabled, the embedding is automatically removed from all indexes.
    ///
    /// # Arguments
    ///
    /// * `id` - The embedding ID to delete
    ///
    /// # Returns
    ///
    /// Returns `Some(Embedding)` if the embedding was found and deleted, `None` if not found,
    /// or an error on serialization or deletion failure.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use storage::StorageManager;
    /// # let storage = StorageManager::new("test_db")?;
    /// if let Some(deleted) = storage.delete_embedding("emb_001")? {
    ///     println!("Deleted embedding");
    /// }
    /// # Ok::<(), storage::DbError>(())
    /// ```
    pub fn delete_embedding(&self, id: &str) -> DbResult<Option<common::models::Embedding>> {
        if id.is_empty() {
            return Err(common::DbError::InvalidOperation(
                "Embedding ID cannot be empty".to_string(),
            ));
        }

        // Get embedding first for zero-copy deserialization
        let embedding = if let Some(emb_ref) = self.get_embedding_ref(id)? {
            emb_ref.deserialize()?
        } else {
            return Ok(None);
        };

        // Now remove it
        self.engine.remove("embeddings", id.as_bytes())
            .map_err(|e| common::DbError::InvalidOperation(format!("Engine error: {}", e)))?;

        // Update indexes
        // Indexing handled externally - see indexing/ARCHITECTURE.md

        Ok(Some(embedding))
    }

    // --- Utility Methods ---

    /// Get a reference to the index manager if indexing is enabled.
    ///
    /// # Returns
    ///
    /// Returns `Some(&IndexManager)` if indexing is enabled, `None` otherwise.
    // NOTE: IndexManager is now created externally and receives DB pointers from storage.
    // See indexing/ARCHITECTURE.md for the correct architecture.
    
    /// Scan all key-value pairs with keys that start with the given prefix.
    ///
    /// # Arguments
    ///
    /// * `prefix` - The byte prefix to match against node keys
    ///
    /// # Returns
    ///
    /// An iterator over `(key, value)` pairs where keys start with `prefix`.
    /// Values are deserialized bytes from the nodes tree.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use storage::StorageManager;
    /// # let storage = StorageManager::new("test_db")?;
    /// for result in storage.scan_prefix(b"chat_") {
    ///     let (key, value) = result?;
    ///     println!("Found key: {:?}", key);
    /// }
    /// # Ok::<(), storage::DbError>(())
    /// ```
    pub fn scan_prefix<'a>(&'a self, prefix: &[u8]) -> impl Iterator<Item = common::DbResult<(Vec<u8>, Vec<u8>)>> + 'a {
        self.engine.scan_prefix("nodes", prefix).map(|result| {
            result.map(|(key, guard)| (key, guard.data().to_vec()))
                .map_err(|e| common::DbError::InvalidOperation(format!("Engine error: {}", e)))
        })
    }
    
    pub fn scan_prefix_nodes_ref<'a>(&'a self, prefix: &[u8]) -> impl Iterator<Item = common::DbResult<(Vec<u8>, ArchivedNodeRef<E::ReadGuard>)>> + 'a {
        self.engine.scan_prefix("nodes", prefix).map(|result| {
            result.map_err(|e| common::DbError::InvalidOperation(format!("Engine error: {}", e)))
                .and_then(|(key, guard)| {
                    ArchivedNodeRef::new(guard)
                        .map(|node_ref| (key, node_ref))
                        .map_err(|e| common::DbError::Serialization(format!("Failed to create node ref: {}", e)))
                })
        })
    }
    
    /// Iterate over all nodes with zero-copy references.
    /// 
    /// Returns an iterator over all nodes using ArchivedNodeRef for true zero-copy access.
    /// This is the FAST path - use this instead of `iter()` when you need to access node fields!
    /// 
    /// # Returns
    /// 
    /// An iterator over `(key, ArchivedNodeRef)` pairs for all nodes in the database.
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use storage::StorageManager;
    /// # let storage = StorageManager::new("test_db")?;
    /// for result in storage.iter_nodes_ref() {
    ///     let (key, node_ref) = result?;
    ///     let archived = node_ref.archived();
    ///     // Access fields directly from mmap - zero-copy!
    /// }
    /// # Ok::<(), common::DbError>(())
    /// ```
    pub fn iter_nodes_ref<'a>(&'a self) -> impl Iterator<Item = common::DbResult<(Vec<u8>, ArchivedNodeRef<E::ReadGuard>)>> + 'a {
        self.scan_prefix_nodes_ref(&[])
    }
    
    /// Iterate over all key-value pairs in the nodes tree.
    ///
    /// # Returns
    ///
    /// An iterator over all `(key, value)` pairs in the database.
    /// Values are deserialized bytes from the nodes tree.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use storage::StorageManager;
    /// # let storage = StorageManager::new("test_db")?;
    /// for result in storage.iter() {
    ///     let (key, value) = result?;
    ///     println!("Key: {:?}, Value size: {} bytes", key, value.len());
    /// }
    /// # Ok::<(), storage::DbError>(())
    /// ```
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = common::DbResult<(Vec<u8>, Vec<u8>)>> + 'a {
        self.engine.iter("nodes").map(|result| {
            result.map(|(key, guard)| (key, guard.data().to_vec()))
                .map_err(|e| common::DbError::InvalidOperation(format!("Engine error: {}", e)))
        })
    }
}

// Type alias for default engine
pub type DefaultStorageManager = StorageManager<crate::engine::MdbxEngine>;

// MdbxEngine-specific methods
impl StorageManager<crate::engine::MdbxEngine> {
    /// Get the database path (for registry introspection)
    pub fn db_path(&self) -> Option<String> {
        self.engine.db_path()
    }
    
    /// Get raw MDBX environment pointer for external services (e.g., indexing)
    ///
    /// # Safety
    ///
    /// This is unsafe because:
    /// - The pointer is only valid while this StorageManager exists
    /// - Caller must follow MDBX's thread-safety rules
    /// - No `mdbx_env_close` should be called on this pointer
    ///
    /// # Usage
    ///
    /// This is intended for services like `indexing` that need to create
    /// their own DBIs within this database's MDBX environment.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use storage::StorageManager;
    /// use indexing::IndexManager;
    ///
    /// let storage = StorageManager::new("./conversations.mdbx")?;
    /// 
    /// // Get pointers for indexing
    /// let env = unsafe { storage.get_raw_env() };
    /// let structural_dbi = storage.get_or_create_dbi("structural_index")?;
    /// let outgoing_dbi = storage.get_or_create_dbi("graph_outgoing")?;
    /// let incoming_dbi = storage.get_or_create_dbi("graph_incoming")?;
    /// 
    /// // Create IndexManager (in caller's code, not storage!)
    /// let index = IndexManager::new_from_storage(
    ///     env, structural_dbi, outgoing_dbi, incoming_dbi, true
    /// )?;
    /// # Ok::<(), common::DbError>(())
    /// ```
    pub unsafe fn get_raw_env(&self) -> *mut mdbx_base::mdbx_sys::MDBX_env {
        self.engine.get_raw_env()
    }
    
    /// Get or create a DBI (table) in this database
    ///
    /// This allows external services to create their own tables within
    /// this database's MDBX environment.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the table/DBI to get or create
    ///
    /// # Errors
    ///
    /// Returns error if DBI creation fails
    pub fn get_or_create_dbi(&self, name: &str) -> DbResult<mdbx_base::mdbx_sys::MDBX_dbi> {
        self.engine.get_or_create_dbi(name)
    }
}
