//! Integration tests for the API.

use std::sync::Arc;
use std::path::PathBuf;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt; // for `oneshot`
use serde_json::json;

use tabagent_api::build_test_router;
use tabagent_server::{AppState, CliArgs, ServerMode};

/// Create REAL server state for integration testing.
/// 
/// IMPORTANT: These tests use the REAL backend (tabagent-server::AppState).
/// If they fail, it means real issues that need fixing - NOT test issues!
/// Better to have honest failing tests than fake passing mocks.
async fn create_test_state(test_name: &str) -> AppState {
    let temp_dir = std::env::temp_dir().join(format!("tabagent_api_test_{}", test_name));
    let db_path = temp_dir.join("db");
    let models_path = temp_dir.join("models");
    
    // Clean up any previous test data
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&db_path).expect("Failed to create test DB dir");
    std::fs::create_dir_all(&models_path).expect("Failed to create test models dir");
    
    let args = CliArgs {
        mode: ServerMode::Http,
        port: 8001,
        config: PathBuf::from("test.toml"),
        db_path,
        model_cache_path: models_path,
        log_level: "error".to_string(), // Reduce noise
        webrtc_enabled: false,
        webrtc_port: 8002,
    };
    AppState::new(&args).await.expect("Failed to create test state")
}

#[tokio::test]
async fn test_health_endpoint() {
    let state = create_test_state("health").await;
    let app = build_test_router(Arc::new(state));

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
    let state = create_test_state("chat").await;
    let app = build_test_router(Arc::new(state));

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
