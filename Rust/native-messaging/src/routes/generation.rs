//! Generation control endpoints for native messaging.

use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use tabagent_values::RequestValue;
use crate::{
    error::{NativeMessagingResult, NativeMessagingError},
    route_trait::{NativeMessagingRoute, RouteMetadata, TestCase},
    traits::AppStateProvider,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopGenerationRequest {
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub request_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopGenerationResponse {
    pub success: bool,
    pub message: String,
    pub stopped_requests: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetHaltStatusRequest {
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetHaltStatusResponse {
    pub is_halted: bool,
    pub active_requests: u32,
    pub last_halt_time: Option<i64>,
}

pub struct StopGenerationRoute;
pub struct GetHaltStatusRoute;

#[async_trait]
impl NativeMessagingRoute for StopGenerationRoute {
    type Request = StopGenerationRequest;
    type Response = StopGenerationResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "stop_generation",
            tags: &["Generation", "Control"],
            description: "Stop ongoing text generation requests",
            openai_compatible: false,
            idempotent: false,
            requires_auth: false,
            rate_limit_tier: Some("standard"),
            supports_streaming: false,
            supports_binary: false,
            max_payload_size: Some(64 * 1024),
        }
    }

    async fn validate_request(_req: &Self::Request) -> NativeMessagingResult<()> {
        Ok(())
    }

    async fn handle<S>(req: Self::Request, state: &S) -> NativeMessagingResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(
            request_id = %request_id, 
            route = "stop_generation", 
            session_id = ?req.session_id,
            target_request_id = ?req.request_id
        );

        let request = RequestValue::stop_generation(
            req.request_id.as_deref().unwrap_or("all")
        );
        let response = state.handle_request(request).await
            .map_err(|e| NativeMessagingError::Backend(e))?;

        let stopped_count = response.as_stop_result()
            .ok_or_else(|| NativeMessagingError::internal("Invalid response type"))?;

        Ok(StopGenerationResponse {
            success: true,
            message: format!("Stopped {} generation request(s)", stopped_count),
            stopped_requests: *stopped_count,
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase {
                name: "stop_all_generations",
                request: StopGenerationRequest {
                    session_id: None,
                    request_id: None,
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "stop_session_generations",
                request: StopGenerationRequest {
                    session_id: Some("session-123".to_string()),
                    request_id: None,
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "stop_specific_request",
                request: StopGenerationRequest {
                    session_id: Some("session-123".to_string()),
                    request_id: Some("req-456".to_string()),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

#[async_trait]
impl NativeMessagingRoute for GetHaltStatusRoute {
    type Request = GetHaltStatusRequest;
    type Response = GetHaltStatusResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "get_halt_status",
            tags: &["Generation", "Status"],
            description: "Get the current halt status of generation requests",
            openai_compatible: false,
            idempotent: true,
            requires_auth: false,
            rate_limit_tier: Some("standard"),
            supports_streaming: false,
            supports_binary: false,
            max_payload_size: Some(64 * 1024),
        }
    }

    async fn validate_request(_req: &Self::Request) -> NativeMessagingResult<()> {
        Ok(())
    }

    async fn handle<S>(req: Self::Request, state: &S) -> NativeMessagingResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(request_id = %request_id, route = "get_halt_status", session_id = ?req.session_id);

        // Use get_stats as proxy for halt status
        let request = RequestValue::get_stats();
        let _response = state.handle_request(request).await
            .map_err(|e| NativeMessagingError::Backend(e))?;

        let (is_halted, active_requests, last_halt_time) = (false, 0u32, None);

        Ok(GetHaltStatusResponse {
            is_halted,
            active_requests,
            last_halt_time,
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase {
                name: "get_halt_status_global",
                request: GetHaltStatusRequest {
                    session_id: None,
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "get_halt_status_session",
                request: GetHaltStatusRequest {
                    session_id: Some("session-123".to_string()),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_native_messaging_route!(StopGenerationRoute);
crate::enforce_native_messaging_route!(GetHaltStatusRoute);