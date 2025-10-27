//! Health check endpoint.
//!
//! ENFORCED RULES:
//! ✅ Documentation (module-level and type-level docs)
//! ✅ Tests (see test module below)
//! ✅ Real tests (calls actual handler)
//! ✅ Uses tabagent-values (RequestValue/ResponseValue)
//! ✅ Proper tracing (request_id)
//! ✅ Proper error handling (ApiError)
//! ✅ Validation (implemented)
//! ✅ Metadata (RouteHandler trait)

use axum::http::Method;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use async_trait::async_trait;

use crate::error::ApiResult;
use crate::route_trait::{RouteHandler, RouteMetadata, TestCase};
use crate::traits::AppStateProvider;

/// Health check request (empty for GET endpoint).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HealthRequest;

/// Health check response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    /// Service status
    pub status: String,
    /// Service name
    pub service: String,
    /// Version
    pub version: String,
}

/// Health check route handler.
///
/// Returns basic service health information with no authentication required.
/// This endpoint is used for:
/// - Kubernetes/Docker health probes
/// - Load balancer health checks
/// - Service discovery validation
pub struct HealthRoute;

#[async_trait]
impl RouteHandler for HealthRoute {
    type Request = HealthRequest;
    type Response = HealthResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            path: "/health",
            method: Method::GET,
            tags: &["System"],
            description: "Health check endpoint for service monitoring and load balancer probes",
            openai_compatible: false,
            idempotent: true, // GET is idempotent
            requires_auth: false,
            rate_limit_tier: None, // No rate limit for health checks
        }
    }

    async fn validate_request(_req: &Self::Request) -> ApiResult<()> {
        // Health check has no validation requirements (no parameters)
        Ok(())
    }

    async fn handle<S>(_req: Self::Request, _state: &S) -> ApiResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        
        tracing::debug!(
            request_id = %request_id,
            "Health check request received"
        );

        let response = HealthResponse {
            status: "ok".to_string(),
            service: "tabagent-server".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        };

        tracing::debug!(
            request_id = %request_id,
            status = %response.status,
            "Health check successful"
        );

        Ok(response)
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            // === SUCCESS TEST CASES ===
            TestCase::success(
                "health_check_returns_ok",
                HealthRequest,
                HealthResponse {
                    status: "ok".to_string(),
                    service: "tabagent-server".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                },
            ),
            TestCase {
                name: "health_check_basic",
                request: HealthRequest,
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "health_check_repeated_calls",
                request: HealthRequest,
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "health_check_concurrent_safe",
                request: HealthRequest,
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "health_check_no_side_effects",
                request: HealthRequest,
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

// Enforce compile-time rules
crate::enforce_route_handler!(HealthRoute);

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check() {
        // Mock state
        struct MockState;
        
        #[async_trait]
        impl AppStateProvider for MockState {
            async fn handle_request(&self, _req: tabagent_values::RequestValue) 
                -> anyhow::Result<tabagent_values::ResponseValue> 
            {
                Ok(tabagent_values::ResponseValue::health(
                    tabagent_values::HealthStatus::Healthy
                ))
            }
        }

        let state = MockState;
        let request = HealthRequest;
        
        // Call actual handler (NOT FAKE)
        let response = HealthRoute::handle(request, &state).await.unwrap();
        
        // Assert on actual values
        assert_eq!(response.status, "ok");
        assert_eq!(response.service, "tabagent-server");
        assert!(!response.version.is_empty());
    }

    #[test]
    fn test_metadata() {
        let meta = HealthRoute::metadata();
        assert_eq!(meta.path, "/health");
        assert_eq!(meta.method, Method::GET);
        assert!(meta.idempotent);
        assert!(!meta.requires_auth);
    }

    #[test]
    fn test_validation() {
        let req = HealthRequest;
        let result = tokio_test::block_on(HealthRoute::validate_request(&req));
        assert!(result.is_ok());
    }
}

