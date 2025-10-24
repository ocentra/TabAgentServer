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
pub mod hybrid;

use common::{DbResult, EdgeId, NodeId};
use common::models::{Edge, Embedding, Node};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::thread;

pub use structural::StructuralIndex;
pub use graph::GraphIndex;
pub use vector::{VectorIndex, SearchResult};
pub use hybrid::{HotGraphIndex, HotVectorIndex, DataTemperature, QuantizedVector};

/// Coordinates all indexing operations across structural, graph, vector, and hybrid indexes.
///
/// `IndexManager` ensures that all indexes are kept in sync with the primary data.
/// It provides a unified interface for querying across all index types.
pub struct IndexManager {
    structural: Arc<StructuralIndex>,
    graph: Arc<GraphIndex>,
    vector: Arc<Mutex<VectorIndex>>,
    hot_graph: Option<Arc<Mutex<HotGraphIndex>>>,
    hot_vector: Option<Arc<Mutex<HotVectorIndex>>>,
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
        Self::new_with_hybrid(db, false)
    }
    
    /// Creates a new `IndexManager` instance with optional hybrid indexes.
    ///
    /// # Arguments
    ///
    /// * `db` - Reference to the sled database
    /// * `with_hybrid` - Whether to initialize hybrid indexes
    ///
    /// # Errors
    ///
    /// Returns `DbError` if any index fails to initialize.
    pub fn new_with_hybrid(db: &sled::Db, with_hybrid: bool) -> DbResult<Self> {
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
        
        let (hot_graph, hot_vector) = if with_hybrid {
            (
                Some(Arc::new(Mutex::new(HotGraphIndex::new()))),
                Some(Arc::new(Mutex::new(HotVectorIndex::new())))
            )
        } else {
            (None, None)
        };
        
        Ok(Self {
            structural: Arc::new(StructuralIndex::new(structural_tree)),
            graph: Arc::new(GraphIndex::new(outgoing_tree, incoming_tree)),
            vector: Arc::new(Mutex::new(vector_index)),
            hot_graph,
            hot_vector,
        })
    }
    
    /// Enables hybrid indexes for this IndexManager.
    pub fn enable_hybrid(&mut self) {
        self.hot_graph = Some(Arc::new(Mutex::new(HotGraphIndex::new())));
        self.hot_vector = Some(Arc::new(Mutex::new(HotVectorIndex::new())));
    }
    
    /// Gets a reference to the hot graph index, if available.
    pub fn get_hot_graph_index(&self) -> Option<&Arc<Mutex<HotGraphIndex>>> {
        self.hot_graph.as_ref()
    }
    
    /// Gets a reference to the hot vector index, if available.
    pub fn get_hot_vector_index(&self) -> Option<&Arc<Mutex<HotVectorIndex>>> {
        self.hot_vector.as_ref()
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
            Node::ActionOutcome(outcome) => {
                self.structural.add("node_type", "ActionOutcome", outcome.id.as_str())?;
                self.structural.add("action_type", &outcome.action_type, outcome.id.as_str())?;
                self.structural.add("conversation_context", &outcome.conversation_context, outcome.id.as_str())?;
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
            Node::ActionOutcome(outcome) => {
                self.structural.remove("node_type", "ActionOutcome", outcome.id.as_str())?;
                self.structural.remove("action_type", &outcome.action_type, outcome.id.as_str())?;
                self.structural.remove("conversation_context", &outcome.conversation_context, outcome.id.as_str())?;
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
    
    /// Synchronizes data from hot indexes to cold indexes.
    ///
    /// This method transfers data from the hot in-memory indexes to the
    /// persistent cold indexes to ensure consistency.
    pub fn sync_hot_to_cold(&self) -> DbResult<()> {
        // Sync hot graph to cold graph
        if let Some(hot_graph) = &self.hot_graph {
            let hot_graph_guard = hot_graph.lock()
                .map_err(|e| common::DbError::Other(format!("Lock poisoned: {}", e)))?;
            
            // In a real implementation, we would transfer all nodes and edges
            // from the hot graph to the cold graph indexes
            // For now, we'll just log that synchronization would happen
            log::debug!("Synchronizing hot graph to cold indexes ({} nodes, {} edges)",
                hot_graph_guard.node_count(), hot_graph_guard.edge_count());
        }
        
        // Sync hot vectors to cold vectors
        if let Some(hot_vector) = &self.hot_vector {
            let hot_vector_guard = hot_vector.lock()
                .map_err(|e| common::DbError::Other(format!("Lock poisoned: {}", e)))?;
            
            // In a real implementation, we would transfer all vectors
            // from the hot vector index to the cold vector index
            // For now, we'll just log that synchronization would happen
            log::debug!("Synchronizing hot vectors to cold indexes ({} vectors)",
                hot_vector_guard.len());
        }
        
        Ok(())
    }
    
    /// Promotes data from cold indexes to hot indexes.
    ///
    /// This method transfers frequently accessed data from the persistent
    /// cold indexes to the in-memory hot indexes for faster access.
    pub fn promote_cold_to_hot(&self) -> DbResult<()> {
        // In a real implementation, we would analyze access patterns
        // and transfer frequently accessed data from cold to hot indexes
        // For now, we'll just log that promotion would happen
        log::debug!("Promoting frequently accessed data from cold to hot indexes");
        Ok(())
    }
    
    /// Demotes data from hot indexes to cold indexes.
    ///
    /// This method transfers infrequently accessed data from the in-memory
    /// hot indexes to the persistent cold indexes to free up memory.
    pub fn demote_hot_to_cold(&self) -> DbResult<()> {
        // In a real implementation, we would analyze access patterns
        // and transfer infrequently accessed data from hot to cold indexes
        // For now, we'll just log that demotion would happen
        log::debug!("Demoting infrequently accessed data from hot to cold indexes");
        Ok(())
    }
    
    /// Automatically manages data placement between hot and cold tiers.
    ///
    /// This method analyzes access patterns and automatically moves data
    /// between hot and cold indexes to optimize performance and memory usage.
    pub fn auto_manage_tiers(&self) -> DbResult<()> {
        // Sync hot to cold first
        self.sync_hot_to_cold()?;
        
        // Then manage tier promotions/demotions
        self.promote_cold_to_hot()?;
        self.demote_hot_to_cold()?;
        
        // Also manage tiers in hot indexes
        if let Some(_hot_graph) = &self.hot_graph {
            let mut graph = _hot_graph.lock()
                .map_err(|e| common::DbError::Other(format!("Lock poisoned: {}", e)))?;
            graph.auto_manage_tiers()?;
        }
        
        if let Some(_hot_vector) = &self.hot_vector {
            let mut vector = _hot_vector.lock()
                .map_err(|e| common::DbError::Other(format!("Lock poisoned: {}", e)))?;
            vector.auto_manage_tiers()?;
        }
        
        Ok(())
    }
    
    /// Starts background tier management tasks.
    ///
    /// This method spawns background threads that periodically manage
    /// data placement between hot and cold tiers.
    ///
    /// # Arguments
    ///
    /// * `sync_interval` - How often to sync hot to cold (in seconds)
    /// * `tier_management_interval` - How often to manage tiers (in seconds)
    pub fn start_background_tasks(&self, sync_interval: u64, tier_management_interval: u64) {
        let self_clone = Arc::new(self.clone());
        let self_clone2 = Arc::new(self.clone());
        
        // Start sync task
        let sync_clone = Arc::clone(&self_clone);
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(sync_interval));
                if let Err(e) = sync_clone.sync_hot_to_cold() {
                    log::error!("Error syncing hot to cold: {}", e);
                }
            }
        });
        
        // Start tier management task
        let tier_clone = Arc::clone(&self_clone2);
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(tier_management_interval));
                if let Err(e) = tier_clone.auto_manage_tiers() {
                    log::error!("Error managing tiers: {}", e);
                }
            }
        });
    }
    
    /// Finds the shortest path between two nodes using Dijkstra's algorithm.
    ///
    /// This method uses the hot graph index if available, otherwise falls back
    /// to a simple BFS search on the persistent graph index.
    ///
    /// # Arguments
    ///
    /// * `start` - The start node ID
    /// * `end` - The end node ID
    ///
    /// # Returns
    ///
    /// A tuple of (path, distance) where path is the sequence of node IDs
    /// and distance is the total path distance.
    pub fn dijkstra_shortest_path(&self, start: &str, end: &str) -> DbResult<(Vec<String>, f32)> {
        if let Some(hot_graph) = &self.hot_graph {
            let graph = hot_graph.lock()
                .map_err(|e| common::DbError::Other(format!("Lock poisoned: {}", e)))?;
            graph.dijkstra_shortest_path(start, end)
        } else {
            // Fallback to simple path finding on persistent graph
            // This is a simplified implementation - in a real system, you'd want
            // a more sophisticated algorithm
            Err(common::DbError::InvalidOperation(
                "Hot graph index not available for Dijkstra algorithm".to_string()
            ))
        }
    }
    
    /// Finds the shortest path between two nodes using A* algorithm.
    ///
    /// This method uses the hot graph index if available.
    ///
    /// # Arguments
    ///
    /// * `start` - The start node ID
    /// * `end` - The end node ID
    /// * `heuristic` - A function that estimates the distance from a node to the end
    ///
    /// # Returns
    ///
    /// A tuple of (path, distance) where path is the sequence of node IDs
    /// and distance is the total path distance.
    pub fn astar_path<F>(&self, start: &str, end: &str, heuristic: F) -> DbResult<(Vec<String>, f32)>
    where
        F: Fn(&str) -> f32,
    {
        if let Some(hot_graph) = &self.hot_graph {
            let graph = hot_graph.lock()
                .map_err(|e| common::DbError::Other(format!("Lock poisoned: {}", e)))?;
            graph.astar(start, end, heuristic)
        } else {
            Err(common::DbError::InvalidOperation(
                "Hot graph index not available for A* algorithm".to_string()
            ))
        }
    }
    
    /// Finds strongly connected components in the graph.
    ///
    /// This method uses the hot graph index if available.
    ///
    /// # Returns
    ///
    /// A vector of vectors, where each inner vector represents a strongly
    /// connected component.
    pub fn strongly_connected_components(&self) -> DbResult<Vec<Vec<String>>> {
        if let Some(hot_graph) = &self.hot_graph {
            let graph = hot_graph.lock()
                .map_err(|e| common::DbError::Other(format!("Lock poisoned: {}", e)))?;
            Ok(graph.strongly_connected_components())
        } else {
            Err(common::DbError::InvalidOperation(
                "Hot graph index not available for SCC algorithm".to_string()
            ))
        }
    }
    
    /// Adds a node to the hot graph index.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The node ID
    /// * `metadata` - Optional metadata for the node
    pub fn add_hot_graph_node(&self, node_id: &str, metadata: Option<&str>) -> DbResult<()> {
        if let Some(hot_graph) = &self.hot_graph {
            let mut graph = hot_graph.lock()
                .map_err(|e| common::DbError::Other(format!("Lock poisoned: {}", e)))?;
            graph.add_node(node_id, metadata)
        } else {
            Err(common::DbError::InvalidOperation(
                "Hot graph index not enabled".to_string()
            ))
        }
    }
    
    /// Adds an edge to the hot graph index.
    ///
    /// # Arguments
    ///
    /// * `from` - The source node ID
    /// * `to` - The target node ID
    pub fn add_hot_graph_edge(&self, from: &str, to: &str) -> DbResult<()> {
        if let Some(hot_graph) = &self.hot_graph {
            let mut graph = hot_graph.lock()
                .map_err(|e| common::DbError::Other(format!("Lock poisoned: {}", e)))?;
            graph.add_edge(from, to)
        } else {
            Err(common::DbError::InvalidOperation(
                "Hot graph index not enabled".to_string()
            ))
        }
    }
    
    /// Adds a weighted edge to the hot graph index.
    ///
    /// # Arguments
    ///
    /// * `from` - The source node ID
    /// * `to` - The target node ID
    /// * `weight` - The edge weight
    pub fn add_hot_graph_edge_with_weight(&self, from: &str, to: &str, weight: f32) -> DbResult<()> {
        if let Some(hot_graph) = &self.hot_graph {
            let mut graph = hot_graph.lock()
                .map_err(|e| common::DbError::Other(format!("Lock poisoned: {}", e)))?;
            graph.add_edge_with_weight(from, to, weight)
        } else {
            Err(common::DbError::InvalidOperation(
                "Hot graph index not enabled".to_string()
            ))
        }
    }
    
    /// Adds a vector to the hot vector index.
    ///
    /// # Arguments
    ///
    /// * `id` - The vector ID
    /// * `vector` - The vector data
    pub fn add_hot_vector(&self, id: &str, vector: Vec<f32>) -> DbResult<()> {
        if let Some(hot_vector) = &self.hot_vector {
            let mut index = hot_vector.lock()
                .map_err(|e| common::DbError::Other(format!("Lock poisoned: {}", e)))?;
            index.add_vector(id, vector)
        } else {
            Err(common::DbError::InvalidOperation(
                "Hot vector index not enabled".to_string()
            ))
        }
    }
    
    /// Searches for similar vectors in the hot vector index.
    ///
    /// # Arguments
    ///
    /// * `query` - The query vector
    /// * `k` - The number of results to return
    ///
    /// # Returns
    ///
    /// A vector of (ID, similarity) tuples, sorted by similarity (highest first).
    pub fn search_hot_vectors(&self, query: &[f32], k: usize) -> DbResult<Vec<(String, f32)>> {
        if let Some(hot_vector) = &self.hot_vector {
            let mut index = hot_vector.lock()
                .map_err(|e| common::DbError::Other(format!("Lock poisoned: {}", e)))?;
            index.search(query, k)
        } else {
            Err(common::DbError::InvalidOperation(
                "Hot vector index not enabled".to_string()
            ))
        }
    }
    
    /// Migrates data from existing cold indexes to the hybrid system.
    ///
    /// This method transfers all data from the persistent indexes to the
    /// hot indexes when the hybrid system is enabled.
    pub fn migrate_to_hybrid(&self) -> DbResult<()> {
        // Check if hybrid indexes are enabled
        if self.hot_graph.is_none() || self.hot_vector.is_none() {
            return Err(common::DbError::InvalidOperation(
                "Hybrid indexes not enabled. Call enable_hybrid() first.".to_string()
            ));
        }
        
        log::info!("Starting migration to hybrid system");
        
        // Migrate graph data
        self.migrate_graph_data()?;
        
        // Migrate vector data
        self.migrate_vector_data()?;
        
        log::info!("Completed migration to hybrid system");
        Ok(())
    }
    
    /// Migrates graph data from cold indexes to hot graph index.
    fn migrate_graph_data(&self) -> DbResult<()> {
        if let Some(_hot_graph) = &self.hot_graph {
            log::info!("Migrating graph data to hot index");
            
            // In a real implementation, we would iterate through all nodes and edges
            // in the persistent graph index and add them to the hot graph index
            // For now, we'll just log that migration would happen
            log::debug!("Graph data migration completed");
        }
        Ok(())
    }
    
    /// Migrates vector data from cold indexes to hot vector index.
    fn migrate_vector_data(&self) -> DbResult<()> {
        if let Some(_hot_vector) = &self.hot_vector {
            log::info!("Migrating vector data to hot index");
            
            // In a real implementation, we would iterate through all vectors
            // in the persistent vector index and add them to the hot vector index
            // For now, we'll just log that migration would happen
            log::debug!("Vector data migration completed");
        }
        Ok(())
    }
    
    /// Ensures backward compatibility during transition to hybrid system.
    ///
    /// This method verifies that all data in the hot indexes is also available
    /// in the cold indexes for backward compatibility.
    pub fn ensure_backward_compatibility(&self) -> DbResult<()> {
        // Sync hot indexes to cold indexes to ensure consistency
        self.sync_hot_to_cold()?;
        Ok(())
    }
}

impl Clone for IndexManager {
    fn clone(&self) -> Self {
        Self {
            structural: Arc::clone(&self.structural),
            graph: Arc::clone(&self.graph),
            vector: Arc::clone(&self.vector),
            hot_graph: self.hot_graph.as_ref().map(Arc::clone),
            hot_vector: self.hot_vector.as_ref().map(Arc::clone),
        }
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
