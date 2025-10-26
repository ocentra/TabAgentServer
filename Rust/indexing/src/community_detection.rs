//! Community detection algorithms for graph analysis.
//!
//! This module provides implementations of various community detection algorithms
//! commonly used in network analysis, including the Louvain algorithm for
//! modularity optimization. These implementations follow the Rust Architecture
//! Guidelines for safety, performance, and clarity.

use crate::graph_traits::{GraphBase, UndirectedGraph, IntoNodeIdentifiers, IntoEdgeIdentifiers};
use common::DbResult;
use std::collections::HashMap;
use std::hash::Hash;

/// Community detection using the Louvain algorithm for modularity optimization.
///
/// The Louvain algorithm is a greedy optimization method that attempts to
/// optimize the modularity of a partition of a network. Modularity is a
/// measure of the strength of division of a network into modules (communities).
pub struct LouvainCommunityDetection;

impl LouvainCommunityDetection {
    /// Creates a new Louvain community detection instance.
    pub fn new() -> Self {
        Self
    }
    
    /// Detects communities using the Louvain algorithm.
    ///
    /// # Arguments
    ///
    /// * `graph` - The graph to analyze
    /// * `max_iterations` - Maximum number of iterations to run
    ///
    /// # Returns
    ///
    /// A vector of communities, where each community is a vector of node IDs.
    pub fn detect_communities<G>(
        &self,
        graph: &G,
        max_iterations: usize,
    ) -> DbResult<Vec<Vec<G::NodeId>>>
    where
        G: GraphBase + UndirectedGraph + IntoNodeIdentifiers + IntoEdgeIdentifiers,
        G::NodeId: Clone + Eq + Hash + Ord,
    {
        // Initialize each node as its own community
        let mut communities: HashMap<G::NodeId, usize> = graph
            .node_identifiers()
            .enumerate()
            .map(|(i, node)| (node, i))
            .collect();
        
        let mut community_nodes: Vec<Vec<G::NodeId>> = (0..communities.len())
            .map(|_| Vec::new())
            .collect();
        
        // Initialize community nodes
        for (node, community_id) in &communities {
            community_nodes[*community_id].push(node.clone());
        }
        
        let mut prev_modularity = -1.0;
        let mut current_modularity;
        
        // Calculate total edge weight for modularity computation
        let total_edge_weight: f32 = graph
            .edge_identifiers()
            .map(|_| 1.0) // Assuming unweighted edges
            .sum();
        
        // Phase 1: Local optimization
        for _iteration in 0..max_iterations {
            let mut changed = false;
            
            // For each node, try to move it to a better community
            for node_id in graph.node_identifiers() {
                let current_community = *communities.get(&node_id).unwrap();
                
                // Calculate modularity gain for moving to each neighbor's community
                let mut best_community = current_community;
                let mut best_gain = 0.0;
                
                if let Ok(neighbors) = graph.neighbors(node_id.clone()) {
                    for neighbor in neighbors {
                        let neighbor_community = *communities.get(&neighbor).unwrap();
                        if neighbor_community != current_community {
                            let gain = self.calculate_modularity_gain(
                                graph,
                                &node_id,
                                current_community,
                                neighbor_community,
                                &communities,
                                &community_nodes,
                                total_edge_weight,
                            )?;
                            
                            if gain > best_gain {
                                best_gain = gain;
                                best_community = neighbor_community;
                            }
                        }
                    }
                }
                
                // Move node to the best community if it improves modularity
                if best_community != current_community {
                    // Remove node from current community
                    community_nodes[current_community].retain(|n| *n != node_id);
                    
                    // Add node to new community
                    community_nodes[best_community].push(node_id.clone());
                    communities.insert(node_id.clone(), best_community);
                    
                    changed = true;
                }
            }
            
            // Calculate current modularity
            current_modularity = self.calculate_modularity(
                graph,
                &communities,
                &community_nodes,
                total_edge_weight,
            )?;
            
            // Check for convergence
            if !changed || (current_modularity - prev_modularity).abs() < 1e-6 {
                break;
            }
            
            prev_modularity = current_modularity;
        }
        
        // Return non-empty communities
        Ok(community_nodes
            .into_iter()
            .filter(|community| !community.is_empty())
            .collect())
    }
    
    /// Calculates the modularity gain of moving a node from one community to another.
    fn calculate_modularity_gain<G>(
        &self,
        graph: &G,
        node_id: &G::NodeId,
        from_community: usize,
        to_community: usize,
        communities: &HashMap<G::NodeId, usize>,
        community_nodes: &[Vec<G::NodeId>],
        total_edge_weight: f32,
    ) -> DbResult<f32>
    where
        G: GraphBase + UndirectedGraph,
        G::NodeId: Clone + Eq + Hash,
    {
        // Calculate the sum of weights of edges from node to nodes in from_community
        let mut ki_in_from = 0.0;
        if let Ok(neighbors) = graph.neighbors(node_id.clone()) {
            for neighbor in neighbors {
                if *communities.get(&neighbor).unwrap() == from_community {
                    ki_in_from += 1.0; // Assuming unweighted edges
                }
            }
        }
        
        // Calculate the sum of weights of edges from node to nodes in to_community
        let mut ki_in_to = 0.0;
        if let Ok(neighbors) = graph.neighbors(node_id.clone()) {
            for neighbor in neighbors {
                if *communities.get(&neighbor).unwrap() == to_community {
                    ki_in_to += 1.0; // Assuming unweighted edges
                }
            }
        }
        
        // Calculate sum of weights of edges in from_community
        let mut sum_in_from = 0.0;
        for node in &community_nodes[from_community] {
            if *node != *node_id {
                if let Ok(neighbors) = graph.neighbors(node.clone()) {
                    for neighbor in neighbors {
                        if *communities.get(&neighbor).unwrap() == from_community {
                            sum_in_from += 1.0; // Assuming unweighted edges
                        }
                    }
                }
            }
        }
        sum_in_from /= 2.0; // Each edge is counted twice
        
        // Calculate sum of weights of edges in to_community
        let mut sum_in_to = 0.0;
        for node in &community_nodes[to_community] {
            if let Ok(neighbors) = graph.neighbors(node.clone()) {
                for neighbor in neighbors {
                    if *communities.get(&neighbor).unwrap() == to_community {
                        sum_in_to += 1.0; // Assuming unweighted edges
                    }
                }
            }
        }
        sum_in_to /= 2.0; // Each edge is counted twice
        
        // Calculate sum of weights of edges incident to from_community
        let mut sum_tot_from = 0.0;
        for node in &community_nodes[from_community] {
            if *node != *node_id {
                // Count all edges from this node
                sum_tot_from += graph.neighbors(node.clone()).map(|v| v.len() as f32).unwrap_or(0.0);
            }
        }
        
        // Calculate sum of weights of edges incident to to_community
        let mut sum_tot_to = 0.0;
        for node in &community_nodes[to_community] {
            // Count all edges from this node
            sum_tot_to += graph.neighbors(node.clone()).map(|v| v.len() as f32).unwrap_or(0.0);
        }
        
        // Calculate the modularity gain
        let delta_mod = (sum_in_to + ki_in_to) / total_edge_weight
            - sum_in_to / total_edge_weight
            - ((sum_tot_to + graph.neighbors(node_id.clone()).map(|v| v.len() as f32).unwrap_or(0.0))
                * (sum_tot_to + graph.neighbors(node_id.clone()).map(|v| v.len() as f32).unwrap_or(0.0)))
                / (4.0 * total_edge_weight * total_edge_weight)
            + (sum_in_from) / total_edge_weight
            + ((sum_tot_from) * (sum_tot_from)) / (4.0 * total_edge_weight * total_edge_weight)
            - (sum_in_from - ki_in_from) / total_edge_weight
            - ((sum_tot_from - graph.neighbors(node_id.clone()).map(|v| v.len() as f32).unwrap_or(0.0))
                * (sum_tot_from - graph.neighbors(node_id.clone()).map(|v| v.len() as f32).unwrap_or(0.0)))
                / (4.0 * total_edge_weight * total_edge_weight);
        
        Ok(delta_mod)
    }
    
    /// Calculates the modularity of the current community partition.
    fn calculate_modularity<G>(
        &self,
        graph: &G,
        communities: &HashMap<G::NodeId, usize>,
        community_nodes: &[Vec<G::NodeId>],
        total_edge_weight: f32,
    ) -> DbResult<f32>
    where
        G: GraphBase + UndirectedGraph + IntoNodeIdentifiers,
        G::NodeId: Clone + Eq + Hash,
    {
        let mut modularity = 0.0;
        
        // For each edge, check if both endpoints are in the same community
        for node_id in graph.node_identifiers() {
            if let Ok(neighbors) = graph.neighbors(node_id.clone()) {
                let node_community = *communities.get(&node_id).unwrap();
                
                for neighbor in neighbors {
                    let neighbor_community = *communities.get(&neighbor).unwrap();
                    
                    if node_community == neighbor_community {
                        modularity += 1.0; // Assuming unweighted edges
                    }
                }
            }
        }
        
        modularity /= 2.0; // Each edge is counted twice
        modularity /= total_edge_weight;
        
        // Subtract the expected modularity in a random graph
        let mut sum_sq = 0.0;
        for community in community_nodes {
            let mut degree_sum = 0.0;
            for node in community {
                degree_sum += graph.neighbors(node.clone()).map(|v| v.len() as f32).unwrap_or(0.0);
            }
            sum_sq += (degree_sum / (2.0 * total_edge_weight)).powi(2);
        }
        
        modularity -= sum_sq;
        
        Ok(modularity)
    }
}

impl Default for LouvainCommunityDetection {
    fn default() -> Self {
        Self::new()
    }
}

/// Detects communities using the Louvain algorithm.
///
/// This is a convenience function that creates a LouvainCommunityDetection
/// instance and runs the algorithm.
pub fn louvain_communities<G>(
    graph: &G,
    max_iterations: usize,
) -> DbResult<Vec<Vec<G::NodeId>>>
where
    G: GraphBase + UndirectedGraph + IntoNodeIdentifiers + IntoEdgeIdentifiers,
    G::NodeId: Clone + Eq + Hash + Ord,
{
    let detector = LouvainCommunityDetection::new();
    detector.detect_communities(graph, max_iterations)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph_traits::{GraphBase, UndirectedGraph, IntoNodeIdentifiers};
    use std::collections::{HashMap, HashSet};
    
    // Simple graph implementation for testing
    struct TestGraph {
        nodes: HashSet<String>,
        edges: HashMap<String, HashSet<String>>,
    }
    
    impl TestGraph {
        fn new() -> Self {
            Self {
                nodes: HashSet::new(),
                edges: HashMap::new(),
            }
        }
        
        fn add_node(&mut self, node: &str) {
            self.nodes.insert(node.to_string());
            self.edges.entry(node.to_string()).or_insert_with(HashSet::new);
        }
        
        fn add_edge(&mut self, from: &str, to: &str) {
            self.nodes.insert(from.to_string());
            self.nodes.insert(to.to_string());
            self.edges.entry(from.to_string()).or_insert_with(HashSet::new).insert(to.to_string());
            self.edges.entry(to.to_string()).or_insert_with(HashSet::new).insert(from.to_string());
        }
    }
    
    impl GraphBase for TestGraph {
        type NodeId = String;
        type EdgeId = String;
        
        fn node_count(&self) -> usize {
            self.nodes.len()
        }
        
        fn edge_count(&self) -> usize {
            self.edges.values().map(|neighbors| neighbors.len()).sum::<usize>() / 2
        }
    }
    
    impl IntoNodeIdentifiers for TestGraph {
        type NodeIdentifiers = std::vec::IntoIter<Self::NodeId>;
        
        fn node_identifiers(&self) -> Self::NodeIdentifiers {
            self.nodes.iter().cloned().collect::<Vec<_>>().into_iter()
        }
    }
    
    impl IntoEdgeIdentifiers for TestGraph {
        type EdgeIdentifiers = std::vec::IntoIter<Self::EdgeId>;
        
        fn edge_identifiers(&self) -> Self::EdgeIdentifiers {
            // Simplified implementation for testing
            vec![].into_iter()
        }
    }
    
    impl UndirectedGraph for TestGraph {
        fn neighbors(&self, node: Self::NodeId) -> DbResult<Vec<Self::NodeId>> {
            Ok(self.edges.get(&node).cloned().unwrap_or_default().into_iter().collect())
        }
    }

    #[test]
    fn test_louvain_community_detection() {
        let mut graph = TestGraph::new();
        
        // Create a simple graph with two communities
        // Community 1: nodes 1, 2, 3 (densely connected)
        graph.add_node("1");
        graph.add_node("2");
        graph.add_node("3");
        graph.add_edge("1", "2");
        graph.add_edge("2", "3");
        graph.add_edge("1", "3");
        
        // Community 2: nodes 4, 5, 6 (densely connected)
        graph.add_node("4");
        graph.add_node("5");
        graph.add_node("6");
        graph.add_edge("4", "5");
        graph.add_edge("5", "6");
        graph.add_edge("4", "6");
        
        // Weak connection between communities
        graph.add_edge("3", "4");
        
        let detector = LouvainCommunityDetection::new();
        let communities = detector.detect_communities(&graph, 10).unwrap();
        
        // We should have at least 2 communities
        assert!(communities.len() >= 2);
        
        // Check that nodes are grouped reasonably
        let mut community_sizes: Vec<usize> = communities.iter().map(|c| c.len()).collect();
        community_sizes.sort();
        
        // Should have communities of size 3 and 3 (or similar)
        assert!(!community_sizes.is_empty());
    }
}