//! Route discovery endpoint.
//!
//! Allows clients to query available API routes dynamically.

use axum::http::Method;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use async_trait::async_trait;

use crate::{
    error::ApiResult,
    route_trait::{RouteHandler, RouteMetadata as InternalRouteMetadata, TestCase},
    traits::AppStateProvider,
};

/// Route discovery request (no parameters).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DiscoveryRequest;

/// Route information exposed to clients.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RouteInfo {
    /// HTTP path
    pub path: String,
    /// HTTP method
    pub method: String,
    /// Route tags for grouping
    pub tags: Vec<String>,
    /// Description
    pub description: String,
    /// OpenAI compatibility flag
    pub openai_compatible: bool,
    /// Idempotency flag
    pub idempotent: bool,
    /// Authentication requirement
    pub requires_auth: bool,
}

/// Route discovery response.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DiscoveryResponse {
    /// List of available routes
    pub routes: Vec<RouteInfo>,
    /// Total route count
    pub total: usize,
}

/// Route discovery route handler.
pub struct DiscoveryRoute;

#[async_trait]
impl RouteHandler for DiscoveryRoute {
    type Request = DiscoveryRequest;
    type Response = DiscoveryResponse;

    fn metadata() -> InternalRouteMetadata {
        InternalRouteMetadata {
            path: "/v1/routes",
            method: Method::GET,
            tags: &["System", "Discovery"],
            description: "List all available API routes for service discovery",
            openai_compatible: false,
            idempotent: true,
            requires_auth: false,
            rate_limit_tier: None,
        }
    }

    async fn validate_request(_req: &Self::Request) -> ApiResult<()> {
        Ok(()) // No validation needed
    }

    async fn handle<S: AppStateProvider>(
        _req: Self::Request,
        _state: &S,
    ) -> ApiResult<Self::Response> {
        let all_routes = crate::routes::list_available_routes();
        
        let routes = all_routes
            .into_iter()
            .map(|r| RouteInfo {
                path: r.path.to_string(),
                method: r.method.to_string(),
                tags: r.tags.iter().map(|s| s.to_string()).collect(),
                description: r.description.to_string(),
                openai_compatible: r.openai_compatible,
                idempotent: r.idempotent,
                requires_auth: r.requires_auth,
            })
            .collect::<Vec<_>>();

        let total = routes.len();

        Ok(DiscoveryResponse { routes, total })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase {
                name: "list_routes",
                request: DiscoveryRequest,
                expected_response: None, // Response will vary
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

// Implement RegisterableRoute to wire route into Axum router
crate::impl_registerable_route!(DiscoveryRoute);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::route_trait::RouteHandler;

    #[tokio::test]
    async fn test_discovery_route() {
        let req = DiscoveryRequest;
        struct MockState;
        
        #[async_trait]
        impl AppStateProvider for MockState {
            async fn handle_request(&self, _req: tabagent_values::RequestValue) -> anyhow::Result<tabagent_values::ResponseValue> {
                Ok(tabagent_values::ResponseValue::health(tabagent_values::HealthStatus::Healthy))
            }
        }
        
        let state = MockState;
        let response = DiscoveryRoute::handle(req, &state).await.unwrap();
        
        assert!(response.total > 0, "Should have routes");
        assert_eq!(response.routes.len(), response.total);
        assert!(response.routes.iter().any(|r| r.path == "/health"));
    }
}

