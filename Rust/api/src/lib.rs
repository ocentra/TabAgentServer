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
//! use tabagent_values::{RequestValue, ResponseValue, HealthStatus};
//!
//! struct MyState;
//!
//! #[async_trait::async_trait]
//! impl AppStateProvider for MyState {
//!     async fn handle_request(&self, req: RequestValue) 
//!         -> anyhow::Result<ResponseValue> 
//!     {
//!         Ok(ResponseValue::health(HealthStatus::Healthy))
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let state = MyState;
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
pub use traits::{AppStateProvider, AppStateWrapper, ApiState};

use std::{net::SocketAddr, sync::Arc};

/// Run the HTTP API server.
///
/// This is the main entry point for starting the Axum server without WebRTC signaling.
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
/// # use tabagent_api::{run_server, AppStateProvider};
/// # use tabagent_values::{RequestValue, ResponseValue, HealthStatus};
/// # struct MyState;
/// # #[async_trait::async_trait]
/// # impl AppStateProvider for MyState {
/// #     async fn handle_request(&self, _req: RequestValue) -> anyhow::Result<ResponseValue> {
/// #         Ok(ResponseValue::health(HealthStatus::Healthy))
/// #     }
/// # }
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// let state = MyState;
/// tabagent_api::run_server(state, 8080).await?;
/// # Ok(())
/// # }
/// ```
pub async fn run_server<S>(state: S, port: u16) -> anyhow::Result<()>
where
    S: AppStateProvider + 'static,
{
    run_server_with_config(Arc::new(state) as Arc<dyn AppStateProvider>, None, ApiConfig { port, ..Default::default() }).await
}

/// Run the HTTP API server with WebRTC signaling support.
///
/// Use this when you want both business logic routes AND WebRTC signaling routes.
///
/// # Arguments
///
/// * `state` - Application state implementing `AppStateProvider`
/// * `webrtc_manager` - WebRTC session manager for signaling
/// * `port` - Port number to bind to
pub async fn run_server_with_webrtc<S>(
    state: S,
    webrtc_manager: Arc<tabagent_webrtc::WebRtcManager>,
    port: u16,
) -> anyhow::Result<()>
where
    S: AppStateProvider + 'static,
{
    run_server_with_config(
        Arc::new(state) as Arc<dyn AppStateProvider>,
        Some(webrtc_manager),
        ApiConfig { port, ..Default::default() }
    ).await
}

/// Run the HTTP API server with custom configuration.
///
/// This allows more control over server behavior via `ApiConfig`.
///
/// # Arguments
///
/// * `state` - Application state as Arc<dyn AppStateProvider> (concrete type for Axum)
/// * `webrtc_manager` - Optional WebRTC manager for signaling routes
/// * `config` - API configuration
///
/// # Errors
///
/// Returns an error if the server fails to start.
pub async fn run_server_with_config(
    state: Arc<dyn AppStateProvider>,
    webrtc_manager: Option<Arc<tabagent_webrtc::WebRtcManager>>,
    config: ApiConfig,
) -> anyhow::Result<()> 
{
    // Bind to address first
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::info!("🚀 TabAgent API server listening on http://{}", addr);
    tracing::info!("🌐 Dashboard:     http://{}/", addr);
    if config.enable_swagger {
        tracing::info!("📘 Swagger UI:    http://{}/swagger-ui/", addr);
        tracing::info!("📗 RapiDoc:       http://{}/rapidoc/", addr);
        tracing::info!("📕 Redoc:         http://{}/redoc/", addr);
        tracing::info!("📄 OpenAPI Spec:  http://{}/api-doc/openapi.json", addr);
    }
    tracing::info!("🎮 WebRTC Demo:   http://{}/demo/webrtc-demo.html", addr);

    // Axum 0.8 fix: Wrap trait object in concrete Clone type
    let wrapped_state = crate::traits::AppStateWrapper(state);
    
    // Configure all routes and middleware (returns MakeService)
    let service = router::configure_routes(wrapped_state, webrtc_manager, &config);
    
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
    let _config = ApiConfig::development();
    let wrapped_state = AppStateWrapper(state);
    
    // Build router (type inferred from register calls)
    use crate::route_trait::RegisterableRoute;
    
    let mut router = axum::Router::new();
    
    // Register all trait-based routes (same as production)
    router = routes::health::HealthRoute::register(router);
    router = routes::chat::ChatRoute::register(router);
    router = routes::chat::ResponsesRoute::register(router);
    router = routes::generate::GenerateRoute::register(router);
    router = routes::embeddings::EmbeddingsRoute::register(router);
    router = routes::models::LoadModelRoute::register(router);
    router = routes::models::UnloadModelRoute::register(router);
    router = routes::models::ListModelsRoute::register(router);
    router = routes::models::ModelInfoRoute::register(router);
    router = routes::system::SystemRoute::register(router);
    router = routes::sessions::GetHistoryRoute::register(router);
    router = routes::sessions::SaveMessageRoute::register(router);
    router = routes::rag::RagRoute::register(router);
    router = routes::params::GetParamsRoute::register(router);
    router = routes::params::SetParamsRoute::register(router);
    router = routes::stats::GetStatsRoute::register(router);
    router = routes::resources::GetResourcesRoute::register(router);
    router = routes::resources::EstimateMemoryRoute::register(router);
    router = routes::management::GetRecipesRoute::register(router);
    router = routes::management::GetEmbeddingModelsRoute::register(router);
    router = routes::management::GetLoadedModelsRoute::register(router);
    router = routes::management::SelectModelRoute::register(router);
    router = routes::management::PullModelRoute::register(router);
    router = routes::management::DeleteModelRoute::register(router);
    router = routes::generation::StopGenerationRoute::register(router);
    router = routes::rerank::RerankRoute::register(router);
    
    // Apply middleware
    use tower_http::{trace::TraceLayer, compression::CompressionLayer};
    router = router
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http());
    
    // Apply state last
    router.with_state(wrapped_state)
}

