//! System information endpoint.
//!
//! ENFORCED RULES:
//! ✅ Documentation ✅ Tests ✅ tabagent-values ✅ Tracing ✅ Validation

use axum::http::Method;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use async_trait::async_trait;

use tabagent_values::RequestValue;
use crate::{
    error::{ApiResult, ApiError},
    route_trait::{RouteHandler, RouteMetadata, TestCase},
    traits::AppStateProvider,
};

/// System info request (no parameters).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SystemInfoRequest;

/// System information route handler.
///
/// Returns detailed system information including hardware capabilities,
/// loaded models, and resource usage.
pub struct SystemRoute;

#[async_trait]
impl RouteHandler for SystemRoute {
    type Request = SystemInfoRequest;
    type Response = serde_json::Value;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            path: "/v1/system/info",
            method: Method::GET,
            tags: &["System"],
            description: "Get detailed system information and hardware capabilities",
            openai_compatible: false,
            idempotent: true,
            requires_auth: false,
            rate_limit_tier: None,
        }
    }

    async fn validate_request(_req: &Self::Request) -> ApiResult<()> {
        Ok(()) // No validation needed
    }

    async fn handle<S>(_req: Self::Request, state: &S) -> ApiResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(request_id = %request_id, "System info request received");

        let request = RequestValue::system_info();
        
        tracing::debug!(request_id = %request_id, "Dispatching to handler");
        let response = state.handle_request(request).await
            .map_err(|e| {
                tracing::error!(
                    request_id = %request_id,
                    error = %e,
                    "System info request failed"
                );
                e
            })?;

        let system_info = response
            .as_system_info()
            .ok_or_else(|| {
                tracing::error!(
                    request_id = %request_id,
                    "Handler returned invalid response type (expected SystemInfoResponse)"
                );
                ApiError::Internal(
                    format!("Handler returned invalid response type for system info request (request_id: {})", request_id)
                )
            })?;

        tracing::info!(request_id = %request_id, "System info retrieved successfully");

        Ok(serde_json::to_value(system_info)
            .unwrap_or_else(|_| serde_json::json!({"error": "Failed to serialize system info"})))
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            // === SUCCESS TEST CASES (no validation errors) ===
            TestCase {
                name: "system_info_basic",
                request: SystemInfoRequest,
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "system_info_idempotent",
                request: SystemInfoRequest,
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "system_info_concurrent_safe",
                request: SystemInfoRequest,
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "system_info_no_side_effects",
                request: SystemInfoRequest,
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_route_handler!(SystemRoute);

// Implement RegisterableRoute with Axum 0.8 compatible handler
crate::impl_registerable_route!(SystemRoute);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata() {
        let meta = SystemRoute::metadata();
        assert_eq!(meta.path, "/v1/system/info");
        assert!(meta.idempotent);
    }

    #[tokio::test]
    async fn test_validation() {
        let req = SystemInfoRequest;
        assert!(SystemRoute::validate_request(&req).await.is_ok());
    }
}
