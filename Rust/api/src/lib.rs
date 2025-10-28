//! TabAgent API Crate
//!
//! Atomic, self-contained HTTP API layer using Axum for TabAgent Server.
//!
//! # Architecture
//!
//! This crate provides a complete REST API with:
//! - 14 production-ready routes
//! - OpenAPI/Swagger documentation
//! - Full middleware stack (CORS, rate limiting, tracing, compression)
//! - Type-safe request/response handling via `tabagent-values`
//!
//! # Usage
//!
//! ```rust,no_run
//! use tabagent_api::{run_server, AppStateProvider};
//! use tabagent_values::{RequestValue, ResponseValue};
//! use std::sync::Arc;
//!
//! struct MyState;
//!
//! #[async_trait::async_trait]
//! impl AppStateProvider for MyState {
//!     async fn handle_request(&self, req: RequestValue) 
//!         -> anyhow::Result<ResponseValue> 
//!     {
//!         Ok(ResponseValue::health("ok"))
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let state = Arc::new(MyState);
//!     tabagent_api::run_server(state, 8080).await
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(unsafe_code)]

mod config;
mod error;
mod traits;
mod router;
mod middleware;
mod routes;
pub mod route_trait;

// Re-export public API
pub use config::ApiConfig;
pub use error::{ApiError, ApiResult};
pub use traits::{AppStateProvider, AppStateWrapper};

use std::{net::SocketAddr, sync::Arc};

/// Run the HTTP API server.
///
/// This is the main entry point for starting the Axum server.
///
/// # Arguments
///
/// * `state` - Application state implementing `AppStateProvider`
/// * `port` - Port number to bind to
///
/// # Errors
///
/// Returns an error if:
/// - The port is already in use
/// - The server fails to bind
/// - The server encounters a fatal error
///
/// # Example
///
/// ```rust,no_run
/// # use std::sync::Arc;
/// # use tabagent_api::{run_server, AppStateProvider};
/// # use tabagent_values::{RequestValue, ResponseValue};
/// # struct MyState;
/// # #[async_trait::async_trait]
/// # impl AppStateProvider for MyState {
/// #     async fn handle_request(&self, _req: RequestValue) -> anyhow::Result<ResponseValue> {
/// #         Ok(ResponseValue::health("ok"))
/// #     }
/// # }
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// let state = Arc::new(MyState);
/// tabagent_api::run_server(state, 8080).await?;
/// # Ok(())
/// # }
/// ```
pub async fn run_server<S>(state: S, port: u16) -> anyhow::Result<()>
where
    S: AppStateProvider + 'static,
{
    run_server_with_config(Arc::new(state) as Arc<dyn AppStateProvider>, ApiConfig { port, ..Default::default() }).await
}

/// Run the HTTP API server with custom configuration.
///
/// This allows more control over server behavior via `ApiConfig`.
///
/// # Arguments
///
/// * `state` - Application state as Arc<dyn AppStateProvider> (concrete type for Axum)
/// * `config` - API configuration
///
/// # Errors
///
/// Returns an error if the server fails to start.
pub async fn run_server_with_config(
    state: Arc<dyn AppStateProvider>,
    config: ApiConfig,
) -> anyhow::Result<()> 
{
    // Bind to address first
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::info!("TabAgent API server listening on http://{}", addr);
    if config.enable_swagger {
        tracing::info!("📘 Swagger UI:    http://{}/swagger-ui/", addr);
        tracing::info!("📗 RapiDoc:       http://{}/rapidoc/", addr);
        tracing::info!("📕 Redoc:         http://{}/redoc/", addr);
        tracing::info!("📄 OpenAPI Spec:  http://{}/api-doc/openapi.json", addr);
    }

    // Axum 0.8 fix: Wrap trait object in concrete Clone type
    let wrapped_state = crate::traits::AppStateWrapper(state);
    
    // Configure all routes and middleware (returns MakeService)
    let service = router::configure_routes(wrapped_state, &config);
    
    // Axum 0.8: Serve the MakeService directly
    axum::serve(listener, service).await?;

    Ok(())
}

/// Build a router for testing purposes.
///
/// This is a convenience function for integration tests.
/// Returns an Axum Router that can be used with `tower::ServiceExt::oneshot`.
#[cfg(any(test, feature = "test-helpers"))]
pub fn build_test_router(
    state: Arc<dyn AppStateProvider>,
) -> axum::Router {
    let config = ApiConfig::development();
    let wrapped_state = AppStateWrapper(state);
    
    // Build router without calling into_make_service (tests need Router, not service)
    let mut router = axum::Router::new();
    
    use crate::route_trait::RegisterableRoute;
    
    // Register just essential routes for testing
    router = routes::health::HealthRoute::register(router);
    router = routes::chat::ChatRoute::register(router);
    
    // Apply middleware
    use tower_http::{trace::TraceLayer, compression::CompressionLayer};
    router = router
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http());
    
    // Apply state last
    router.with_state(wrapped_state)
}

