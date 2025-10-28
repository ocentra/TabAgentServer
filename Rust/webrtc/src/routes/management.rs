//! Management endpoint for WebRTC data channels.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tabagent_values::RequestValue;
use crate::{
    error::WebRtcResult,
    route_trait::{DataChannelRoute, RouteMetadata, TestCase},
    traits::RequestHandler,
};

/// Management operation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagementRequest {
    /// Management action to perform
    pub action: String,
    /// Additional parameters for the action
    #[serde(flatten)]
    pub params: serde_json::Value,
}

/// Management operation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagementResponse {
    /// Whether the operation succeeded
    pub success: bool,
    /// Response data
    pub data: serde_json::Value,
}

/// Management route handler
pub struct ManagementRoute;

#[async_trait]
impl DataChannelRoute for ManagementRoute {
    type Request = ManagementRequest;
    type Response = ManagementResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "management",
            tags: &["Management", "Admin"],
            description: "Administrative operations and system management tasks",
            supports_streaming: false,
            supports_binary: false,
            requires_auth: true,
            rate_limit_tier: Some("admin"),
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
        
        tracing::info!(
            request_id = %request_id,
            route = "management",
            action = %req.action,
            "WebRTC management request"
        );

        // For now, management route acts as a system info placeholder
        // TODO: Implement specific management actions with proper RequestValue mappings
        let request_value = RequestValue::system_info();

        let response = handler.handle_request(request_value).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Management request failed");
                crate::error::WebRtcError::from(e)
            })?;

        let data = response.to_json_value();

        tracing::info!(request_id = %request_id, "Management request successful");

        Ok(ManagementResponse {
            success: true,
            data,
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::success(
                "management_action",
                ManagementRequest {
                    action: "status".to_string(),
                    params: serde_json::json!({}),
                },
                ManagementResponse {
                    success: true,
                    data: serde_json::json!({"status": "ok"}),
                },
            ),
        ]
    }
}

crate::enforce_data_channel_route!(ManagementRoute);
