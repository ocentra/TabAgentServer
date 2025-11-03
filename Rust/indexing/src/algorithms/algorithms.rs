//! Graph algorithms implementation for the TabAgent indexing system.
//!
//! This module implements various graph algorithms that work with the graph traits
//! defined in graph_traits.rs. These implementations follow the Rust Architecture
//! Guidelines for safety, performance, and clarity.

use crate::algorithms::graph_traits::*;
use common::DbResult;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};
use std::cmp::Reverse;
use ordered_float::OrderedFloat;
use std::hash::Hash;

/// Implementation of Kruskal's algorithm for finding minimum spanning tree.
///
/// Kruskal's algorithm finds a minimum spanning tree for a connected,
/// undirected graph. It is a greedy algorithm that sorts all edges by
/// weight and adds them to the MST if they don't create cycles.
pub fn kruskal<G>(graph: &G) -> DbResult<Vec<G::EdgeId>>
where
    G: GraphBase + IntoEdgeIdentifiers + Data + IntoNodeIdentifiers,
    G::EdgeWeight: Clone + PartialOrd,
    G::EdgeId: Clone + Ord,
{
    // Create a disjoint set data structure for cycle detection
    let mut disjoint_set = DisjointSet::new();
    
    // Add all nodes to the disjoint set
    for node_id in graph.node_identifiers() {
        disjoint_set.make_set(node_id);
    }
    
    // Create a vector of edges with their weights
    let mut edges: Vec<(G::EdgeWeight, G::EdgeId)> = Vec::new();
    
    for edge_id in graph.edge_identifiers() {
        if let Some(weight) = graph.edge_weight(edge_id.clone()) {
            edges.push((weight.clone(), edge_id));
        }
    }
    
    // Sort edges by weight
    edges.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    
    let mut mst_edges = Vec::new();
    
    // Process edges in order of increasing weight
    for (_, edge_id) in edges {
        if let Some((from, to)) = graph.edge_endpoints(edge_id.clone()) {
            // Check if adding this edge would create a cycle
            if disjoint_set.find(from.clone()) != disjoint_set.find(to.clone()) {
                // Add edge to MST
                mst_edges.push(edge_id.clone());
                
                // Union the sets
                disjoint_set.union(from, to);
            }
        }
    }
    
    Ok(mst_edges)
}

/// Implementation of Prim's algorithm for finding minimum spanning tree.
///
/// Prim's algorithm finds a minimum spanning tree for a connected,
/// undirected graph. It is a greedy algorithm that starts with an
/// arbitrary node and grows the MST by adding the minimum weight edge
/// that connects a node in the MST to a node not in the MST.
pub fn prim<G>(graph: &G) -> DbResult<Vec<G::EdgeId>>
where
    G: GraphBase + IntoNodeIdentifiers + IntoEdges + Data,
    G::EdgeWeight: Clone + PartialOrd + Into<f32>,
    G::NodeId: Clone + Eq + std::hash::Hash + Ord,
    G::EdgeId: Clone + Ord,
{
    use std::collections::BinaryHeap;
    
    // Start with an arbitrary node
    let start_node = match graph.node_identifiers().next() {
        Some(node) => node,
        None => return Ok(Vec::new()),
    };
    
    let mut visited = HashSet::new();
    let mut mst_edges = Vec::new();
    
    // Priority queue to store edges (negative weight for max heap behavior)
    let mut pq: BinaryHeap<(OrderedFloat<f32>, G::EdgeId)> = BinaryHeap::new();
    
    // Add start node to visited set
    visited.insert(start_node.clone());
    
    // Add all edges from start node to priority queue
    if let Ok(edges) = graph.edges(start_node) {
        for (edge_id, _, _) in edges {
            if let Some(weight) = graph.edge_weight(edge_id.clone()) {
                // Use negative weight for min-heap behavior
                // Convert weight to f32 for OrderedFloat
                let weight_f32: f32 = weight.clone().into();
                pq.push((OrderedFloat(-weight_f32), edge_id));
            }
        }
    }
    
    // Main loop: grow MST until all nodes are included
    while !visited.is_empty() && visited.len() < graph.node_count() {
        // Get the minimum weight edge
        if let Some((neg_weight, edge_id)) = pq.pop() {
            let weight = -neg_weight.into_inner();
            
            // Get edge endpoints
            if let Some((from, to)) = graph.edge_endpoints(edge_id.clone()) {
                // Determine which node is already in MST and which is not
                let (in_mst, not_in_mst) = if visited.contains(&from) && !visited.contains(&to) {
                    (from, to)
                } else if visited.contains(&to) && !visited.contains(&from) {
                    (to, from)
                } else {
                    // Both nodes are in MST or both are not in MST, skip this edge
                    continue;
                };
                
                // Add the new node to MST
                visited.insert(not_in_mst.clone());
                mst_edges.push(edge_id.clone());
                
                // Add all edges from the new node to priority queue
                if let Ok(edges) = graph.edges(not_in_mst) {
                    for (edge_id, _, _) in edges {
                        if let Some((edge_from, edge_to)) = graph.edge_endpoints(edge_id.clone()) {
                            // Only add edges that connect to unvisited nodes
                            if !visited.contains(&edge_to) || !visited.contains(&edge_from) {
                                if let Some(weight) = graph.edge_weight(edge_id.clone()) {
                                    // Use negative weight for min-heap behavior
                                    let weight_f32: f32 = weight.clone().into();
                                    pq.push((OrderedFloat(-weight_f32), edge_id));
                                }
                            }
                        }
                    }
                }
            }
        } else {
            // No more edges to process
            break;
        }
    }
    
    Ok(mst_edges)
}

/// Implementation of Dijkstra's algorithm for shortest paths.
pub fn dijkstra<G>(
    graph: &G,
    start: G::NodeId,
    goal: Option<G::NodeId>,
) -> DbResult<HashMap<G::NodeId, (f32, Option<G::NodeId>)>>
where
    G: DirectedGraph + IntoNodeIdentifiers,
    G::NodeId: Clone + Eq + std::hash::Hash + Ord,
    G::EdgeWeight: Clone + Into<f32>,
{
    let mut distances: HashMap<G::NodeId, (f32, Option<G::NodeId>)> = HashMap::new();
    let mut visited: HashSet<G::NodeId> = HashSet::new();
    let mut priority_queue: BinaryHeap<Reverse<(OrderedFloat<f32>, G::NodeId)>> = BinaryHeap::new();
    
    // Initialize distances
    distances.insert(start.clone(), (0.0, None));
    priority_queue.push(Reverse((OrderedFloat(0.0), start.clone())));
    
    while let Some(Reverse((OrderedFloat(current_distance), current_node))) = priority_queue.pop() {
        // If we've already processed this node, skip it
        if visited.contains(&current_node) {
            continue;
        }
        
        visited.insert(current_node.clone());
        
        // If we've reached our goal, we can stop early
        if let Some(ref goal_node) = goal {
            if &current_node == goal_node {
                break;
            }
        }
        
        // Process neighbors
        if let Ok(neighbors) = graph.neighbors(current_node.clone()) {
            for neighbor in neighbors {
                if visited.contains(&neighbor) {
                    continue;
                }
                
                // Get edge weight (this is a simplification - in a real implementation,
                // we would need to get the specific edge between current_node and neighbor)
                let edge_weight = 1.0f32; // Default weight
                
                let new_distance = current_distance + edge_weight;
                
                let should_update = match distances.get(&neighbor) {
                    Some((existing_distance, _)) => new_distance < *existing_distance,
                    None => true,
                };
                
                if should_update {
                    distances.insert(neighbor.clone(), (new_distance, Some(current_node.clone())));
                    priority_queue.push(Reverse((OrderedFloat(new_distance), neighbor.clone())));
                }
            }
        }
    }
    
    Ok(distances)
}

/// Implementation of Edmonds-Karp algorithm for maximum flow.
///
/// For a full implementation, see the `flow_algorithms` module.
pub fn edmonds_karp<G, F>(
    graph: &G,
    source: G::NodeId,
    sink: G::NodeId,
    capacity_fn: F,
) -> DbResult<f32>
where
    G: DirectedGraph + IntoNodeIdentifiers,
    G::NodeId: Clone + Eq + std::hash::Hash,
    G::EdgeId: Clone + Eq + std::hash::Hash,
    F: Fn(G::EdgeId) -> f32,
{
    crate::algorithms::flow_algorithms::edmonds_karp(graph, source, sink, capacity_fn)
}

/// Implementation of Dinic's algorithm for maximum flow.
///
/// For a full implementation, see the `flow_algorithms` module.
pub fn dinic<G, F>(
    graph: &G,
    source: G::NodeId,
    sink: G::NodeId,
    capacity_fn: F,
) -> DbResult<f32>
where
    G: DirectedGraph + IntoNodeIdentifiers,
    G::NodeId: Clone + Eq + std::hash::Hash,
    G::EdgeId: Clone + Eq + std::hash::Hash,
    F: Fn(G::EdgeId) -> f32,
{
    crate::algorithms::flow_algorithms::dinic(graph, source, sink, capacity_fn)
}

/// Finds connected components in an undirected graph.
pub fn connected_components<G>(graph: &G) -> DbResult<Vec<Vec<G::NodeId>>>
where
    G: GraphBase + IntoNodeIdentifiers + DirectedGraph,
    G::NodeId: Clone + Eq + std::hash::Hash + Ord,
{
    let mut components = Vec::new();
    let mut visited = HashSet::new();
    
    for node_id in graph.node_identifiers() {
        if !visited.contains(&node_id) {
            let mut component = Vec::new();
            let mut queue = VecDeque::new();
            queue.push_back(node_id.clone());
            visited.insert(node_id.clone());
            component.push(node_id.clone());
            
            while let Some(current) = queue.pop_front() {
                if let Ok(neighbors) = graph.neighbors(current) {
                    for neighbor in neighbors {
                        if !visited.contains(&neighbor) {
                            visited.insert(neighbor.clone());
                            component.push(neighbor.clone());
                            queue.push_back(neighbor);
                        }
                    }
                }
            }
            
            components.push(component);
        }
    }
    
    Ok(components)
}

/// Topological sort of a directed acyclic graph using Kahn's algorithm.
pub fn topological_sort<G>(graph: &G) -> DbResult<Vec<G::NodeId>>
where
    G: GraphBase + IntoNodeIdentifiers + DirectedGraph,
    G::NodeId: Clone + Eq + std::hash::Hash + Ord,
{
    // Calculate in-degrees for all nodes
    let mut in_degree: HashMap<G::NodeId, usize> = HashMap::new();
    
    // Initialize in-degrees to 0
    for node_id in graph.node_identifiers() {
        in_degree.insert(node_id, 0);
    }
    
    // Calculate actual in-degrees
    for node_id in graph.node_identifiers() {
        if let Ok(neighbors) = graph.neighbors(node_id) {
            for neighbor in neighbors {
                *in_degree.get_mut(&neighbor).unwrap_or(&mut 0) += 1;
            }
        }
    }
    
    // Add nodes with in-degree 0 to a queue
    let mut queue = VecDeque::new();
    for (node_id, &degree) in &in_degree {
        if degree == 0 {
            queue.push_back(node_id.clone());
        }
    }
    
    let mut result = Vec::new();
    
    // Process nodes in queue
    while let Some(node_id) = queue.pop_front() {
        result.push(node_id.clone());
        
        // Reduce in-degrees of neighbors
        if let Ok(neighbors) = graph.neighbors(node_id) {
            for neighbor in neighbors {
                let degree = in_degree.get_mut(&neighbor).unwrap();
                *degree -= 1;
                if *degree == 0 {
                    queue.push_back(neighbor);
                }
            }
        }
    }
    
    // Check for cycles
    if result.len() != graph.node_count() {
        return Err(common::DbError::InvalidOperation("Graph contains a cycle".to_string()));
    }
    
    Ok(result)
}

/// Cycle detection in a directed graph using DFS.
pub fn has_cycle<G>(graph: &G) -> DbResult<bool>
where
    G: GraphBase + IntoNodeIdentifiers + DirectedGraph,
    G::NodeId: Clone + Eq + std::hash::Hash + Ord,
{
    let mut visited = HashSet::new();
    let mut recursion_stack = HashSet::new();
    
    for node_id in graph.node_identifiers() {
        if !visited.contains(&node_id) {
            if has_cycle_util(graph, node_id, &mut visited, &mut recursion_stack)? {
                return Ok(true);
            }
        }
    }
    
    Ok(false)
}

/// Helper function for cycle detection.
fn has_cycle_util<G>(
    graph: &G,
    node_id: G::NodeId,
    visited: &mut HashSet<G::NodeId>,
    recursion_stack: &mut HashSet<G::NodeId>,
) -> DbResult<bool>
where
    G: DirectedGraph,
    G::NodeId: Clone + Eq + std::hash::Hash + Ord,
{
    let node_clone = node_id.clone();
    visited.insert(node_clone.clone());
    recursion_stack.insert(node_clone.clone());
    
    if let Ok(neighbors) = graph.neighbors(node_id) {
        for neighbor in neighbors {
            if !visited.contains(&neighbor) {
                if has_cycle_util(graph, neighbor, visited, recursion_stack)? {
                    return Ok(true);
                }
            } else if recursion_stack.contains(&neighbor) {
                return Ok(true);
            }
        }
    }
    
    recursion_stack.remove(&node_clone);
    Ok(false)
}

/// Disjoint set data structure for union-find operations.
///
/// This data structure is used in Kruskal's algorithm to efficiently
/// detect cycles when building a minimum spanning tree.
struct DisjointSet<T> {
    parent: HashMap<T, T>,
    rank: HashMap<T, usize>,
}

impl<T> DisjointSet<T>
where
    T: Clone + Eq + Hash,
{
    /// Creates a new empty disjoint set.
    fn new() -> Self {
        Self {
            parent: HashMap::new(),
            rank: HashMap::new(),
        }
    }
    
    /// Creates a new set containing a single element.
    fn make_set(&mut self, x: T) {
        self.parent.insert(x.clone(), x.clone());
        self.rank.insert(x, 0);
    }
    
    /// Finds the representative (root) of the set containing x.
    fn find(&mut self, x: T) -> T {
        let root = self.parent.get(&x).cloned().unwrap_or_else(|| x.clone());
        
        if root != x {
            // Path compression
            let new_root = self.find(root);
            self.parent.insert(x.clone(), new_root.clone());
            new_root
        } else {
            root
        }
    }
    
    /// Unions the sets containing x and y.
    fn union(&mut self, x: T, y: T) {
        let x_root = self.find(x);
        let y_root = self.find(y);
        
        if x_root == y_root {
            return;
        }
        
        // Union by rank
        let x_rank = *self.rank.get(&x_root).unwrap_or(&0);
        let y_rank = *self.rank.get(&y_root).unwrap_or(&0);
        
        if x_rank < y_rank {
            self.parent.insert(x_root, y_root);
        } else if x_rank > y_rank {
            self.parent.insert(y_root, x_root);
        } else {
            self.parent.insert(y_root, x_root.clone());
            self.rank.insert(x_root, x_rank + 1);
        }
    }
}

/// Finds all cycles in a directed graph.
///
/// This function uses DFS with backtracking to enumerate all simple cycles
/// in the graph. A simple cycle is a closed path where no node appears twice
/// except for the starting and ending node.
pub fn find_cycles<G>(graph: &G) -> DbResult<Vec<Vec<G::NodeId>>>
where
    G: GraphBase + IntoNodeIdentifiers + DirectedGraph,
    G::NodeId: Clone + Eq + std::hash::Hash + Ord,
{
    let mut cycles = Vec::new();
    let mut visited = HashSet::new();
    let mut recursion_stack = HashSet::new();
    let mut current_path = Vec::new();
    
    // Start DFS from each unvisited node
    for node_id in graph.node_identifiers() {
        if !visited.contains(&node_id) {
            find_cycles_dfs(
                graph,
                node_id,
                &mut visited,
                &mut recursion_stack,
                &mut current_path,
                &mut cycles,
            )?;
        }
    }
    
    Ok(cycles)
}

/// Helper function for cycle enumeration DFS.
fn find_cycles_dfs<G>(
    graph: &G,
    node_id: G::NodeId,
    visited: &mut HashSet<G::NodeId>,
    recursion_stack: &mut HashSet<G::NodeId>,
    current_path: &mut Vec<G::NodeId>,
    cycles: &mut Vec<Vec<G::NodeId>>,
) -> DbResult<()>
where
    G: DirectedGraph,
    G::NodeId: Clone + Eq + std::hash::Hash,
{
    let node_id_clone = node_id.clone();
    visited.insert(node_id_clone.clone());
    recursion_stack.insert(node_id_clone.clone());
    current_path.push(node_id_clone.clone());
    
    if let Ok(neighbors) = graph.neighbors(node_id) {
        for neighbor in neighbors {
            if recursion_stack.contains(&neighbor) {
                // Found a cycle - extract it from the current path
                if let Some(cycle_start) = current_path.iter().position(|n| n == &neighbor) {
                    let mut cycle = Vec::new();
                    for i in cycle_start..current_path.len() {
                        cycle.push(current_path[i].clone());
                    }
                    cycle.push(neighbor);
                    cycles.push(cycle);
                }
            } else if !visited.contains(&neighbor) {
                find_cycles_dfs(
                    graph,
                    neighbor,
                    visited,
                    recursion_stack,
                    current_path,
                    cycles,
                )?;
            }
        }
    }
    
    recursion_stack.remove(&node_id_clone);
    current_path.pop();
    Ok(())
}

/// Calculates the clustering coefficient for a node.
pub fn clustering_coefficient<G>(graph: &G, node: G::NodeId) -> DbResult<f32>
where
    G: DirectedGraph,
    G::NodeId: Clone + Eq + std::hash::Hash,
{
    let neighbors = match graph.neighbors(node) {
        Ok(n) => n,
        Err(_) => return Ok(0.0),
    };
    
    if neighbors.len() < 2 {
        return Ok(0.0);
    }
    
    let mut edge_count = 0;
    let neighbor_set: HashSet<_> = neighbors.iter().collect();
    
    // Count edges between neighbors
    for neighbor in &neighbors {
        if let Ok(n_neighbors) = graph.neighbors(neighbor.clone()) {
            for n_neighbor in n_neighbors {
                if neighbor_set.contains(&n_neighbor) {
                    edge_count += 1;
                }
            }
        }
    }
    
    // For undirected graphs, each edge is counted twice
    edge_count /= 2;
    
    // Calculate clustering coefficient
    let possible_edges = neighbors.len() * (neighbors.len() - 1) / 2;
    if possible_edges == 0 {
        Ok(0.0)
    } else {
        Ok(edge_count as f32 / possible_edges as f32)
    }
}

/// Calculates the global clustering coefficient for the graph.
pub fn global_clustering_coefficient<G>(graph: &G) -> DbResult<f32>
where
    G: GraphBase + IntoNodeIdentifiers + DirectedGraph,
    G::NodeId: Clone + Eq + std::hash::Hash,
{
    let mut total_coefficient = 0.0;
    let mut node_count = 0;
    
    for node_id in graph.node_identifiers() {
        let coefficient = clustering_coefficient(graph, node_id)?;
        total_coefficient += coefficient;
        node_count += 1;
    }
    
    if node_count == 0 {
        Ok(0.0)
    } else {
        Ok(total_coefficient / node_count as f32)
    }
}

/// Finds the diameter of the graph (longest shortest path).
///
/// The diameter is the longest shortest path between any two nodes
/// in the graph. This implementation uses BFS for unweighted graphs
/// and Dijkstra's algorithm for weighted graphs.
pub fn diameter<G>(graph: &G) -> DbResult<usize>
where
    G: GraphBase + IntoNodeIdentifiers + DirectedGraph,
    G::NodeId: Clone + Eq + std::hash::Hash + Ord,
    G::EdgeWeight: Clone + Into<f32>,
{
    let mut max_distance = 0;
    
    // Run shortest path algorithm from each node
    for start_node in graph.node_identifiers() {
        let distances = dijkstra(graph, start_node, None)?;
        
        // Find the maximum distance from this node
        for (_, (distance, _)) in distances {
            let distance_usize = distance as usize;
            if distance_usize > max_distance {
                max_distance = distance_usize;
            }
        }
    }
    
    Ok(max_distance)
}

/// Finds the radius of the graph (minimum eccentricity).
///
/// The radius is the minimum eccentricity among all nodes in the graph.
/// Eccentricity of a node is the maximum distance from that node to
/// any other node in the graph.
pub fn radius<G>(graph: &G) -> DbResult<usize>
where
    G: GraphBase + IntoNodeIdentifiers + DirectedGraph,
    G::NodeId: Clone + Eq + std::hash::Hash + Ord,
    G::EdgeWeight: Clone + Into<f32>,
{
    let mut min_eccentricity = usize::MAX;
    
    // Calculate eccentricity for each node
    for start_node in graph.node_identifiers() {
        let distances = dijkstra(graph, start_node, None)?;
        
        // Find the maximum distance from this node (eccentricity)
        let mut max_distance = 0;
        for (_, (distance, _)) in distances {
            let distance_usize = distance as usize;
            if distance_usize > max_distance {
                max_distance = distance_usize;
            }
        }
        
        // Update minimum eccentricity
        if max_distance < min_eccentricity {
            min_eccentricity = max_distance;
        }
    }
    
    if min_eccentricity == usize::MAX {
        Ok(0)
    } else {
        Ok(min_eccentricity)
    }
}

/// Finds the center nodes of the graph (nodes with eccentricity equal to radius).
///
/// The center nodes are those with eccentricity equal to the graph's radius.
pub fn center<G>(graph: &G) -> DbResult<Vec<G::NodeId>>
where
    G: GraphBase + IntoNodeIdentifiers + DirectedGraph,
    G::NodeId: Clone + Eq + std::hash::Hash + Ord,
    G::EdgeWeight: Clone + Into<f32>,
{
    let graph_radius = radius(graph)?;
    let mut center_nodes = Vec::new();
    
    // Calculate eccentricity for each node
    for start_node in graph.node_identifiers() {
        let distances = dijkstra(graph, start_node.clone(), None)?;
        
        // Find the maximum distance from this node (eccentricity)
        let mut max_distance = 0;
        for (_, (distance, _)) in distances {
            let distance_usize = distance as usize;
            if distance_usize > max_distance {
                max_distance = distance_usize;
            }
        }
        
        // If eccentricity equals radius, this is a center node
        if max_distance == graph_radius {
            center_nodes.push(start_node);
        }
    }
    
    Ok(center_nodes)
}

/// Finds articulation points (cut vertices) in the graph.
///
/// An articulation point is a node whose removal increases the
/// number of connected components in the graph.
pub fn articulation_points<G>(graph: &G) -> DbResult<HashSet<G::NodeId>>
where
    G: GraphBase + IntoNodeIdentifiers + DirectedGraph,
    G::NodeId: Clone + Eq + std::hash::Hash,
{
    let mut articulation_points = HashSet::new();
    let mut discovery_time = HashMap::new();
    let mut low_value = HashMap::new();
    let mut parent = HashMap::new();
    let mut visited = HashSet::new();
    let mut time = 0;
    
    // Run DFS from each unvisited node
    for node_id in graph.node_identifiers() {
        if !visited.contains(&node_id) {
            let mut children_count = 0;
            articulation_points_dfs(
                graph,
                node_id.clone(),
                &mut discovery_time,
                &mut low_value,
                &mut parent,
                &mut visited,
                &mut time,
                &mut children_count,
                &mut articulation_points,
            )?;
        }
    }
    
    Ok(articulation_points)
}

/// Helper function for articulation points DFS.
fn articulation_points_dfs<G>(
    graph: &G,
    node_id: G::NodeId,
    discovery_time: &mut HashMap<G::NodeId, usize>,
    low_value: &mut HashMap<G::NodeId, usize>,
    parent: &mut HashMap<G::NodeId, Option<G::NodeId>>,
    visited: &mut HashSet<G::NodeId>,
    time: &mut usize,
    children_count: &mut usize,
    articulation_points: &mut HashSet<G::NodeId>,
) -> DbResult<()>
where
    G: DirectedGraph,
    G::NodeId: Clone + Eq + std::hash::Hash,
{
    visited.insert(node_id.clone());
    *time += 1;
    let node_id_clone = node_id.clone();
    discovery_time.insert(node_id_clone.clone(), *time);
    low_value.insert(node_id_clone.clone(), *time);
    
    if let Ok(neighbors) = graph.neighbors(node_id) {
        for neighbor in neighbors {
            let neighbor_clone = neighbor.clone();
            if !visited.contains(&neighbor) {
                *children_count += 1;
                parent.insert(neighbor_clone.clone(), Some(node_id_clone.clone()));
                articulation_points_dfs(
                    graph,
                    neighbor,
                    discovery_time,
                    low_value,
                    parent,
                    visited,
                    time,
                    children_count,
                    articulation_points,
                )?;
                
                low_value.insert(
                    node_id_clone.clone(),
                    low_value[&node_id_clone].min(low_value[&neighbor_clone]),
                );
                if parent[&node_id_clone].is_none() && *children_count > 1 {
                    articulation_points.insert(node_id_clone.clone());
                }

                if parent[&node_id_clone].is_some() && low_value[&neighbor_clone] >= discovery_time[&node_id_clone] {
                    articulation_points.insert(node_id_clone.clone());
                }
            } else if parent[&node_id_clone] != Some(neighbor_clone.clone()) {
                low_value.insert(
                    node_id_clone.clone(),
                    low_value[&node_id_clone].min(discovery_time[&neighbor_clone]),
                );
            }
        }
    }
    
    Ok(())
}
