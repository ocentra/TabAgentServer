//! Error types for the MCP crate.
//!
//! Following RAG guidelines:
//! - Use thiserror for library error types
//! - Provide specific error variants
//! - Include context in error messages

use thiserror::Error;

/// Custom error type for MCP operations.
///
/// # RAG Compliance
///
/// - Specific error types (not stringly-typed)
/// - Rich context for debugging
/// - Implements std::error::Error via thiserror
#[derive(Error, Debug)]
pub enum McpError {
    /// Storage/database error.
    #[error("Storage error: {0}")]
    Storage(#[from] common::DbError),

    /// Serialization/deserialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// AppState communication error.
    #[error("AppState communication error: {0}")]
    AppState(String),

    /// Invalid parameter provided.
    #[error("Invalid parameter: {field} - {reason}")]
    InvalidParameter {
        field: String,
        reason: String,
    },

    /// Tool not found.
    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    /// Log query failed.
    #[error("Log query failed: {0}")]
    LogQueryFailed(String),

    /// Model operation failed.
    #[error("Model operation failed: {0}")]
    ModelOperationFailed(String),

    /// System operation failed.
    #[error("System operation failed: {0}")]
    SystemOperationFailed(String),

    /// Generation operation failed.
    #[error("Generation operation failed: {0}")]
    GenerationOperationFailed(String),

    /// RAG operation failed.
    #[error("RAG operation failed: {0}")]
    RagOperationFailed(String),
}

/// Result type for MCP operations.
pub type McpResult<T> = Result<T, McpError>;

impl From<serde_json::Error> for McpError {
    fn from(err: serde_json::Error) -> Self {
        McpError::Serialization(err.to_string())
    }
}

impl From<anyhow::Error> for McpError {
    fn from(err: anyhow::Error) -> Self {
        McpError::AppState(err.to_string())
    }
}

