//! Traits for native messaging integration.
//!
//! This module defines the contract between the native messaging layer
//! and the application state, mirroring the patterns established in the
//! API and WebRTC crates for consistency.

use tabagent_values::{RequestValue, ResponseValue};
use async_trait::async_trait;

/// Trait that application state must implement to handle native messaging requests.
///
/// This trait is identical to the AppStateProvider trait in the API crate,
/// ensuring that the same backend services can handle requests from HTTP,
/// WebRTC, and native messaging clients without modification.
///
/// # Example
///
/// ```rust
/// use tabagent_native_messaging::AppStateProvider;
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
#[async_trait]
pub trait AppStateProvider: Send + Sync + 'static {
    /// Handle an incoming native messaging request.
    ///
    /// This method receives a type-safe `RequestValue` and must return
    /// a `ResponseValue` or an error. The implementation should be identical
    /// to the HTTP API handler to ensure consistent behavior.
    ///
    /// # Arguments
    ///
    /// * `request` - The incoming request (already parsed and validated)
    ///
    /// # Returns
    ///
    /// * `Ok(ResponseValue)` - Successful response
    /// * `Err(anyhow::Error)` - Error that will be converted to native messaging error response
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
// This allows using Arc<dyn AppStateProvider> as a concrete state type
// Without this, Arc<T> doesn't automatically implement T even if T is a trait
#[async_trait]
impl AppStateProvider for std::sync::Arc<dyn AppStateProvider> {
    async fn handle_request(&self, request: RequestValue) -> anyhow::Result<ResponseValue> {
        // Delegate to the inner trait object by dereferencing the Arc
        (**self).handle_request(request).await
    }
}

/// Trait for request handlers that can be used in middleware chains.
///
/// This trait allows for composable request processing with middleware
/// for logging, authentication, rate limiting, etc.
#[async_trait]
pub trait RequestHandler: Send + Sync {
    /// Handle a request.
    ///
    /// # Arguments
    ///
    /// * `request` - The incoming request
    ///
    /// # Returns
    ///
    /// * `Ok(ResponseValue)` - Successful response
    /// * `Err(anyhow::Error)` - Error that should be handled by error middleware
    async fn handle(&self, request: RequestValue) -> anyhow::Result<ResponseValue>;
}

/// Trait for middleware that can be applied to request processing.
///
/// Middleware can modify requests, responses, or handle cross-cutting
/// concerns like logging, authentication, and rate limiting.
#[async_trait]
pub trait Middleware: Send + Sync {
    /// Process a request through this middleware.
    ///
    /// # Arguments
    ///
    /// * `request` - The incoming request (may be modified)
    /// * `handler` - The next handler in the chain
    ///
    /// # Returns
    ///
    /// * `Ok(ResponseValue)` - Successful response (may be modified)
    /// * `Err(anyhow::Error)` - Error that should be handled appropriately
    async fn process(
        &self,
        request: RequestValue,
        handler: &dyn RequestHandler,
    ) -> anyhow::Result<ResponseValue>;
}

/// Trait for route-specific authentication and authorization.
///
/// This trait allows routes to define their own authentication
/// and authorization requirements.
#[async_trait]
pub trait RouteAuth: Send + Sync {
    /// Check if the request is authenticated.
    ///
    /// # Arguments
    ///
    /// * `request` - The incoming request
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Request is authenticated
    /// * `Err(anyhow::Error)` - Authentication failed
    async fn authenticate(&self, request: &RequestValue) -> anyhow::Result<()>;
    
    /// Check if the request is authorized for this route.
    ///
    /// # Arguments
    ///
    /// * `request` - The incoming request
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Request is authorized
    /// * `Err(anyhow::Error)` - Authorization failed
    async fn authorize(&self, request: &RequestValue) -> anyhow::Result<()>;
}

/// Default implementation that allows all requests (no auth).
pub struct NoAuth;

#[async_trait]
impl RouteAuth for NoAuth {
    async fn authenticate(&self, _request: &RequestValue) -> anyhow::Result<()> {
        Ok(())
    }
    
    async fn authorize(&self, _request: &RequestValue) -> anyhow::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tabagent_values::{HealthStatus, TokenUsage};
    
    struct MockAppState;
    
    #[async_trait]
    impl AppStateProvider for MockAppState {
        async fn handle_request(&self, _request: RequestValue) -> anyhow::Result<ResponseValue> {
            // Return a simple health response for all requests in tests
            Ok(ResponseValue::health(HealthStatus::Healthy))
        }
    }
    
    #[tokio::test]
    async fn test_app_state_provider() {
        let state = MockAppState;
        let request = RequestValue::system_info();
        
        let response = state.handle_request(request).await.unwrap();
        // Just verify we get a response - don't check specific values
        assert!(response.as_health().is_some());
    }
    
    #[tokio::test]
    async fn test_arc_app_state_provider() {
        let state = std::sync::Arc::new(MockAppState);
        let request = RequestValue::system_info();
        
        let response = state.handle_request(request).await.unwrap();
        // Just verify we get a response - don't check specific values
        assert!(response.as_health().is_some());
    }
    
    #[tokio::test]
    async fn test_no_auth() {
        let auth = NoAuth;
        let request = RequestValue::system_info();
        
        assert!(auth.authenticate(&request).await.is_ok());
        assert!(auth.authorize(&request).await.is_ok());
    }
}