//! Error types for the API.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Result type for API operations.
pub type ApiResult<T> = Result<T, ApiError>;

/// API error types following RFC 7807 Problem Details.
#[derive(Debug)]
pub enum ApiError {
    /// Bad request (400) - malformed request
    BadRequest(String),

    /// Validation error (400) - specific field validation failed
    ValidationError {
        /// The field that failed validation
        field: String,
        /// The validation error message
        message: String,
        /// Optional request ID for tracking
        request_id: Option<String>,
    },

    /// Not found (404) - resource doesn't exist
    NotFound(String),

    /// Unprocessable entity (422) - valid format, invalid semantics
    UnprocessableEntity(String),

    /// Internal server error (500)
    InternalError(String),
    
    /// Internal server error (alias for convenience)
    Internal(String),

    /// Service unavailable (503) - temporary failure
    ServiceUnavailable(String),

    /// Gateway timeout (504) - request took too long
    Timeout(String),

    /// Rate limit exceeded (429)
    RateLimitExceeded(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            Self::ValidationError { field, message, .. } => {
                write!(f, "Validation Error [field: {}]: {}", field, message)
            },
            Self::NotFound(msg) => write!(f, "Not Found: {}", msg),
            Self::UnprocessableEntity(msg) => write!(f, "Unprocessable Entity: {}", msg),
            Self::InternalError(msg) | Self::Internal(msg) => write!(f, "Internal Error: {}", msg),
            Self::ServiceUnavailable(msg) => write!(f, "Service Unavailable: {}", msg),
            Self::Timeout(msg) => write!(f, "Timeout: {}", msg),
            Self::RateLimitExceeded(msg) => write!(f, "Rate Limit Exceeded: {}", msg),
        }
    }
}

impl std::error::Error for ApiError {}

/// RFC 7807 Problem Details response.
#[derive(Debug, Serialize, Deserialize)]
struct ProblemDetails {
    /// URI reference identifying the problem type
    #[serde(rename = "type")]
    type_uri: String,

    /// Short, human-readable summary
    title: String,

    /// HTTP status code
    status: u16,

    /// Human-readable explanation
    detail: String,

    /// URI reference identifying the specific occurrence
    #[serde(skip_serializing_if = "Option::is_none")]
    instance: Option<String>,
    
    /// Request ID for tracing
    #[serde(skip_serializing_if = "Option::is_none")]
    request_id: Option<String>,
    
    /// Field-specific validation errors
    #[serde(skip_serializing_if = "Option::is_none")]
    errors: Option<serde_json::Value>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, title, detail, request_id, errors) = match &self {
            Self::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                "Bad Request",
                msg.clone(),
                None,
                None,
            ),
            Self::ValidationError { field, message, request_id } => (
                StatusCode::BAD_REQUEST,
                "Validation Error",
                format!("Field '{}': {}", field, message),
                request_id.clone(),
                Some(serde_json::json!({ field: message })),
            ),
            Self::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                "Not Found",
                msg.clone(),
                None,
                None,
            ),
            Self::UnprocessableEntity(msg) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "Unprocessable Entity",
                msg.clone(),
                None,
                None,
            ),
            Self::InternalError(msg) | Self::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error",
                msg.clone(),
                None,
                None,
            ),
            Self::ServiceUnavailable(msg) => (
                StatusCode::SERVICE_UNAVAILABLE,
                "Service Unavailable",
                msg.clone(),
                None,
                None,
            ),
            Self::Timeout(msg) => (
                StatusCode::GATEWAY_TIMEOUT,
                "Gateway Timeout",
                msg.clone(),
                None,
                None,
            ),
            Self::RateLimitExceeded(msg) => (
                StatusCode::TOO_MANY_REQUESTS,
                "Too Many Requests",
                msg.clone(),
                None,
                None,
            ),
        };

        let problem = ProblemDetails {
            type_uri: format!("https://tabagent.dev/errors/{}", 
                title.to_lowercase().replace(' ', "-")),
            title: title.to_string(),
            status: status.as_u16(),
            detail,
            instance: None,
            request_id,
            errors,
        };

        (status, Json(problem)).into_response()
    }
}

/// Convert `tabagent_values::BackendError` to `ApiError`.
///
/// This is the PRIMARY error mapping from backend to API layer.
/// It provides helpful, actionable error messages to clients.
impl From<tabagent_values::BackendError> for ApiError {
    fn from(err: tabagent_values::BackendError) -> Self {
        use tabagent_values::BackendError;

        match err {
            BackendError::ModelNotLoaded { model } => {
                ApiError::ServiceUnavailable(
                    format!(
                        "Model '{}' is not currently loaded. Load it with: POST /v1/models/load {{\"model_id\": \"{}\"}}",
                        model, model
                    )
                )
            }
            BackendError::ModelNotFound { model } => {
                ApiError::NotFound(
                    format!(
                        "Model '{}' not found in registry. View available models at: GET /v1/models",
                        model
                    )
                )
            }
            BackendError::OutOfMemory { required_mb, available_mb } => {
                ApiError::ServiceUnavailable(
                    format!(
                        "Insufficient memory: model requires {}MB but only {}MB available. Try unloading other models with: POST /v1/models/unload",
                        required_mb, available_mb
                    )
                )
            }
            BackendError::GenerationTimeout { timeout_seconds } => {
                ApiError::Timeout(
                    format!(
                        "Generation exceeded {} second timeout. Try using: POST /v1/generation/stop to cancel stuck generations",
                        timeout_seconds
                    )
                )
            }
            BackendError::InvalidInput { field, reason } => {
                ApiError::ValidationError {
                    field,
                    message: reason,
                    request_id: Some(uuid::Uuid::new_v4().to_string()),
                }
            }
            BackendError::CudaError { code, message } => {
                ApiError::InternalError(
                    format!(
                        "GPU error (CUDA code {}): {}. Check that CUDA is properly installed and GPU is available",
                        code, message
                    )
                )
            }
            BackendError::ModelCorrupted { model, reason } => {
                ApiError::UnprocessableEntity(
                    format!(
                        "Model '{}' file is corrupted: {}. Try re-downloading with: POST /v1/pull {{\"model\": \"{}\"}}",
                        model, reason, model
                    )
                )
            }
            BackendError::ResourceLimitExceeded { resource, limit, current } => {
                ApiError::RateLimitExceeded(
                    format!(
                        "Resource '{}' limit exceeded: {}/{} used. Wait a moment and try again",
                        resource, current, limit
                    )
                )
            }
            BackendError::SessionNotFound { session_id } => {
                ApiError::NotFound(
                    format!(
                        "Session '{}' not found. Create a new session by sending a message to: POST /v1/sessions/<session_id>/messages",
                        session_id
                    )
                )
            }
            BackendError::EmbeddingModelNotAvailable { required_for } => {
                ApiError::ServiceUnavailable(
                    format!(
                        "Embedding model required for '{}' but not available. Load an embedding model with: GET /v1/embedding-models then POST /v1/models/load",
                        required_for
                    )
                )
            }
            BackendError::VectorStoreError { operation, reason } => {
                ApiError::InternalError(
                    format!(
                        "Vector store error during '{}': {}. The vector database may need to be rebuilt",
                        operation, reason
                    )
                )
            }
            BackendError::InternalError { message, context } => {
                if let Some(ctx) = context {
                    ApiError::InternalError(format!("{} (context: {})", message, ctx))
                } else {
                    ApiError::InternalError(message)
                }
            }
            BackendError::ConfigurationError { setting, reason } => {
                ApiError::InternalError(
                    format!(
                        "Configuration error for '{}': {}. Check your server configuration",
                        setting, reason
                    )
                )
            }
            BackendError::NotImplemented { feature } => {
                ApiError::InternalError(
                    format!(
                        "Feature '{}' is not yet implemented. Check the API documentation for supported features",
                        feature
                    )
                )
            }
        }
    }
}

/// Convert `anyhow::Error` to `ApiError`.
///
/// This is a FALLBACK for backends that haven't migrated to `BackendError` yet.
/// Prefer using `BackendError` directly for better error messages.
impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        // Try to downcast to BackendError first
        if let Some(backend_err) = err.downcast_ref::<tabagent_values::BackendError>() {
            return ApiError::from(backend_err.clone());
        }

        // Fallback: pattern match error message (less precise)
        let err_str = err.to_string();

        if err_str.contains("not found") || err_str.contains("Not Found") {
            ApiError::NotFound(err_str)
        } else if err_str.contains("not loaded") || err_str.contains("Not Loaded") {
            ApiError::ServiceUnavailable(err_str)
        } else if err_str.contains("timeout") || err_str.contains("Timeout") {
            ApiError::Timeout(err_str)
        } else if err_str.contains("invalid") || err_str.contains("Invalid") {
            ApiError::BadRequest(err_str)
        } else {
            // Default to internal error for unknown errors
            ApiError::InternalError(err_str)
        }
    }
}

