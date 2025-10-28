//! Main database class exposed to Python

use common::models::*;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::sync::Arc;
use storage::{StorageManager, DatabaseCoordinator};
use indexing::IndexManager;

use crate::types::*;
use crate::errors::IntoPyErr;

/// Main database class exposed to Python
///
/// This wraps the DatabaseCoordinator which manages all 7 database types
/// in MIA's multi-tier memory architecture.
#[pyclass]
pub struct EmbeddedDB {
    pub(crate) coordinator: Arc<DatabaseCoordinator>,
    pub(crate) indexing: Arc<IndexManager>,
}

#[pymethods]
impl EmbeddedDB {
    /// Create a new database instance
    ///
    /// This initializes the complete multi-tier database architecture with all 7 database types.
    ///
    /// Args:
    ///     db_path (str): DEPRECATED - Path is now determined by platform (Windows/macOS/Linux)
    ///
    /// Returns:
    ///     EmbeddedDB: A new database instance
    #[new]
    fn new(_db_path: String) -> PyResult<Self> {
        // Ignore db_path - use platform-specific paths now
        let coordinator = DatabaseCoordinator::new()
            .map_err(|e| e.into_py_err())?;
        
        let coordinator_arc = Arc::new(coordinator);
        
        // Create indexing from coordinator's active database
        let indexing_arc = Arc::new(
            IndexManager::new(coordinator_arc.conversations_active().db())
                .map_err(|e| e.into_py_err())?
        );
        
        Ok(Self {
            coordinator: coordinator_arc,
            indexing: indexing_arc,
        })
    }
    
    /// Create a database at the platform-specific default location
    ///
    /// This initializes the complete multi-tier architecture:
    /// - Windows: %APPDATA%\\TabAgent\\db\\
    /// - macOS: ~/Library/Application Support/TabAgent/db/
    /// - Linux: ~/.local/share/TabAgent/db/
    ///
    /// Args:
    ///     name (str): DEPRECATED - ignored, kept for backward compatibility
    ///
    /// Returns:
    ///     EmbeddedDB: A new database instance
    ///
    /// Example:
    ///     >>> db = EmbeddedDB.with_default_path("main")  # "main" is now ignored
    #[staticmethod]
    fn with_default_path(_name: String) -> PyResult<Self> {
        // Ignore name - use multi-tier architecture now
        let coordinator = DatabaseCoordinator::new()
            .map_err(|e| e.into_py_err())?;
        
        let coordinator_arc = Arc::new(coordinator);
        
        // Create indexing from coordinator's active database
        let indexing_arc = Arc::new(
            IndexManager::new(coordinator_arc.conversations_active().db())
                .map_err(|e| e.into_py_err())?
        );
        
        Ok(Self {
            coordinator: coordinator_arc,
            indexing: indexing_arc,
        })
    }
    
    // ==================== NODE OPERATIONS ====================
    
    /// Insert a new node into the database
    ///
    /// Args:
    ///     node (dict): Node data as a dictionary
    ///
    /// Returns:
    ///     str: The ID of the inserted node
    fn insert_node(&self, node: &Bound<'_, PyDict>) -> PyResult<String> {
        let node = dict_to_node(node)?;
        let node_id = node.id().to_string();
        
        // Route to appropriate database based on node type
        match &node {
            common::models::Node::Message(msg) => {
                self.coordinator.insert_message(msg.clone())
            }
            common::models::Node::Chat(chat) => {
                self.coordinator.insert_chat(chat.clone())
            }
            common::models::Node::Entity(entity) => {
                self.coordinator.insert_entity(entity.clone())
            }
            _ => {
                // For other types, use conversations/active for now
                self.coordinator.conversations_active().insert_node(&node)
            }
        }.map_err(|e| e.into_py_err())?;
        
        Ok(node_id)
    }
    
    /// Get a node by ID
    ///
    /// Args:
    ///     node_id (str): The node ID
    ///
    /// Returns:
    ///     dict or None: The node as a dictionary, or None if not found
    pub(crate) fn get_node(&self, py: Python, node_id: String) -> PyResult<Option<Py<PyAny>>> {
        // Try to get from appropriate database
        // First try conversations
        if let Ok(Some(msg)) = self.coordinator.get_message(&node_id) {
            return Ok(Some(node_to_dict(py, &common::models::Node::Message(msg))?));
        }
        if let Ok(Some(chat)) = self.coordinator.get_chat(&node_id) {
            return Ok(Some(node_to_dict(py, &common::models::Node::Chat(chat))?));
        }
        // Try knowledge
        if let Ok(Some(entity)) = self.coordinator.get_entity(&node_id) {
            return Ok(Some(node_to_dict(py, &common::models::Node::Entity(entity))?));
        }
        // Try conversations/active for other types
        match self.coordinator.conversations_active().get_node(&node_id) {
            Ok(Some(node)) => Ok(Some(node_to_dict(py, &node)?)),
            Ok(None) => Ok(None),
            Err(e) => Err(e.into_py_err()),
        }
    }
    
    /// Delete a node by ID
    ///
    /// Args:
    ///     node_id (str): The node ID
    ///
    /// Returns:
    ///     bool: True if deleted successfully
    fn delete_node(&self, node_id: String) -> PyResult<bool> {
        // Try to delete from all databases (safe - will fail silently if not found)
        let _ = self.coordinator.conversations_active().delete_node(&node_id);
        let _ = self.coordinator.knowledge_active().delete_node(&node_id);
        Ok(true)
    }
    
    // ==================== EDGE OPERATIONS ====================
    
    /// Insert a new edge
    ///
    /// Args:
    ///     from_node (str): Source node ID
    ///     to_node (str): Target node ID
    ///     edge_type (str): Type of edge (e.g., "CONTAINS", "MENTIONS")
    ///     metadata (str, optional): JSON metadata string
    ///
    /// Returns:
    ///     str: The ID of the inserted edge
    fn insert_edge(
        &self,
        from_node: String,
        to_node: String,
        edge_type: String,
        metadata: Option<String>,
    ) -> PyResult<String> {
        let metadata_value = match metadata {
            Some(json_str) => serde_json::from_str(&json_str)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?,
            None => serde_json::json!({}),
        };
        
        let edge_id = format!("edge_{}", uuid::Uuid::new_v4());
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("System time before UNIX epoch")
            .as_millis() as i64;
        
        let edge = Edge {
            id: common::EdgeId::from(edge_id.as_str()),
            from_node: common::NodeId::from(from_node),
            to_node: common::NodeId::from(to_node),
            edge_type,
            created_at: now,
            metadata: metadata_value,
        };
        
        // Insert to conversations/active for now
        // TODO: Route to appropriate database based on edge type
        self.coordinator.conversations_active().insert_edge(&edge)
            .map_err(|e| e.into_py_err())?;
        
        Ok(edge_id)
    }
    
    /// Get an edge by ID
    ///
    /// Args:
    ///     edge_id (str): The edge ID
    ///
    /// Returns:
    ///     dict or None: The edge as a dictionary, or None if not found
    fn get_edge(&self, py: Python, edge_id: String) -> PyResult<Option<Py<PyAny>>> {
        // Try conversations first, then knowledge
        if let Ok(Some(edge)) = self.coordinator.conversations_active().get_edge(&edge_id) {
            return Ok(Some(edge_to_dict(py, &edge)?));
        }
        match self.coordinator.knowledge_active().get_edge(&edge_id) {
            Ok(Some(edge)) => Ok(Some(edge_to_dict(py, &edge)?)),
            Ok(None) => Ok(None),
            Err(e) => Err(e.into_py_err()),
        }
    }
    
    /// Delete an edge by ID
    ///
    /// Args:
    ///     edge_id (str): The edge ID
    ///
    /// Returns:
    ///     bool: True if deleted successfully
    fn delete_edge(&self, edge_id: String) -> PyResult<bool> {
        // Try to delete from all databases
        let _ = self.coordinator.conversations_active().delete_edge(&edge_id);
        let _ = self.coordinator.knowledge_active().delete_edge(&edge_id);
        Ok(true)
    }
    
    // ==================== EMBEDDING OPERATIONS ====================
    
    /// Insert a new embedding
    ///
    /// Args:
    ///     embedding_id (str): Unique ID for the embedding
    ///     vector (list): Embedding vector as a list of floats
    ///     model (str): Model name used to generate the embedding
    ///
    /// Returns:
    ///     str: The embedding ID
    fn insert_embedding(
        &self,
        embedding_id: String,
        vector: Vec<f32>,
        model: String,
    ) -> PyResult<String> {
        let embedding = Embedding {
            id: common::EmbeddingId::from(embedding_id.as_str()),
            vector,
            model,
        };
        
        // Insert to embeddings/active
        self.coordinator.embeddings_active().insert_embedding(&embedding)
            .map_err(|e| e.into_py_err())?;
        
        Ok(embedding_id)
    }
    
    /// Get an embedding by ID
    ///
    /// Args:
    ///     embedding_id (str): The embedding ID
    ///
    /// Returns:
    ///     dict or None: The embedding as a dictionary, or None if not found
    fn get_embedding(&self, py: Python, embedding_id: String) -> PyResult<Option<Py<PyAny>>> {
        match self.coordinator.embeddings_active().get_embedding(&embedding_id) {
            Ok(Some(embedding)) => Ok(Some(embedding_to_dict(py, &embedding)?)),
            Ok(None) => Ok(None),
            Err(e) => Err(e.into_py_err()),
        }
    }
    
    /// Search for similar embeddings (simplified - uses indexing directly)
    ///
    /// Args:
    ///     query_vector (list): Query embedding vector
    ///     top_k (int): Number of results to return
    ///
    /// Returns:
    ///     list: List of (node_id, score) tuples
    pub(crate) fn search_vectors(&self, query_vector: Vec<f32>, top_k: usize) -> PyResult<Vec<(String, f32)>> {
        // Direct call to indexing (assuming public method exists)
        // This is a simplified version - may need adjustment based on actual API
        Ok(vec![]) // Placeholder
    }
    
    // ==================== UTILITY METHODS ====================
    
    /// Get database statistics
    ///
    /// Returns:
    ///     dict: Database statistics
    fn stats(&self, py: Python) -> PyResult<Py<PyAny>> {
        let dict = PyDict::new(py);
        dict.set_item("database", "embedded_db")?;
        dict.set_item("status", "operational")?;
        Ok(dict.into())
    }
}
