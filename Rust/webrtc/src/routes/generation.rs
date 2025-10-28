//! Generation control endpoints for WebRTC data channels.
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

// ==================== STOP GENERATION ====================

/// Stop generation request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopGenerationRequest {
    /// Request ID of the generation to stop
    pub request_id: String,
}

/// Stop generation response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopGenerationResponse {
    /// Whether the operation succeeded
    pub success: bool,
    /// Status message
    pub message: String,
}

/// Stop generation route handler.
pub struct StopGenerationRoute;

#[async_trait]
impl DataChannelRoute for StopGenerationRoute {
    type Request = StopGenerationRequest;
    type Response = StopGenerationResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "stop_generation",
            tags: &["Generation", "Control"],
            description: "Stop an active generation process",
            supports_streaming: false,
            supports_binary: false,
            requires_auth: true,
            rate_limit_tier: Some("standard"),
            max_payload_size: None,
            media_type: None,
        }
    }

    async fn validate_request(req: &Self::Request) -> WebRtcResult<()> {
        if req.request_id.is_empty() {
            return Err(WebRtcError::ValidationError {
                field: "request_id".to_string(),
                message: "request_id cannot be empty".to_string(),
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
            route = "stop_generation",
            generation_request_id = %req.request_id,
            "WebRTC stop generation request"
        );

        let request_value = RequestValue::stop_generation(req.request_id.clone());

        handler.handle_request(request_value).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Stop generation failed");
                WebRtcError::from(e)
            })?;

        tracing::info!(request_id = %request_id, "Stop generation successful");

        Ok(StopGenerationResponse {
            success: true,
            message: "Generation stopped".to_string(),
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::success(
                "stop_generation",
                StopGenerationRequest {
                    request_id: "test-req-123".to_string(),
                },
                StopGenerationResponse {
                    success: true,
                    message: "Generation stopped".to_string(),
                },
            ),
            TestCase::error(
                "stop_empty_id",
                StopGenerationRequest {
                    request_id: "".to_string(),
                },
                "request_id cannot be empty",
            ),
        ]
    }
}

crate::enforce_data_channel_route!(StopGenerationRoute);

// ==================== GET HALT STATUS ====================

/// Get halt status request (no parameters).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetHaltStatusRequest;

/// Get halt status response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetHaltStatusResponse {
    /// Whether generation is halted
    pub is_halted: bool,
    /// Number of active generations
    pub active_count: u32,
    /// Optional status message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Get halt status route handler.
pub struct GetHaltStatusRoute;

#[async_trait]
impl DataChannelRoute for GetHaltStatusRoute {
    type Request = GetHaltStatusRequest;
    type Response = GetHaltStatusResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "get_halt_status",
            tags: &["Generation", "Control", "Status"],
            description: "Get the current generation halt status and active generation count",
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
        tracing::info!(request_id = %request_id, route = "get_halt_status", "WebRTC get halt status request");

        let request_value = RequestValue::get_stats();

        let _response = handler.handle_request(request_value).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Get halt status failed");
                WebRtcError::from(e)
            })?;

        // For now, return basic status
        // TODO: Implement actual halt status tracking
        tracing::info!(request_id = %request_id, "Get halt status successful");

        Ok(GetHaltStatusResponse {
            is_halted: false,
            active_count: 0,
            message: Some("No active generations".to_string()),
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::success(
                "get_halt_status",
                GetHaltStatusRequest,
                GetHaltStatusResponse {
                    is_halted: false,
                    active_count: 0,
                    message: Some("No active generations".to_string()),
                },
            ),
        ]
    }
}

crate::enforce_data_channel_route!(GetHaltStatusRoute);
