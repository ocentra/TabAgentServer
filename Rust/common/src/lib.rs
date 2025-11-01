//! Common types, traits, and utilities shared across all TabAgent crates.
//!
//! This crate provides foundational infrastructure used by all TabAgent components:
//! - **Unified Backend Trait** (`AppStateProvider`): Single trait for all entry points
//! - **Database Types**: IDs, errors, serialization helpers
//! - **Platform Types**: Models, actions, inference settings
//!
//! # Architecture
//!
//! The `common` crate sits at the bottom of the dependency hierarchy:
//! - Has NO dependencies on other workspace crates (except `tabagent-values`)
//! - Provides shared types and traits that all other crates use
//! - Ensures type consistency and DRY principles across the entire system
//!
//! # Key Exports
//!
//! ## Backend Infrastructure (UNIFIED)
//! - `AppStateProvider`: Unified trait for backend request handling
//! - `AppStateWrapper`: Axum-compatible wrapper for trait objects
//!
//! ## Routing Infrastructure (TRANSPORT-SPECIFIC)
//! Routing traits are intentionally separate per transport:
//! - `api::route_trait::RouteHandler`: HTTP-specific (Axum, status codes)
//! - `native-messaging::route_trait::NativeMessagingRoute`: Chrome protocol
//! - `webrtc::route_trait::DataChannelRoute`: WebRTC data channels
//!
//! This separation of concerns allows transport-specific error handling,
//! registration, and validation while keeping the business logic unified.
//!
//! ## Database Types
//! - `NodeId`, `EdgeId`, `EmbeddingId`: Type-safe IDs for knowledge graph
//! - `DbError`, `DbResult`: Error handling for database operations

pub mod models;
pub mod platform;
pub mod actions;
pub mod errors;
pub mod inference_settings;
pub mod hardware_constants;
pub mod backend;
pub mod grpc;
pub mod ml_client;
pub mod python_process_manager;
pub mod logging;

// Re-export for convenience
pub use ml_client::MlClient;
pub use python_process_manager::PythonProcessManager;

// Re-export commonly used types
pub use inference_settings::InferenceSettings;
pub use hardware_constants as hw_const;
pub use backend::{AppStateProvider, AppStateWrapper};

// --- Core Newtype Wrappers (RAG Rule 8.1) ---

/// Unique identifier for a node in the knowledge graph.
///
/// Nodes represent entities like chats, messages, summaries, attachments, and entities.
/// 
/// **Type Safety**: Using newtype pattern instead of alias prevents accidental mixing
/// of NodeId with EdgeId or EmbeddingId at compile time.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct NodeId(String);

impl NodeId {
    /// Create a new NodeId from a string
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    
    /// Generate a new unique NodeId
    pub fn generate() -> Self {
        use rand::Rng;
        let id: u64 = rand::rng().random();
        Self(format!("{:016x}", id))
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
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


/// Result type alias for database operations.
pub type DbResult<T> = Result<T, DbError>;

// --- Metadata Helper ---

/// Helper module for serializing/deserializing JSON fields.
///
/// Converts between serde_json::Value and String for rkyv compatibility.
pub mod json_metadata {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    /// Serializes a String as-is
    pub fn to_str<S>(value: &String, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        value.serialize(serializer)
    }

    /// Deserializes from serde_json::Value to String
    pub fn from_str<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        serde_json::to_string(&value).map_err(serde::de::Error::custom)
    }

    /// Serializes from serde_json::Value to JSON string for serde
    pub fn serialize_from_value<S>(value: &serde_json::Value, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let json_str = serde_json::to_string(value).map_err(serde::ser::Error::custom)?;
        json_str.serialize(serializer)
    }

    /// Serializes a String as-is
    pub fn to_value<S>(value: &String, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        value.serialize(serializer)
    }

    /// Deserializes from serde_json::Value to String
    pub fn from_value<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        serde_json::to_string(&value).map_err(serde::de::Error::custom)
    }
}


