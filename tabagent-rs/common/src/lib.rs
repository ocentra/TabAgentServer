//! Common types and utilities shared across the embedded database crates.
//!
//! This crate provides foundational types, type aliases, and error definitions
//! that are used by multiple crates in the workspace (storage, indexing, query-engine, etc.).
//!
//! # Architecture
//!
//! The `common` crate sits at the bottom of the dependency hierarchy:
//! - Has NO dependencies on other workspace crates
//! - Provides shared types that all other crates can use
//! - Ensures type consistency across the entire system

pub mod models;
pub mod platform;

// --- Core Type Aliases ---

/// Unique identifier for a node in the knowledge graph.
///
/// Nodes represent entities like chats, messages, summaries, attachments, and entities.
pub type NodeId = String;

/// Unique identifier for an edge connecting two nodes.
///
/// Edges represent typed relationships like "CONTAINS_MESSAGE", "MENTIONS", etc.
pub type EdgeId = String;

/// Unique identifier for a vector embedding.
///
/// Embeddings are high-dimensional vectors used for semantic search.
pub type EmbeddingId = String;

// --- Error Types ---

/// Common error type for database operations.
///
/// This error type is used across all database crates for consistency.
/// Each crate may extend this with its own error variants.
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    /// Error from the sled storage backend.
    #[error("Sled database error: {0}")]
    Sled(#[from] sled::Error),

    /// Error during serialization/deserialization.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Requested entity not found.
    #[error("Entity not found: {0}")]
    NotFound(String),

    /// Invalid operation or arguments.
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Other errors (e.g., lock poisoning).
    #[error("{0}")]
    Other(String),
}

// Implement From for bincode errors manually to convert to String
impl From<bincode::Error> for DbError {
    fn from(err: bincode::Error) -> Self {
        DbError::Serialization(err.to_string())
    }
}

/// Result type alias for database operations.
pub type DbResult<T> = Result<T, DbError>;

// --- Metadata Helper ---

/// Helper module for serializing serde_json::Value with bincode.
///
/// Bincode doesn't support serde_json::Value directly, so we serialize it as a JSON string.
/// This module provides custom serialization functions that can be used with
/// `#[serde(with = "common::json_metadata")]`.
pub mod json_metadata {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    /// Serializes a serde_json::Value as a JSON string for bincode compatibility.
    pub fn serialize<S>(value: &serde_json::Value, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let json_string = serde_json::to_string(value).map_err(serde::ser::Error::custom)?;
        json_string.serialize(serializer)
    }

    /// Deserializes a JSON string back to a serde_json::Value.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<serde_json::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let json_string = String::deserialize(deserializer)?;
        serde_json::from_str(&json_string).map_err(serde::de::Error::custom)
    }
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_aliases() {
        let node_id: NodeId = "node_123".to_string();
        let edge_id: EdgeId = "edge_456".to_string();
        let embedding_id: EmbeddingId = "embed_789".to_string();

        assert_eq!(node_id, "node_123");
        assert_eq!(edge_id, "edge_456");
        assert_eq!(embedding_id, "embed_789");
    }

    #[test]
    fn test_error_display() {
        let err = DbError::NotFound("test_id".to_string());
        assert_eq!(err.to_string(), "Entity not found: test_id");

        let err = DbError::InvalidOperation("test operation".to_string());
        assert_eq!(err.to_string(), "Invalid operation: test operation");
    }
}
