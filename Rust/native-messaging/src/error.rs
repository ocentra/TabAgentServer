//! Error types and handling for native messaging.
//!
//! This module provides consistent error handling that matches the patterns
//! established in the API and WebRTC crates, ensuring identical error
//! responses and logging behavior.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Result type for native messaging operations.
pub type NativeMessagingResult<T> = Result<T, NativeMessagingError>;

/// Error types for native messaging operations.
///
/// This error hierarchy mirrors the API and WebRTC error patterns
/// to ensure consistent error handling across all communication layers.
#[derive(Debug, thiserror::Error)]
pub enum NativeMessagingError {
    /// Protocol-level errors (malformed messages, invalid JSON, etc.)
    #[error("Protocol error: {0}")]
    Protocol(String),
    
    /// Request validation errors
    #[error("Validation error in field '{field}': {message}")]
    ValidationError {
        /// Field name that failed validation
        field: String,
        /// Validation error message
        message: String,
    },
    
    /// Route not found or unsupported
    #[error("Route not found: {route}")]
    RouteNotFound {
        /// Route identifier that was not found
        route: String,
    },
    
    /// Bad request (client error)
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    /// Internal server error
    #[error("Internal error: {0}")]
    Internal(String),
    
    /// Backend service error
    #[error("Backend error: {0}")]
    Backend(anyhow::Error),
    
    /// I/O errors (stdin/stdout communication)
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    /// JSON serialization/deserialization errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    /// Rate limiting error
    #[error("Rate limit exceeded: {message}")]
    RateLimitExceeded {
        /// Rate limit error message
        message: String,
        /// Retry after seconds
        retry_after: Option<u64>,
    },
    
    /// Authentication/authorization error
    #[error("Authentication error: {0}")]
    Auth(String),
}

/// Error response format for Chrome extensions.
///
/// This structure provides consistent error information that Chrome
/// extensions can parse and handle appropriately.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error code for programmatic handling
    pub code: String,
    
    /// Human-readable error message
    pub message: String,
    
    /// Additional error details and context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    
    /// Request ID for tracing (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

impl NativeMessagingError {
    /// Create a protocol error.
    pub fn protocol<S: Into<String>>(message: S) -> Self {
        Self::Protocol(message.into())
    }
    
    /// Create a validation error.
    pub fn validation<S: Into<String>>(field: S, message: S) -> Self {
        Self::ValidationError {
            field: field.into(),
            message: message.into(),
        }
    }
    
    /// Create a route not found error.
    pub fn route_not_found<S: Into<String>>(route: S) -> Self {
        Self::RouteNotFound {
            route: route.into(),
        }
    }
    
    /// Create a bad request error.
    pub fn bad_request<S: Into<String>>(message: S) -> Self {
        Self::BadRequest(message.into())
    }
    
    /// Create an internal error.
    pub fn internal<S: Into<String>>(message: S) -> Self {
        Self::Internal(message.into())
    }
    
    /// Create a rate limit exceeded error.
    pub fn rate_limit_exceeded<S: Into<String>>(message: S, retry_after: Option<u64>) -> Self {
        Self::RateLimitExceeded {
            message: message.into(),
            retry_after,
        }
    }
    
    /// Create an authentication error.
    pub fn auth<S: Into<String>>(message: S) -> Self {
        Self::Auth(message.into())
    }
    
    /// Get the error code for this error type.
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::Protocol(_) => "PROTOCOL_ERROR",
            Self::ValidationError { .. } => "VALIDATION_ERROR",
            Self::RouteNotFound { .. } => "ROUTE_NOT_FOUND",
            Self::BadRequest(_) => "BAD_REQUEST",
            Self::Internal(_) => "INTERNAL_ERROR",
            Self::Backend(_) => "BACKEND_ERROR",
            Self::Io(_) => "IO_ERROR",
            Self::Json(_) => "JSON_ERROR",
            Self::RateLimitExceeded { .. } => "RATE_LIMIT_EXCEEDED",
            Self::Auth(_) => "AUTH_ERROR",
        }
    }
    
    /// Check if this error is a client error (4xx equivalent).
    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            Self::Protocol(_) 
            | Self::ValidationError { .. } 
            | Self::RouteNotFound { .. } 
            | Self::BadRequest(_)
            | Self::Json(_)
            | Self::RateLimitExceeded { .. }
            | Self::Auth(_)
        )
    }
    
    /// Check if this error is a server error (5xx equivalent).
    pub fn is_server_error(&self) -> bool {
        matches!(
            self,
            Self::Internal(_) 
            | Self::Backend(_) 
            | Self::Io(_)
        )
    }
}

impl From<NativeMessagingError> for ErrorResponse {
    fn from(error: NativeMessagingError) -> Self {
        let code = error.error_code().to_string();
        let message = error.to_string();
        
        let details = match &error {
            NativeMessagingError::ValidationError { field, message } => {
                Some(serde_json::json!({
                    "field": field,
                    "validation_message": message
                }))
            }
            NativeMessagingError::RouteNotFound { route } => {
                Some(serde_json::json!({
                    "route": route
                }))
            }
            NativeMessagingError::RateLimitExceeded { retry_after, .. } => {
                Some(serde_json::json!({
                    "retry_after": retry_after
                }))
            }
            _ => None,
        };
        
        Self {
            code,
            message,
            details,
            request_id: None,
        }
    }
}

// Error conversions from API and WebRTC crates to maintain consistency

/// Convert from tabagent-api errors (when available)
impl From<anyhow::Error> for NativeMessagingError {
    fn from(error: anyhow::Error) -> Self {
        // Try to downcast to known error types first
        if let Some(io_error) = error.downcast_ref::<std::io::Error>() {
            return Self::Io(io_error.kind().into());
        }
        
        if let Some(json_error) = error.downcast_ref::<serde_json::Error>() {
            return Self::BadRequest(format!("JSON parsing error: {}", json_error));
        }
        
        // Check error message for common patterns
        let error_msg = error.to_string();
        if error_msg.contains("validation") || error_msg.contains("invalid") {
            return Self::BadRequest(error_msg);
        }
        
        if error_msg.contains("not found") {
            return Self::BadRequest(error_msg);
        }
        
        // Default to backend error
        Self::Backend(error)
    }
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_codes() {
        assert_eq!(NativeMessagingError::protocol("test").error_code(), "PROTOCOL_ERROR");
        assert_eq!(NativeMessagingError::validation("field", "message").error_code(), "VALIDATION_ERROR");
        assert_eq!(NativeMessagingError::route_not_found("test").error_code(), "ROUTE_NOT_FOUND");
        assert_eq!(NativeMessagingError::bad_request("test").error_code(), "BAD_REQUEST");
        assert_eq!(NativeMessagingError::internal("test").error_code(), "INTERNAL_ERROR");
        assert_eq!(NativeMessagingError::auth("test").error_code(), "AUTH_ERROR");
    }
    
    #[test]
    fn test_error_classification() {
        assert!(NativeMessagingError::protocol("test").is_client_error());
        assert!(NativeMessagingError::validation("field", "message").is_client_error());
        assert!(NativeMessagingError::route_not_found("test").is_client_error());
        assert!(NativeMessagingError::bad_request("test").is_client_error());
        
        assert!(NativeMessagingError::internal("test").is_server_error());
        assert!(NativeMessagingError::Backend(anyhow::anyhow!("test")).is_server_error());
    }
    
    #[test]
    fn test_error_response_conversion() {
        let error = NativeMessagingError::validation("temperature", "must be between 0.0 and 2.0");
        let response: ErrorResponse = error.into();
        
        assert_eq!(response.code, "VALIDATION_ERROR");
        assert!(response.message.contains("temperature"));
        assert!(response.details.is_some());
        
        if let Some(details) = response.details {
            assert_eq!(details["field"], "temperature");
            assert_eq!(details["validation_message"], "must be between 0.0 and 2.0");
        }
    }
    
    #[test]
    fn test_rate_limit_error() {
        let error = NativeMessagingError::rate_limit_exceeded("Too many requests", Some(60));
        let response: ErrorResponse = error.into();
        
        assert_eq!(response.code, "RATE_LIMIT_EXCEEDED");
        assert!(response.details.is_some());
        
        if let Some(details) = response.details {
            assert_eq!(details["retry_after"], 60);
        }
    }
    
    #[test]
    fn test_anyhow_error_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let anyhow_error = anyhow::Error::from(io_error);
        let native_error = NativeMessagingError::from(anyhow_error);
        
        assert!(matches!(native_error, NativeMessagingError::Io(_)));
    }
}