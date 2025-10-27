//! Integration tests for the API.

use std::sync::Arc;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt; // for `oneshot`
use serde_json::json;

use tabagent_api::{AppStateProvider, ApiConfig};
use tabagent_values::{RequestValue, ResponseValue};

/// Mock application state for testing.
struct MockState;

#[async_trait::async_trait]
impl AppStateProvider for MockState {
    async fn handle_request(&self, _request: RequestValue) -> anyhow::Result<ResponseValue> {
        Ok(ResponseValue::health("ok"))
    }
}

#[tokio::test]
async fn test_health_endpoint() {
    let state = Arc::new(MockState);
    let app = tabagent_api::router::build_router(state);

    let response = app
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_openapi_spec() {
    let state = Arc::new(MockState);
    let config = ApiConfig::development();
    let app = tabagent_api::router::build_router_with_config(state, config);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api-doc/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_chat_completions() {
    let state = Arc::new(MockState);
    let app = tabagent_api::router::build_router(state);

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

    assert_eq!(response.status(), StatusCode::OK);
}

