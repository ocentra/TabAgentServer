//! Optimized graph index implementation using integer indices.
//!
//! This module provides a more efficient graph implementation that uses
//! integer indices instead of string keys for better performance and memory usage.
//! It follows the graph traits defined in graph_traits.rs and implements
//! the HotGraphIndex functionality with better performance characteristics.

use crate::algorithms::graph_traits::*;
use common::{DbError, DbResult};
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

/// Node data structure for the optimized graph.
#[derive(Debug, Clone)]
pub struct NodeData {
    /// Node metadata
    pub metadata: Option<String>,
    
    /// Access tracking for temperature management
    pub access_tracker: AccessTracker,
}

/// Edge data structure for the optimized graph.
#[derive(Debug, Clone)]
pub struct EdgeData {
    /// Source node index
    pub from: NodeIndex,
    
    /// Target node index
    pub to: NodeIndex,
    
    /// Edge weight
    pub weight: f32,
    
    /// Access tracking for temperature management
    pub access_tracker: AccessTracker,
}

/// Access tracking information for temperature management.
#[derive(Debug, Clone)]
pub struct AccessTracker {
    /// Last access timestamp
    pub last_access: u64,
    
    /// Access count
    pub access_count: usize,
    
    /// Creation timestamp
    pub created_at: u64,
}

impl AccessTracker {
    /// Creates a new access tracker.
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        Self {
            last_access: now,
            access_count: 0,
            created_at: now,
        }
    }
    
    /// Records an access.
    pub fn record_access(&mut self) {
        self.last_access = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.access_count += 1;
    }
    
    /// Gets the age of the item in seconds.
    pub fn age(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now.saturating_sub(self.created_at)
    }
    
    /// Gets time since last access in seconds.
    pub fn time_since_access(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now.saturating_sub(self.last_access)
    }
}

/// Optimized graph index with integer indices for better performance.
pub struct OptimizedGraphIndex {
    /// Adjacency list representation for fast lookups
    adjacency_list: Vec<Vec<NodeIndex>>,
    
    /// Reverse adjacency list for incoming edges
    reverse_adjacency_list: Vec<Vec<NodeIndex>>,
    
    /// Node data
    nodes: Vec<Option<NodeData>>,
    
    /// Edge data
    edges: Vec<Option<EdgeData>>,
    
    /// Mapping from string node IDs to integer indices
    node_id_to_index: HashMap<String, NodeIndex>,
    
    /// Mapping from integer indices to string node IDs
    index_to_node_id: HashMap<NodeIndex, String>,
    
    /// Free node indices for reuse
    free_node_indices: Vec<NodeIndex>,
    
    /// Free edge indices for reuse
    free_edge_indices: Vec<EdgeIndex>,
    
    /// Performance monitoring statistics
    stats: HotGraphStats,
    
    /// Cache for frequently computed paths
    path_cache: HashMap<(NodeIndex, NodeIndex), Option<Vec<NodeIndex>>>,
    
    /// Precomputed centrality scores for fast access
    centrality_cache: HashMap<NodeIndex, f32>,
}

/// Performance monitoring statistics for OptimizedGraphIndex
#[derive(Debug, Clone)]
pub struct HotGraphStats {
    /// Number of nodes in the graph
    pub node_count: usize,
    
    /// Number of edges in the graph
    pub edge_count: usize,
    
    /// Total number of queries performed
    pub query_count: usize,
    
    /// Total time spent on queries (in microseconds)
    pub total_query_time_micros: u64,
    
    /// Number of tier promotions
    pub promotions: usize,
    
    /// Number of tier demotions
    pub demotions: usize,
}

impl HotGraphStats {
    /// Creates new statistics with default values
    pub fn new() -> Self {
        Self {
            node_count: 0,
            edge_count: 0,
            query_count: 0,
            total_query_time_micros: 0,
            promotions: 0,
            demotions: 0,
        }
    }
}

impl OptimizedGraphIndex {
    /// Creates a new OptimizedGraphIndex.
    pub fn new() -> Self {
        Self {
            adjacency_list: Vec::new(),
            reverse_adjacency_list: Vec::new(),
            nodes: Vec::new(),
            edges: Vec::new(),
            node_id_to_index: HashMap::new(),
            index_to_node_id: HashMap::new(),
            free_node_indices: Vec::new(),
            free_edge_indices: Vec::new(),
            stats: HotGraphStats::new(),
            path_cache: HashMap::new(),
            centrality_cache: HashMap::new(),
        }
    }
    
    /// Gets or creates a node index for a string node ID.
    fn get_or_create_node_index(&mut self, node_id: &str) -> DbResult<NodeIndex> {
        if let Some(index) = self.node_id_to_index.get(node_id) {
            Ok(*index)
        } else {
            // Create a new node
            let index = if let Some(free_index) = self.free_node_indices.pop() {
                // Reuse a free index
                free_index
            } else {
                // Create a new index
                NodeIndex(self.nodes.len())
            };
            
            // Ensure the adjacency lists are large enough
            while self.adjacency_list.len() <= index.0 {
                self.adjacency_list.push(Vec::new());
                self.reverse_adjacency_list.push(Vec::new());
            }
            
            // Add the node data
            if index.0 >= self.nodes.len() {
                self.nodes.resize(index.0 + 1, None);
            }
            self.nodes[index.0] = Some(NodeData {
                metadata: None,
                access_tracker: AccessTracker::new(),
            });
            
            // Update the mappings
            self.node_id_to_index.insert(node_id.to_string(), index);
            self.index_to_node_id.insert(index, node_id.to_string());
            
            self.stats.node_count += 1;
            Ok(index)
        }
    }
    
    /// Gets a node index for a string node ID, returning None if not found.
    fn get_node_index(&self, node_id: &str) -> Option<NodeIndex> {
        self.node_id_to_index.get(node_id).copied()
    }
    
    /// Gets a string node ID for a node index.
    fn get_node_id(&self, index: NodeIndex) -> Option<&String> {
        self.index_to_node_id.get(&index)
    }
    
    /// Adds a node to the graph.
    pub fn add_node(&mut self, node_id: &str, metadata: Option<&str>) -> DbResult<()> {
        let index = self.get_or_create_node_index(node_id)?;
        
        // Update metadata if provided
        if let Some(node_data) = &mut self.nodes[index.0] {
            if let Some(meta) = metadata {
                node_data.metadata = Some(meta.to_string());
            }
            node_data.access_tracker.record_access();
        }
        
        Ok(())
    }
    
    /// Removes a node from the graph.
    pub fn remove_node(&mut self, node_id: &str) -> DbResult<bool> {
        let index = match self.get_node_index(node_id) {
            Some(index) => index,
            None => return Ok(false),
        };
        
        // Remove from mappings
        self.node_id_to_index.remove(node_id);
        self.index_to_node_id.remove(&index);
        
        // Add to free indices for reuse
        self.free_node_indices.push(index);
        
        // Remove node data
        if index.0 < self.nodes.len() {
            self.nodes[index.0] = None;
        }
        
        // Remove references to this node from other nodes' adjacency lists
        for neighbors in &mut self.adjacency_list {
            neighbors.retain(|&n| n != index);
        }
        
        for neighbors in &mut self.reverse_adjacency_list {
            neighbors.retain(|&n| n != index);
        }
        
        self.stats.node_count -= 1;
        Ok(true)
    }
    
    /// Adds an edge to the graph.
    pub fn add_edge(&mut self, from: &str, to: &str) -> DbResult<()> {
        self.add_edge_with_weight(from, to, 1.0)
    }
    
    /// Adds a weighted edge to the graph.
    pub fn add_edge_with_weight(&mut self, from: &str, to: &str, weight: f32) -> DbResult<()> {
        let from_index = self.get_or_create_node_index(from)?;
        let to_index = self.get_or_create_node_index(to)?;
        
        // Create a new edge
        let edge_index = if let Some(free_index) = self.free_edge_indices.pop() {
            // Reuse a free index
            free_index
        } else {
            // Create a new index
            EdgeIndex(self.edges.len())
        };
        
        // Add the edge data
        if edge_index.0 >= self.edges.len() {
            self.edges.resize(edge_index.0 + 1, None);
        }
        self.edges[edge_index.0] = Some(EdgeData {
            from: from_index,
            to: to_index,
            weight,
            access_tracker: AccessTracker::new(),
        });
        
        // Add edge to adjacency lists
        self.adjacency_list[from_index.0].push(to_index);
        self.reverse_adjacency_list[to_index.0].push(from_index);
        
        self.stats.edge_count += 1;
        Ok(())
    }
    
    /// Removes an edge from the graph.
    pub fn remove_edge(&mut self, from: &str, to: &str) -> DbResult<bool> {
        let from_index = match self.get_node_index(from) {
            Some(index) => index,
            None => return Ok(false),
        };
        
        let to_index = match self.get_node_index(to) {
            Some(index) => index,
            None => return Ok(false),
        };
        
        // Remove from adjacency lists
        let mut removed = false;
        
        if from_index.0 < self.adjacency_list.len() {
            let initial_len = self.adjacency_list[from_index.0].len();
            self.adjacency_list[from_index.0].retain(|&n| n != to_index);
            removed = self.adjacency_list[from_index.0].len() < initial_len;
        }
        
        if to_index.0 < self.reverse_adjacency_list.len() {
            self.reverse_adjacency_list[to_index.0].retain(|&n| n != from_index);
        }
        
        if removed {
            self.stats.edge_count -= 1;
        }
        
        Ok(removed)
    }
    
    /// Gets outgoing neighbors of a node.
    pub fn get_outgoing_neighbors(&mut self, node_id: &str) -> DbResult<Vec<String>> {
        let index = match self.get_node_index(node_id) {
            Some(index) => index,
            None => return Ok(Vec::new()),
        };
        
        // Record access for temperature management
        if let Some(node_data) = &mut self.nodes[index.0] {
            node_data.access_tracker.record_access();
        }
        
        self.stats.query_count += 1;
        
        if index.0 < self.adjacency_list.len() {
            let neighbors = &self.adjacency_list[index.0];
            let mut result = Vec::with_capacity(neighbors.len());
            for &neighbor_index in neighbors {
                if let Some(neighbor_id) = self.get_node_id(neighbor_index) {
                    result.push(neighbor_id.clone());
                }
            }
            Ok(result)
        } else {
            Ok(Vec::new())
        }
    }
    
    /// Gets incoming neighbors of a node.
    pub fn get_incoming_neighbors(&mut self, node_id: &str) -> DbResult<Vec<String>> {
        let index = match self.get_node_index(node_id) {
            Some(index) => index,
            None => return Ok(Vec::new()),
        };
        
        // Record access for temperature management
        if let Some(node_data) = &mut self.nodes[index.0] {
            node_data.access_tracker.record_access();
        }
        
        self.stats.query_count += 1;
        
        if index.0 < self.reverse_adjacency_list.len() {
            let neighbors = &self.reverse_adjacency_list[index.0];
            let mut result = Vec::with_capacity(neighbors.len());
            for &neighbor_index in neighbors {
                if let Some(neighbor_id) = self.get_node_id(neighbor_index) {
                    result.push(neighbor_id.clone());
                }
            }
            Ok(result)
        } else {
            Ok(Vec::new())
        }
    }
    
    /// Gets the weight of an edge.
    pub fn get_edge_weight(&self, from: &str, to: &str) -> Option<f32> {
        let from_index = self.get_node_index(from)?;
        let to_index = self.get_node_index(to)?;
        
        // Search for the edge in the adjacency list
        if from_index.0 < self.adjacency_list.len() {
            for &neighbor_index in &self.adjacency_list[from_index.0] {
                if neighbor_index == to_index {
                    // Find the edge data
                    for edge_data in &self.edges {
                        if let Some(edge) = edge_data {
                            if edge.from == from_index && edge.to == to_index {
                                return Some(edge.weight);
                            }
                        }
                    }
                }
            }
        }
        
        None
    }
    
    /// Gets the number of nodes in the graph.
    pub fn node_count(&self) -> usize {
        self.stats.node_count
    }
    
    /// Gets the number of edges in the graph.
    pub fn edge_count(&self) -> usize {
        self.stats.edge_count
    }
}

// Implementation of graph traits for OptimizedGraphIndex
impl GraphBase for OptimizedGraphIndex {
    type NodeId = NodeIndex;
    type EdgeId = EdgeIndex;
    
    fn node_count(&self) -> usize {
        self.node_count()
    }
    
    fn edge_count(&self) -> usize {
        self.edge_count()
    }
}

impl Data for OptimizedGraphIndex {
    type NodeWeight = NodeData;
    type EdgeWeight = EdgeData;
    
    fn node_weight(&self, node_id: Self::NodeId) -> Option<&Self::NodeWeight> {
        if node_id.0 < self.nodes.len() {
            self.nodes[node_id.0].as_ref()
        } else {
            None
        }
    }
    
    fn edge_weight(&self, edge_id: Self::EdgeId) -> Option<&Self::EdgeWeight> {
        if edge_id.0 < self.edges.len() {
            self.edges[edge_id.0].as_ref()
        } else {
            None
        }
    }
    
    fn edge_endpoints(&self, edge_id: Self::EdgeId) -> Option<(Self::NodeId, Self::NodeId)> {
        // This is a simplified implementation
        // In a real implementation, we would need to store the edge endpoints
        None
    }
}

impl DirectedGraph for OptimizedGraphIndex {
    fn neighbors(&self, node_id: Self::NodeId) -> DbResult<Vec<Self::NodeId>> {
        if node_id.0 < self.adjacency_list.len() {
            Ok(self.adjacency_list[node_id.0].clone())
        } else {
            Ok(Vec::new())
        }
    }
    
    fn neighbors_directed(&self, node_id: Self::NodeId, direction: Direction) -> DbResult<Vec<Self::NodeId>> {
        match direction {
            Direction::Outgoing => self.neighbors(node_id),
            Direction::Incoming => {
                if node_id.0 < self.reverse_adjacency_list.len() {
                    Ok(self.reverse_adjacency_list[node_id.0].clone())
                } else {
                    Ok(Vec::new())
                }
            }
        }
    }
    
    fn edges(&self, node_id: Self::NodeId) -> DbResult<Vec<(Self::EdgeId, Self::NodeId, Self::NodeId)>> {
        // This is a simplified implementation
        // In a real implementation, we would need to search through all edges
        // to find those connected to the given node
        Ok(Vec::new())
    }
}

impl GraphMut for OptimizedGraphIndex {
    fn add_node(&mut self, weight: Self::NodeWeight) -> DbResult<Self::NodeId> {
        // This is a simplified implementation
        // In a real implementation, we would need to handle the weight properly
        let index = if let Some(free_index) = self.free_node_indices.pop() {
            free_index
        } else {
            NodeIndex(self.nodes.len())
        };
        
        // Ensure the adjacency lists are large enough
        while self.adjacency_list.len() <= index.0 {
            self.adjacency_list.push(Vec::new());
            self.reverse_adjacency_list.push(Vec::new());
        }
        
        // Add the node data
        if index.0 >= self.nodes.len() {
            self.nodes.resize(index.0 + 1, None);
        }
        self.nodes[index.0] = Some(weight);
        
        self.stats.node_count += 1;
        Ok(index)
    }
    
    fn remove_node(&mut self, node_id: Self::NodeId) -> DbResult<Option<Self::NodeWeight>> {
        if node_id.0 < self.nodes.len() {
            let node_data = self.nodes[node_id.0].take();
            if node_data.is_some() {
                self.stats.node_count -= 1;
                self.free_node_indices.push(node_id);
            }
            Ok(node_data)
        } else {
            Ok(None)
        }
    }
    
    fn add_edge(&mut self, from: Self::NodeId, to: Self::NodeId, weight: Self::EdgeWeight) -> DbResult<Self::EdgeId> {
        // Add edge to adjacency lists
        if from.0 < self.adjacency_list.len() {
            self.adjacency_list[from.0].push(to);
        }
        if to.0 < self.reverse_adjacency_list.len() {
            self.reverse_adjacency_list[to.0].push(from);
        }
        
        // Create a new edge
        let edge_index = if let Some(free_index) = self.free_edge_indices.pop() {
            free_index
        } else {
            EdgeIndex(self.edges.len())
        };
        
        // Add the edge data
        if edge_index.0 >= self.edges.len() {
            self.edges.resize(edge_index.0 + 1, None);
        }
        self.edges[edge_index.0] = Some(weight);
        
        self.stats.edge_count += 1;
        Ok(edge_index)
    }
    
    fn remove_edge(&mut self, edge_id: Self::EdgeId) -> DbResult<Option<Self::EdgeWeight>> {
        if edge_id.0 < self.edges.len() {
            let edge_data = self.edges[edge_id.0].take();
            if edge_data.is_some() {
                self.stats.edge_count -= 1;
                self.free_edge_indices.push(edge_id);
            }
            Ok(edge_data)
        } else {
            Ok(None)
        }
    }
    
    fn update_node_weight(&mut self, node_id: Self::NodeId, weight: Self::NodeWeight) -> DbResult<()> {
        if node_id.0 < self.nodes.len() {
            self.nodes[node_id.0] = Some(weight);
            Ok(())
        } else {
            Err(DbError::NotFound(format!("Node {:?} not found", node_id)))
        }
    }
    
    fn update_edge_weight(&mut self, edge_id: Self::EdgeId, weight: Self::EdgeWeight) -> DbResult<()> {
        if edge_id.0 < self.edges.len() {
            self.edges[edge_id.0] = Some(weight);
            Ok(())
        } else {
            Err(DbError::NotFound(format!("Edge {:?} not found", edge_id)))
        }
    }
}

impl IntoNodeIdentifiers for OptimizedGraphIndex {
    type NodeIdentifiers = std::vec::IntoIter<Self::NodeId>;
    
    fn node_identifiers(&self) -> Self::NodeIdentifiers {
        (0..self.nodes.len())
            .filter_map(|i| {
                if self.nodes[i].is_some() {
                    Some(NodeIndex(i))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .into_iter()
    }
}

impl IntoEdgeIdentifiers for OptimizedGraphIndex {
    type EdgeIdentifiers = std::vec::IntoIter<Self::EdgeId>;
    
    fn edge_identifiers(&self) -> Self::EdgeIdentifiers {
        (0..self.edges.len())
            .filter_map(|i| {
                if self.edges[i].is_some() {
                    Some(EdgeIndex(i))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .into_iter()
    }
}

impl Visitable for OptimizedGraphIndex {
    type Map = HashSet<NodeIndex>;
    
    fn visit_map(&self) -> Self::Map {
        HashSet::new()
    }
    
    fn reset_map(&self, map: &mut Self::Map) {
        map.clear();
    }
}

impl VisitMap<NodeIndex> for HashSet<NodeIndex> {
    fn visit(&mut self, node_id: &NodeIndex) -> bool {
        self.insert(*node_id)
    }
    
    fn is_visited(&self, node_id: &NodeIndex) -> bool {
        self.contains(node_id)
    }
}
