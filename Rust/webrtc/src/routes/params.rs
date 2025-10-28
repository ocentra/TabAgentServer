//! Parameters management endpoints for WebRTC data channels.
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

// ==================== GET PARAMS ====================

/// Get parameters request (no parameters).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetParamsRequest;

/// Get parameters response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetParamsResponse {
    /// Current parameter values
    pub params: serde_json::Value,
}

/// Get parameters route handler.
pub struct GetParamsRoute;

#[async_trait]
impl DataChannelRoute for GetParamsRoute {
    type Request = GetParamsRequest;
    type Response = GetParamsResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "get_params",
            tags: &["Configuration", "Parameters"],
            description: "Get current system parameters and configuration settings",
            supports_streaming: false,
            supports_binary: false,
            requires_auth: true,
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
        tracing::info!(request_id = %request_id, route = "get_params", "WebRTC get params request");

        let request_value = RequestValue::get_params();

        let response = handler.handle_request(request_value).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Get params request failed");
                WebRtcError::from(e)
            })?;

        let params = response.to_json_value();

        tracing::info!(request_id = %request_id, "Get params successful");

        Ok(GetParamsResponse { params })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::success(
                "get_params",
                GetParamsRequest,
                GetParamsResponse {
                    params: serde_json::json!({"param1": "value1"}),
                },
            ),
        ]
    }
}

crate::enforce_data_channel_route!(GetParamsRoute);

// ==================== SET PARAMS ====================

/// Set parameters request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetParamsRequest {
    /// Parameter values to set
    pub params: serde_json::Value,
}

/// Set parameters response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetParamsResponse {
    /// Whether the operation succeeded
    pub success: bool,
    /// Updated parameter values
    pub params: serde_json::Value,
}

/// Set parameters route handler.
pub struct SetParamsRoute;

#[async_trait]
impl DataChannelRoute for SetParamsRoute {
    type Request = SetParamsRequest;
    type Response = SetParamsResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "set_params",
            tags: &["Configuration", "Parameters"],
            description: "Update system parameters and configuration settings",
            supports_streaming: false,
            supports_binary: false,
            requires_auth: true,
            rate_limit_tier: Some("management"),
            max_payload_size: Some(64 * 1024), // 64KB
            media_type: None,
        }
    }

    async fn validate_request(req: &Self::Request) -> WebRtcResult<()> {
        if !req.params.is_object() {
            return Err(WebRtcError::ValidationError {
                field: "params".to_string(),
                message: "params must be a JSON object".to_string(),
            });
        }
        Ok(())
    }

    async fn handle<H>(req: Self::Request, handler: &H) -> WebRtcResult<Self::Response>
    where
        H: RequestHandler + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(request_id = %request_id, route = "set_params", "WebRTC set params request");

        let request_value = RequestValue::set_params(req.params.clone());

        let response = handler.handle_request(request_value).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Set params request failed");
                WebRtcError::from(e)
            })?;

        let params = response.to_json_value();

        tracing::info!(request_id = %request_id, "Set params successful");

        Ok(SetParamsResponse {
            success: true,
            params,
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::success(
                "set_params",
                SetParamsRequest {
                    params: serde_json::json!({"temperature": 0.7}),
                },
                SetParamsResponse {
                    success: true,
                    params: serde_json::json!({"temperature": 0.7}),
                },
            ),
            TestCase::error(
                "set_params_invalid_type",
                SetParamsRequest {
                    params: serde_json::json!("not an object"),
                },
                "params must be a JSON object",
            ),
        ]
    }
}

crate::enforce_data_channel_route!(SetParamsRoute);
