//! Params endpoint for WebRTC data channels.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tabagent_values::RequestValue;
use crate::{
    error::WebRtcResult,
    route_trait::{DataChannelRoute, RouteMetadata, TestCase},
    traits::RequestHandler,
};

/// Generation parameters request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ParamsRequest {
    /// Get current parameters
    Get,
    /// Set new parameters
    Set { 
        /// Parameter values to set
        params: serde_json::Value 
    },
}

/// Generation parameters response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamsResponse {
    /// Whether the operation succeeded
    pub success: bool,
    /// Current parameter values
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

/// Parameters management route handler
pub struct ParamsRoute;

#[async_trait]
impl DataChannelRoute for ParamsRoute {
    type Request = ParamsRequest;
    type Response = ParamsResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "params",
            tags: &["Configuration", "Parameters"],
            description: "Get and set system parameters and configuration settings",
            supports_streaming: false,
            supports_binary: false,
            requires_auth: true,
            rate_limit_tier: Some("standard"),
            max_payload_size: None,
            media_type: None,
        }
    }

    async fn validate_request(_req: &Self::Request) -> WebRtcResult<()> {
        Ok(())
    }

    async fn handle<H>(req: Self::Request, handler: &H) -> WebRtcResult<Self::Response>
    where
        H: RequestHandler + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        
        tracing::info!(request_id = %request_id, route = "params", action = ?req, "WebRTC params request");

        let request_value = match req {
            ParamsRequest::Get => RequestValue::get_params(),
            ParamsRequest::Set { params } => RequestValue::set_params(params),
        };

        let response = handler.handle_request(request_value).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Params request failed");
                crate::error::WebRtcError::from(e)
            })?;

        let params_data = Some(response.to_json_value());

        tracing::info!(request_id = %request_id, "Params request successful");

        Ok(ParamsResponse {
            success: true,
            params: params_data,
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::success(
                "get_params",
                ParamsRequest::Get,
                ParamsResponse {
                    success: true,
                    params: Some(serde_json::json!({"param1": "value1"})),
                },
            ),
        ]
    }
}

crate::enforce_data_channel_route!(ParamsRoute);
