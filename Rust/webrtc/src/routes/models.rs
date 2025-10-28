//! Models endpoint for WebRTC data channels.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tabagent_values::RequestValue;
use crate::{
    error::{WebRtcResult, WebRtcError},
    route_trait::{DataChannelRoute, RouteMetadata, TestCase},
    traits::RequestHandler,
};

/// Model management request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ModelsRequest {
    /// List available models
    List,
    /// Load a model
    Load { 
        /// Model identifier to load
        model_id: String 
    },
    /// Unload a model
    Unload { 
        /// Model identifier to unload
        model_id: String 
    },
    /// Get model information
    Info { 
        /// Model identifier
        model_id: String 
    },
}

/// Model management response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsResponse {
    /// Whether the operation succeeded
    pub success: bool,
    /// Optional status message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Response data (models list or model info)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Model management route handler
pub struct ModelsRoute;

#[async_trait]
impl DataChannelRoute for ModelsRoute {
    type Request = ModelsRequest;
    type Response = ModelsResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "models",
            tags: &["Models", "Management"],
            description: "Manage AI models - list, load, unload, and get model information",
            supports_streaming: false,
            supports_binary: false,
            requires_auth: true,
            rate_limit_tier: Some("standard"),
            max_payload_size: None,
            media_type: None,
        }
    }

    async fn validate_request(req: &Self::Request) -> WebRtcResult<()> {
        match req {
            ModelsRequest::Load { model_id } | ModelsRequest::Unload { model_id } | ModelsRequest::Info { model_id } => {
                if model_id.is_empty() {
                    return Err(WebRtcError::ValidationError {
                        field: "model_id".to_string(),
                        message: "model_id cannot be empty".to_string(),
                    });
                }
            }
            ModelsRequest::List => {}
        }
        Ok(())
    }

    async fn handle<H>(req: Self::Request, handler: &H) -> WebRtcResult<Self::Response>
    where
        H: RequestHandler + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        
        tracing::info!(request_id = %request_id, route = "models", action = ?req, "WebRTC models request");

        let request_value = match &req {
            ModelsRequest::List => RequestValue::list_models(),
            ModelsRequest::Load { model_id } => RequestValue::load_model(model_id.clone(), None),
            ModelsRequest::Unload { model_id } => RequestValue::unload_model(model_id.clone()),
            ModelsRequest::Info { model_id } => RequestValue::model_info(model_id.clone()),
        };

        let response = handler.handle_request(request_value).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Models request failed");
                WebRtcError::from(e)
            })?;

        let response_json = response.to_json_value();

        tracing::info!(request_id = %request_id, "Models request successful");

        Ok(ModelsResponse {
            success: true,
            message: Some("Operation completed".to_string()),
            data: Some(response_json),
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::success(
                "list_models",
                ModelsRequest::List,
                ModelsResponse {
                    success: true,
                    message: Some("Models listed".to_string()),
                    data: None,
                },
            ),
            TestCase::error(
                "load_empty_id",
                ModelsRequest::Load { model_id: "".to_string() },
                "model_id cannot be empty",
            ),
        ]
    }
}

crate::enforce_data_channel_route!(ModelsRoute);
