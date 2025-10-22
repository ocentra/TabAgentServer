//! Query data models for the converged query pipeline.
//!
//! These structures define the API contract for expressing multi-faceted queries
//! that combine structural filters, graph traversals, and semantic search.

use common::{models::Node, models::Edge, NodeId};
use serde::{Deserialize, Serialize};

// --- Query Structures ---

/// The top-level structure for a converged query.
///
/// A converged query can combine multiple query facets:
/// - Structural filters on indexed properties
/// - Graph filters based on relationships
/// - Semantic search using vector embeddings
///
/// # Examples
///
/// ```
/// use query::models::{ConvergedQuery, SemanticQuery, StructuralFilter, FilterOperator};
/// use serde_json::json;
///
/// let query = ConvergedQuery {
///     semantic_query: Some(SemanticQuery {
///         vector: vec![0.1, 0.2, 0.3],
///         similarity_threshold: Some(0.8),
///     }),
///     structural_filters: Some(vec![StructuralFilter {
///         property_name: "node_type".to_string(),
///         operator: FilterOperator::Equals,
///         value: json!("Message"),
///     }]),
///     graph_filter: None,
///     limit: 10,
///     offset: 0,
///};
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConvergedQuery {
    /// Optional semantic search component
    pub semantic_query: Option<SemanticQuery>,
    
    /// Optional structural filters (AND'd together)
    pub structural_filters: Option<Vec<StructuralFilter>>,
    
    /// Optional graph traversal filter
    pub graph_filter: Option<GraphFilter>,
    
    /// Maximum number of results to return
    pub limit: usize,
    
    /// Number of results to skip (for pagination)
    pub offset: usize,
}

/// Defines a semantic search component using vector embeddings.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SemanticQuery {
    /// The query vector for similarity search
    pub vector: Vec<f32>,
    
    /// Optional threshold for filtering by similarity score
    /// Results with scores below this threshold will be excluded
    pub similarity_threshold: Option<f32>,
}

/// Defines a filter on a core, indexed property of a node.
///
/// Structural filters enable fast, exact matching on indexed fields
/// like `node_type`, `chat_id`, `sender`, `timestamp`, etc.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StructuralFilter {
    /// The property name to filter on (e.g., "chat_id", "sender")
    pub property_name: String,
    
    /// The comparison operator
    pub operator: FilterOperator,
    
    /// The value to compare against
    pub value: serde_json::Value,
}

/// Comparison operators for structural filters.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum FilterOperator {
    /// Exact equality match
    Equals,
    
    /// Not equal to
    NotEquals,
    
    /// Greater than (for numeric/timestamp fields)
    GreaterThan,
    
    /// Less than (for numeric/timestamp fields)
    LessThan,
    
    /// Greater than or equal to
    GreaterThanOrEqual,
    
    /// Less than or equal to
    LessThanOrEqual,
}

/// Defines a filter based on graph relationships.
///
/// Graph filters enable traversal-based queries like:
/// - "Find all nodes connected to X"
/// - "Find nodes within 2 hops of Y via MENTIONS edges"
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GraphFilter {
    /// The starting node for the traversal
    pub start_node_id: NodeId,
    
    /// Direction of edges to follow
    pub direction: EdgeDirection,
    
    /// Optional filter for specific edge types (e.g., "MENTIONS")
    pub edge_type: Option<String>,
    
    /// Traversal depth (1 for direct neighbors, >1 for multi-hop)
    pub depth: u32,
}

/// Direction for edge traversal in graph filters.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum EdgeDirection {
    /// Follow outgoing edges (from start node to others)
    Outbound,
    
    /// Follow incoming edges (from others to start node)
    Inbound,
    
    /// Follow edges in both directions
    Both,
}

// --- Result Structures ---

/// Represents a single item in the query result set.
///
/// Contains the node and an optional similarity score if semantic
/// search was performed.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QueryResult {
    /// The node matching the query
    pub node: Node,
    
    /// Similarity score (0.0-1.0) if semantic search was used
    pub similarity_score: Option<f32>,
}

/// Represents a path found during a graph traversal.
///
/// Useful for shortest path queries or visualizing relationships.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Path {
    /// Ordered list of nodes in the path
    pub nodes: Vec<Node>,
    
    /// Ordered list of edges connecting the nodes
    pub edges: Vec<Edge>,
}

