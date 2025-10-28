//! Unified backend trait for all TabAgent entry points.
//!
//! This module defines the single, canonical trait that ALL entry points
//! (HTTP API, Native Messaging, WebRTC) use to communicate with the backend.
//!
//! # Design Principles
//!
//! 1. **DRY**: ONE trait definition, used everywhere
//! 2. **Type Safety**: Uses `tabagent-values` for requests/responses
//! 3. **Transport Agnostic**: Same backend handles HTTP, stdin, WebRTC
//! 4. **Async**: Uses async-trait for non-blocking operations
//!
//! # Architecture
//!
//! ```text
//! ┌──────────┐  ┌───────────────┐  ┌─────────┐
//! │   API    │  │ Native Msg    │  │ WebRTC  │
//! │ (HTTP)   │  │ (stdin/out)   │  │ (Data)  │
//! └────┬─────┘  └───────┬───────┘  └────┬────┘
//!      │                │                │
//!      └────────────────┼────────────────┘
//!                       │
//!                       ▼
//!           ┌───────────────────────┐
//!           │  AppStateProvider     │  <- SINGLE TRAIT
//!           │  handle_request()     │
//!           └───────────┬───────────┘
//!                       │
//!                       ▼
//!           ┌───────────────────────┐
//!           │  Backend Handler      │
//!           │  (GGUF/ONNX/Python)   │
//!           └───────────────────────┘
//! ```

use async_trait::async_trait;
use tabagent_values::{RequestValue, ResponseValue};

/// Unified backend trait that all entry points use.
///
/// Every entry point (API, Native Messaging, WebRTC) calls this trait
/// to handle requests. The implementation routes to the appropriate
/// backend (GGUF, ONNX, Python) based on the request type and model.
///
/// # Example Implementation
///
/// ```rust
/// use common::backend::AppStateProvider;
/// use tabagent_values::{RequestValue, ResponseValue, HealthStatus};
/// use async_trait::async_trait;
///
/// struct MyBackend {
///     // ... backend state
/// }
///
/// #[async_trait]
/// impl AppStateProvider for MyBackend {
///     async fn handle_request(&self, request: RequestValue) 
///         -> anyhow::Result<ResponseValue> 
///     {
///         match request.request_type() {
///             // Route to appropriate handler
///             _ => Ok(ResponseValue::health(HealthStatus::Healthy))
///         }
///     }
/// }
/// ```
#[async_trait]
pub trait AppStateProvider: Send + Sync + 'static {
    /// Handle an incoming request from any transport.
    ///
    /// This method receives a type-safe `RequestValue` (from `tabagent-values`)
    /// and must return a `ResponseValue` or an error.
    ///
    /// # Transport Independence
    ///
    /// This method does NOT know or care where the request came from:
    /// - HTTP POST from API
    /// - JSON message from stdin (native messaging)
    /// - Binary message from WebRTC data channel
    ///
    /// All transports use the same `RequestValue` type, ensuring identical
    /// behavior regardless of how the request arrived.
    ///
    /// # Arguments
    ///
    /// * `request` - The incoming request (already parsed and validated by transport layer)
    ///
    /// # Returns
    ///
    /// * `Ok(ResponseValue)` - Successful response
    /// * `Err(anyhow::Error)` - Error (will be converted to transport-specific format)
    ///
    /// # Errors
    ///
    /// Implementations should return errors for:
    /// - Model not found/loaded
    /// - Invalid parameters (after transport validation)
    /// - Backend failures (ONNX runtime errors, GGUF errors, Python bridge errors)
    /// - Database errors
    /// - Resource exhaustion (OOM, disk space)
    async fn handle_request(&self, request: RequestValue) -> anyhow::Result<ResponseValue>;
}

// --- Blanket Implementations for Common Wrapper Types ---

/// Blanket implementation for `Arc<dyn AppStateProvider>`.
///
/// This allows using `Arc<dyn AppStateProvider>` as a concrete state type
/// without manually implementing the trait for Arc.
#[async_trait]
impl AppStateProvider for std::sync::Arc<dyn AppStateProvider> {
    async fn handle_request(&self, request: RequestValue) -> anyhow::Result<ResponseValue> {
        // Delegate to the inner trait object by dereferencing the Arc
        (**self).handle_request(request).await
    }
}

/// Blanket implementation for `Box<dyn AppStateProvider>`.
///
/// This allows using `Box<dyn AppStateProvider>` for owned trait objects.
#[async_trait]
impl AppStateProvider for Box<dyn AppStateProvider> {
    async fn handle_request(&self, request: RequestValue) -> anyhow::Result<ResponseValue> {
        // Delegate to the inner trait object by dereferencing the Box
        (**self).handle_request(request).await
    }
}

/// Concrete wrapper for trait object state (Axum 0.8 compatibility).
///
/// Axum 0.8+ requires Router state to be `Clone`. This wrapper provides
/// a concrete Clone type that wraps the trait object, allowing middleware
/// layers to work properly with `into_make_service()`.
///
/// # Usage with Axum
///
/// ```rust,ignore
/// use axum::Router;
/// use common::backend::{AppStateProvider, AppStateWrapper};
/// use std::sync::Arc;
///
/// let backend: Arc<dyn AppStateProvider> = /* your implementation */;
/// let wrapper = AppStateWrapper(backend);
/// 
/// let app = Router::new()
///     .route("/health", get(health_handler))
///     .with_state(wrapper); // Wrapper is Clone
/// ```
#[derive(Clone)]
pub struct AppStateWrapper(pub std::sync::Arc<dyn AppStateProvider>);

#[async_trait]
impl AppStateProvider for AppStateWrapper {
    async fn handle_request(&self, request: RequestValue) -> anyhow::Result<ResponseValue> {
        self.0.handle_request(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tabagent_values::HealthStatus;

    struct MockBackend;

    #[async_trait]
    impl AppStateProvider for MockBackend {
        async fn handle_request(&self, _request: RequestValue) -> anyhow::Result<ResponseValue> {
            Ok(ResponseValue::health(HealthStatus::Healthy))
        }
    }

    #[tokio::test]
    async fn test_direct_backend() {
        let backend = MockBackend;
        let request = RequestValue::system_info();
        let response = backend.handle_request(request).await.unwrap();
        assert!(response.as_health().is_some());
    }

    #[tokio::test]
    async fn test_arc_backend() {
        let backend = std::sync::Arc::new(MockBackend);
        let request = RequestValue::system_info();
        let response = backend.handle_request(request).await.unwrap();
        assert!(response.as_health().is_some());
    }

    #[tokio::test]
    async fn test_box_backend() {
        let backend: Box<dyn AppStateProvider> = Box::new(MockBackend);
        let request = RequestValue::system_info();
        let response = backend.handle_request(request).await.unwrap();
        assert!(response.as_health().is_some());
    }

    #[tokio::test]
    async fn test_wrapper_backend() {
        let backend = AppStateWrapper(std::sync::Arc::new(MockBackend));
        let request = RequestValue::system_info();
        let response = backend.handle_request(request).await.unwrap();
        assert!(response.as_health().is_some());
    }

    #[tokio::test]
    async fn test_wrapper_is_clone() {
        let backend = AppStateWrapper(std::sync::Arc::new(MockBackend));
        let clone = backend.clone();
        
        let request = RequestValue::system_info();
        let response = clone.handle_request(request).await.unwrap();
        assert!(response.as_health().is_some());
    }
}

