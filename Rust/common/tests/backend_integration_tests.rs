//! Integration tests for the unified backend trait system.
//!
//! These tests verify that the `AppStateProvider` trait and its wrapper types
//! work correctly across different usage patterns, especially for Axum 0.8 compatibility.

use common::{AppStateProvider, AppStateWrapper};
use tabagent_values::{RequestValue, ResponseValue, HealthStatus, Message, MessageRole};
use async_trait::async_trait;
use std::sync::Arc;

// === Mock Backend for Testing ===

struct MockBackend {
    name: String,
}

impl MockBackend {
    fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[async_trait]
impl AppStateProvider for MockBackend {
    async fn handle_request(&self, request: RequestValue) -> anyhow::Result<ResponseValue> {
        // Route based on request type
        match request.request_type() {
            tabagent_values::RequestType::Health => {
                Ok(ResponseValue::health(HealthStatus::Healthy))
            }
            tabagent_values::RequestType::SystemInfo => {
                Ok(ResponseValue::health(HealthStatus::Healthy))
            }
            tabagent_values::RequestType::Chat { model, .. } => {
                Ok(ResponseValue::chat(
                    "test-request-id",
                    model,
                    format!("Response from {}", self.name),
                    tabagent_values::TokenUsage::new(10, 20),
                ))
            }
            _ => Ok(ResponseValue::health(HealthStatus::Healthy)),
        }
    }
}

// === Test Direct Implementation ===

#[tokio::test]
async fn test_direct_implementation() {
    let backend = MockBackend::new("direct");
    
    // Test health request
    let request = RequestValue::health();
    let response = backend.handle_request(request).await.unwrap();
    assert!(response.as_health().is_some());
    
    // Test system info request
    let request = RequestValue::system_info();
    let response = backend.handle_request(request).await.unwrap();
    assert!(response.as_health().is_some());
}

// === Test Arc Wrapper (Critical for Axum 0.8) ===

#[tokio::test]
async fn test_arc_wrapper() {
    let backend = Arc::new(MockBackend::new("arc"));
    
    // Should work with Arc due to blanket impl
    let request = RequestValue::health();
    let response = backend.handle_request(request).await.unwrap();
    assert!(response.as_health().is_some());
    
    // Test that Arc can be cloned
    let clone = backend.clone();
    let request = RequestValue::system_info();
    let response = clone.handle_request(request).await.unwrap();
    assert!(response.as_health().is_some());
}

// === Test Box Wrapper ===

#[tokio::test]
async fn test_box_wrapper() {
    let backend: Box<dyn AppStateProvider> = Box::new(MockBackend::new("box"));
    
    // Should work with Box due to blanket impl
    let request = RequestValue::health();
    let response = backend.handle_request(request).await.unwrap();
    assert!(response.as_health().is_some());
}

// === Test AppStateWrapper (Axum 0.8 Concrete Type) ===

#[tokio::test]
async fn test_app_state_wrapper() {
    let backend = AppStateWrapper(Arc::new(MockBackend::new("wrapper")));
    
    // Test request handling
    let request = RequestValue::health();
    let response = backend.handle_request(request).await.unwrap();
    assert!(response.as_health().is_some());
    
    // Test Clone trait (CRITICAL for Axum 0.8)
    let clone = backend.clone();
    let request = RequestValue::system_info();
    let response = clone.handle_request(request).await.unwrap();
    assert!(response.as_health().is_some());
}

// === Test Trait Object with Arc<dyn> ===

#[tokio::test]
async fn test_arc_dyn_trait_object() {
    let backend: Arc<dyn AppStateProvider> = Arc::new(MockBackend::new("arc-dyn"));
    
    // Should work due to blanket impl for Arc<dyn AppStateProvider>
    let request = RequestValue::health();
    let response = backend.handle_request(request).await.unwrap();
    assert!(response.as_health().is_some());
}

// === Test AppStateWrapper with Arc<dyn> ===

#[tokio::test]
async fn test_wrapper_with_arc_dyn() {
    let backend: Arc<dyn AppStateProvider> = Arc::new(MockBackend::new("wrapper-arc-dyn"));
    let wrapper = AppStateWrapper(backend);
    
    // Test that wrapper works with Arc<dyn>
    let request = RequestValue::health();
    let response = wrapper.handle_request(request).await.unwrap();
    assert!(response.as_health().is_some());
    
    // Test Clone
    let clone = wrapper.clone();
    let request = RequestValue::system_info();
    let response = clone.handle_request(request).await.unwrap();
    assert!(response.as_health().is_some());
}

// === Test Multiple Request Types ===

#[tokio::test]
async fn test_multiple_request_types() {
    let backend = AppStateWrapper(Arc::new(MockBackend::new("multi")));
    
    // Health
    let request = RequestValue::health();
    let response = backend.handle_request(request).await.unwrap();
    assert!(response.as_health().is_some());
    
    // System Info
    let request = RequestValue::system_info();
    let response = backend.handle_request(request).await.unwrap();
    assert!(response.as_health().is_some());
    
    // Chat
    let request = RequestValue::chat(
        "test-model",
        vec![Message {
            role: MessageRole::User,
            content: "Hello".to_string(),
            name: None,
        }],
        None,
    );
    let response = backend.handle_request(request).await.unwrap();
    assert!(response.as_chat().is_some());
}

// === Test Error Handling ===

struct ErrorBackend;

#[async_trait]
impl AppStateProvider for ErrorBackend {
    async fn handle_request(&self, _request: RequestValue) -> anyhow::Result<ResponseValue> {
        Err(anyhow::anyhow!("Simulated error"))
    }
}

#[tokio::test]
async fn test_error_propagation() {
    let backend = AppStateWrapper(Arc::new(ErrorBackend));
    
    let request = RequestValue::health();
    let result = backend.handle_request(request).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Simulated error"));
}

// === Test Send + Sync Bounds ===

#[test]
fn test_send_sync_bounds() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    
    // AppStateWrapper must be Send + Sync for Axum 0.8
    assert_send::<AppStateWrapper>();
    assert_sync::<AppStateWrapper>();
    
    // Arc<dyn AppStateProvider> must be Send + Sync
    assert_send::<Arc<dyn AppStateProvider>>();
    assert_sync::<Arc<dyn AppStateProvider>>();
}

// === Test Clone is Cheap (Arc-based) ===

#[test]
fn test_clone_is_cheap() {
    let backend = AppStateWrapper(Arc::new(MockBackend::new("clone-test")));
    
    // Get the Arc count
    let original_count = Arc::strong_count(&backend.0);
    
    // Clone should increment Arc count, not deep copy
    let clone1 = backend.clone();
    let clone2 = backend.clone();
    
    assert_eq!(Arc::strong_count(&backend.0), original_count + 2);
    assert_eq!(Arc::strong_count(&clone1.0), original_count + 2);
    assert_eq!(Arc::strong_count(&clone2.0), original_count + 2);
}

