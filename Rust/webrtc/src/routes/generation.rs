//! Generation control endpoint for WebRTC data channels.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tabagent_values::RequestValue;
use crate::{
    error::{WebRtcResult, WebRtcError},
    route_trait::{DataChannelRoute, RouteMetadata, TestCase},
    traits::RequestHandler,
};

/// Generation control request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationRequest {
    /// Action to perform
    pub action: GenerationAction,
    /// Request ID for stop action
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

/// Actions for generation control
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GenerationAction {
    /// Stop an active generation
    Stop,
    /// Get generation status
    GetStatus,
}

/// Generation control response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationResponse {
    /// Whether the operation succeeded
    pub success: bool,
    /// Status message
    pub message: String,
}

/// Generation control route handler
pub struct GenerationRoute;

#[async_trait]
impl DataChannelRoute for GenerationRoute {
    type Request = GenerationRequest;
    type Response = GenerationResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "generation",
            tags: &["Generation", "Control"],
            description: "Control ongoing generation processes - stop active generations and check status",
            supports_streaming: false,
            supports_binary: false,
            requires_auth: true,
            rate_limit_tier: Some("standard"),
            max_payload_size: None,
            media_type: None,
        }
    }

    async fn validate_request(req: &Self::Request) -> WebRtcResult<()> {
        match req.action {
            GenerationAction::Stop => {
                if req.request_id.is_none() || req.request_id.as_ref().unwrap().is_empty() {
                    return Err(WebRtcError::ValidationError {
                        field: "request_id".to_string(),
                        message: "request_id is required for stop action".to_string(),
                    });
                }
            }
            GenerationAction::GetStatus => {}
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
            route = "generation",
            action = ?req.action,
            "WebRTC generation control request"
        );

        let request_value = match req.action {
            GenerationAction::Stop => {
                RequestValue::stop_generation(req.request_id.unwrap())
            }
            GenerationAction::GetStatus => {
                RequestValue::get_stats()
            }
        };

        let _response = handler.handle_request(request_value).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Generation control failed");
                WebRtcError::from(e)
            })?;

        tracing::info!(request_id = %request_id, "Generation control successful");

        Ok(GenerationResponse {
            success: true,
            message: "Operation completed".to_string(),
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::success(
                "get_status",
                GenerationRequest {
                    action: GenerationAction::GetStatus,
                    request_id: None,
                },
                GenerationResponse {
                    success: true,
                    message: "Status retrieved".to_string(),
                },
            ),
            TestCase::error(
                "stop_without_id",
                GenerationRequest {
                    action: GenerationAction::Stop,
                    request_id: None,
                },
                "request_id is required",
            ),
        ]
    }
}

crate::enforce_data_channel_route!(GenerationRoute);
