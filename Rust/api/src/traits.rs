//! Traits for API integration.
//!
//! This module defines the contract between the API layer and the application state.

use tabagent_values::{RequestValue, ResponseValue};
use async_trait::async_trait;

/// Trait that application state must implement to handle API requests.
///
/// The server binary implements this trait to process incoming HTTP requests
/// by routing them through the handler logic to the appropriate backends
/// (ONNX, GGUF, Python).
///
/// # Example
///
/// ```rust
/// use tabagent_api::AppStateProvider;
/// use tabagent_values::{RequestValue, ResponseValue};
/// use async_trait::async_trait;
///
/// struct MyAppState {
///     // Your state fields
/// }
///
/// #[async_trait]
/// impl AppStateProvider for MyAppState {
///     async fn handle_request(&self, request: RequestValue) 
///         -> anyhow::Result<ResponseValue> 
///     {
///         // Route to appropriate handler based on request type
///         match request.request_type() {
///             _ => Ok(ResponseValue::health("ok"))
///         }
///     }
/// }
/// ```
/// App state provider trait (must be object-safe for trait objects).
#[async_trait]
pub trait AppStateProvider: Send + Sync + 'static {
    /// Handle an incoming API request.
    ///
    /// This method receives a type-safe `RequestValue` and must return
    /// a `ResponseValue` or an error.
    ///
    /// # Arguments
    ///
    /// * `request` - The incoming request (already parsed and validated)
    ///
    /// # Returns
    ///
    /// * `Ok(ResponseValue)` - Successful response
    /// * `Err(anyhow::Error)` - Error that will be converted to HTTP error response
    ///
    /// # Errors
    ///
    /// Implementations should return errors for:
    /// - Model not found/loaded
    /// - Invalid parameters
    /// - Backend failures (ONNX, GGUF, Python)
    /// - Database errors
    /// - Resource exhaustion
    async fn handle_request(&self, request: RequestValue) -> anyhow::Result<ResponseValue>;
}

// Blanket implementation for Arc<dyn AppStateProvider>
// This allows using Arc<dyn AppStateProvider> as a concrete state type in Axum 0.8
// Without this, Arc<T> doesn't automatically implement T even if T is a trait
#[async_trait]
impl AppStateProvider for std::sync::Arc<dyn AppStateProvider> {
    async fn handle_request(&self, request: RequestValue) -> anyhow::Result<ResponseValue> {
        // Delegate to the inner trait object by dereferencing the Arc
        (**self).handle_request(request).await
    }
}

