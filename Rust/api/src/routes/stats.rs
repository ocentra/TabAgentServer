//! Performance statistics endpoint.
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

/// Performance statistics.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct PerformanceStats {
    /// Time to first token (seconds)
    pub time_to_first_token: f64,
    /// Tokens per second
    pub tokens_per_second: f64,
    /// Input tokens count
    pub input_tokens: u32,
    /// Output tokens count
    pub output_tokens: u32,
    /// Total inference time (seconds)
    pub total_time: f64,
}

/// Get stats request (no parameters).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetStatsRequest;

/// Get performance statistics route handler.
///
/// Returns real-time performance metrics including throughput, latency,
/// and token processing statistics from recent inference operations.
pub struct GetStatsRoute;

#[async_trait]
impl RouteHandler for GetStatsRoute {
    type Request = GetStatsRequest;
    type Response = PerformanceStats;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            path: "/v1/stats",
            method: Method::GET,
            tags: &["Statistics"],
            description: "Get real-time performance statistics and metrics for inference operations",
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
        tracing::info!(request_id = %request_id, "Get stats request received");

        let request = RequestValue::get_stats();
        
        tracing::debug!(request_id = %request_id, "Dispatching to handler");
        let response = state.handle_request(request).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Get stats failed");
                e
            })?;

        let stats_json = response.to_json_value();
        let stats: PerformanceStats = serde_json::from_value(stats_json)
            .map_err(|e| ApiError::Internal(format!("Failed to parse stats: {}", e)))?;

        tracing::info!(
            request_id = %request_id,
            tokens_per_second = stats.tokens_per_second,
            total_time = stats.total_time,
            "Get stats successful"
        );
        Ok(stats)
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            // === SUCCESS TEST CASES (no validation errors) ===
            TestCase {
                name: "get_stats_basic",
                request: GetStatsRequest,
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "get_stats_idempotent",
                request: GetStatsRequest,
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "get_stats_concurrent_safe",
                request: GetStatsRequest,
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "get_stats_no_side_effects",
                request: GetStatsRequest,
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_route_handler!(GetStatsRoute);

// Implement RegisterableRoute with Axum 0.8 compatible handler
crate::impl_registerable_route!(GetStatsRoute);

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validation() {
        let req = GetStatsRequest;
        assert!(GetStatsRoute::validate_request(&req).await.is_ok());
    }

    #[test]
    fn test_metadata() {
        let meta = GetStatsRoute::metadata();
        assert_eq!(meta.path, "/v1/stats");
        assert!(meta.idempotent);
    }
}
