//! Graph traits for associated data and graph construction.

use crate::visit::Data;

/// Access node and edge weights (associated data).
pub trait DataMap: Data {
    fn node_weight(&self, id: Self::NodeId) -> Option<&Self::NodeWeight>;
    fn edge_weight(&self, id: Self::EdgeId) -> Option<&Self::EdgeWeight>;
}

/// Access node and edge weights mutably.
pub trait DataMapMut: DataMap {
    fn node_weight_mut(&mut self, id: Self::NodeId) -> Option<&mut Self::NodeWeight>;
    fn edge_weight_mut(&mut self, id: Self::EdgeId) -> Option<&mut Self::EdgeWeight>;
}

/// A graph element type (node or edge with weights)
pub enum Element<N, E> {
    Node {
        weight: N,
    },
    Edge {
        source: usize,
        target: usize,
        weight: E,
    },
}

/// Marker trait for building graphs from elements
pub trait FromElements: Sized {
    fn from_elements<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Element<Self::NodeWeight, Self::EdgeWeight>>,
        Self: Data;
}

/// Trait for building graphs (adding nodes and edges)
pub trait Build: Data + crate::visit::NodeCount {
    fn add_node(&mut self, weight: Self::NodeWeight) -> Self::NodeId;
    fn add_edge(&mut self, a: Self::NodeId, b: Self::NodeId, weight: Self::EdgeWeight) -> Option<Self::EdgeId>;
    fn update_edge(&mut self, a: Self::NodeId, b: Self::NodeId, weight: Self::EdgeWeight) -> Self::EdgeId;
}
