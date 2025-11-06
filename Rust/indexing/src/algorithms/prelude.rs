//! Prelude module for common graph algorithm imports.
//!
//! This module re-exports commonly used types and traits that are already
//! available elsewhere in the codebase (DRY!).

// Re-export from root level
pub use crate::scored::{MinScored, MaxScored};
pub use crate::unionfind::UnionFind;
pub use crate::data::*;
pub use crate::visit::*;
pub use crate::EdgeType;

// Direction and Incoming/Outgoing (already defined in lib.rs)
pub use crate::Direction;
pub use crate::Direction::{Incoming, Outgoing};

// Our custom traits from graph_traits (extensions/additions)
pub use super::graph_traits::{
    DirectedGraph, UndirectedGraph, GraphMut, 
    BfsTraversal, DfsTraversal, ShortestPath, 
    Centrality, CommunityDetection, MinimumSpanningTree, Flow,
    NodeIndex as GraphNodeIndex, EdgeIndex as GraphEdgeIndex,
};

// Algorithm utilities
pub use super::algo::{Measure, Infinity, NegativeCycle};
