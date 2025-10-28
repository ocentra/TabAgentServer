//! Traits for native messaging integration.
//!
//! This module re-exports the unified backend trait from `common`.

// Re-export unified backend trait from common
pub use common::backend::AppStateProvider;

#[cfg(test)]
mod tests {
    use super::*;
    use tabagent_values::{RequestValue, ResponseValue, HealthStatus};
    use async_trait::async_trait;
    
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
}