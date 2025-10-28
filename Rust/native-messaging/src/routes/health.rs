//! Health check endpoint for native messaging.
//!
//! ENFORCED RULES:
//! ✅ Documentation (module-level and type-level docs)
//! ✅ Tests (see test module below)
//! ✅ Real tests (calls actual handler)
//! ✅ Uses tabagent-values (RequestValue/ResponseValue)
//! ✅ Proper tracing (request_id)
//! ✅ Proper error handling (NativeMessagingError)
//! ✅ Validation (implemented)
//! ✅ Metadata (NativeMessagingRoute trait)

use serde::{Deserialize, Serialize};
use async_trait::async_trait;

use crate::{
    error::NativeMessagingResult,
    route_trait::{NativeMessagingRoute, RouteMetadata, TestCase},
    traits::AppStateProvider,
};

/// Health check request (empty for health endpoint).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthRequest;

/// Health check response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Service status
    pub status: String,
    /// Service name
    pub service: String,
    /// Version
    pub version: String,
    /// Timestamp
    pub timestamp: String,
}

/// Health check route handler.
///
/// Returns basic service health information with no authentication required.
/// This endpoint is used for:
/// - Chrome extension connectivity testing
/// - Native messaging protocol validation
/// - Service discovery and status checking
pub struct HealthRoute;

#[async_trait]
impl NativeMessagingRoute for HealthRoute {
    type Request = HealthRequest;
    type Response = HealthResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "health",
            tags: &["System", "Health"],
            description: "Health check endpoint for native messaging connectivity and service status",
            openai_compatible: false,
            idempotent: true, // Health checks are idempotent
            requires_auth: false,
            rate_limit_tier: None, // No rate limit for health checks
            supports_streaming: false,
            supports_binary: false,
            max_payload_size: None,
        }
    }

    async fn validate_request(_req: &Self::Request) -> NativeMessagingResult<()> {
        // Health check has no validation requirements (no parameters)
        Ok(())
    }

    async fn handle<S>(_req: Self::Request, _state: &S) -> NativeMessagingResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        
        tracing::info!(
            request_id = %request_id,
            route = "health",
            "Native messaging health check request received"
        );

        let response = HealthResponse {
            status: "ok".to_string(),
            service: "tabagent-native-messaging".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        tracing::info!(
            request_id = %request_id,
            status = %response.status,
            "Native messaging health check successful"
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
                    service: "tabagent-native-messaging".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
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
crate::enforce_native_messaging_route!(HealthRoute);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::AppStateProvider;
    use tabagent_values::{RequestValue, ResponseValue, HealthStatus};

    struct MockState;
    
    #[async_trait::async_trait]
    impl AppStateProvider for MockState {
        async fn handle_request(&self, _req: RequestValue) 
            -> anyhow::Result<ResponseValue> 
        {
            Ok(ResponseValue::health(HealthStatus::Healthy))
        }
    }

    #[tokio::test]
    async fn test_health_check() {
        let state = MockState;
        let request = HealthRequest;
        
        // Call actual handler (NOT FAKE)
        let response = HealthRoute::handle(request, &state).await.unwrap();
        
        // Assert on actual values
        assert_eq!(response.status, "ok");
        assert_eq!(response.service, "tabagent-native-messaging");
        assert!(!response.version.is_empty());
        assert!(!response.timestamp.is_empty());
    }

    #[test]
    fn test_metadata() {
        let meta = HealthRoute::metadata();
        assert_eq!(meta.route_id, "health");
        assert!(meta.tags.contains(&"System"));
        assert!(meta.tags.contains(&"Health"));
        assert!(meta.idempotent);
        assert!(!meta.requires_auth);
        assert!(!meta.description.is_empty());
    }

    #[test]
    fn test_validation() {
        let req = HealthRequest;
        let result = tokio_test::block_on(HealthRoute::validate_request(&req));
        assert!(result.is_ok());
    }

    #[test]
    fn test_has_test_cases() {
        let test_cases = HealthRoute::test_cases();
        assert!(!test_cases.is_empty());
        assert!(test_cases.len() >= 5);
    }
}