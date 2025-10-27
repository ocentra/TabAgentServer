//! Error types for WebRTC operations
//!
//! Mirrors API error handling with RFC 7807 Problem Details support.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Result type for WebRTC operations
pub type WebRtcResult<T> = Result<T, WebRtcError>;

/// Errors that can occur during WebRTC operations
///
/// These errors map directly to HTTP-like status concepts for consistency
/// with the API layer, enabling unified error handling across all transport layers.
#[derive(Debug)]
pub enum WebRtcError {
    /// Bad request (400) - malformed signaling request
    BadRequest(String),
    
    /// Validation error (400) - specific field validation failed
    ValidationError {
        field: String,
        message: String,
    },
    
    /// Session not found (404)
    SessionNotFound(String),
    
    /// Invalid state (409) - operation not allowed in current session state
    InvalidState {
        session_id: String,
        expected: String,
        actual: String,
    },
    
    /// Session limit reached (429) - rate limit
    SessionLimitReached { current: usize, max: usize },
    
    /// Internal error (500)
    InternalError(String),
    
    /// Service unavailable (503)
    ServiceUnavailable(String),
    
    /// Generic error with message
    Other(String),
}

impl fmt::Display for WebRtcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            Self::ValidationError { field, message } => {
                write!(f, "Validation Error [{}]: {}", field, message)
            }
            Self::SessionNotFound(id) => write!(f, "Session Not Found: {}", id),
            Self::InvalidState {
                session_id,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "Invalid State [{}]: expected {}, got {}",
                    session_id, expected, actual
                )
            }
            Self::SessionLimitReached { current, max } => {
                write!(f, "Session Limit Reached: {}/{}", current, max)
            }
            Self::InternalError(msg) => write!(f, "Internal Error: {}", msg),
            Self::ServiceUnavailable(msg) => write!(f, "Service Unavailable: {}", msg),
            Self::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for WebRtcError {}

/// RFC 7807 Problem Details response for WebRTC errors
#[derive(Debug, Serialize, Deserialize)]
pub struct ProblemDetails {
    /// URI reference identifying the problem type
    #[serde(rename = "type")]
    pub type_uri: String,

    /// Short, human-readable summary
    pub title: String,

    /// Status code (HTTP-like)
    pub status: u16,

    /// Human-readable explanation
    pub detail: String,

    /// Session ID or request ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
    
    /// Field-specific validation errors
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<serde_json::Value>,
}

impl WebRtcError {
    /// Convert to RFC 7807 Problem Details JSON
    pub fn to_problem_details(&self, session_id: Option<String>) -> ProblemDetails {
        let (status, title, detail) = match self {
            Self::BadRequest(msg) => (400, "Bad Request", msg.clone()),
            Self::ValidationError { field, message } => (
                400,
                "Validation Error",
                format!("Field '{}': {}", field, message),
            ),
            Self::SessionNotFound(id) => (404, "Not Found", format!("Session {} not found", id)),
            Self::InvalidState { session_id, expected, actual } => (
                409,
                "Conflict",
                format!(
                    "Session {} is in state '{}', expected '{}'",
                    session_id, actual, expected
                ),
            ),
            Self::SessionLimitReached { current, max } => (
                429,
                "Too Many Requests",
                format!("Session limit reached: {}/{}", current, max),
            ),
            Self::InternalError(msg) => (500, "Internal Server Error", msg.clone()),
            Self::ServiceUnavailable(msg) => (503, "Service Unavailable", msg.clone()),
            Self::Other(msg) => (500, "Error", msg.clone()),
        };

        ProblemDetails {
            type_uri: format!("https://tabagent.dev/errors/{}", title.replace(' ', "-").to_lowercase()),
            title: title.to_string(),
            status,
            detail,
            instance: session_id,
            errors: None,
        }
    }
    
    /// Convert to JSON error response (for data channels)
    pub fn to_json_response(&self, session_id: Option<String>) -> serde_json::Value {
        let problem = self.to_problem_details(session_id);
        serde_json::to_value(&problem).unwrap_or_else(|_| {
            serde_json::json!({
                "error": self.to_string()
            })
        })
    }
}

impl From<serde_json::Error> for WebRtcError {
    fn from(err: serde_json::Error) -> Self {
        Self::BadRequest(format!("Invalid JSON: {}", err))
    }
}

impl From<anyhow::Error> for WebRtcError {
    fn from(err: anyhow::Error) -> Self {
        // Try to downcast to BackendError first (same as API!)
        if let Some(backend_err) = err.downcast_ref::<tabagent_values::BackendError>() {
            match backend_err {
                tabagent_values::BackendError::ModelNotLoaded { .. } => {
                    Self::ServiceUnavailable(err.to_string())
                }
                tabagent_values::BackendError::OutOfMemory { .. } => {
                    Self::ServiceUnavailable(err.to_string())
                }
                tabagent_values::BackendError::SessionNotFound { .. } => {
                    Self::SessionNotFound(err.to_string())
                }
                _ => Self::InternalError(err.to_string()),
            }
        } else {
            Self::Other(err.to_string())
        }
    }
}

