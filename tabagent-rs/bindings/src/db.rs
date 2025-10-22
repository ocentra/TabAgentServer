//! Main database class exposed to Python

use common::models::*;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::sync::Arc;
use storage::StorageManager;
use indexing::IndexManager;

use crate::types::*;
use crate::errors::IntoPyErr;

/// Main database class exposed to Python
#[pyclass]
pub struct EmbeddedDB {
    pub(crate) storage: Arc<StorageManager>,
    pub(crate) indexing: Arc<IndexManager>,
}

#[pymethods]
impl EmbeddedDB {
    /// Create a new database instance
    ///
    /// Args:
    ///     db_path (str): Path to the database directory
    ///
    /// Returns:
    ///     EmbeddedDB: A new database instance
    #[new]
    fn new(db_path: String) -> PyResult<Self> {
        let storage = StorageManager::with_indexing(&db_path)
            .map_err(|e| e.into_py_err())?;
        
        let storage_arc = Arc::new(storage);
        
        // Create a new Arc for indexing (simplified for now)
        let indexing_arc = Arc::new(
            IndexManager::new(storage_arc.db())
                .map_err(|e| e.into_py_err())?
        );
        
        Ok(Self {
            storage: storage_arc,
            indexing: indexing_arc,
        })
    }
    
    /// Create a database at the platform-specific default location
    ///
    /// This uses platform-appropriate paths:
    /// - Windows: %APPDATA%\\TabAgent\\db\\{name}
    /// - macOS: ~/Library/Application Support/TabAgent/db/{name}
    /// - Linux: ~/.local/share/TabAgent/db/{name}
    ///
    /// Args:
    ///     name (str): Database name (e.g., "main", "cache")
    ///
    /// Returns:
    ///     EmbeddedDB: A new database instance
    ///
    /// Example:
    ///     >>> db = EmbeddedDB.with_default_path("main")
    #[staticmethod]
    fn with_default_path(name: String) -> PyResult<Self> {
        let storage = StorageManager::with_default_path_and_indexing(&name)
            .map_err(|e| e.into_py_err())?;
        
        let storage_arc = Arc::new(storage);
        
        // Create a new Arc for indexing
        let indexing_arc = Arc::new(
            IndexManager::new(storage_arc.db())
                .map_err(|e| e.into_py_err())?
        );
        
        Ok(Self {
            storage: storage_arc,
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
    fn insert_node(&self, node: &PyDict) -> PyResult<String> {
        let node = dict_to_node(node)?;
        let node_id = node.id().to_string();
        self.storage.insert_node(&node)
            .map_err(|e| e.into_py_err())?;
        Ok(node_id)
    }
    
    /// Get a node by ID
    ///
    /// Args:
    ///     node_id (str): The node ID
    ///
    /// Returns:
    ///     dict or None: The node as a dictionary, or None if not found
    pub(crate) fn get_node(&self, py: Python, node_id: String) -> PyResult<Option<PyObject>> {
        match self.storage.get_node(&node_id) {
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
        self.storage.delete_node(&node_id)
            .map_err(|e| e.into_py_err())?;
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
            .unwrap()
            .as_millis() as i64;
        
        let edge = Edge {
            id: edge_id.clone(),
            from_node,
            to_node,
            edge_type,
            created_at: now,
            metadata: metadata_value,
        };
        
        self.storage.insert_edge(&edge)
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
    fn get_edge(&self, py: Python, edge_id: String) -> PyResult<Option<PyObject>> {
        match self.storage.get_edge(&edge_id) {
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
        self.storage.delete_edge(&edge_id)
            .map_err(|e| e.into_py_err())?;
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
            id: embedding_id.clone(),
            vector,
            model,
        };
        
        self.storage.insert_embedding(&embedding)
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
    fn get_embedding(&self, py: Python, embedding_id: String) -> PyResult<Option<PyObject>> {
        match self.storage.get_embedding(&embedding_id) {
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
    fn stats(&self, py: Python) -> PyResult<PyObject> {
        let dict = PyDict::new(py);
        dict.set_item("database", "embedded_db")?;
        dict.set_item("status", "operational")?;
        Ok(dict.into())
    }
}
