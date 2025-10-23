//! Query engine for the embedded database.
//!
//! This crate implements the Converged Query Pipeline, which fuses structural filters,
//! graph traversals, and semantic search into a unified, multi-stage query execution engine.
//!
//! # Architecture
//!
//! The query pipeline executes in two stages:
//!
//! ## Stage 1: Candidate Set Generation
//! - Apply structural filters using secondary indexes for fast exact matching
//! - Apply graph filters using traversal algorithms
//! - Intersect the results to get an accurate candidate set
//!
//! ## Stage 2: Semantic Re-ranking
//! - Perform vector search only on the candidate set (if semantic query provided)
//! - Rank results by similarity score
//! - Apply pagination and return final results
//!
//! # Examples
//!
//! ```
//! use query::{QueryManager, models::*};
//! use storage::StorageManager;
//! use indexing::IndexManager;
//! use serde_json::json;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Initialize components
//! let storage = StorageManager::new("test_db")?;
//! let indexing = IndexManager::new(storage.db())?;
//! let query_mgr = QueryManager::new(&storage, &indexing);
//!
//! // Build a converged query
//! let query = ConvergedQuery {
//!     structural_filters: Some(vec![StructuralFilter {
//!         property_name: "chat_id".to_string(),
//!         operator: FilterOperator::Equals,
//!         value: json!("chat_123"),
//!     }]),
//!     semantic_query: Some(SemanticQuery {
//!         vector: vec![0.1, 0.2, 0.3],
//!         similarity_threshold: Some(0.7),
//!     }),
//!     graph_filter: None,
//!     limit: 10,
//!     offset: 0,
//! };
//!
//! // Execute the query
//! let results = query_mgr.query(&query)?;
//! # Ok(())
//! # }
//! ```

pub mod models;

use common::{DbError, NodeId};
use hashbrown::{HashMap, HashSet};
use indexing::IndexManager;
use models::*;
use storage::DatabaseCoordinator;

// Re-export key types for convenience
pub use models::{ConvergedQuery, QueryResult};

/// Error type for query operations.
#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    /// Error from the storage layer
    #[error("Storage error: {0}")]
    Storage(#[from] DbError),
    
    /// Invalid query parameters
    #[error("Invalid query: {0}")]
    InvalidQuery(String),
    
    /// Graph traversal error
    #[error("Graph traversal error: {0}")]
    GraphTraversal(String),
}

/// Result type for query operations.
pub type QueryResult2<T> = Result<T, QueryError>;

/// The central query manager that orchestrates multi-stage query execution.
///
/// `QueryManager` holds references to the storage and indexing layers and
/// implements the Converged Query Pipeline, executing queries in an optimized
/// multi-stage process.
pub struct QueryManager<'a> {
    coordinator: &'a DatabaseCoordinator,
    indexing: &'a IndexManager,
    query_cache: HashMap<String, Vec<QueryResult>>,
}

impl<'a> QueryManager<'a> {
    /// Creates a new QueryManager with references to storage and indexing layers.
    ///
    /// # Examples
    ///
    /// ```
    /// # use query::QueryManager;
    /// # use storage::StorageManager;
    /// # use indexing::IndexManager;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let storage = StorageManager::new("test_db")?;
    /// let indexing = IndexManager::new(storage.db())?;
    /// let query_mgr = QueryManager::new(&storage, &indexing);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(coordinator: &'a DatabaseCoordinator, indexing: &'a IndexManager) -> Self {
        Self { coordinator, indexing, query_cache: HashMap::new() }
    }

    /// The main entry point for all queries in the database.
    ///
    /// This method implements the full Converged Query Pipeline, executing
    /// structural filters, graph traversals, and semantic search in an
    /// optimized multi-stage process.
    ///
    /// # Arguments
    ///
    /// * `query` - The converged query specification
    ///
    /// # Returns
    ///
    /// A vector of `QueryResult` items, ordered by relevance if semantic
    /// search was performed, or in arbitrary order otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use query::{QueryManager, models::*};
    /// # use storage::StorageManager;
    /// # use indexing::IndexManager;
    /// # use serde_json::json;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let storage = StorageManager::new("test_db")?;
    /// # let indexing = IndexManager::new(storage.db())?;
    /// # let query_mgr = QueryManager::new(&storage, &indexing);
    /// let query = ConvergedQuery {
    ///     structural_filters: Some(vec![StructuralFilter {
    ///         property_name: "node_type".to_string(),
    ///         operator: FilterOperator::Equals,
    ///         value: json!("Message"),
    ///     }]),
    ///     semantic_query: None,
    ///     graph_filter: None,
    ///     limit: 10,
    ///     offset: 0,
    /// };
    ///
    /// let results = query_mgr.query(&query)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn query(&self, query: &ConvergedQuery) -> QueryResult2<Vec<QueryResult>> {
        // --- STAGE 1: Candidate Set Generation ---

        // 1a. Apply structural filters
        let structural_candidates = self.apply_structural_filters(&query.structural_filters)?;

        // 1b. Apply graph filters
        let graph_candidates = self.apply_graph_filter(&query.graph_filter)?;

        // 1c. Intersect candidate sets to get the final, accurate set
        let final_candidates = self.intersect_candidates(structural_candidates, graph_candidates);

        // --- STAGE 2: Fetching & Semantic Re-ranking ---

        let results = if let Some(semantic_query) = &query.semantic_query {
            // If there's a semantic component, search within the candidate set and re-rank
            self.fetch_and_rank_by_similarity(
                &semantic_query.vector,
                final_candidates,
                semantic_query.similarity_threshold,
                query.limit,
                query.offset,
            )?
        } else {
            // If no semantic component, just fetch the filtered candidates
            self.fetch_nodes(final_candidates, query.limit, query.offset)?
        };

        Ok(results)
    }

    // --- STAGE 1 HELPER METHODS: Candidate Set Generation ---

    /// Applies all structural filters and returns the intersection of matching node IDs.
    ///
    /// If no structural filters are provided, returns `None` (meaning "all nodes").
    /// Multiple filters are combined with AND logic.
    fn apply_structural_filters(
        &self,
        filters: &Option<Vec<StructuralFilter>>,
    ) -> QueryResult2<Option<HashSet<NodeId>>> {
        let Some(filters) = filters else {
            return Ok(None); // No filters = all nodes
        };

        if filters.is_empty() {
            return Ok(None);
        }

        // Start with the first filter's results
        let mut result_set: Option<HashSet<NodeId>> = None;

        for filter in filters {
            let filter_results = self.apply_single_structural_filter(filter)?;

            result_set = match result_set {
                None => Some(filter_results),
                Some(existing) => {
                    // Intersect with existing results (AND logic)
                    Some(existing.intersection(&filter_results).cloned().collect())
                }
            };

            // Early exit if intersection becomes empty
            if result_set.as_ref().map_or(false, |s| s.is_empty()) {
                return Ok(Some(HashSet::new()));
            }
        }

        Ok(result_set)
    }

    /// Applies a single structural filter using the indexing layer.
    fn apply_single_structural_filter(
        &self,
        filter: &StructuralFilter,
    ) -> QueryResult2<HashSet<NodeId>> {
        let value_str = match &filter.value {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            other => serde_json::to_string(other).map_err(|e| {
                QueryError::InvalidQuery(format!("Cannot convert filter value to string: {}", e))
            })?,
        };

        match filter.operator {
            FilterOperator::Equals => {
                let nodes = self
                    .indexing
                    .get_nodes_by_property(&filter.property_name, &value_str)?;
                Ok(nodes.into_iter().collect())
            }
            FilterOperator::NotEquals => {
                // This requires fetching all nodes and filtering
                // For now, return an error as it's not efficiently supported
                Err(QueryError::InvalidQuery(
                    "NotEquals operator not yet supported efficiently".to_string(),
                ))
            }
            FilterOperator::GreaterThan
            | FilterOperator::LessThan
            | FilterOperator::GreaterThanOrEqual
            | FilterOperator::LessThanOrEqual => {
                // Range queries would require a different index structure (e.g., B-tree)
                Err(QueryError::InvalidQuery(
                    "Comparison operators not yet supported".to_string(),
                ))
            }
        }
    }

    /// Applies a graph filter by traversing the graph from the start node.
    ///
    /// Returns `None` if no graph filter is provided (meaning "all nodes").
    fn apply_graph_filter(
        &self,
        filter: &Option<GraphFilter>,
    ) -> QueryResult2<Option<HashSet<NodeId>>> {
        let Some(filter) = filter else {
            return Ok(None); // No graph filter = all nodes
        };

        let mut visited = HashSet::new();
        let mut current_level = HashSet::new();
        current_level.insert(filter.start_node_id.clone());

        // BFS traversal up to the specified depth
        for _ in 0..filter.depth {
            let mut next_level = HashSet::new();

            for node_id in &current_level {
                if visited.contains(node_id) {
                    continue;
                }
                visited.insert(node_id.clone());

                // Get edge IDs based on direction
                let edge_ids = match filter.direction {
                    EdgeDirection::Outbound => self.indexing.get_outgoing_edges(node_id.as_str())?,
                    EdgeDirection::Inbound => self.indexing.get_incoming_edges(node_id.as_str())?,
                    EdgeDirection::Both => {
                        let mut all_edges = self.indexing.get_outgoing_edges(node_id.as_str())?;
                        all_edges.extend(self.indexing.get_incoming_edges(node_id.as_str())?);
                        all_edges
                    }
                };

                // Fetch actual Edge objects and traverse
                for edge_id in edge_ids {
                    let Some(edge) = self.coordinator.conversations_active().get_edge(edge_id.as_str())? else {
                        continue; // Edge was deleted
                    };

                    // Filter by edge type if specified
                    if let Some(ref edge_type) = filter.edge_type {
                        if &edge.edge_type != edge_type {
                            continue;
                        }
                    }

                    // Add the target node to the next level
                    let target_node = match filter.direction {
                        EdgeDirection::Outbound => &edge.to_node,
                        EdgeDirection::Inbound => &edge.from_node,
                        EdgeDirection::Both => {
                            // For 'Both', we need to determine which end is not the current node
                            if &edge.from_node == node_id {
                                &edge.to_node
                            } else {
                                &edge.from_node
                            }
                        }
                    };

                    next_level.insert(target_node.clone());
                }
            }

            current_level = next_level;

            if current_level.is_empty() {
                break; // No more nodes to explore
            }
        }

        Ok(Some(visited))
    }

    /// Intersects multiple candidate sets.
    ///
    /// - If both are `None`, returns `None` (all nodes)
    /// - If one is `None`, returns the other (one filter applied)
    /// - If both are `Some`, returns their intersection (both filters applied)
    fn intersect_candidates(
        &self,
        set1: Option<HashSet<NodeId>>,
        set2: Option<HashSet<NodeId>>,
    ) -> Option<HashSet<NodeId>> {
        match (set1, set2) {
            (None, None) => None, // No filters = all nodes
            (Some(s), None) | (None, Some(s)) => Some(s), // One filter
            (Some(s1), Some(s2)) => Some(s1.intersection(&s2).cloned().collect()), // Both filters
        }
    }

    // --- STAGE 2 HELPER METHODS: Fetching & Ranking ---

    /// Fetches and ranks nodes by similarity to the query vector.
    ///
    /// If `candidates` is `None`, searches the entire vector index.
    /// If `candidates` is `Some`, only searches within those specific nodes.
    fn fetch_and_rank_by_similarity(
        &self,
        query_vector: &[f32],
        candidates: Option<HashSet<NodeId>>,
        similarity_threshold: Option<f32>,
        limit: usize,
        offset: usize,
    ) -> QueryResult2<Vec<QueryResult>> {
        // Perform vector search
        let search_results = self.indexing.search_vectors(query_vector, limit + offset)?;

        // Filter by candidates if provided
        let mut results: Vec<QueryResult> = search_results
            .into_iter()
            .filter_map(|search_result| {
                // Filter by candidate set if provided
                if let Some(ref candidates) = candidates {
                    // Convert EmbeddingId to NodeId for comparison
                    if !candidates.contains(&NodeId::from(search_result.id.as_str())) {
                        return None;
                    }
                }

                // Filter by similarity threshold if provided
                if let Some(threshold) = similarity_threshold {
                    if search_result.distance > threshold {
                        return None;
                    }
                }

                // Fetch the node
                match self.coordinator.get_message(search_result.id.as_str()).ok()? {
                    Some(msg) => Some(QueryResult {
                        node: common::models::Node::Message(msg),
                        similarity_score: Some(search_result.distance),
                    }),
                    None => None,
                }
            })
            .collect();

        // Apply pagination
        results = results.into_iter().skip(offset).take(limit).collect();

        Ok(results)
    }

    /// Fetches nodes from the candidate set without semantic ranking.
    ///
    /// If `candidates` is `None`, this would fetch all nodes (expensive!).
    /// For now, we return an error if no candidates are provided to avoid
    /// accidentally fetching the entire database.
    fn fetch_nodes(
        &self,
        candidates: Option<HashSet<NodeId>>,
        limit: usize,
        offset: usize,
    ) -> QueryResult2<Vec<QueryResult>> {
        let Some(candidates) = candidates else {
            return Err(QueryError::InvalidQuery(
                "Cannot fetch all nodes without filters. Add a structural or graph filter."
                    .to_string(),
            ));
        };

        let results: Vec<QueryResult> = candidates
            .into_iter()
            .skip(offset)
            .take(limit)
            .filter_map(|node_id| {
                match self.coordinator.get_message(node_id.as_str()).ok()? {
                    Some(msg) => Some(QueryResult {
                        node: common::models::Node::Message(msg),
                        similarity_score: None,
                    }),
                    None => None,
                }
            })
            .collect();

        Ok(results)
    }

    // --- HIGH-LEVEL CONVENIENCE APIs ---

    /// Finds the shortest path between two nodes using BFS.
    ///
    /// Returns `None` if no path exists, otherwise returns a `Path` object
    /// containing the ordered nodes and edges.
    ///
    /// # Examples
    ///
    /// ```
    /// # use query::QueryManager;
    /// # use storage::StorageManager;
    /// # use indexing::IndexManager;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let storage = StorageManager::new("test_db")?;
    /// # let indexing = IndexManager::new(storage.db())?;
    /// # let query_mgr = QueryManager::new(&storage, &indexing);
    /// let path = query_mgr.find_shortest_path("node_1", "node_2")?;
    ///
    /// if let Some(path) = path {
    ///     println!("Found path with {} nodes", path.nodes.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn find_shortest_path(
        &self,
        start_node_id: &str,
        end_node_id: &str,
    ) -> QueryResult2<Option<Path>> {
        use std::collections::VecDeque;

        // BFS to find shortest path
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent_map: HashMap<NodeId, (NodeId, common::models::Edge)> = HashMap::new();

        let start_node_id = NodeId::from(start_node_id);
        let end_node_id = NodeId::from(end_node_id);

        queue.push_back(start_node_id.clone());
        visited.insert(start_node_id.clone());

        let mut found = false;

        while let Some(current_id) = queue.pop_front() {
            if current_id == end_node_id {
                found = true;
                break;
            }

            // Get all outgoing edge IDs
            let edge_ids = self.indexing.get_outgoing_edges(current_id.as_str())?;

            for edge_id in edge_ids {
                let Some(edge) = self.coordinator.conversations_active().get_edge(edge_id.as_str())? else {
                    continue; // Edge was deleted
                };

                if !visited.contains(&edge.to_node) {
                    visited.insert(edge.to_node.clone());
                    parent_map.insert(edge.to_node.clone(), (current_id.clone(), edge.clone()));
                    queue.push_back(edge.to_node.clone());
                }
            }
        }

        if !found {
            return Ok(None); // No path found
        }

        // Reconstruct the path
        let mut path_nodes = Vec::new();
        let mut path_edges = Vec::new();
        let mut current = end_node_id.clone();

        // Build path in reverse
        while let Some((parent_id, edge)) = parent_map.get(&current) {
            path_edges.push(edge.clone());
            current = parent_id.clone();
        }

        // Reverse to get correct order
        path_edges.reverse();

        // Fetch all nodes in the path
        path_nodes.push(
            self.coordinator.conversations_active()
                .get_node(start_node_id.as_str())?
                .ok_or_else(|| QueryError::GraphTraversal("Start node not found".to_string()))?,
        );

        for edge in &path_edges {
            let node = self.coordinator.conversations_active()
                .get_node(edge.to_node.as_str())?
                .ok_or_else(|| QueryError::GraphTraversal("Path node not found".to_string()))?;
            path_nodes.push(node);
        }

        Ok(Some(Path {
            nodes: path_nodes,
            edges: path_edges,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::NodeId;
    use common::models::{Chat, Node};
    use serde_json::json;
    use storage::DatabaseCoordinator;

    #[test]
    fn test_structural_query() -> Result<(), Box<dyn std::error::Error>> {
        let coordinator = DatabaseCoordinator::new()?;
        let storage = coordinator.conversations_active();
        let indexing = storage.index_manager()
            .ok_or("Index manager not available")?;
        let query_mgr = QueryManager::new(&coordinator, indexing);

        // Insert test data
        let chat = Node::Chat(Chat {
            id: NodeId::from("chat_1"),
            title: "Test Chat".to_string(),
            topic: "Testing".to_string(),
            created_at: 1234567890,
            updated_at: 1234567890,
            message_ids: vec![],
            summary_ids: vec![],
            embedding_id: None,
            metadata: json!({}),
        });
        storage.insert_node(&chat)?;

        // Query for the chat
        let query = ConvergedQuery {
            structural_filters: Some(vec![StructuralFilter {
                property_name: "node_type".to_string(),
                operator: FilterOperator::Equals,
                value: json!("Chat"),
            }]),
            semantic_query: None,
            graph_filter: None,
            limit: 10,
            offset: 0,
        };

        let results = query_mgr.query(&query)?;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].node.id().as_str(), "chat_1");

        Ok(())
    }

    #[test]
    fn test_empty_result_set() -> Result<(), Box<dyn std::error::Error>> {
        let coordinator = DatabaseCoordinator::new()?;
        let storage = coordinator.conversations_active();
        let indexing = storage.index_manager()
            .ok_or("Index manager not available")?;
        let query_mgr = QueryManager::new(&coordinator, indexing);

        let query = ConvergedQuery {
            structural_filters: Some(vec![StructuralFilter {
                property_name: "node_type".to_string(),
                operator: FilterOperator::Equals,
                value: json!("NonExistentType"),
            }]),
            semantic_query: None,
            graph_filter: None,
            limit: 10,
            offset: 0,
        };

        let results = query_mgr.query(&query)?;
        assert_eq!(results.len(), 0);

        Ok(())
    }
}
