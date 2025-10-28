//! System info endpoint for WebRTC data channels.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tabagent_values::{RequestValue, response::{SystemInfo, CpuInfo, MemoryInfo}};
use crate::{
    error::WebRtcResult,
    route_trait::{DataChannelRoute, RouteMetadata, TestCase},
    traits::RequestHandler,
};

/// System information request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemRequest;

/// System information response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResponse {
    /// System information (CPU, memory, GPU)
    pub system: SystemInfo,
}

/// System information route handler
pub struct SystemRoute;

#[async_trait]
impl DataChannelRoute for SystemRoute {
    type Request = SystemRequest;
    type Response = SystemResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "system",
            tags: &["System", "Info"],
            description: "Get system information including OS, architecture, CPU, and memory details",
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
        
        tracing::info!(request_id = %request_id, route = "system", "WebRTC system info request");

        let request_value = RequestValue::system_info();
        let response = handler.handle_request(request_value).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "System info request failed");
                crate::error::WebRtcError::from(e)
            })?;

        let system_info = response.as_system_info()
            .ok_or_else(|| crate::error::WebRtcError::InternalError("Invalid response".to_string()))?
            .clone();

        tracing::info!(request_id = %request_id, "System info request successful");

        Ok(SystemResponse {
            system: system_info,
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::success(
                "system_info",
                SystemRequest,
                SystemResponse {
                    system: SystemInfo {
                        cpu: CpuInfo {
                            model: "Intel Core".to_string(),
                            cores: 8,
                            threads: 16,
                        },
                        memory: MemoryInfo {
                            total_bytes: 16_000_000_000,
                            available_bytes: 8_000_000_000,
                        },
                        gpu: None,
                    },
                },
            ),
        ]
    }
}

crate::enforce_data_channel_route!(SystemRoute);
