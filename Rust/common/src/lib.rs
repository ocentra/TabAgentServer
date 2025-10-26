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
pub mod actions;
pub mod errors;
pub mod inference_settings;
pub mod hardware_constants;

// Re-export commonly used types
pub use inference_settings::InferenceSettings;
pub use hardware_constants as hw_const;

// --- Core Newtype Wrappers (RAG Rule 8.1) ---

/// Unique identifier for a node in the knowledge graph.
///
/// Nodes represent entities like chats, messages, summaries, attachments, and entities.
/// 
/// **Type Safety**: Using newtype pattern instead of alias prevents accidental mixing
/// of NodeId with EdgeId or EmbeddingId at compile time.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct NodeId(String);

impl NodeId {
    /// Create a new NodeId from a string
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    
    /// Get the inner string reference
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    /// Consume and return the inner string
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for NodeId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for NodeId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Unique identifier for an edge connecting two nodes.
///
/// Edges represent typed relationships like "CONTAINS_MESSAGE", "MENTIONS", etc.
/// 
/// **Type Safety**: Using newtype pattern prevents mixing with NodeId or EmbeddingId.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct EdgeId(String);

impl EdgeId {
    /// Create a new EdgeId from a string
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    
    /// Get the inner string reference
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    /// Consume and return the inner string
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl std::fmt::Display for EdgeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for EdgeId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for EdgeId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Unique identifier for a vector embedding.
///
/// Embeddings are high-dimensional vectors used for semantic search.
/// 
/// **Type Safety**: Using newtype pattern prevents mixing with NodeId or EdgeId.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct EmbeddingId(String);

impl EmbeddingId {
    /// Create a new EmbeddingId from a string
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    
    /// Get the inner string reference
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    /// Consume and return the inner string
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl std::fmt::Display for EmbeddingId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for EmbeddingId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for EmbeddingId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

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


