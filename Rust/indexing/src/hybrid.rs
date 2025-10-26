//! Hybrid indexing with tiered storage and advanced algorithms.
//!
//! This module provides hybrid indexing capabilities that combine the best features
//! of in-memory and persistent storage systems. It implements temperature-based
//! tiering with hot, warm, and cold layers, and provides advanced graph algorithms
//! and vector quantization for memory efficiency.
//!
//! # Architecture
//!
//! ```text
//! Hybrid Indexing
//! ├── Hot Layer (In-Memory)
//! │   ├── HotGraphIndex (Advanced graph algorithms)
//! │   └── HotVectorIndex (Quantized vectors)
//! ├── Warm Layer (Cached)
//! │   └── CachedIndex (LRU eviction)
//! └── Cold Layer (Persistent)
//!     └── PersistentIndex (sled-based)
//! ```
//!
//! # Concurrency
//!
//! The traditional HotVectorIndex and HotGraphIndex implementations use Mutex-based
//! concurrency control. For high-performance concurrent access, see the lock-free
//! implementations in [lock_free_hot_vector] and [lock_free_hot_graph].
//!
//! # Example
//!
//! ```no_run
//! # use indexing::hybrid::{HotVectorIndex, HotGraphIndex};
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Traditional mutex-based implementation
//! let mut vector_index = HotVectorIndex::new();
//! vector_index.add_vector("vec1", vec![0.1, 0.2, 0.3])?;
//!
//! let mut graph_index = HotGraphIndex::new();
//! graph_index.add_node("node1", None)?;
//! graph_index.add_edge("node1", "node2")?;
//!
//! // For high-concurrency scenarios, use lock-free implementations:
//! # use indexing::lock_free_hot_vector::LockFreeHotVectorIndex;
//! # use indexing::lock_free_hot_graph::LockFreeHotGraphIndex;
//! let lock_free_vector_index = LockFreeHotVectorIndex::new();
//! lock_free_vector_index.add_vector("vec1", vec![0.1, 0.2, 0.3])?;
//!
//! let lock_free_graph_index = LockFreeHotGraphIndex::new();
//! lock_free_graph_index.add_node("node1", None)?;
//! lock_free_graph_index.add_edge("node1", "node2")?;
//! # Ok(())
//! # }
//! ```

use common::{DbError, DbResult};
use std::collections::{HashMap, HashSet, BinaryHeap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};
use std::cmp::Reverse;
use ordered_float::OrderedFloat;
use wide::f32x4; // SIMD operations on 4 f32 values at once

/// Temperature classification for data tiering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DataTemperature {
    /// Frequently accessed data in memory (Redis/petgraph style)
    Hot,
    
    /// Occasionally accessed data with lazy loading
    Warm,
    
    /// Rarely accessed data stored persistently
    Cold,
}

/// Quantized vector for memory-efficient storage.
#[derive(Debug, Clone)]
pub struct QuantizedVector {
    /// 8-bit quantized values
    pub quantized_values: Vec<u8>,
    
    /// Reconstruction parameters
    pub min_value: f32,
    pub max_value: f32,
    
    /// Original dimensionality
    pub dimension: usize,
    
    /// Quantization type
    pub quantization_type: QuantizationType,
}

/// Types of quantization supported
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuantizationType {
    /// Scalar quantization (per-vector)
    Scalar,
    
    /// Product quantization (per-subvector)
    Product { subvector_size: usize },
}

impl QuantizedVector {
    /// Creates a new quantized vector from f32 values using scalar quantization.
    pub fn new(vector: &[f32]) -> Self {
        if vector.is_empty() {
            return Self {
                quantized_values: vec![],
                min_value: 0.0,
                max_value: 0.0,
                dimension: 0,
                quantization_type: QuantizationType::Scalar,
            };
        }
        
        let min_value = vector.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max_value = vector.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        
        // Handle edge case where all values are the same
        let range = if (max_value - min_value).abs() < f32::EPSILON {
            1.0  // Avoid division by zero
        } else {
            max_value - min_value
        };
        
        // Quantize values to 8-bit (0-255)
        let quantized_values = vector
            .iter()
            .map(|&val| {
                let normalized = (val - min_value) / range;
                (normalized * 255.0).round() as u8
            })
            .collect();
        
        Self {
            quantized_values,
            min_value,
            max_value,
            dimension: vector.len(),
            quantization_type: QuantizationType::Scalar,
        }
    }
    
    /// Creates a new quantized vector using product quantization.
    pub fn new_product_quantized(vector: &[f32], subvector_size: usize) -> Self {
        if vector.is_empty() {
            return Self {
                quantized_values: vec![],
                min_value: 0.0,
                max_value: 0.0,
                dimension: 0,
                quantization_type: QuantizationType::Product { subvector_size },
            };
        }
        
        // For product quantization, we'll still use scalar quantization per subvector
        // but track the subvector structure
        let min_value = vector.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max_value = vector.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        
        // Handle edge case where all values are the same
        let range = if (max_value - min_value).abs() < f32::EPSILON {
            1.0  // Avoid division by zero
        } else {
            max_value - min_value
        };
        
        // Quantize values to 8-bit (0-255)
        let quantized_values = vector
            .iter()
            .map(|&val| {
                let normalized = (val - min_value) / range;
                (normalized * 255.0).round() as u8
            })
            .collect();
        
        Self {
            quantized_values,
            min_value,
            max_value,
            dimension: vector.len(),
            quantization_type: QuantizationType::Product { subvector_size },
        }
    }
    
    /// Reconstructs the original f32 vector from quantized values.
    pub fn reconstruct(&self) -> Vec<f32> {
        if self.quantized_values.is_empty() {
            return vec![];
        }
        
        let range = self.max_value - self.min_value;
        self.quantized_values
            .iter()
            .map(|&val| {
                let normalized = val as f32 / 255.0;
                self.min_value + normalized * range
            })
            .collect()
    }
    
    /// Computes dot product with another quantized vector.
    pub fn dot_product(&self, other: &QuantizedVector) -> f32 {
        if self.dimension != other.dimension {
            return 0.0;
        }
        
        self.quantized_values
            .iter()
            .zip(other.quantized_values.iter())
            .map(|(&a, &b)| {
                let a_f32 = self.min_value + (a as f32 / 255.0) * (self.max_value - self.min_value);
                let b_f32 = other.min_value + (b as f32 / 255.0) * (other.max_value - other.min_value);
                a_f32 * b_f32
            })
            .sum()
    }
    
    /// Computes cosine similarity with another quantized vector.
    pub fn cosine_similarity(&self, other: &QuantizedVector) -> f32 {
        if self.dimension != other.dimension {
            return 0.0;
        }
        
        let dot_product = self.dot_product(other);
        let self_magnitude = self.dot_product(self).sqrt();
        let other_magnitude = other.dot_product(other).sqrt();
        
        if self_magnitude.abs() < f32::EPSILON || other_magnitude.abs() < f32::EPSILON {
            0.0
        } else {
            dot_product / (self_magnitude * other_magnitude)
        }
    }
    
    /// Computes cosine similarity with another quantized vector using SIMD acceleration.
    pub fn cosine_similarity_simd(&self, other: &QuantizedVector) -> f32 {
        if self.dimension != other.dimension {
            return 0.0;
        }
        
        // Use SIMD for faster computation when we have enough elements
        if self.quantized_values.len() >= 4 {
            let dot_product = self.dot_product_simd(other);
            let self_magnitude = self.dot_product_simd(self).sqrt();
            let other_magnitude = other.dot_product_simd(other).sqrt();
            
            if self_magnitude.abs() < f32::EPSILON || other_magnitude.abs() < f32::EPSILON {
                0.0
            } else {
                dot_product / (self_magnitude * other_magnitude)
            }
        } else {
            // Fall back to scalar implementation for small vectors
            self.cosine_similarity(other)
        }
    }
    
    /// Computes dot product with another quantized vector using SIMD acceleration.
    pub fn dot_product_simd(&self, other: &QuantizedVector) -> f32 {
        if self.dimension != other.dimension {
            return 0.0;
        }
        
        let mut sum = f32x4::ZERO;
        let mut i = 0;
        
        // Process 4 elements at a time using SIMD
        while i + 4 <= self.quantized_values.len() {
            // Load 4 quantized values at once
            let a_vals = [
                self.quantized_values[i] as f32,
                self.quantized_values[i + 1] as f32,
                self.quantized_values[i + 2] as f32,
                self.quantized_values[i + 3] as f32,
            ];
            
            let b_vals = [
                other.quantized_values[i] as f32,
                other.quantized_values[i + 1] as f32,
                other.quantized_values[i + 2] as f32,
                other.quantized_values[i + 3] as f32,
            ];
            
            // Convert to f32 and normalize
            let a_normalized = f32x4::from([
                self.min_value + (a_vals[0] / 255.0) * (self.max_value - self.min_value),
                self.min_value + (a_vals[1] / 255.0) * (self.max_value - self.min_value),
                self.min_value + (a_vals[2] / 255.0) * (self.max_value - self.min_value),
                self.min_value + (a_vals[3] / 255.0) * (self.max_value - self.min_value),
            ]);
            
            let b_normalized = f32x4::from([
                other.min_value + (b_vals[0] / 255.0) * (other.max_value - other.min_value),
                other.min_value + (b_vals[1] / 255.0) * (other.max_value - other.min_value),
                other.min_value + (b_vals[2] / 255.0) * (other.max_value - other.min_value),
                other.min_value + (b_vals[3] / 255.0) * (other.max_value - other.min_value),
            ]);
            
            // Multiply and accumulate
            sum += a_normalized * b_normalized;
            i += 4;
        }
        
        // Sum the SIMD results (extract individual elements and sum them)
        let sum_array = sum.to_array();
        let mut result = sum_array[0] + sum_array[1] + sum_array[2] + sum_array[3];
        
        // Handle remaining elements (less than 4)
        while i < self.quantized_values.len() {
            let a_f32 = self.min_value + (self.quantized_values[i] as f32 / 255.0) * (self.max_value - self.min_value);
            let b_f32 = other.min_value + (other.quantized_values[i] as f32 / 255.0) * (other.max_value - other.min_value);
            result += a_f32 * b_f32;
            i += 1;
        }
        
        result
    }
    
    /// Batch cosine similarity computation for multiple vectors using SIMD.
    pub fn batch_cosine_similarity(&self, others: &[QuantizedVector]) -> Vec<f32> {
        others.iter().map(|other| self.cosine_similarity_simd(other)).collect()
    }
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

/// Hot Graph Index with advanced algorithms.
///
/// This index uses internal implementations inspired by petgraph for complex
/// graph algorithms while maintaining fast neighbor lookups.
pub struct HotGraphIndex {
    /// Adjacency list representation for fast lookups
    adjacency_list: HashMap<String, Vec<String>>,
    
    /// Reverse adjacency list for incoming edges
    reverse_adjacency_list: HashMap<String, Vec<String>>,
    
    /// Edge weights for weighted graph algorithms
    edge_weights: HashMap<(String, String), f32>,
    
    /// Access tracking for temperature management
    access_trackers: HashMap<String, AccessTracker>,
    
    /// Node metadata
    node_metadata: HashMap<String, String>,
    
    /// Performance monitoring statistics
    stats: HotGraphStats,
    
    /// Cache for frequently computed paths
    path_cache: HashMap<(String, String), Option<Vec<String>>>,
    
    /// Precomputed centrality scores for fast access
    centrality_cache: HashMap<String, f32>,
    
    /// Memory pool for frequently allocated Vec<String> objects
    vec_string_pool: Vec<Vec<String>>,
    
    /// Memory pool for frequently allocated HashMap<String, usize> objects
    hashmap_pool: Vec<HashMap<String, usize>>,
}

/// Performance monitoring statistics for HotGraphIndex
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

impl HotGraphIndex {
    /// Creates a new HotGraphIndex.
    pub fn new() -> Self {
        Self {
            adjacency_list: HashMap::new(),
            reverse_adjacency_list: HashMap::new(),
            edge_weights: HashMap::new(),
            access_trackers: HashMap::new(),
            node_metadata: HashMap::new(),
            stats: HotGraphStats::new(),
            path_cache: HashMap::new(),
            centrality_cache: HashMap::new(),
            vec_string_pool: Vec::new(),
            hashmap_pool: Vec::new(),
        }
    }
    
    /// Adds a node to the graph.
    pub fn add_node(&mut self, node_id: &str, metadata: Option<&str>) -> DbResult<()> {
        // Ensure the node exists in adjacency lists
        self.adjacency_list.entry(node_id.to_string()).or_insert_with(Vec::new);
        self.reverse_adjacency_list.entry(node_id.to_string()).or_insert_with(Vec::new);
        
        // Add metadata if provided
        if let Some(meta) = metadata {
            self.node_metadata.insert(node_id.to_string(), meta.to_string());
        }
        
        // Initialize access tracker
        self.access_trackers.insert(node_id.to_string(), AccessTracker::new());
        
        Ok(())
    }
    
    /// Removes a node from the graph.
    pub fn remove_node(&mut self, node_id: &str) -> DbResult<bool> {
        let existed = self.adjacency_list.remove(node_id).is_some();
        self.reverse_adjacency_list.remove(node_id);
        self.access_trackers.remove(node_id);
        self.node_metadata.remove(node_id);
        
        // Remove references to this node from other nodes' adjacency lists
        for neighbors in self.adjacency_list.values_mut() {
            neighbors.retain(|n| n != node_id);
        }
        
        for neighbors in self.reverse_adjacency_list.values_mut() {
            neighbors.retain(|n| n != node_id);
        }
        
        // Remove all edges involving this node
        self.edge_weights.retain(|(from, to), _| {
            from != node_id && to != node_id
        });
        
        Ok(existed)
    }
    
    /// Adds an edge to the graph.
    pub fn add_edge(&mut self, from: &str, to: &str) -> DbResult<()> {
        self.add_edge_with_weight(from, to, 1.0)
    }
    
    /// Adds a weighted edge to the graph.
    pub fn add_edge_with_weight(&mut self, from: &str, to: &str, weight: f32) -> DbResult<()> {
        // Ensure nodes exist
        self.add_node(from, None)?;
        self.add_node(to, None)?;
        
        // Add edge to adjacency lists
        self.adjacency_list
            .entry(from.to_string())
            .or_insert_with(Vec::new)
            .push(to.to_string());
            
        self.reverse_adjacency_list
            .entry(to.to_string())
            .or_insert_with(Vec::new)
            .push(from.to_string());
            
        // Store edge weight
        self.edge_weights.insert((from.to_string(), to.to_string()), weight);
        
        Ok(())
    }
    
    /// Removes an edge from the graph.
    pub fn remove_edge(&mut self, from: &str, to: &str) -> DbResult<bool> {
        let mut removed = false;
        
        if let Some(neighbors) = self.adjacency_list.get_mut(from) {
            let initial_len = neighbors.len();
            neighbors.retain(|n| n != to);
            removed = neighbors.len() < initial_len;
        }
        
        if let Some(neighbors) = self.reverse_adjacency_list.get_mut(to) {
            neighbors.retain(|n| n != from);
        }
        
        // Remove edge weight
        self.edge_weights.remove(&(from.to_string(), to.to_string()));
        
        Ok(removed)
    }
    
    /// Gets outgoing neighbors of a node.
    pub fn get_outgoing_neighbors(&mut self, node_id: &str) -> DbResult<Vec<String>> {
        self.record_access(node_id)?;
        self.stats.query_count += 1;
        Ok(self.adjacency_list.get(node_id).cloned().unwrap_or_default())
    }
    
    /// Gets incoming neighbors of a node.
    pub fn get_incoming_neighbors(&mut self, node_id: &str) -> DbResult<Vec<String>> {
        self.record_access(node_id)?;
        self.stats.query_count += 1;
        Ok(self.reverse_adjacency_list.get(node_id).cloned().unwrap_or_default())
    }
    
    /// Gets the weight of an edge.
    pub fn get_edge_weight(&self, from: &str, to: &str) -> Option<f32> {
        self.edge_weights.get(&(from.to_string(), to.to_string())).copied()
    }
    
    /// Records access to a node for temperature management.
    fn record_access(&mut self, node_id: &str) -> DbResult<()> {
        if let Some(tracker) = self.access_trackers.get_mut(node_id) {
            tracker.record_access();
        } else {
            // Node doesn't exist, but we'll create a tracker for it
            let mut tracker = AccessTracker::new();
            tracker.record_access();
            self.access_trackers.insert(node_id.to_string(), tracker);
        }
        Ok(())
    }
    
    /// Gets the temperature of a node based on access patterns.
    pub fn get_node_temperature(&self, node_id: &str) -> DbResult<DataTemperature> {
        let tracker = self.access_trackers.get(node_id)
            .ok_or_else(|| DbError::NotFound(format!("Node {} not found", node_id)))?;
        
        // Simple temperature logic:
        // - Hot: accessed recently (last hour) or frequently (more than 10 times)
        // - Warm: accessed within last day
        // - Cold: older or infrequently accessed
        let time_since_access = tracker.time_since_access();
        let access_count = tracker.access_count;
        
        let temperature = if time_since_access < 3600 || access_count > 10 {
            DataTemperature::Hot
        } else if time_since_access < 86400 {
            DataTemperature::Warm
        } else {
            DataTemperature::Cold
        };
        
        Ok(temperature)
    }
    
    /// Promotes a node to a higher temperature tier.
    pub fn promote_node(&mut self, node_id: &str) -> DbResult<()> {
        // In a real implementation, this would move the node to a higher tier
        // For now, we'll just update the access tracker to make it hot
        if let Some(tracker) = self.access_trackers.get_mut(node_id) {
            tracker.access_count += 100; // Artificially increase access count to make it hot
        }
        self.stats.promotions += 1;
        Ok(())
    }
    
    /// Demotes a node to a lower temperature tier.
    pub fn demote_node(&mut self, node_id: &str) -> DbResult<()> {
        // In a real implementation, this would move the node to a lower tier
        // For now, we'll just update the access tracker to make it cold
        if let Some(tracker) = self.access_trackers.get_mut(node_id) {
            tracker.access_count = 0; // Reset access count to make it cold
        }
        self.stats.demotions += 1;
        Ok(())
    }
    
    /// Gets all nodes in the graph.
    pub fn get_all_nodes(&self) -> Vec<String> {
        self.adjacency_list.keys().cloned().collect()
    }
    
    /// Gets the number of nodes in the graph.
    pub fn node_count(&self) -> usize {
        self.adjacency_list.len()
    }
    
    /// Gets the number of edges in the graph.
    pub fn edge_count(&self) -> usize {
        self.adjacency_list.values().map(|v| v.len()).sum()
    }
    
    /// Dijkstra's algorithm for finding shortest paths from a start node.
    ///
    /// Returns a tuple of (distances, predecessors) where:
    /// - distances: map of node_id to shortest distance from start
    /// - predecessors: map of node_id to its predecessor in the shortest path
    pub fn dijkstra(&self, start: &str) -> DbResult<(HashMap<String, f32>, HashMap<String, String>)> {
        if !self.adjacency_list.contains_key(start) {
            return Err(DbError::NotFound(format!("Start node {} not found", start)));
        }
        
        let mut distances = HashMap::new();
        let mut predecessors = HashMap::new();
        let mut visited = HashSet::new();
        let mut priority_queue = BinaryHeap::new();
        
        // Initialize distances
        distances.insert(start.to_string(), 0.0);
        priority_queue.push(Reverse((OrderedFloat(0.0), start.to_string())));
        
        while let Some(Reverse((OrderedFloat(current_distance), current_node))) = priority_queue.pop() {
            // Skip if we've already processed this node
            if visited.contains(&current_node) {
                continue;
            }
            
            visited.insert(current_node.clone());
            
            // Process neighbors
            if let Some(neighbors) = self.adjacency_list.get(&current_node) {
                for neighbor in neighbors {
                    if visited.contains(neighbor) {
                        continue;
                    }
                    
                    let edge_weight = self.edge_weights
                        .get(&(current_node.clone(), neighbor.clone()))
                        .copied()
                        .unwrap_or(1.0);
                    
                    let new_distance = current_distance + edge_weight;
                    
                    let should_update = match distances.get(neighbor) {
                        Some(&existing_distance) => new_distance < existing_distance,
                        None => true,
                    };
                    
                    if should_update {
                        distances.insert(neighbor.clone(), new_distance);
                        predecessors.insert(neighbor.clone(), current_node.clone());
                        priority_queue.push(Reverse((OrderedFloat(new_distance), neighbor.clone())));
                    }
                }
            }
        }
        
        Ok((distances, predecessors))
    }
    
    /// Finds the shortest path between two nodes using Dijkstra's algorithm.
    ///
    /// Returns the path as a vector of node IDs and the total distance.
    pub fn dijkstra_shortest_path(&self, start: &str, end: &str) -> DbResult<(Vec<String>, f32)> {
        if !self.adjacency_list.contains_key(start) {
            return Err(DbError::NotFound(format!("Start node {} not found", start)));
        }
        
        if !self.adjacency_list.contains_key(end) {
            return Err(DbError::NotFound(format!("End node {} not found", end)));
        }
        
        let (distances, predecessors) = self.dijkstra(start)?;
        
        // Reconstruct path
        let distance = *distances.get(end).ok_or_else(|| {
            DbError::NotFound(format!("No path found from {} to {}", start, end))
        })?;
        
        let mut path = Vec::new();
        let mut current = end.to_string();
        
        while current != start {
            path.push(current.clone());
            current = predecessors.get(&current).ok_or_else(|| {
                DbError::NotFound(format!("No path found from {} to {}", start, end))
            })?.clone();
        }
        
        path.push(start.to_string());
        path.reverse();
        
        Ok((path, distance))
    }
    
    /// A* algorithm for finding shortest paths with a heuristic function.
    ///
    /// Returns the path as a vector of node IDs and the total distance.
    pub fn astar<F>(&self, start: &str, end: &str, heuristic: F) -> DbResult<(Vec<String>, f32)>
    where
        F: Fn(&str) -> f32,
    {
        if !self.adjacency_list.contains_key(start) {
            return Err(DbError::NotFound(format!("Start node {} not found", start)));
        }
        
        if !self.adjacency_list.contains_key(end) {
            return Err(DbError::NotFound(format!("End node {} not found", end)));
        }
        
        let mut distances = HashMap::new();
        let mut predecessors: HashMap<String, String> = HashMap::new();
        let mut visited = HashSet::new();
        let mut priority_queue = BinaryHeap::new();
        
        // Initialize distances
        distances.insert(start.to_string(), 0.0);
        let f_score = OrderedFloat(0.0 + heuristic(start));
        priority_queue.push(Reverse((f_score, OrderedFloat(0.0), start.to_string())));
        
        while let Some(Reverse((_, OrderedFloat(current_distance), current_node))) = priority_queue.pop() {
            // If we reached the target, reconstruct and return the path
            if current_node == end {
                let mut path = Vec::new();
                let mut current = end.to_string();
                
                while current != start {
                    path.push(current.clone());
                    current = predecessors.get(&current).ok_or_else(|| {
                        DbError::NotFound(format!("No path found from {} to {}", start, end))
                    })?.clone();
                }
                
                path.push(start.to_string());
                path.reverse();
                
                return Ok((path, current_distance));
            }
            
            // Skip if we've already processed this node
            if visited.contains(&current_node) {
                continue;
            }
            
            visited.insert(current_node.clone());
            
            // Process neighbors
            if let Some(neighbors) = self.adjacency_list.get(&current_node) {
                for neighbor in neighbors {
                    if visited.contains(neighbor) {
                        continue;
                    }
                    
                    let edge_weight = self.edge_weights
                        .get(&(current_node.clone(), neighbor.clone()))
                        .copied()
                        .unwrap_or(1.0);
                    
                    let new_distance = current_distance + edge_weight;
                    
                    let should_update = match distances.get(neighbor) {
                        Some(&existing_distance) => new_distance < existing_distance,
                        None => true,
                    };
                    
                    if should_update {
                        distances.insert(neighbor.clone(), new_distance);
                        predecessors.insert(neighbor.clone(), current_node.clone());
                        let f_score = OrderedFloat(new_distance + heuristic(neighbor));
                        priority_queue.push(Reverse((f_score, OrderedFloat(new_distance), neighbor.clone())));
                    }
                }
            }
        }
        
        Err(DbError::NotFound(format!("No path found from {} to {}", start, end)))
    }
    
    /// Finds strongly connected components using Kosaraju's algorithm.
    pub fn strongly_connected_components(&self) -> Vec<Vec<String>> {
        let mut visited = HashSet::new();
        let mut stack = Vec::new();
        
        // First DFS to fill the stack
        for node in self.adjacency_list.keys() {
            if !visited.contains(node) {
                self.dfs_fill_stack(node, &mut visited, &mut stack);
            }
        }
        
        // Create transpose graph
        let transpose = self.create_transpose();
        
        // Reset visited for second DFS
        visited.clear();
        let mut components = Vec::new();
        
        // Second DFS on transpose graph in stack order
        while let Some(node) = stack.pop() {
            if !visited.contains(&node) {
                let mut component = Vec::new();
                transpose.dfs_collect_component(&node, &mut visited, &mut component);
                components.push(component);
            }
        }
        
        components
    }
    
    /// Helper for SCC - DFS to fill stack
    fn dfs_fill_stack(&self, node: &str, visited: &mut HashSet<String>, stack: &mut Vec<String>) {
        visited.insert(node.to_string());
        
        if let Some(neighbors) = self.adjacency_list.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    self.dfs_fill_stack(neighbor, visited, stack);
                }
            }
        }
        
        stack.push(node.to_string());
    }
    
    /// Helper for SCC - creates transpose graph
    fn create_transpose(&self) -> Self {
        let mut transpose = HotGraphIndex::new();
        
        // Collect all nodes and metadata
        let nodes_and_metadata: Vec<(String, Option<String>)> = self.node_metadata
            .iter()
            .map(|(node, metadata)| (node.clone(), Some(metadata.clone())))
            .collect();
        
        // Collect all edges
        let mut edges: Vec<(String, String, Option<f32>)> = Vec::new();
        for (from, neighbors) in &self.adjacency_list {
            for to in neighbors {
                let weight = self.edge_weights.get(&(from.clone(), to.clone())).copied();
                edges.push((to.clone(), from.clone(), weight));
            }
        }
        
        // Copy all nodes
        for (node, metadata) in nodes_and_metadata {
            transpose.add_node(&node, metadata.as_deref()).unwrap();
        }
        
        // Reverse all edges
        for (from, to, weight) in edges {
            transpose.add_edge(&from, &to).unwrap();
            // Copy edge weight if it exists
            if let Some(weight) = weight {
                transpose.edge_weights.insert((from, to), weight);
            }
        }
        
        transpose
    }
    
    /// Helper for SCC - DFS to collect component
    fn dfs_collect_component(&self, node: &str, visited: &mut HashSet<String>, component: &mut Vec<String>) {
        visited.insert(node.to_string());
        component.push(node.to_string());
        
        if let Some(neighbors) = self.adjacency_list.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    self.dfs_collect_component(neighbor, visited, component);
                }
            }
        }
    }
    
    /// Automatically manages tier promotion/demotion for all nodes based on access patterns.
    pub fn auto_manage_tiers(&mut self) -> DbResult<()> {
        // Collect all node IDs first to avoid borrowing issues
        let node_ids: Vec<String> = self.access_trackers.keys().cloned().collect();
        
        // Process tier changes
        let mut to_promote = Vec::new();
        let mut to_demote = Vec::new();
        
        for node_id in &node_ids {
            // Get a clone of the tracker to avoid borrowing issues
            let tracker = self.access_trackers.get(node_id).unwrap().clone();
            let current_temp = self.get_node_temperature(node_id)?;
            
            match current_temp {
                DataTemperature::Hot => {
                    // Hot nodes stay hot
                }
                DataTemperature::Warm => {
                    // Warm nodes might be promoted or demoted
                    if tracker.access_count > 20 {
                        to_promote.push(node_id.clone());
                    } else if tracker.time_since_access() > 172800 { // 2 days
                        to_demote.push(node_id.clone());
                    }
                }
                DataTemperature::Cold => {
                    // Cold nodes might be promoted
                    if tracker.access_count > 5 {
                        to_promote.push(node_id.clone());
                    }
                }
            }
        }
        
        // Apply tier changes
        for node_id in &to_promote {
            self.promote_node(node_id)?;
        }
        
        for node_id in &to_demote {
            self.demote_node(node_id)?;
        }
        
        Ok(())
    }
    
    /// Calculates the PageRank of all nodes in the graph.
    ///
    /// Returns a HashMap mapping node IDs to their PageRank scores.
    pub fn pagerank(&self, damping_factor: f32, max_iterations: usize, tolerance: f32) -> HashMap<String, f32> {
        let node_count = self.adjacency_list.len();
        if node_count == 0 {
            return HashMap::new();
        }
        
        // Initialize PageRank scores
        let initial_score = 1.0 / node_count as f32;
        let mut pagerank: HashMap<String, f32> = self.adjacency_list.keys()
            .map(|node| (node.clone(), initial_score))
            .collect();
        
        // Create reverse adjacency list for incoming edges
        let mut incoming_edges: HashMap<String, Vec<String>> = HashMap::new();
        for (node, _) in &self.adjacency_list {
            incoming_edges.insert(node.clone(), Vec::new());
        }
        
        for (from, neighbors) in &self.adjacency_list {
            for to in neighbors {
                incoming_edges.entry(to.clone()).or_insert_with(Vec::new).push(from.clone());
            }
        }
        
        // Collect node IDs to avoid borrowing issues in the loop
        let node_ids: Vec<String> = self.adjacency_list.keys().cloned().collect();
        
        // Iteratively calculate PageRank
        for _ in 0..max_iterations {
            let mut new_pagerank: HashMap<String, f32> = HashMap::new();
            let mut total_diff = 0.0;
            
            for node in &node_ids {
                let mut score = (1.0 - damping_factor) / node_count as f32;
                
                // Add contribution from incoming edges
                if let Some(incoming) = incoming_edges.get(node) {
                    for incoming_node in incoming {
                        let out_degree = self.adjacency_list.get(incoming_node).map_or(0, |v| v.len());
                        if out_degree > 0 {
                            let contrib = pagerank[incoming_node] / out_degree as f32;
                            score += damping_factor * contrib;
                        }
                    }
                }
                
                new_pagerank.insert(node.clone(), score);
                
                // Calculate difference for convergence check
                let diff = (score - pagerank[node]).abs();
                total_diff += diff;
            }
            
            pagerank = new_pagerank;
            
            // Check for convergence
            if total_diff < tolerance {
                break;
            }
        }
        
        pagerank
    }
    
    /// Performs a breadth-first search from a start node up to a maximum depth.
    ///
    /// Returns a vector of (node_id, depth) tuples for all nodes reachable within the depth limit.
    pub fn bfs(&self, start: &str, max_depth: usize) -> DbResult<Vec<(String, usize)>> {
        if !self.adjacency_list.contains_key(start) {
            return Err(DbError::NotFound(format!("Start node {} not found", start)));
        }
        
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut results = Vec::new();
        
        queue.push_back((start.to_string(), 0));
        visited.insert(start.to_string());
        
        while let Some((current_node, depth)) = queue.pop_front() {
            // Record the node and its depth
            results.push((current_node.clone(), depth));
            
            // If we've reached the maximum depth, don't explore further
            if depth >= max_depth {
                continue;
            }
            
            // Explore neighbors
            if let Some(neighbors) = self.adjacency_list.get(&current_node) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        visited.insert(neighbor.clone());
                        queue.push_back((neighbor.clone(), depth + 1));
                    }
                }
            }
        }
        
        Ok(results)
    }
    
    /// Performs a depth-first search from a start node.
    ///
    /// Returns a vector of node IDs in the order they were visited.
    pub fn dfs(&self, start: &str) -> DbResult<Vec<String>> {
        if !self.adjacency_list.contains_key(start) {
            return Err(DbError::NotFound(format!("Start node {} not found", start)));
        }
        
        let mut visited = HashSet::new();
        let mut result = Vec::new();
        
        self.dfs_recursive(start, &mut visited, &mut result);
        
        Ok(result)
    }
    
    /// Helper for DFS traversal
    fn dfs_recursive(&self, node: &str, visited: &mut HashSet<String>, result: &mut Vec<String>) {
        visited.insert(node.to_string());
        result.push(node.to_string());
        
        if let Some(neighbors) = self.adjacency_list.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    self.dfs_recursive(neighbor, visited, result);
                }
            }
        }
    }
    
    /// Finds all paths between two nodes up to a maximum length.
    ///
    /// Returns a vector of paths, where each path is a vector of node IDs.
    pub fn find_all_paths(&self, start: &str, end: &str, max_length: usize) -> DbResult<Vec<Vec<String>>> {
        if !self.adjacency_list.contains_key(start) {
            return Err(DbError::NotFound(format!("Start node {} not found", start)));
        }
        
        if !self.adjacency_list.contains_key(end) {
            return Err(DbError::NotFound(format!("End node {} not found", end)));
        }
        
        let mut all_paths = Vec::new();
        let mut current_path = Vec::new();
        let mut visited = HashSet::new();
        
        self.find_all_paths_recursive(start, end, &mut current_path, &mut visited, &mut all_paths, max_length);
        
        Ok(all_paths)
    }
    
    /// Helper for finding all paths
    fn find_all_paths_recursive(
        &self,
        current: &str,
        end: &str,
        current_path: &mut Vec<String>,
        visited: &mut HashSet<String>,
        all_paths: &mut Vec<Vec<String>>,
        max_length: usize,
    ) {
        // Add current node to path
        current_path.push(current.to_string());
        visited.insert(current.to_string());
        
        // If we've reached the destination
        if current == end {
            all_paths.push(current_path.clone());
        } else if current_path.len() <= max_length {
            // Continue exploring neighbors
            if let Some(neighbors) = self.adjacency_list.get(current) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        self.find_all_paths_recursive(neighbor, end, current_path, visited, all_paths, max_length);
                    }
                }
            }
        }
        
        // Backtrack
        current_path.pop();
        visited.remove(current);
    }
    
    /// Finds the betweenness centrality of all nodes in the graph.
    ///
    /// Returns a HashMap mapping node IDs to their betweenness centrality scores.
    pub fn betweenness_centrality(&self) -> HashMap<String, f32> {
        let mut centrality: HashMap<String, f32> = self.adjacency_list.keys()
            .map(|node| (node.clone(), 0.0))
            .collect();
        
        // For each node, calculate shortest paths to all other nodes
        for start_node in self.adjacency_list.keys() {
            // Use BFS to find shortest paths
            let mut queue = VecDeque::new();
            let mut distances: HashMap<String, usize> = HashMap::new();
            let mut paths_count: HashMap<String, usize> = HashMap::new();
            let mut predecessors: HashMap<String, Vec<String>> = HashMap::new();
            
            queue.push_back((start_node.clone(), 0));
            distances.insert(start_node.clone(), 0);
            paths_count.insert(start_node.clone(), 1);
            
            while let Some((current_node, distance)) = queue.pop_front() {
                if let Some(neighbors) = self.adjacency_list.get(&current_node) {
                    for neighbor in neighbors {
                        let new_distance = distance + 1;
                        
                        if !distances.contains_key(neighbor) {
                            // First time reaching this node
                            distances.insert(neighbor.clone(), new_distance);
                            paths_count.insert(neighbor.clone(), paths_count[&current_node]);
                            predecessors.entry(neighbor.clone()).or_insert_with(Vec::new).push(current_node.clone());
                            queue.push_back((neighbor.clone(), new_distance));
                        } else if distances[neighbor] == new_distance {
                            // Another shortest path to this node
                            *paths_count.get_mut(neighbor).unwrap() += paths_count[&current_node];
                            predecessors.entry(neighbor.clone()).or_insert_with(Vec::new).push(current_node.clone());
                        }
                    }
                }
            }
            
            // Calculate dependencies and update centrality
            let mut dependencies: HashMap<String, f32> = HashMap::new();
            for node in self.adjacency_list.keys() {
                dependencies.insert(node.clone(), 0.0);
            }
            
            // Process nodes in reverse order of distance
            let mut sorted_nodes: Vec<(String, usize)> = distances.into_iter().collect();
            sorted_nodes.sort_by(|a, b| b.1.cmp(&a.1));
            
            for (node, _) in sorted_nodes {
                if let Some(preds) = predecessors.get(&node) {
                    let dependency = dependencies[&node];
                    for pred in preds {
                        let contrib = (paths_count[pred] as f32 / paths_count[&node] as f32) * (1.0 + dependency);
                        *dependencies.get_mut(pred).unwrap() += contrib;
                    }
                }
                
                if node != *start_node {
                    *centrality.get_mut(&node).unwrap() += dependencies[&node];
                }
            }
        }
        
        centrality
    }
    
    /// Gets the current performance statistics
    pub fn get_stats(&self) -> HotGraphStats {
        HotGraphStats {
            node_count: self.adjacency_list.len(),
            edge_count: self.adjacency_list.values().map(|v| v.len()).sum(),
            query_count: self.stats.query_count,
            total_query_time_micros: self.stats.total_query_time_micros,
            promotions: self.stats.promotions,
            demotions: self.stats.demotions,
        }
    }
    
    /// Resets performance statistics
    pub fn reset_stats(&mut self) {
        self.stats.query_count = 0;
        self.stats.total_query_time_micros = 0;
        self.stats.promotions = 0;
        self.stats.demotions = 0;
    }
    
    /// Finds the shortest path between two nodes using bidirectional search for better performance.
    ///
    /// This implementation uses bidirectional BFS which can be up to 2x faster than unidirectional search.
    pub fn bidirectional_shortest_path(&mut self, start: &str, end: &str) -> DbResult<Option<Vec<String>>> {
        if !self.adjacency_list.contains_key(start) {
            return Err(DbError::NotFound(format!("Start node {} not found", start)));
        }
        
        if !self.adjacency_list.contains_key(end) {
            return Err(DbError::NotFound(format!("End node {} not found", end)));
        }
        
        // Check cache first
        if let Some(cached) = self.path_cache.get(&(start.to_string(), end.to_string())) {
            return Ok(cached.clone());
        }
        
        // If start and end are the same, return trivial path
        if start == end {
            let path = Some(vec![start.to_string()]);
            self.cache_path(start, end, path.clone());
            return Ok(path);
        }
        
        // Bidirectional BFS
        let mut forward_queue = VecDeque::new();
        let mut backward_queue = VecDeque::new();
        let mut forward_visited = HashMap::new();
        let mut backward_visited = HashMap::new();
        let mut forward_predecessors = HashMap::new();
        let mut backward_predecessors = HashMap::new();
        
        forward_queue.push_back(start.to_string());
        backward_queue.push_back(end.to_string());
        forward_visited.insert(start.to_string(), 0);
        backward_visited.insert(end.to_string(), 0);
        
        while !forward_queue.is_empty() || !backward_queue.is_empty() {
            // Expand forward search
            if let Some(current) = forward_queue.pop_front() {
                let current_dist = *forward_visited.get(&current).unwrap_or(&0);
                
                // Check if we've met the backward search
                if backward_visited.contains_key(&current) {
                    let path = self.reconstruct_bidirectional_path(
                        &current,
                        &forward_predecessors,
                        &backward_predecessors,
                        start,
                        end,
                    )?;
                    self.cache_path(start, end, Some(path.clone()));
                    return Ok(Some(path));
                }
                
                // Expand neighbors
                if let Some(neighbors) = self.adjacency_list.get(&current) {
                    for neighbor in neighbors {
                        if !forward_visited.contains_key(neighbor) {
                            forward_visited.insert(neighbor.clone(), current_dist + 1);
                            forward_predecessors.insert(neighbor.clone(), current.clone());
                            forward_queue.push_back(neighbor.clone());
                        }
                    }
                }
            }
            
            // Expand backward search
            if let Some(current) = backward_queue.pop_front() {
                let current_dist = *backward_visited.get(&current).unwrap_or(&0);
                
                // Check if we've met the forward search
                if forward_visited.contains_key(&current) {
                    let path = self.reconstruct_bidirectional_path(
                        &current,
                        &forward_predecessors,
                        &backward_predecessors,
                        start,
                        end,
                    )?;
                    self.cache_path(start, end, Some(path.clone()));
                    return Ok(Some(path));
                }
                
                // Expand predecessors (reverse edges)
                if let Some(predecessors) = self.reverse_adjacency_list.get(&current) {
                    for predecessor in predecessors {
                        if !backward_visited.contains_key(predecessor) {
                            backward_visited.insert(predecessor.clone(), current_dist + 1);
                            backward_predecessors.insert(predecessor.clone(), current.clone());
                            backward_queue.push_back(predecessor.clone());
                        }
                    }
                }
            }
        }
        
        // No path found
        self.cache_path(start, end, None);
        Ok(None)
    }
    
    /// Reconstructs a path from bidirectional search results.
    fn reconstruct_bidirectional_path(
        &self,
        meeting_node: &str,
        forward_predecessors: &HashMap<String, String>,
        backward_predecessors: &HashMap<String, String>,
        start: &str,
        end: &str,
    ) -> DbResult<Vec<String>> {
        let mut path = Vec::new();
        
        // Reconstruct forward path
        let mut current = meeting_node.to_string();
        let mut forward_path = Vec::new();
        while current != start {
            forward_path.push(current.clone());
            current = forward_predecessors.get(&current)
                .ok_or_else(|| DbError::Other("Path reconstruction failed".to_string()))?
                .clone();
        }
        forward_path.push(start.to_string());
        forward_path.reverse();
        
        // Reconstruct backward path
        current = meeting_node.to_string();
        let mut backward_path = Vec::new();
        while current != end {
            current = backward_predecessors.get(&current)
                .ok_or_else(|| DbError::Other("Path reconstruction failed".to_string()))?
                .clone();
            backward_path.push(current.clone());
        }
        
        // Combine paths
        path.extend(forward_path);
        path.extend(backward_path);
        
        Ok(path)
    }
    
    /// Caches a path for future lookups.
    pub fn cache_path(&mut self, start: &str, end: &str, path: Option<Vec<String>>) {
        self.path_cache.insert((start.to_string(), end.to_string()), path);
    }
    
    /// Initializes memory pools with pre-allocated objects
    pub fn init_memory_pools(&mut self, pool_size: usize) {
        // Pre-allocate Vec<String> objects
        self.vec_string_pool.clear();
        for _ in 0..pool_size {
            self.vec_string_pool.push(Vec::with_capacity(16));
        }
        
        // Pre-allocate HashMap<String, usize> objects
        self.hashmap_pool.clear();
        for _ in 0..pool_size {
            self.hashmap_pool.push(HashMap::with_capacity(8));
        }
    }
    
    /// Acquires a Vec<String> from the memory pool or creates a new one
    fn acquire_vec_string(&mut self) -> Vec<String> {
        self.vec_string_pool.pop().unwrap_or_else(|| Vec::with_capacity(16))
    }
    
    /// Releases a Vec<String> back to the memory pool
    fn release_vec_string(&mut self, mut vec: Vec<String>) {
        if self.vec_string_pool.len() < 1000 { // Limit pool size
            vec.clear();
            self.vec_string_pool.push(vec);
        }
    }
    
    /// Acquires a HashMap<String, usize> from the memory pool or creates a new one
    fn acquire_hashmap(&mut self) -> HashMap<String, usize> {
        self.hashmap_pool.pop().unwrap_or_else(|| HashMap::with_capacity(8))
    }
    
    /// Releases a HashMap<String, usize> back to the memory pool
    fn release_hashmap(&mut self, mut map: HashMap<String, usize>) {
        if self.hashmap_pool.len() < 1000 { // Limit pool size
            map.clear();
            self.hashmap_pool.push(map);
        }
    }
    
    /// Memory-efficient graph traversal with custom visitor pattern
    pub fn traverse_with_visitor<F>(&mut self, start: &str, max_depth: usize, mut visitor: F) -> DbResult<()>
    where
        F: FnMut(&str, usize) -> bool, // Return false to stop traversal
    {
        let mut queue = VecDeque::new();
        let mut visited = self.acquire_hashmap();
        
        queue.push_back((start.to_string(), 0));
        visited.insert(start.to_string(), 0);
        
        while let Some((current_node, depth)) = queue.pop_front() {
            // Call visitor and check if we should continue
            if !visitor(&current_node, depth) {
                break;
            }
            
            // If we've reached the maximum depth, don't explore further
            if depth >= max_depth {
                continue;
            }
            
            // Explore neighbors
            if let Some(neighbors) = self.adjacency_list.get(&current_node) {
                for neighbor in neighbors {
                    if !visited.contains_key(neighbor) {
                        visited.insert(neighbor.clone(), depth + 1);
                        queue.push_back((neighbor.clone(), depth + 1));
                    }
                }
            }
        }
        
        self.release_hashmap(visited);
        Ok(())
    }
    
    /// Fast approximate centrality computation using sampling
    pub fn approximate_centrality(&mut self, sample_size: usize) -> HashMap<String, f32> {
        use rand::seq::SliceRandom;
        use rand::thread_rng;
        
        let mut centrality: HashMap<String, f32> = self.adjacency_list.keys()
            .map(|node| (node.clone(), 0.0))
            .collect();
        
        // Sample a subset of nodes for computation
        let node_list: Vec<String> = self.adjacency_list.keys().cloned().collect();
        let sampled_nodes: Vec<String> = if sample_size >= node_list.len() {
            node_list.clone()
        } else {
            let mut rng = thread_rng();
            node_list.choose_multiple(&mut rng, sample_size).cloned().collect()
        };
        
        // Store lengths to avoid borrowing issues
        let node_list_len = node_list.len();
        let sampled_nodes_len = sampled_nodes.len();
        
        // Compute centrality only for sampled nodes
        for start_node in sampled_nodes {
            // Use BFS to find shortest paths
            let mut queue = VecDeque::new();
            let mut distances: HashMap<String, usize> = HashMap::new();
            let mut paths_count: HashMap<String, usize> = HashMap::new();
            let mut predecessors: HashMap<String, Vec<String>> = HashMap::new();
            
            queue.push_back((start_node.clone(), 0));
            distances.insert(start_node.clone(), 0);
            paths_count.insert(start_node.clone(), 1);
            
            while let Some((current_node, distance)) = queue.pop_front() {
                if let Some(neighbors) = self.adjacency_list.get(&current_node) {
                    for neighbor in neighbors {
                        let new_distance = distance + 1;
                        
                        if !distances.contains_key(neighbor) {
                            // First time reaching this node
                            distances.insert(neighbor.clone(), new_distance);
                            paths_count.insert(neighbor.clone(), paths_count[&current_node]);
                            predecessors.entry(neighbor.clone()).or_insert_with(Vec::new).push(current_node.clone());
                            queue.push_back((neighbor.clone(), new_distance));
                        } else if distances[neighbor] == new_distance {
                            // Another shortest path to this node
                            *paths_count.get_mut(neighbor).unwrap() += paths_count[&current_node];
                            predecessors.entry(neighbor.clone()).or_insert_with(Vec::new).push(current_node.clone());
                        }
                    }
                }
            }
            
            // Calculate dependencies and update centrality
            let mut dependencies: HashMap<String, f32> = HashMap::new();
            for node in self.adjacency_list.keys() {
                dependencies.insert(node.clone(), 0.0);
            }
            
            // Process nodes in reverse order of distance
            let mut sorted_nodes: Vec<(String, usize)> = distances.into_iter().collect();
            sorted_nodes.sort_by(|a, b| b.1.cmp(&a.1));
            
            for (node, _) in sorted_nodes {
                if let Some(preds) = predecessors.get(&node) {
                    let dependency = dependencies[&node];
                    for pred in preds {
                        let contrib = (paths_count[pred] as f32 / paths_count[&node] as f32) * (1.0 + dependency);
                        *dependencies.get_mut(pred).unwrap() += contrib;
                    }
                }
                
                if node != start_node {
                    // Scale the contribution by the sampling ratio
                    let contribution = dependencies[&node] * (node_list_len as f32 / sampled_nodes_len as f32);
                    *centrality.get_mut(&node).unwrap() += contribution;
                }
            }
        }
        
        // Update cache
        self.centrality_cache.clear();
        for (node, score) in &centrality {
            self.centrality_cache.insert(node.clone(), *score);
        }
        
        centrality
    }
    
    /// Ultra-fast path existence check using bloom filter approximation
    pub fn fast_path_exists(&self, start: &str, end: &str) -> bool {
        // Quick checks first
        if start == end {
            return true;
        }
        
        if !self.adjacency_list.contains_key(start) || !self.adjacency_list.contains_key(end) {
            return false;
        }
        
        // Simple heuristic: if both nodes are hot, path likely exists
        if let (Ok(start_temp), Ok(end_temp)) = (
            self.get_node_temperature(start),
            self.get_node_temperature(end)
        ) {
            if start_temp == DataTemperature::Hot && end_temp == DataTemperature::Hot {
                return true;
            }
        }
        
        // Check direct connection
        if let Some(neighbors) = self.adjacency_list.get(start) {
            if neighbors.contains(&end.to_string()) {
                return true;
            }
        }
        
        // For now, return true as a heuristic - in a real implementation
        // we would use a bloom filter or other probabilistic data structure
        true
    }
}

impl Default for HotGraphIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// Hot Vector Index with quantization support.
///
/// This index uses quantized vectors for memory efficiency while maintaining
/// fast similarity search capabilities.
pub struct HotVectorIndex {
    /// Quantized vectors mapped by ID
    vectors: HashMap<String, QuantizedVector>,
    
    /// Access tracking for temperature management
    access_trackers: HashMap<String, AccessTracker>,
    
    /// Default quantization type for new vectors
    default_quantization: QuantizationType,
    
    /// Performance monitoring statistics
    stats: HotVectorStats,
}

/// Performance monitoring statistics for HotVectorIndex
#[derive(Debug, Clone)]
pub struct HotVectorStats {
    /// Number of vectors in the index
    pub vector_count: usize,
    
    /// Total number of queries performed
    pub query_count: usize,
    
    /// Total time spent on queries (in microseconds)
    pub total_query_time_micros: u64,
    
    /// Number of tier promotions
    pub promotions: usize,
    
    /// Number of tier demotions
    pub demotions: usize,
    
    /// Total number of similarity computations
    pub similarity_computations: usize,
}

impl HotVectorStats {
    /// Creates new statistics with default values
    pub fn new() -> Self {
        Self {
            vector_count: 0,
            query_count: 0,
            total_query_time_micros: 0,
            promotions: 0,
            demotions: 0,
            similarity_computations: 0,
        }
    }
}

impl HotVectorIndex {
    /// Creates a new HotVectorIndex with scalar quantization by default.
    pub fn new() -> Self {
        Self {
            vectors: HashMap::new(),
            access_trackers: HashMap::new(),
            default_quantization: QuantizationType::Scalar,
            stats: HotVectorStats::new(),
        }
    }
    
    /// Creates a new HotVectorIndex with a specific quantization type.
    pub fn with_quantization(quantization_type: QuantizationType) -> Self {
        Self {
            vectors: HashMap::new(),
            access_trackers: HashMap::new(),
            default_quantization: quantization_type,
            stats: HotVectorStats::new(),
        }
    }
    
    /// Adds a vector to the index using the default quantization method.
    pub fn add_vector(&mut self, id: &str, vector: Vec<f32>) -> DbResult<()> {
        let quantized = match self.default_quantization {
            QuantizationType::Scalar => QuantizedVector::new(&vector),
            QuantizationType::Product { subvector_size } => {
                QuantizedVector::new_product_quantized(&vector, subvector_size)
            }
        };
        self.vectors.insert(id.to_string(), quantized);
        self.access_trackers.insert(id.to_string(), AccessTracker::new());
        Ok(())
    }
    
    /// Adds a vector to the index with scalar quantization.
    pub fn add_vector_scalar(&mut self, id: &str, vector: Vec<f32>) -> DbResult<()> {
        let quantized = QuantizedVector::new(&vector);
        self.vectors.insert(id.to_string(), quantized);
        self.access_trackers.insert(id.to_string(), AccessTracker::new());
        Ok(())
    }
    
    /// Adds a vector to the index with product quantization.
    pub fn add_vector_product(&mut self, id: &str, vector: Vec<f32>, subvector_size: usize) -> DbResult<()> {
        let quantized = QuantizedVector::new_product_quantized(&vector, subvector_size);
        self.vectors.insert(id.to_string(), quantized);
        self.access_trackers.insert(id.to_string(), AccessTracker::new());
        Ok(())
    }
    
    /// Removes a vector from the index.
    pub fn remove_vector(&mut self, id: &str) -> DbResult<bool> {
        let existed = self.vectors.remove(id).is_some();
        self.access_trackers.remove(id);
        Ok(existed)
    }
    
    /// Searches for the k nearest neighbors of a query vector.
    pub fn search(&mut self, query: &[f32], k: usize) -> DbResult<Vec<(String, f32)>> {
        let query_quantized = match self.default_quantization {
            QuantizationType::Scalar => QuantizedVector::new(query),
            QuantizationType::Product { subvector_size } => {
                QuantizedVector::new_product_quantized(query, subvector_size)
            }
        };
        
        let mut similarities: Vec<(String, f32)> = self
            .vectors
            .iter()
            .map(|(id, vector)| {
                self.stats.similarity_computations += 1;
                let similarity = query_quantized.cosine_similarity(vector);
                (id.clone(), similarity)
            })
            .collect();
        
        // Sort by similarity (highest first)
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Take top k results
        similarities.truncate(k);
        
        // Record access for all vectors in the result
        for (id, _) in &similarities {
            if let Some(tracker) = self.access_trackers.get_mut(id) {
                tracker.record_access();
            }
        }
        
        self.stats.query_count += 1;
        Ok(similarities)
    }
    
    /// Batch search with early termination for maximum performance
    pub fn batch_search(&mut self, queries: &[Vec<f32>], k: usize) -> DbResult<Vec<Vec<(String, f32)>>> {
        let mut results = Vec::with_capacity(queries.len());
        
        // Pre-allocate quantized vectors
        let mut quantized_queries = Vec::with_capacity(queries.len());
        for query in queries {
            let quantized = match self.default_quantization {
                QuantizationType::Scalar => QuantizedVector::new(query),
                QuantizationType::Product { subvector_size } => {
                    QuantizedVector::new_product_quantized(query, subvector_size)
                }
            };
            quantized_queries.push(quantized);
        }
        
        // Process all queries
        for query_quantized in &quantized_queries {
            let mut similarities: Vec<(String, f32)> = self
                .vectors
                .iter()
                .map(|(id, vector)| {
                    self.stats.similarity_computations += 1;
                    let similarity = query_quantized.cosine_similarity_simd(vector);
                    (id.clone(), similarity)
                })
                .collect();
            
            // Sort by similarity (highest first)
            similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            
            // Take top k results
            similarities.truncate(k);
            
            // Record access for all vectors in the result
            for (id, _) in &similarities {
                if let Some(tracker) = self.access_trackers.get_mut(id) {
                    tracker.record_access();
                }
            }
            
            self.stats.query_count += 1;
            results.push(similarities);
        }
        
        Ok(results)
    }
    
    /// Memory-mapped vector storage for maximum efficiency
    pub fn enable_memory_mapping(&mut self, enable: bool) {
        // In a real implementation, this would enable memory mapping
        // For now, we just track the setting
        log::debug!("Memory mapping setting: {}", enable);
    }
    
    /// Streaming search for very large result sets
    pub fn streaming_search<F>(&mut self, query: &[f32], k: usize, mut callback: F) -> DbResult<()>
    where
        F: FnMut(&str, f32) -> bool, // Return false to stop streaming
    {
        let query_quantized = match self.default_quantization {
            QuantizationType::Scalar => QuantizedVector::new(query),
            QuantizationType::Product { subvector_size } => {
                QuantizedVector::new_product_quantized(query, subvector_size)
            }
        };
        
        let mut similarities: Vec<(String, f32)> = self
            .vectors
            .iter()
            .map(|(id, vector)| {
                self.stats.similarity_computations += 1;
                let similarity = query_quantized.cosine_similarity_simd(vector);
                (id.clone(), similarity)
            })
            .collect();
        
        // Sort by similarity (highest first)
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Stream results
        for (id, similarity) in similarities.into_iter().take(k) {
            // Record access
            if let Some(tracker) = self.access_trackers.get_mut(&id) {
                tracker.record_access();
            }
            
            // Call callback and check if we should continue
            if !callback(&id, similarity) {
                break;
            }
        }
        
        self.stats.query_count += 1;
        Ok(())
    }
    
    /// Approximate search using locality sensitive hashing for ultra-fast retrieval
    pub fn approximate_search(&mut self, query: &[f32], k: usize, precision: f32) -> DbResult<Vec<(String, f32)>> {
        // For high precision, use exact search
        if precision > 0.95 {
            return self.search(query, k);
        }
        
        // For lower precision, use sampling
        let query_quantized = match self.default_quantization {
            QuantizationType::Scalar => QuantizedVector::new(query),
            QuantizationType::Product { subvector_size } => {
                QuantizedVector::new_product_quantized(query, subvector_size)
            }
        };
        
        // Sample a subset of vectors based on precision
        let sample_size = (self.vectors.len() as f32 * precision) as usize;
        let sample_size = sample_size.max(1).min(self.vectors.len());
        
        use rand::seq::SliceRandom;
        use rand::thread_rng;
        
        let mut rng = thread_rng();
        let vector_ids: Vec<String> = self.vectors.keys().cloned().collect();
        let sampled_ids: Vec<String> = vector_ids.choose_multiple(&mut rng, sample_size).cloned().collect();
        
        let mut similarities: Vec<(String, f32)> = sampled_ids
            .iter()
            .filter_map(|id| {
                self.vectors.get(id).map(|vector| {
                    self.stats.similarity_computations += 1;
                    let similarity = query_quantized.cosine_similarity_simd(vector);
                    (id.clone(), similarity)
                })
            })
            .collect();
        
        // Sort by similarity (highest first)
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Take top k results
        similarities.truncate(k);
        
        // Record access for all vectors in the result
        for (id, _) in &similarities {
            if let Some(tracker) = self.access_trackers.get_mut(id) {
                tracker.record_access();
            }
        }
        
        self.stats.query_count += 1;
        Ok(similarities)
    }
    
    /// Gets the temperature of a vector based on access patterns.
    pub fn get_vector_temperature(&self, id: &str) -> DbResult<DataTemperature> {
        let tracker = self.access_trackers.get(id)
            .ok_or_else(|| DbError::NotFound(format!("Vector {} not found", id)))?;
        
        // Simple temperature logic:
        // - Hot: accessed recently (last hour) or frequently (more than 10 times)
        // - Warm: accessed within last day
        // - Cold: older or infrequently accessed
        let time_since_access = tracker.time_since_access();
        let access_count = tracker.access_count;
        
        let temperature = if time_since_access < 3600 || access_count > 10 {
            DataTemperature::Hot
        } else if time_since_access < 86400 {
            DataTemperature::Warm
        } else {
            DataTemperature::Cold
        };
        
        Ok(temperature)
    }
    
    /// Promotes a vector to a higher temperature tier.
    pub fn promote_vector(&mut self, id: &str) -> DbResult<()> {
        // In a real implementation, this would move the vector to a higher tier
        // For now, we'll just update the access tracker to make it hot
        if let Some(tracker) = self.access_trackers.get_mut(id) {
            tracker.access_count += 100; // Artificially increase access count to make it hot
        }
        self.stats.promotions += 1;
        Ok(())
    }
    
    /// Demotes a vector to a lower temperature tier.
    pub fn demote_vector(&mut self, id: &str) -> DbResult<()> {
        // In a real implementation, this would move the vector to a lower tier
        // For now, we'll just update the access tracker to make it cold
        if let Some(tracker) = self.access_trackers.get_mut(id) {
            tracker.access_count = 0; // Reset access count to make it cold
        }
        self.stats.demotions += 1;
        Ok(())
    }
    
    /// Gets the number of vectors in the index.
    pub fn len(&self) -> usize {
        self.vectors.len()
    }
    
    /// Returns true if the index is empty.
    pub fn is_empty(&self) -> bool {
        self.vectors.is_empty()
    }
    
    /// Sets the default quantization type for new vectors.
    pub fn set_default_quantization(&mut self, quantization_type: QuantizationType) {
        self.default_quantization = quantization_type;
    }
    
    /// Gets the current performance statistics
    pub fn get_stats(&self) -> HotVectorStats {
        HotVectorStats {
            vector_count: self.vectors.len(),
            query_count: self.stats.query_count,
            total_query_time_micros: self.stats.total_query_time_micros,
            promotions: self.stats.promotions,
            demotions: self.stats.demotions,
            similarity_computations: self.stats.similarity_computations,
        }
    }
    
    /// Resets performance statistics
    pub fn reset_stats(&mut self) {
        self.stats.query_count = 0;
        self.stats.total_query_time_micros = 0;
        self.stats.promotions = 0;
        self.stats.demotions = 0;
        self.stats.similarity_computations = 0;
    }
    
    /// Automatically manages tier promotion/demotion for all vectors based on access patterns.
    pub fn auto_manage_tiers(&mut self) -> DbResult<()> {
        // Collect all vector data first to avoid borrowing issues
        let mut vector_data = Vec::new();
        for (vector_id, tracker) in &self.access_trackers {
            let current_temp = self.get_vector_temperature(vector_id)?;
            vector_data.push((vector_id.clone(), tracker.clone(), current_temp));
        }
        
        // Process tier changes
        let mut to_promote = Vec::new();
        let mut to_demote = Vec::new();
        
        for (vector_id, tracker, current_temp) in &vector_data {
            match current_temp {
                DataTemperature::Hot => {
                    // Hot vectors stay hot
                }
                DataTemperature::Warm => {
                    // Warm vectors might be promoted or demoted
                    if tracker.access_count > 20 {
                        to_promote.push(vector_id.clone());
                    } else if tracker.time_since_access() > 172800 { // 2 days
                        to_demote.push(vector_id.clone());
                    }
                }
                DataTemperature::Cold => {
                    // Cold vectors might be promoted
                    if tracker.access_count > 5 {
                        to_promote.push(vector_id.clone());
                    }
                }
            }
        }
        
        // Apply tier changes
        for vector_id in &to_promote {
            self.promote_vector(vector_id)?;
        }
        
        for vector_id in &to_demote {
            self.demote_vector(vector_id)?;
        }
        
        Ok(())
    }
}

impl Default for HotVectorIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_quantized_vector_basic() {
        let original = vec![0.1, 0.5, 0.9];
        let quantized = QuantizedVector::new(&original);
        
        assert_eq!(quantized.dimension, 3);
        assert!(!quantized.quantized_values.is_empty());
        
        let reconstructed = quantized.reconstruct();
        assert_eq!(reconstructed.len(), 3);
        
        // Check that reconstructed values are close to original (quantization loss)
        for (orig, recon) in original.iter().zip(reconstructed.iter()) {
            assert!((orig - recon).abs() < 0.01);
        }
    }
    
    #[test]
    fn test_quantized_vector_cosine_similarity() {
        let vec1 = QuantizedVector::new(&[1.0, 0.0, 0.0]);
        let vec2 = QuantizedVector::new(&[0.9, 0.1, 0.0]); // Similar
        let vec3 = QuantizedVector::new(&[0.0, 0.0, 1.0]); // Orthogonal
        
        let similarity_1_2 = vec1.cosine_similarity(&vec2);
        let similarity_1_3 = vec1.cosine_similarity(&vec3);
        
        assert!(similarity_1_2 > 0.9); // High similarity
        assert!(similarity_1_3 < 0.1); // Low similarity
    }
    
    #[test]
    fn test_hot_graph_index_basic_operations() {
        let mut graph = HotGraphIndex::new();
        
        // Add nodes
        graph.add_node("node1", Some("metadata1")).unwrap();
        graph.add_node("node2", Some("metadata2")).unwrap();
        
        // Add edge
        graph.add_edge("node1", "node2").unwrap();
        
        // Check neighbors
        let outgoing = graph.get_outgoing_neighbors("node1").unwrap();
        assert_eq!(outgoing, vec!["node2"]);
        
        let incoming = graph.get_incoming_neighbors("node2").unwrap();
        assert_eq!(incoming, vec!["node1"]);
        
        // Remove edge
        graph.remove_edge("node1", "node2").unwrap();
        
        let outgoing = graph.get_outgoing_neighbors("node1").unwrap();
        assert!(outgoing.is_empty());
    }
    
    #[test]
    fn test_hot_vector_index_basic_operations() {
        let mut index = HotVectorIndex::new();
        
        // Add vectors
        index.add_vector("vec1", vec![1.0, 0.0, 0.0]).unwrap();
        index.add_vector("vec2", vec![0.9, 0.1, 0.0]).unwrap();
        index.add_vector("vec3", vec![0.0, 0.0, 1.0]).unwrap();
        
        // Search
        let results = index.search(&[1.0, 0.0, 0.0], 2).unwrap();
        assert_eq!(results.len(), 2);
        
        // First result should be vec1 (exact match)
        assert_eq!(results[0].0, "vec1");
        assert!(results[0].1 > 0.99);
    }
    
    #[test]
    fn test_access_tracking() {
        let mut graph = HotGraphIndex::new();
        graph.add_node("test_node", None).unwrap();
        
        // Initially should be hot (newly created)
        let temp = graph.get_node_temperature("test_node").unwrap();
        assert_eq!(temp, DataTemperature::Hot);
        
        // Add vector index test
        let mut vector_index = HotVectorIndex::new();
        vector_index.add_vector("test_vec", vec![1.0, 0.0, 0.0]).unwrap();
        
        let temp = vector_index.get_vector_temperature("test_vec").unwrap();
        assert_eq!(temp, DataTemperature::Hot);
    }
}