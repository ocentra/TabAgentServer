//! Stats endpoint for WebRTC data channels.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tabagent_values::RequestValue;
use crate::{
    error::WebRtcResult,
    route_trait::{DataChannelRoute, RouteMetadata, TestCase},
    traits::RequestHandler,
};

/// Performance statistics request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsRequest;

/// Performance statistics response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsResponse {
    /// Total number of requests processed
    pub total_requests: u64,
    /// Number of successful requests
    pub successful_requests: u64,
    /// Number of failed requests
    pub failed_requests: u64,
    /// Average response time in milliseconds
    pub average_response_time_ms: f64,
    /// System uptime in seconds
    pub uptime_seconds: u64,
}

/// Performance statistics route handler
pub struct StatsRoute;

#[async_trait]
impl DataChannelRoute for StatsRoute {
    type Request = StatsRequest;
    type Response = StatsResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "stats",
            tags: &["System", "Statistics", "Monitoring"],
            description: "Get system statistics including request counts, response times, and uptime metrics",
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
        
        tracing::info!(request_id = %request_id, route = "stats", "WebRTC stats request");

        let request_value = RequestValue::get_stats();
        let response = handler.handle_request(request_value).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Stats request failed");
                crate::error::WebRtcError::from(e)
            })?;

        let response_json = response.to_json_value();

        // Extract stats from response
        let total_requests = response_json.get("total_requests")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        
        let successful_requests = response_json.get("successful_requests")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        
        let failed_requests = response_json.get("failed_requests")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        
        let average_response_time_ms = response_json.get("average_response_time_ms")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        
        let uptime_seconds = response_json.get("uptime_seconds")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        tracing::info!(
            request_id = %request_id,
            total_requests = total_requests,
            uptime = uptime_seconds,
            "Stats request successful"
        );

        Ok(StatsResponse {
            total_requests,
            successful_requests,
            failed_requests,
            average_response_time_ms,
            uptime_seconds,
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::success(
                "get_stats",
                StatsRequest,
                StatsResponse {
                    total_requests: 1000,
                    successful_requests: 950,
                    failed_requests: 50,
                    average_response_time_ms: 125.5,
                    uptime_seconds: 86400,
                },
            ),
        ]
    }
}

crate::enforce_data_channel_route!(StatsRoute);
