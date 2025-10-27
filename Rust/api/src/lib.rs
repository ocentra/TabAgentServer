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
pub use traits::AppStateProvider;

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
    use axum::Router;
    
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

    // CORRECT Axum 0.8 pattern with CONCRETE state type:
    // 1. Use Arc<dyn AppStateProvider> as CONCRETE state (not generic S)
    // 2. Create Router::new().with_state(state) to get Router<Arc<dyn AppStateProvider>>
    // 3. Add all routes
    // 4. into_make_service() works on concrete Router<Arc<dyn AppStateProvider>>!
    
    // Step 1: Create stateful router with CONCRETE state type
    let app = Router::new().with_state(state);
    
    // Step 2: Configure all routes and middleware
    let app = router::configure_routes(app, &config);
    
    // Step 3: Convert router to service and serve
    // Axum 0.8 pattern: use into_make_service_with_connect_info
    let service = app.into_make_service_with_connect_info::<SocketAddr>();
    axum::serve(listener, service).await?;

    Ok(())
}

