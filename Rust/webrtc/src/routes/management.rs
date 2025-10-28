//! Model management endpoints for WebRTC data channels.
//!
//! ENFORCED RULES:
//! ✅ Documentation ✅ Tests ✅ tabagent-values ✅ Tracing ✅ Validation

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tabagent_values::RequestValue;
use crate::{
    error::{WebRtcResult, WebRtcError},
    route_trait::{DataChannelRoute, RouteMetadata, TestCase},
    traits::RequestHandler,
};

// ==================== PULL MODEL ====================

/// Pull model request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullModelRequest {
    /// Model identifier
    pub model: String,
    /// Optional model variant (e.g., "4bit", "8bit")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
}

/// Pull model response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullModelResponse {
    /// Whether the operation succeeded
    pub success: bool,
    /// Model identifier
    pub model: String,
    /// Status message
    pub message: String,
    /// Optional download progress (0.0-1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<f32>,
}

/// Pull model route handler.
pub struct PullModelRoute;

#[async_trait]
impl DataChannelRoute for PullModelRoute {
    type Request = PullModelRequest;
    type Response = PullModelResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "pull_model",
            tags: &["Models", "Management"],
            description: "Download and install a model",
            supports_streaming: false,
            supports_binary: false,
            requires_auth: true,
            rate_limit_tier: Some("management"),
            max_payload_size: None,
            media_type: None,
        }
    }

    async fn validate_request(req: &Self::Request) -> WebRtcResult<()> {
        if req.model.is_empty() {
            return Err(WebRtcError::ValidationError {
                field: "model".to_string(),
                message: "model cannot be empty".to_string(),
            });
        }
        Ok(())
    }

    async fn handle<H>(req: Self::Request, handler: &H) -> WebRtcResult<Self::Response>
    where
        H: RequestHandler + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(
            request_id = %request_id,
            route = "pull_model",
            model = %req.model,
            "WebRTC pull model request"
        );

        let request_value = RequestValue::pull_model(&req.model, req.variant.clone());

        let response = handler.handle_request(request_value).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Pull model failed");
                WebRtcError::from(e)
            })?;

        let (success, progress) = response.as_pull_result()
            .ok_or_else(|| WebRtcError::InternalError("Invalid response type".to_string()))?;

        tracing::info!(request_id = %request_id, success = success, "Pull model completed");

        Ok(PullModelResponse {
            success: *success,
            model: req.model,
            message: if *success {
                "Model pulled successfully".to_string()
            } else {
                "Failed to pull model".to_string()
            },
            progress: *progress,
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::success(
                "pull_model",
                PullModelRequest {
                    model: "gpt-3.5-turbo".to_string(),
                    variant: Some("4bit".to_string()),
                },
                PullModelResponse {
                    success: true,
                    model: "gpt-3.5-turbo".to_string(),
                    message: "Model pulled successfully".to_string(),
                    progress: Some(1.0),
                },
            ),
            TestCase::error(
                "pull_empty_model",
                PullModelRequest {
                    model: "".to_string(),
                    variant: None,
                },
                "model cannot be empty",
            ),
        ]
    }
}

crate::enforce_data_channel_route!(PullModelRoute);

// ==================== DELETE MODEL ====================

/// Delete model request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteModelRequest {
    /// Model identifier
    pub model: String,
}

/// Delete model response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteModelResponse {
    /// Whether the operation succeeded
    pub success: bool,
    /// Model identifier
    pub model: String,
    /// Status message
    pub message: String,
}

/// Delete model route handler.
pub struct DeleteModelRoute;

#[async_trait]
impl DataChannelRoute for DeleteModelRoute {
    type Request = DeleteModelRequest;
    type Response = DeleteModelResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "delete_model",
            tags: &["Models", "Management"],
            description: "Delete a model from storage",
            supports_streaming: false,
            supports_binary: false,
            requires_auth: true,
            rate_limit_tier: Some("management"),
            max_payload_size: None,
            media_type: None,
        }
    }

    async fn validate_request(req: &Self::Request) -> WebRtcResult<()> {
        if req.model.is_empty() {
            return Err(WebRtcError::ValidationError {
                field: "model".to_string(),
                message: "model cannot be empty".to_string(),
            });
        }
        Ok(())
    }

    async fn handle<H>(req: Self::Request, handler: &H) -> WebRtcResult<Self::Response>
    where
        H: RequestHandler + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(
            request_id = %request_id,
            route = "delete_model",
            model = %req.model,
            "WebRTC delete model request"
        );

        let request_value = RequestValue::delete_model(&req.model);

        let response = handler.handle_request(request_value).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Delete model failed");
                WebRtcError::from(e)
            })?;

        let success = response.as_delete_result()
            .ok_or_else(|| WebRtcError::InternalError("Invalid response type".to_string()))?;

        tracing::info!(request_id = %request_id, success = success, "Delete model completed");

        Ok(DeleteModelResponse {
            success: *success,
            model: req.model,
            message: if *success {
                "Model deleted successfully".to_string()
            } else {
                "Failed to delete model".to_string()
            },
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::success(
                "delete_model",
                DeleteModelRequest {
                    model: "gpt-3.5-turbo".to_string(),
                },
                DeleteModelResponse {
                    success: true,
                    model: "gpt-3.5-turbo".to_string(),
                    message: "Model deleted successfully".to_string(),
                },
            ),
            TestCase::error(
                "delete_empty_model",
                DeleteModelRequest {
                    model: "".to_string(),
                },
                "model cannot be empty",
            ),
        ]
    }
}

crate::enforce_data_channel_route!(DeleteModelRoute);

// ==================== GET LOADED MODELS ====================

/// Get loaded models request (no parameters).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetLoadedModelsRequest;

/// Loaded model information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadedModelInfo {
    /// Model identifier
    pub id: String,
    /// Model name
    pub name: String,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Load time (Unix timestamp)
    pub load_time: i64,
}

/// Get loaded models response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetLoadedModelsResponse {
    /// List of loaded models
    pub models: Vec<LoadedModelInfo>,
}

/// Get loaded models route handler.
pub struct GetLoadedModelsRoute;

#[async_trait]
impl DataChannelRoute for GetLoadedModelsRoute {
    type Request = GetLoadedModelsRequest;
    type Response = GetLoadedModelsResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "get_loaded_models",
            tags: &["Models", "Management", "Status"],
            description: "Get list of currently loaded models with resource usage",
            supports_streaming: false,
            supports_binary: false,
            requires_auth: false,
            rate_limit_tier: Some("standard"),
            max_payload_size: None,
            media_type: None,
        }
    }

    async fn validate_request(_req: &Self::Request) -> WebRtcResult<()> {
        Ok(()) // No validation needed
    }

    async fn handle<H>(_req: Self::Request, handler: &H) -> WebRtcResult<Self::Response>
    where
        H: RequestHandler + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(request_id = %request_id, route = "get_loaded_models", "WebRTC get loaded models request");

        let request_value = RequestValue::get_loaded_models();

        let response = handler.handle_request(request_value).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Get loaded models failed");
                WebRtcError::from(e)
            })?;

        let loaded_models = response.as_loaded_models()
            .ok_or_else(|| WebRtcError::InternalError("Invalid response type".to_string()))?;

        let models = loaded_models.iter().map(|m| LoadedModelInfo {
            id: m.id.clone(),
            name: m.name.clone(),
            memory_usage: m.memory_usage,
            load_time: m.load_time,
        }).collect();

        tracing::info!(request_id = %request_id, model_count = loaded_models.len(), "Get loaded models successful");

        Ok(GetLoadedModelsResponse { models })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::success(
                "get_loaded_models",
                GetLoadedModelsRequest,
                GetLoadedModelsResponse {
                    models: vec![],
                },
            ),
        ]
    }
}

crate::enforce_data_channel_route!(GetLoadedModelsRoute);

// ==================== TIER 2 ROUTES (STUBS) ====================

// TODO: TIER 2 - Implement these routes

/// Select model request (TIER 2 - Not implemented).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectModelRequest {
    /// Model identifier
    pub model: String,
}

/// Select model response (TIER 2 - Not implemented).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectModelResponse {
    /// Whether the operation succeeded
    pub success: bool,
}

/// Select model route (TIER 2 stub).
#[allow(dead_code)]
pub struct SelectModelRoute;

/// Get embedding models request (TIER 2 - Not implemented).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetEmbeddingModelsRequest;

/// Get embedding models response (TIER 2 - Not implemented).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetEmbeddingModelsResponse {
    /// List of available embedding models
    pub models: Vec<String>,
}

/// Get embedding models route (TIER 2 stub).
#[allow(dead_code)]
pub struct GetEmbeddingModelsRoute;

/// Get recipes request (TIER 2 - Not implemented).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetRecipesRequest;

/// Get recipes response (TIER 2 - Not implemented).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetRecipesResponse {
    /// List of available recipes
    pub recipes: Vec<String>,
}

/// Get recipes route (TIER 2 stub).
#[allow(dead_code)]
pub struct GetRecipesRoute;

/// Get registered models request (TIER 2 - Not implemented).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetRegisteredModelsRequest;

/// Get registered models response (TIER 2 - Not implemented).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetRegisteredModelsResponse {
    /// List of registered models
    pub models: Vec<String>,
}

/// Get registered models route (TIER 2 stub).
#[allow(dead_code)]
pub struct GetRegisteredModelsRoute;
