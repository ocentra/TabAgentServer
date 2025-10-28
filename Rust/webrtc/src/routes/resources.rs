//! Resources endpoint for WebRTC data channels.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tabagent_values::RequestValue;
use crate::{
    error::WebRtcResult,
    route_trait::{DataChannelRoute, RouteMetadata, TestCase},
    traits::RequestHandler,
};

/// Resource usage request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesRequest;

/// Resource usage response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesResponse {
    /// CPU usage percentage
    pub cpu_usage: f32,
    /// Memory currently used (bytes)
    pub memory_used: u64,
    /// Total available memory (bytes)
    pub memory_total: u64,
    /// Optional GPU usage percentage
    pub gpu_usage: Option<f32>,
}

/// Resource monitoring route handler
pub struct ResourcesRoute;

#[async_trait]
impl DataChannelRoute for ResourcesRoute {
    type Request = ResourcesRequest;
    type Response = ResourcesResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "resources",
            tags: &["System", "Resources", "Monitoring"],
            description: "Get current system resource usage including CPU, memory, and GPU utilization",
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

    async fn handle<H>(_req: Self::Request, handler: &H) -> WebRtcResult<Self::Response>
    where
        H: RequestHandler + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        
        tracing::info!(request_id = %request_id, route = "resources", "WebRTC resources request");

        let request_value = RequestValue::get_resources();
        let response = handler.handle_request(request_value).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Resources request failed");
                crate::error::WebRtcError::from(e)
            })?;

        let response_json = response.to_json_value();

        // Extract resource data from response
        let cpu_usage = response_json.get("cpu_usage")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;
        
        let memory_used = response_json.get("memory_used")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        
        let memory_total = response_json.get("memory_total")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        
        let gpu_usage = response_json.get("gpu_usage")
            .and_then(|v| v.as_f64())
            .map(|v| v as f32);

        tracing::info!(
            request_id = %request_id,
            cpu_usage = cpu_usage,
            memory_used = memory_used,
            "Resources request successful"
        );

        Ok(ResourcesResponse {
            cpu_usage,
            memory_used,
            memory_total,
            gpu_usage,
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::success(
                "get_resources",
                ResourcesRequest,
                ResourcesResponse {
                    cpu_usage: 45.5,
                    memory_used: 8_000_000_000,
                    memory_total: 16_000_000_000,
                    gpu_usage: Some(30.2),
                },
            ),
        ]
    }
}

crate::enforce_data_channel_route!(ResourcesRoute);
