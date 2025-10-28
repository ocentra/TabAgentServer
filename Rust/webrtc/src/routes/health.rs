//! Health check endpoint for WebRTC data channels.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tabagent_values::{RequestValue, HealthStatus};
use crate::{
    error::WebRtcResult,
    route_trait::{DataChannelRoute, RouteMetadata, TestCase},
    traits::RequestHandler,
};

/// Health check request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthRequest;

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// System health status
    pub status: String,
    /// Timestamp of health check
    pub timestamp: String,
}

/// Health check route handler
pub struct HealthRoute;

#[async_trait]
impl DataChannelRoute for HealthRoute {
    type Request = HealthRequest;
    type Response = HealthResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "health",
            tags: &["System", "Health"],
            description: "Health check endpoint for WebRTC data channel connectivity and system status",
            supports_streaming: false,
            supports_binary: false,
            requires_auth: false,
            rate_limit_tier: None,
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
        
        tracing::info!(
            request_id = %request_id,
            route = "health",
            "WebRTC health check"
        );

        let request_value = RequestValue::system_info();
        let response = handler.handle_request(request_value).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Health check failed");
                crate::error::WebRtcError::from(e)
            })?;

        let health_response = if let Some((status, timestamp)) = response.as_health() {
            HealthResponse {
                status: format!("{:?}", status),
                timestamp: timestamp.to_rfc3339(),
            }
        } else {
            HealthResponse {
                status: format!("{:?}", HealthStatus::Unhealthy),
                timestamp: chrono::Utc::now().to_rfc3339(),
            }
        };

        tracing::info!(request_id = %request_id, status = %health_response.status, "Health check successful");

        Ok(health_response)
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::success(
                "health_check",
                HealthRequest,
                HealthResponse {
                    status: "Healthy".to_string(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
            ),
        ]
    }
}

crate::enforce_data_channel_route!(HealthRoute);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata() {
        let metadata = HealthRoute::metadata();
        assert_eq!(metadata.route_id, "health");
        assert!(!metadata.description.is_empty());
    }
}

