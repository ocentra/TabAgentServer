//! Error types for the value system.
//!
//! Following RAG guidelines:
//! - Use thiserror for library error types
//! - Provide specific error variants
//! - Include context in error messages

use thiserror::Error;
use crate::types::ValueType;

/// Result type for value operations.
pub type ValueResult<T> = Result<T, ValueError>;

/// Errors that can occur when working with values.
///
/// # RAG Compliance
///
/// - Specific error types (not stringly-typed)
/// - Rich context for debugging
/// - Implements std::error::Error via thiserror
#[derive(Error, Debug, Clone)]
pub enum ValueError {
    /// Type mismatch during downcast or conversion.
    #[error("Type mismatch: expected {expected}, got {actual:?}")]
    TypeMismatch {
        expected: &'static str,
        actual: ValueType,
    },

    /// Invalid value for the given type.
    #[error("Invalid value: {message}")]
    InvalidValue {
        message: String,
    },

    /// Serialization error.
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Deserialization error.
    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    /// Missing required field.
    #[error("Missing required field: {field}")]
    MissingField {
        field: &'static str,
    },

    /// Value out of valid range.
    #[error("Value out of range: {message}")]
    OutOfRange {
        message: String,
    },
}

impl From<serde_json::Error> for ValueError {
    fn from(err: serde_json::Error) -> Self {
        ValueError::SerializationError(err.to_string())
    }
}

// ============================================================================
// Backend Error Types - Contract for Downstream Errors
// ============================================================================

/// Backend error types - the contract for downstream errors.
///
/// The backend (tabagent-server) must return these typed errors instead of generic anyhow::Error.
/// This allows the API layer to:
/// - Map errors to appropriate HTTP status codes
/// - Provide helpful, actionable error messages to clients
/// - Include relevant context (request_id, field names, etc.)
///
/// # Usage in Backend
///
/// ```rust,ignore
/// // Backend should return BackendError instead of anyhow::Error
/// fn load_model(model: &str) -> Result<(), BackendError> {
///     if !model_exists(model) {
///         return Err(BackendError::ModelNotFound {
///             model: model.to_string(),
///         });
///     }
///     // ... load model
///     Ok(())
/// }
/// ```
#[derive(Error, Debug, Clone)]
pub enum BackendError {
    /// Model not loaded (503 Service Unavailable)
    #[error("Model '{model}' is not loaded")]
    ModelNotLoaded {
        model: String,
    },

    /// Model not found in registry (404 Not Found)
    #[error("Model '{model}' not found")]
    ModelNotFound {
        model: String,
    },

    /// Insufficient memory to load/run model (503 Service Unavailable)
    #[error("Insufficient memory: requires {required_mb}MB, only {available_mb}MB available")]
    OutOfMemory {
        required_mb: u64,
        available_mb: u64,
    },

    /// Generation exceeded timeout (504 Gateway Timeout)
    #[error("Generation exceeded {timeout_seconds} second timeout")]
    GenerationTimeout {
        timeout_seconds: u64,
    },

    /// Invalid input from backend perspective (400 Bad Request)
    #[error("Invalid input for field '{field}': {reason}")]
    InvalidInput {
        field: String,
        reason: String,
    },

    /// CUDA/GPU error (500 Internal Server Error)
    #[error("CUDA error (code {code}): {message}")]
    CudaError {
        code: i32,
        message: String,
    },

    /// Model file corruption or format error (422 Unprocessable Entity)
    #[error("Model '{model}' is corrupted: {reason}")]
    ModelCorrupted {
        model: String,
        reason: String,
    },

    /// Resource limit exceeded (429 Too Many Requests)
    #[error("Resource '{resource}' limit exceeded: {current}/{limit} used")]
    ResourceLimitExceeded {
        resource: String,
        limit: u64,
        current: u64,
    },

    /// Session not found (404 Not Found)
    #[error("Session '{session_id}' not found")]
    SessionNotFound {
        session_id: String,
    },

    /// Embedding model required but not available (503 Service Unavailable)
    #[error("Embedding model required for '{required_for}' but not available")]
    EmbeddingModelNotAvailable {
        required_for: String,
    },

    /// RAG/Vector store error (500 Internal Server Error)
    #[error("Vector store error during '{operation}': {reason}")]
    VectorStoreError {
        operation: String,
        reason: String,
    },

    /// Generic internal error (500 Internal Server Error)
    #[error("Internal error: {message}{}", context.as_ref().map(|c| format!(" (context: {})", c)).unwrap_or_default())]
    InternalError {
        message: String,
        context: Option<String>,
    },

    /// Configuration error (500 Internal Server Error)
    #[error("Configuration error for '{setting}': {reason}")]
    ConfigurationError {
        setting: String,
        reason: String,
    },

    /// Unsupported operation (501 Not Implemented)
    #[error("Feature '{feature}' is not implemented")]
    NotImplemented {
        feature: String,
    },
}

/// Result type for backend operations.
pub type BackendResult<T> = Result<T, BackendError>;

/// Convert from anyhow::Error to BackendError.
///
/// This provides a fallback for backends that haven't fully migrated to typed errors yet.
/// It attempts to parse the error message to categorize it appropriately.
impl From<anyhow::Error> for BackendError {
    fn from(err: anyhow::Error) -> Self {
        let err_str = err.to_string();

        // Pattern match common error messages
        if err_str.contains("not loaded") || err_str.contains("Not Loaded") {
            Self::ModelNotLoaded {
                model: "unknown".to_string(),
            }
        } else if err_str.contains("not found") || err_str.contains("Not Found") {
            Self::ModelNotFound {
                model: "unknown".to_string(),
            }
        } else if err_str.contains("out of memory") || err_str.contains("OOM") {
            Self::OutOfMemory {
                required_mb: 0,
                available_mb: 0,
            }
        } else if err_str.contains("timeout") || err_str.contains("Timeout") {
            Self::GenerationTimeout {
                timeout_seconds: 0,
            }
        } else if err_str.contains("CUDA") || err_str.contains("cuda") {
            Self::CudaError {
                code: 0,
                message: err_str,
            }
        } else if err_str.contains("corrupt") || err_str.contains("Corrupt") {
            Self::ModelCorrupted {
                model: "unknown".to_string(),
                reason: err_str,
            }
        } else {
            // Default to internal error
            Self::InternalError {
                message: err_str,
                context: None,
            }
        }
    }
}

