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

// ==================== LIST MODELS ====================

/// List models request (no parameters).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListModelsRequest;

/// Model information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Model identifier
    pub id: String,
    /// Object type (always "model")
    pub object: String,
    /// Creation timestamp
    pub created: i64,
    /// Owner organization
    pub owned_by: String,
}

/// Model list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListModelsResponse {
    /// List of models
    pub models: Vec<ModelInfo>,
}

/// List models route handler.
pub struct ListModelsRoute;

#[async_trait]
impl DataChannelRoute for ListModelsRoute {
    type Request = ListModelsRequest;
    type Response = ListModelsResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "models",
            tags: &["Models", "Management"],
            description: "List all loaded models",
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
        tracing::info!(request_id = %request_id, route = "models", "WebRTC list models request");

        let request_value = RequestValue::list_models();

        let response = handler.handle_request(request_value).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "List models request failed");
                WebRtcError::from(e)
            })?;

        let models = response.as_model_list()
            .ok_or_else(|| WebRtcError::InternalError("Invalid response type".to_string()))?;

        tracing::info!(request_id = %request_id, model_count = models.len(), "List models successful");

        Ok(ListModelsResponse {
            models: models.iter().map(|m| ModelInfo {
                id: m.id.clone(),
                object: "model".to_string(),
                created: 0,
                owned_by: "tabagent".to_string(),
            }).collect(),
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::success(
                "list_models",
                ListModelsRequest,
                ListModelsResponse {
                    models: vec![],
                },
            ),
        ]
    }
}

crate::enforce_data_channel_route!(ListModelsRoute);

// ==================== LOAD MODEL ====================

/// Load model request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadModelRequest {
    /// Model identifier to load
    pub model: String,
}

/// Load model response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadModelResponse {
    /// Whether the operation succeeded
    pub success: bool,
    /// Model identifier that was loaded
    pub model: String,
    /// Status message
    pub message: String,
}

/// Load model route handler.
pub struct LoadModelRoute;

#[async_trait]
impl DataChannelRoute for LoadModelRoute {
    type Request = LoadModelRequest;
    type Response = LoadModelResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "load_model",
            tags: &["Models", "Management"],
            description: "Load an AI model into memory",
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
        tracing::info!(request_id = %request_id, route = "load_model", model = %req.model, "WebRTC load model request");

        let request_value = RequestValue::load_model(req.model.clone(), None);

        handler.handle_request(request_value).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Load model request failed");
                WebRtcError::from(e)
            })?;

        tracing::info!(request_id = %request_id, "Load model successful");

        Ok(LoadModelResponse {
            success: true,
            model: req.model,
            message: "Model loaded successfully".to_string(),
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::success(
                "load_model",
                LoadModelRequest {
                    model: "test-model".to_string(),
                },
                LoadModelResponse {
                    success: true,
                    model: "test-model".to_string(),
                    message: "Model loaded successfully".to_string(),
                },
            ),
            TestCase::error(
                "load_empty_model",
                LoadModelRequest {
                    model: "".to_string(),
                },
                "model cannot be empty",
            ),
        ]
    }
}

crate::enforce_data_channel_route!(LoadModelRoute);

// ==================== UNLOAD MODEL ====================

/// Unload model request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnloadModelRequest {
    /// Model identifier to unload
    pub model: String,
}

/// Unload model response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnloadModelResponse {
    /// Whether the operation succeeded
    pub success: bool,
    /// Model identifier that was unloaded
    pub model: String,
    /// Status message
    pub message: String,
}

/// Unload model route handler.
pub struct UnloadModelRoute;

#[async_trait]
impl DataChannelRoute for UnloadModelRoute {
    type Request = UnloadModelRequest;
    type Response = UnloadModelResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "unload_model",
            tags: &["Models", "Management"],
            description: "Unload an AI model from memory",
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
        tracing::info!(request_id = %request_id, route = "unload_model", model = %req.model, "WebRTC unload model request");

        let request_value = RequestValue::unload_model(req.model.clone());

        handler.handle_request(request_value).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Unload model request failed");
                WebRtcError::from(e)
            })?;

        tracing::info!(request_id = %request_id, "Unload model successful");

        Ok(UnloadModelResponse {
            success: true,
            model: req.model,
            message: "Model unloaded successfully".to_string(),
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::success(
                "unload_model",
                UnloadModelRequest {
                    model: "test-model".to_string(),
                },
                UnloadModelResponse {
                    success: true,
                    model: "test-model".to_string(),
                    message: "Model unloaded successfully".to_string(),
                },
            ),
            TestCase::error(
                "unload_empty_model",
                UnloadModelRequest {
                    model: "".to_string(),
                },
                "model cannot be empty",
            ),
        ]
    }
}

crate::enforce_data_channel_route!(UnloadModelRoute);

// ==================== MODEL INFO ====================

/// Model info request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfoRequest {
    /// Model identifier
    pub model: String,
}

/// Model info response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfoResponse {
    /// Model identifier
    pub id: String,
    /// Object type (always "model")
    pub object: String,
    /// Creation timestamp
    pub created: i64,
    /// Owner organization
    pub owned_by: String,
    /// Whether model is loaded
    pub loaded: bool,
    /// Model size in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
    /// Number of parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<u64>,
}

/// Model info route handler.
pub struct ModelInfoRoute;

#[async_trait]
impl DataChannelRoute for ModelInfoRoute {
    type Request = ModelInfoRequest;
    type Response = ModelInfoResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "model_info",
            tags: &["Models", "Management"],
            description: "Get detailed information about a specific model",
            supports_streaming: false,
            supports_binary: false,
            requires_auth: false,
            rate_limit_tier: Some("standard"),
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
        tracing::info!(request_id = %request_id, route = "model_info", model = %req.model, "WebRTC model info request");

        let request_value = RequestValue::model_info(req.model.clone());

        let _response = handler.handle_request(request_value).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Model info request failed");
                WebRtcError::from(e)
            })?;

        // TODO: Parse response once backend implements as_model_info()
        tracing::info!(request_id = %request_id, "Model info successful");

        Ok(ModelInfoResponse {
            id: req.model.clone(),
            object: "model".to_string(),
            created: 0,
            owned_by: "tabagent".to_string(),
            loaded: true,
            size_bytes: None,
            parameters: None,
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::success(
                "model_info",
                ModelInfoRequest {
                    model: "test-model".to_string(),
                },
                ModelInfoResponse {
                    id: "test-model".to_string(),
                    object: "model".to_string(),
                    created: 0,
                    owned_by: "tabagent".to_string(),
                    loaded: true,
                    size_bytes: None,
                    parameters: None,
                },
            ),
            TestCase::error(
                "model_info_empty_model",
                ModelInfoRequest {
                    model: "".to_string(),
                },
                "model cannot be empty",
            ),
        ]
    }
}

crate::enforce_data_channel_route!(ModelInfoRoute);
