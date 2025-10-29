//! Error types for TabAgent server.
//!
//! # RAG Compliance
//! - Uses thiserror for library-style errors
//! - Specific error variants (not stringly-typed)
//! - Implements proper error propagation

use thiserror::Error;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// Result type for server operations.
#[allow(dead_code)] // Defined for future use
pub type ServerResult<T> = Result<T, ServerError>;

/// Server error types (RAG: Use enums for type safety).
#[derive(Error, Debug)]
#[allow(dead_code)] // Error variants defined for future use
pub enum ServerError {
    /// Database error
    #[error("Database error: {0}")]
    Database(#[from] anyhow::Error),

    /// Model cache error
    #[error("Model cache error: {0}")]
    ModelCache(String),

    /// Model not found
    #[error("Model not found: {0}")]
    ModelNotFound(String),

    /// Model not loaded
    #[error("Model not loaded: {0}")]
    ModelNotLoaded(String),

    /// Unsupported backend
    #[error("Unsupported backend: {0}")]
    UnsupportedBackend(String),

    /// Model loading error
    #[error("Failed to load model: {0}")]
    ModelLoadError(String),
    
    /// Model load error (alias)
    #[error("Load error: {0}")]
    LoadError(String),
    
    /// Cache error
    #[error("Cache error: {0}")]
    CacheError(String),
    
    /// Database error (string variant)
    #[error("Database error: {0}")]
    DatabaseError(String),

    /// Inference error
    #[error("Inference error: {0}")]
    InferenceError(String),

    /// Invalid request
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// WebRTC error
    #[error("WebRTC error: {0}")]
    WebRtc(String),

    /// Python bridge error
    #[error("Python inference error: {0}")]
    PythonError(String),

    /// Timeout error
    #[error("Operation timed out")]
    Timeout,

    /// Internal server error
    #[error("Internal server error: {0}")]
    Internal(String),
    
    /// Feature not yet implemented
    #[error("Not implemented: {0}")]
    NotImplemented(String),
}

// Implement IntoResponse for Axum (RAG: Proper error handling in HTTP layer)
impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match self {
            ServerError::Database(ref e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "database_error",
                e.to_string(),
            ),
            ServerError::ModelCache(ref msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "model_cache_error",
                msg.clone(),
            ),
            ServerError::ModelNotFound(ref model) => (
                StatusCode::NOT_FOUND,
                "model_not_found",
                format!("Model '{}' not found", model),
            ),
            ServerError::ModelNotLoaded(ref model) => (
                StatusCode::BAD_REQUEST,
                "model_not_loaded",
                format!("Model '{}' is not loaded", model),
            ),
            ServerError::UnsupportedBackend(ref backend) => (
                StatusCode::BAD_REQUEST,
                "unsupported_backend",
                format!("Unsupported backend: {}", backend),
            ),
            ServerError::ModelLoadError(ref msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "model_load_error",
                msg.clone(),
            ),
            ServerError::LoadError(ref msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "load_error",
                msg.clone(),
            ),
            ServerError::CacheError(ref msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "cache_error",
                msg.clone(),
            ),
            ServerError::DatabaseError(ref msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "database_error",
                msg.clone(),
            ),
            ServerError::InferenceError(ref msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "inference_error",
                msg.clone(),
            ),
            ServerError::InvalidRequest(ref msg) => (
                StatusCode::BAD_REQUEST,
                "invalid_request",
                msg.clone(),
            ),
            ServerError::SerializationError(ref e) => (
                StatusCode::BAD_REQUEST,
                "serialization_error",
                e.to_string(),
            ),
            ServerError::Io(ref e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "io_error",
                e.to_string(),
            ),
            ServerError::WebRtc(ref msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "webrtc_error",
                msg.clone(),
            ),
            ServerError::PythonError(ref msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "python_error",
                msg.clone(),
            ),
            ServerError::Timeout => (
                StatusCode::REQUEST_TIMEOUT,
                "timeout",
                "Request timed out".to_string(),
            ),
            ServerError::Internal(ref msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                msg.clone(),
            ),
            ServerError::NotImplemented(ref msg) => (
                StatusCode::NOT_IMPLEMENTED,
                "not_implemented",
                msg.clone(),
            ),
        };

        let body = Json(json!({
            "error": {
                "code": error_code,
                "message": message,
            }
        }));

        (status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ServerError::ModelNotFound("gpt-3.5-turbo".to_string());
        assert_eq!(err.to_string(), "Model not found: gpt-3.5-turbo");
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let server_err: ServerError = io_err.into();
        assert!(matches!(server_err, ServerError::Io(_)));
    }
}

