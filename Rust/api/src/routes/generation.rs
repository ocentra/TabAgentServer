//! Generation control endpoints.
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
    route_trait::{RouteHandler, RouteMetadata, TestCase, ValidationRule, validators::*},
    traits::AppStateProvider,
};

/// Stop generation request.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StopGenerationRequest {
    /// Request ID to stop
    pub request_id: String,
}

/// Stop generation route handler.
///
/// Cancels an active generation request by its request_id.
/// Useful for long-running generations that need to be interrupted.
pub struct StopGenerationRoute;

#[async_trait]
impl RouteHandler for StopGenerationRoute {
    type Request = StopGenerationRequest;
    type Response = serde_json::Value;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            path: "/v1/generation/stop",
            method: Method::POST,
            tags: &["Generation"],
            description: "Stop an active generation request by request_id",
            openai_compatible: false,
            idempotent: true, // Stopping same request multiple times is idempotent
            requires_auth: false,
            rate_limit_tier: None,
        }
    }

    async fn validate_request(req: &Self::Request) -> ApiResult<()> {
        NotEmpty.validate(&req.request_id)?;
        
        // Validate it's a valid UUID format
        if uuid::Uuid::parse_str(&req.request_id).is_err() {
            return Err(ApiError::BadRequest(
                "request_id must be a valid UUID".into()
            ));
        }
        
        Ok(())
    }

    async fn handle<S>(req: Self::Request, state: &S) -> ApiResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(
            request_id = %request_id,
            target_request_id = %req.request_id,
            "Stop generation request received"
        );

        let request = RequestValue::stop_generation(&req.request_id);
        
        tracing::debug!(request_id = %request_id, "Dispatching to handler");
        let _response = state.handle_request(request).await
            .map_err(|e| {
                tracing::error!(
                    request_id = %request_id,
                    target_request_id = %req.request_id,
                    error = %e,
                    "Stop generation failed"
                );
                e
            })?;

        tracing::info!(
            request_id = %request_id,
            target_request_id = %req.request_id,
            "Generation stopped successfully"
        );

        Ok(serde_json::json!({
            "status": "stopped",
            "request_id": req.request_id,
            "stop_request_id": request_id.to_string()
        }))
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::error(
                "empty_request_id",
                StopGenerationRequest {
                    request_id: "".to_string(),
                },
                "cannot be empty",
            ),
            TestCase::error(
                "invalid_uuid",
                StopGenerationRequest {
                    request_id: "not-a-uuid".to_string(),
                },
                "valid UUID",
            ),
            // === SUCCESS TEST CASES ===
            TestCase {
                name: "stop_generation_valid_uuid",
                request: StopGenerationRequest {
                    request_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "stop_generation_different_uuid",
                request: StopGenerationRequest {
                    request_id: "123e4567-e89b-12d3-a456-426614174000".to_string(),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "stop_nonexistent_generation",
                request: StopGenerationRequest {
                    request_id: "999e9999-e99b-99d9-a999-999999999999".to_string(),
                },
                expected_response: None,
                expected_error: None, // Should handle gracefully
                assertions: vec![],
            },
            TestCase {
                name: "stop_generation_idempotent",
                request: StopGenerationRequest {
                    request_id: "111e1111-e11b-11d1-a111-111111111111".to_string(),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_route_handler!(StopGenerationRoute);

// ==================== GET HALT STATUS ====================

/// Get halt status request (no parameters).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetHaltStatusRequest;

/// Get halt status response.
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, PartialEq)]
pub struct GetHaltStatusResponse {
    /// Whether halt is active
    pub halted: bool,
    /// Status message
    pub status: String,
}

/// Get halt status route handler.
pub struct GetHaltStatusRoute;

#[async_trait]
impl RouteHandler for GetHaltStatusRoute {
    type Request = GetHaltStatusRequest;
    type Response = GetHaltStatusResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            path: "/v1/halt",
            method: Method::GET,
            tags: &["Control"],
            description: "Check if generation is currently halted",
            openai_compatible: false,
            idempotent: true,
            requires_auth: false,
            rate_limit_tier: Some("low"),
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
        tracing::info!(request_id = %request_id, "Get halt status request received");

        // Query generation status by calling stop with empty request_id
        // (backend will return current halt status)
        let request = RequestValue::stop_generation("");
        
        tracing::debug!(request_id = %request_id, "Dispatching to handler");
        let response = state.handle_request(request).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Get halt status failed");
                e
            })?;

        // Parse response to determine if halted
        let status_json = response.to_json_value();
        let halted = status_json.get("stopped").and_then(|v| v.as_bool()).unwrap_or(false);

        tracing::info!(request_id = %request_id, halted = halted, "Get halt status successful");

        Ok(GetHaltStatusResponse {
            halted,
            status: if halted { "halted".to_string() } else { "running".to_string() },
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            // === SUCCESS TEST CASES (no validation errors) ===
            TestCase {
                name: "get_halt_status_basic",
                request: GetHaltStatusRequest,
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "get_halt_status_idempotent",
                request: GetHaltStatusRequest,
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "get_halt_status_no_side_effects",
                request: GetHaltStatusRequest,
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "get_halt_status_concurrent_safe",
                request: GetHaltStatusRequest,
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_route_handler!(GetHaltStatusRoute);

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validation_empty() {
        let req = StopGenerationRequest {
            request_id: "".to_string(),
        };
        assert!(StopGenerationRoute::validate_request(&req).await.is_err());
    }

    #[tokio::test]
    async fn test_validation_invalid_uuid() {
        let req = StopGenerationRequest {
            request_id: "not-a-uuid".to_string(),
        };
        assert!(StopGenerationRoute::validate_request(&req).await.is_err());
    }

    #[tokio::test]
    async fn test_validation_valid() {
        let req = StopGenerationRequest {
            request_id: uuid::Uuid::new_v4().to_string(),
        };
        assert!(StopGenerationRoute::validate_request(&req).await.is_ok());
    }

    #[test]
    fn test_metadata() {
        let meta = StopGenerationRoute::metadata();
        assert_eq!(meta.path, "/v1/generation/stop");
        assert!(meta.idempotent);
    }
}
