//! TabAgent Native Messaging Crate
//!
//! Chrome extension communication layer for TabAgent Server.
//!
//! # Architecture
//!
//! This crate provides native messaging host functionality with:
//! - 100% API parity with HTTP and WebRTC endpoints (36+ routes)
//! - Chrome native messaging protocol compliance
//! - Identical architectural patterns as API/WebRTC crates
//! - Type-safe request/response handling via `tabagent-values`
//! - Compile-time route enforcement and validation
//!
//! # Usage
//!
//! ```rust,no_run
//! use tabagent_native_messaging::{NativeMessagingHost, NativeMessagingConfig, AppStateProvider};
//! use tabagent_values::{RequestValue, ResponseValue, HealthStatus};
//! use std::sync::Arc;
//!
//! struct MyState;
//!
//! #[async_trait::async_trait]
//! impl AppStateProvider for MyState {
//!     async fn handle_request(&self, _req: RequestValue) 
//!         -> anyhow::Result<ResponseValue> 
//!     {
//!         Ok(ResponseValue::health(HealthStatus::Healthy))
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let state = Arc::new(MyState);
//!     let config = NativeMessagingConfig::default();
//!     tabagent_native_messaging::run_host(state, config).await
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(unsafe_code)]

mod config;
mod error;
mod traits;
mod protocol;
mod router;
// middleware module removed - was never used
mod routes;
pub mod route_trait;

// Re-export public API
pub use config::NativeMessagingConfig;
pub use error::{NativeMessagingError, NativeMessagingResult};
pub use traits::AppStateProvider;
pub use protocol::{NativeMessagingProtocol, IncomingMessage, OutgoingMessage};
pub use router::MessageRouter;

use std::sync::Arc;

/// Native messaging host for Chrome extensions.
pub struct NativeMessagingHost {
    router: MessageRouter,
    protocol: NativeMessagingProtocol,
}

impl NativeMessagingHost {
    /// Create a new native messaging host.
    pub fn new(state: Arc<dyn AppStateProvider>, config: NativeMessagingConfig) -> Self {
        let mut router = MessageRouter::new(state);
        router.register_all_routes();
        let protocol = NativeMessagingProtocol::new(config);
        
        Self { router, protocol }
    }
    
    /// Process a single message.
    pub async fn process_message(&self, message: IncomingMessage) -> NativeMessagingResult<OutgoingMessage> {
        self.router.dispatch(message).await
    }
    
    /// Run the main message processing loop.
    pub async fn run(&self) -> anyhow::Result<()> {
        tracing::info!("TabAgent Native Messaging Host starting message loop");
        
        loop {
            match self.protocol.read_message().await {
                Ok(message) => {
                    let request_id = message.request_id.clone();
                    
                    tracing::debug!(
                        request_id = %request_id,
                        route = %message.route,
                        "Processing native messaging request"
                    );
                    
                    // Process message through router
                    let response = match self.router.dispatch(message).await {
                        Ok(response) => response,
                        Err(e) => {
                            tracing::error!(
                                request_id = %request_id,
                                error = %e,
                                "Request processing failed"
                            );
                            
                            OutgoingMessage {
                                request_id: request_id.clone(),
                                success: false,
                                data: None,
                                error: Some(e.into()),
                            }
                        }
                    };
                    
                    // Send response
                    if let Err(e) = self.protocol.write_message(&response).await {
                        tracing::error!(
                            request_id = %request_id,
                            error = %e,
                            "Failed to send response"
                        );
                        // Continue processing - don't exit on write errors
                    }
                }
                Err(e) => {
                    tracing::error!(error = %e, "Failed to read message");
                    
                    // For protocol errors, try to send error response if possible
                    if let Ok(error_response) = serde_json::to_value(&OutgoingMessage {
                        request_id: "unknown".to_string(),
                        success: false,
                        data: None,
                        error: Some(e.into()),
                    }) {
                        let _ = self.protocol.write_raw_message(&error_response).await;
                    }
                    
                    // Continue processing - don't exit on read errors
                }
            }
        }
    }
}

/// Run the native messaging host.
///
/// This is the main entry point for starting the native messaging host
/// that communicates with Chrome extensions via stdin/stdout.
///
/// # Arguments
///
/// * `state` - Application state implementing `AppStateProvider`
/// * `config` - Native messaging configuration
///
/// # Errors
///
/// Returns an error if:
/// - stdin/stdout communication fails
/// - Message parsing fails
/// - Backend services are unavailable
/// - Configuration is invalid
///
/// # Example
///
/// ```rust,no_run
/// # use std::sync::Arc;
/// # use tabagent_native_messaging::{run_host, AppStateProvider, NativeMessagingConfig};
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
/// let state = Arc::new(MyState);
/// let config = NativeMessagingConfig::default();
/// tabagent_native_messaging::run_host(state, config).await?;
/// # Ok(())
/// # }
/// ```
pub async fn run_host<S>(state: S, config: NativeMessagingConfig) -> anyhow::Result<()>
where
    S: AppStateProvider + 'static,
{
    run_host_with_state(Arc::new(state) as Arc<dyn AppStateProvider>, config).await
}

/// Run the native messaging host with concrete state type.
///
/// This allows more control over state management and is used internally
/// by the main `run_host` function.
///
/// # Arguments
///
/// * `state` - Application state as Arc<dyn AppStateProvider>
/// * `config` - Native messaging configuration
///
/// # Errors
///
/// Returns an error if the host fails to start or encounters a fatal error.
pub async fn run_host_with_state(
    state: Arc<dyn AppStateProvider>,
    config: NativeMessagingConfig,
) -> anyhow::Result<()> 
{
    let host = NativeMessagingHost::new(state, config);
    host.run().await
}