//! Integration tests for the API.

use std::sync::Arc;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt; // for `oneshot`
use serde_json::json;

use tabagent_api::{AppStateProvider, build_test_router};
use tabagent_values::{RequestValue, ResponseValue, HealthStatus, RequestType, TokenUsage};

/// Mock application state for testing.
struct MockState;

#[async_trait::async_trait]
impl AppStateProvider for MockState {
    async fn handle_request(&self, request: RequestValue) -> anyhow::Result<ResponseValue> {
        // Return appropriate response based on request type
        match request.request_type() {
            RequestType::Chat { .. } => {
                Ok(ResponseValue::chat(
                    "chat-123",
                    "test-model",
                    "Mock response",
                    TokenUsage {
                        prompt_tokens: 10,
                        completion_tokens: 5,
                        total_tokens: 15,
                    },
                ))
            }
            RequestType::SystemInfo => Ok(ResponseValue::health(HealthStatus::Healthy)),
            _ => Ok(ResponseValue::health(HealthStatus::Healthy)),
        }
    }
}

#[tokio::test]
async fn test_health_endpoint() {
    let state = Arc::new(MockState);
    let app = build_test_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .header("content-type", "application/json")
                .body(Body::from("null"))  // Unit struct deserializes from JSON null
                .unwrap()
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

// Note: OpenAPI spec test removed - requires full router with swagger enabled
// The test helper builds a minimal router for basic endpoint testing

#[tokio::test]
async fn test_chat_completions() {
    let state = Arc::new(MockState);
    let app = build_test_router(state);

    let request_body = json!({
        "model": "test-model",
        "messages": [
            {"role": "user", "content": "Hello"}
        ]
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/chat/completions")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(request_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Chat route is registered so this should work
    assert_eq!(response.status(), StatusCode::OK);
}
