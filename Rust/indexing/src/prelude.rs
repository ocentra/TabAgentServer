//! Prelude module for commonly used imports.
//!
//! This module re-exports types and traits that are frequently used throughout
//! the codebase for common graph operations.

// Direction and edge types
pub use crate::{Direction, Incoming, Outgoing, Directed, Undirected, EdgeType};

// Visit traits and types
pub use crate::visit::*;

// Scored types for priority queues
pub use crate::scored::*;

// Data access traits
pub use crate::data::*;

// Union-Find
pub use crate::unionfind::*;

