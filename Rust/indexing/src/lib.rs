//! Indexing layer for fast data retrieval.
//!
//! This crate provides three types of indexes to enable different query patterns:
//! - **Structural indexes**: Fast property-based filtering (O(log n))
//! - **Graph indexes**: Efficient relationship traversal (O(1) neighbor lookup)
//! - **Vector indexes**: Semantic similarity search using HNSW (O(log n))
//!
//! # Architecture
//!
//! ```text
//! IndexManager
//!   ├── Structural Index (B-tree on properties)
//!   ├── Graph Index (Adjacency lists)
//!   └── Vector Index (HNSW for semantic search)
//! ```
//!
//! All indexes are updated **automatically** when data changes in the storage layer,
//! ensuring consistency.
//!
//! # Example
//!
//! ```no_run
//! # use indexing::IndexManager;
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a sled database
//! let db = sled::open("my_database")?;
//! let index_manager = IndexManager::new(&db)?;
//!
//! // Indexes update automatically when nodes are added
//! // Query by property
//! let messages = index_manager.get_nodes_by_property("chat_id", "chat_123")?;
//!
//! // Traverse graph
//! let edges = index_manager.get_outgoing_edges("chat_123")?;
//!
//! // Semantic search
//! let query = vec![0.1; 384]; // From Python ML model
//! let similar = index_manager.search_vectors(&query, 10)?;
//! # Ok(())
//! # }
//! ```

pub mod structural;
pub mod graph;
pub mod vector;

use common::{DbResult, EdgeId, NodeId};
use common::models::{Edge, Embedding, Node};
use std::sync::{Arc, Mutex};

pub use structural::StructuralIndex;
pub use graph::GraphIndex;
pub use vector::{VectorIndex, SearchResult};

/// Coordinates all indexing operations across structural, graph, and vector indexes.
///
/// `IndexManager` ensures that all indexes are kept in sync with the primary data.
/// It provides a unified interface for querying across all index types.
pub struct IndexManager {
    structural: Arc<StructuralIndex>,
    graph: Arc<GraphIndex>,
    vector: Arc<Mutex<VectorIndex>>,
}

impl IndexManager {
    /// Creates a new `IndexManager` instance.
    ///
    /// This initializes all three index types (structural, graph, vector) using
    /// the provided `sled::Db` instance.
    ///
    /// # Arguments
    ///
    /// * `db` - Reference to the sled database
    ///
    /// # Errors
    ///
    /// Returns `DbError` if any index fails to initialize.
    pub fn new(db: &sled::Db) -> DbResult<Self> {
        // Open the required sled trees for each index type
        let structural_tree = db.open_tree("structural_index")?;
        let outgoing_tree = db.open_tree("graph_outgoing")?;
        let incoming_tree = db.open_tree("graph_incoming")?;
        
        // Note: VectorIndex persists to disk
        // TODO: In production, derive path from db location. For now, use a temp path.
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let vector_path = std::env::temp_dir().join(format!("vec_idx_{}", timestamp));
        let vector_index = VectorIndex::new(vector_path)?;
        
        Ok(Self {
            structural: Arc::new(StructuralIndex::new(structural_tree)),
            graph: Arc::new(GraphIndex::new(outgoing_tree, incoming_tree)),
            vector: Arc::new(Mutex::new(vector_index)),
        })
    }

    /// Indexes a node across all relevant indexes.
    ///
    /// This is called automatically when a node is inserted or updated.
    pub fn index_node(&self, node: &Node) -> DbResult<()> {
        match node {
            Node::Chat(chat) => {
                self.structural.add("node_type", "Chat", chat.id.as_str())?;
                self.structural.add("topic", &chat.topic, chat.id.as_str())?;
            }
            Node::Message(msg) => {
                self.structural.add("node_type", "Message", msg.id.as_str())?;
                self.structural.add("chat_id", msg.chat_id.as_str(), msg.id.as_str())?;
                self.structural.add("sender", &msg.sender, msg.id.as_str())?;
            }
            Node::Entity(entity) => {
                self.structural.add("node_type", "Entity", entity.id.as_str())?;
                self.structural.add("entity_type", &entity.entity_type, entity.id.as_str())?;
                self.structural.add("label", &entity.label, entity.id.as_str())?;
            }
            Node::Summary(summary) => {
                self.structural.add("node_type", "Summary", summary.id.as_str())?;
                self.structural.add("chat_id", summary.chat_id.as_str(), summary.id.as_str())?;
            }
            Node::Attachment(att) => {
                self.structural.add("node_type", "Attachment", att.id.as_str())?;
                self.structural.add("message_id", att.message_id.as_str(), att.id.as_str())?;
                self.structural.add("mime_type", &att.mime_type, att.id.as_str())?;
            }
            Node::WebSearch(search) => {
                self.structural.add("node_type", "WebSearch", search.id.as_str())?;
            }
            Node::ScrapedPage(page) => {
                self.structural.add("node_type", "ScrapedPage", page.id.as_str())?;
                self.structural.add("url", &page.url, page.id.as_str())?;
            }
            Node::Bookmark(bookmark) => {
                self.structural.add("node_type", "Bookmark", bookmark.id.as_str())?;
                self.structural.add("url", &bookmark.url, bookmark.id.as_str())?;
            }
            Node::ImageMetadata(img) => {
                self.structural.add("node_type", "ImageMetadata", img.id.as_str())?;
            }
            Node::AudioTranscript(audio) => {
                self.structural.add("node_type", "AudioTranscript", audio.id.as_str())?;
            }
            Node::ModelInfo(model) => {
                self.structural.add("node_type", "ModelInfo", model.id.as_str())?;
                self.structural.add("model_name", &model.name, model.id.as_str())?;
            }
        }
        Ok(())
    }

    /// Removes a node from all indexes.
    pub fn unindex_node(&self, node: &Node) -> DbResult<()> {
        match node {
            Node::Chat(chat) => {
                self.structural.remove("node_type", "Chat", chat.id.as_str())?;
                self.structural.remove("topic", &chat.topic, chat.id.as_str())?;
            }
            Node::Message(msg) => {
                self.structural.remove("node_type", "Message", msg.id.as_str())?;
                self.structural.remove("chat_id", msg.chat_id.as_str(), msg.id.as_str())?;
                self.structural.remove("sender", &msg.sender, msg.id.as_str())?;
            }
            Node::Entity(entity) => {
                self.structural.remove("node_type", "Entity", entity.id.as_str())?;
                self.structural.remove("entity_type", &entity.entity_type, entity.id.as_str())?;
                self.structural.remove("label", &entity.label, entity.id.as_str())?;
            }
            Node::Summary(summary) => {
                self.structural.remove("node_type", "Summary", summary.id.as_str())?;
                self.structural.remove("chat_id", summary.chat_id.as_str(), summary.id.as_str())?;
            }
            Node::Attachment(att) => {
                self.structural.remove("node_type", "Attachment", att.id.as_str())?;
                self.structural.remove("message_id", att.message_id.as_str(), att.id.as_str())?;
                self.structural.remove("mime_type", &att.mime_type, att.id.as_str())?;
            }
            Node::WebSearch(search) => {
                self.structural.remove("node_type", "WebSearch", search.id.as_str())?;
            }
            Node::ScrapedPage(page) => {
                self.structural.remove("node_type", "ScrapedPage", page.id.as_str())?;
                self.structural.remove("url", &page.url, page.id.as_str())?;
            }
            Node::Bookmark(bookmark) => {
                self.structural.remove("node_type", "Bookmark", bookmark.id.as_str())?;
                self.structural.remove("url", &bookmark.url, bookmark.id.as_str())?;
            }
            Node::ImageMetadata(img) => {
                self.structural.remove("node_type", "ImageMetadata", img.id.as_str())?;
            }
            Node::AudioTranscript(audio) => {
                self.structural.remove("node_type", "AudioTranscript", audio.id.as_str())?;
            }
            Node::ModelInfo(model) => {
                self.structural.remove("node_type", "ModelInfo", model.id.as_str())?;
                self.structural.remove("model_name", &model.name, model.id.as_str())?;
            }
        }
        Ok(())
    }

    /// Indexes an edge in the graph index.
    pub fn index_edge(&self, edge: &Edge) -> DbResult<()> {
        self.graph.add_edge(edge)
    }

    /// Removes an edge from the graph index.
    pub fn unindex_edge(&self, edge: &Edge) -> DbResult<()> {
        self.graph.remove_edge(edge)
    }

    /// Indexes an embedding in the vector index.
    pub fn index_embedding(&self, embedding: &Embedding) -> DbResult<()> {
        let mut vec_idx = self.vector.lock()
            .map_err(|e| common::DbError::Other(format!("Lock poisoned: {}", e)))?;
        vec_idx.add_vector(embedding.id.as_str(), embedding.vector.clone())
    }

    /// Removes an embedding from the vector index.
    pub fn unindex_embedding(&self, embedding_id: &str) -> DbResult<()> {
        let mut vec_idx = self.vector.lock()
            .map_err(|e| common::DbError::Other(format!("Lock poisoned: {}", e)))?;
        vec_idx.remove_vector(embedding_id)?;
        Ok(())
    }

    // --- Query Methods ---

    /// Queries the structural index for nodes matching a property value.
    ///
    /// # Arguments
    ///
    /// * `property` - The property name (e.g., "chat_id", "sender")
    /// * `value` - The value to match
    ///
    /// # Returns
    ///
    /// A vector of node IDs matching the query.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use indexing::IndexManager;
    /// # fn example(index_manager: &IndexManager) -> Result<(), Box<dyn std::error::Error>> {
    /// // Find all messages in a specific chat
    /// let message_ids = index_manager.get_nodes_by_property("chat_id", "chat_123")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_nodes_by_property(&self, property: &str, value: &str) -> DbResult<Vec<NodeId>> {
        self.structural.get(property, value)
    }

    /// Gets all outgoing edges from a node.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The ID of the node
    ///
    /// # Returns
    ///
    /// A vector of edge IDs for outgoing edges.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use indexing::IndexManager;
    /// # fn example(index_manager: &IndexManager) -> Result<(), Box<dyn std::error::Error>> {
    /// // Find all edges pointing from a chat node
    /// let outgoing = index_manager.get_outgoing_edges("chat_123")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_outgoing_edges(&self, node_id: &str) -> DbResult<Vec<EdgeId>> {
        self.graph.get_outgoing(node_id)
    }

    /// Gets all incoming edges to a node.
    pub fn get_incoming_edges(&self, node_id: &str) -> DbResult<Vec<EdgeId>> {
        self.graph.get_incoming(node_id)
    }

    /// Performs semantic similarity search using vector embeddings.
    ///
    /// # Arguments
    ///
    /// * `query` - The query vector
    /// * `k` - Number of nearest neighbors to return
    ///
    /// # Returns
    ///
    /// A vector of `SearchResult` structs with embedding IDs and distances.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use indexing::IndexManager;
    /// # fn example(index_manager: &IndexManager) -> Result<(), Box<dyn std::error::Error>> {
    /// // Semantic search (vector from ML model)
    /// let query_vector = vec![0.1; 384];
    /// let similar = index_manager.search_vectors(&query_vector, 10)?;
    /// for result in similar {
    ///     println!("ID: {}, Distance: {}", result.id, result.distance);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn search_vectors(&self, query: &[f32], k: usize) -> DbResult<Vec<SearchResult>> {
        let vec_idx = self.vector.lock()
            .map_err(|e| common::DbError::Other(format!("Lock poisoned: {}", e)))?;
        vec_idx.search(query, k)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use serde_json::json;
    use common::{NodeId, EdgeId, EmbeddingId};
    use common::models::{Chat, Message, Edge, Embedding};
    
    fn create_test_manager() -> (IndexManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db = sled::open(temp_dir.path()).unwrap();
        let manager = IndexManager::new(&db).unwrap();
        (manager, temp_dir)
    }

    #[test]
    fn test_index_chat_node() {
        let (manager, _temp) = create_test_manager();
        
        let chat = Node::Chat(Chat {
            id: NodeId::from("chat_001"),
            title: "Test Chat".to_string(),
            topic: "Testing".to_string(),
            created_at: 1697500000000,
            updated_at: 1697500000000,
            message_ids: vec![],
            summary_ids: vec![],
            embedding_id: None,
            metadata: json!({}),
        });
        
        manager.index_node(&chat).unwrap();
        
        let results = manager.get_nodes_by_property("node_type", "Chat").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], NodeId::from("chat_001"));
        
        let topic_results = manager.get_nodes_by_property("topic", "Testing").unwrap();
        assert_eq!(topic_results.len(), 1);
    }

    #[test]
    fn test_index_message_node() {
        let (manager, _temp) = create_test_manager();
        
        let message = Node::Message(Message {
            id: NodeId::from("msg_001"),
            chat_id: NodeId::from("chat_123"),
            sender: "user".to_string(),
            timestamp: 1697500000000,
            text_content: "Hello".to_string(),
            attachment_ids: vec![],
            embedding_id: None,
            metadata: json!({}),
        });
        
        manager.index_node(&message).unwrap();
        
        let results = manager.get_nodes_by_property("chat_id", "chat_123").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], NodeId::from("msg_001"));
    }

    #[test]
    fn test_index_edge() {
        let (manager, _temp) = create_test_manager();
        
        let edge = Edge {
            id: EdgeId::from("edge_001"),
            from_node: NodeId::from("chat_123"),
            to_node: NodeId::from("msg_456"),
            edge_type: "CONTAINS".to_string(),
            created_at: 1697500000000,
            metadata: json!({}),
        };
        
        manager.index_edge(&edge).unwrap();
        
        let outgoing = manager.get_outgoing_edges("chat_123").unwrap();
        assert_eq!(outgoing.len(), 1);
        assert_eq!(outgoing[0], EdgeId::from("edge_001"));
    }

    #[test]
    fn test_index_embedding() {
        let (manager, _temp) = create_test_manager();
        
        let embedding = Embedding {
            id: EmbeddingId::from("embed_001"),
            vector: vec![0.1, 0.2, 0.3],
            model: "test-model".to_string(),
        };
        
        manager.index_embedding(&embedding).unwrap();
        
        let query = vec![0.1, 0.2, 0.3];
        let results = manager.search_vectors(&query, 5).unwrap();
        assert!(!results.is_empty());
    }
}
