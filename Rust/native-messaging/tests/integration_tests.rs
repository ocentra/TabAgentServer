//! Integration tests for native-messaging.
//!
//! IMPORTANT: These tests use the REAL backend (appstate::AppState).
//! If they fail, it means real issues that need fixing - NOT test issues!
//! Better to have honest failing tests than fake passing mocks.

use std::sync::Arc;
use serde_json::json;

use tabagent_native_messaging::{
    NativeMessagingHost, NativeMessagingConfig, IncomingMessage
};
use appstate::{AppState, AppStateConfig};

/// Create REAL server state for integration testing.
/// 
/// This uses the actual AppState backend with a temporary database.
async fn create_test_state(test_name: &str) -> AppState {
    let temp_dir = std::env::temp_dir().join(format!("tabagent_nm_test_{}", test_name));
    let db_path = temp_dir.join("db");
    let models_path = temp_dir.join("models");
    
    // Clean up any previous test data
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&db_path).expect("Failed to create test DB dir");
    std::fs::create_dir_all(&models_path).expect("Failed to create test models dir");
    
    let config = AppStateConfig {
        db_path,
        model_cache_path: models_path,
    };
    
    AppState::new(config).await.expect("Failed to create test state")
}

/// Create a test host with real AppState backend.
async fn create_test_host(test_name: &str) -> NativeMessagingHost {
    let state = create_test_state(test_name).await;
    let config = NativeMessagingConfig::default();
    NativeMessagingHost::new(Arc::new(state), config)
}

#[tokio::test]
async fn test_health_route() {
    let host = create_test_host("health").await;
    
    let message = IncomingMessage {
        request_id: "test-health-1".to_string(),
        route: "health".to_string(),
        payload: json!(null),
    };
    
    let response = host.process_message(message).await.expect("Health route should succeed");
    
    assert!(response.success, "Health check should succeed");
    assert_eq!(response.request_id, "test-health-1");
    assert!(response.data.is_some(), "Health response should have data");
    assert!(response.error.is_none(), "Health response should have no error");
}

#[tokio::test]
async fn test_system_info_route() {
    let host = create_test_host("system_info").await;
    
    let message = IncomingMessage {
        request_id: "test-system-1".to_string(),
        route: "system".to_string(),
        payload: json!(null),
    };
    
    let response = host.process_message(message).await.expect("System route should succeed");
    
    assert!(response.success, "System info should succeed");
    assert_eq!(response.request_id, "test-system-1");
    assert!(response.data.is_some(), "System response should have data");
}

#[tokio::test]
async fn test_hardware_info_route() {
    let host = create_test_host("hardware").await;
    
    let message = IncomingMessage {
        request_id: "test-hw-1".to_string(),
        route: "get_hardware_info".to_string(),
        payload: json!(null),
    };
    
    let response = host.process_message(message).await.expect("Hardware info route should succeed");
    
    assert!(response.success, "Hardware info should succeed");
    assert_eq!(response.request_id, "test-hw-1");
    assert!(response.data.is_some(), "Hardware response should have data");
}

#[tokio::test]
async fn test_hf_token_status_route() {
    let host = create_test_host("hf_status").await;
    
    let message = IncomingMessage {
        request_id: "test-hf-1".to_string(),
        route: "get_hf_token_status".to_string(),
        payload: json!(null),
    };
    
    let response = host.process_message(message).await.expect("HF status route should succeed");
    
    assert!(response.success, "HF status should succeed");
    assert_eq!(response.request_id, "test-hf-1");
    assert!(response.data.is_some(), "HF status response should have data");
}

#[tokio::test]
async fn test_models_list_route() {
    let host = create_test_host("models_list").await;
    
    let message = IncomingMessage {
        request_id: "test-models-1".to_string(),
        route: "models".to_string(),
        payload: json!(null),
    };
    
    let response = host.process_message(message).await.expect("Models list route should succeed");
    
    assert!(response.success, "Models list should succeed. Error: {:?}", response.error);
    assert_eq!(response.request_id, "test-models-1");
    // Models list returns ResponseValue::chat() so data might be in message content
    // With no models loaded, it should return empty list []
}

#[tokio::test]
async fn test_chat_with_nonexistent_model() {
    let host = create_test_host("chat_fail").await;
    
    let message = IncomingMessage {
        request_id: "test-chat-1".to_string(),
        route: "chat".to_string(),
        payload: json!({
            "model": "nonexistent-model",
            "messages": [
                {"role": "user", "content": "Hello"}
            ]
        }),
    };
    
    let response = host.process_message(message).await.expect("Chat route should respond");
    
    // Should fail because model doesn't exist - this is expected!
    assert!(!response.success, "Chat with nonexistent model should fail");
    assert_eq!(response.request_id, "test-chat-1");
    assert!(response.error.is_some(), "Should have error message");
    
    let error = response.error.as_ref().unwrap();
    assert!(
        error.message.contains("not found") || error.message.contains("Model"),
        "Error should mention model not found, got: {}", error.message
    );
}

#[tokio::test]
async fn test_invalid_route() {
    let host = create_test_host("invalid_route").await;
    
    let message = IncomingMessage {
        request_id: "test-invalid-1".to_string(),
        route: "this/route/does/not/exist".to_string(),
        payload: json!(null),
    };
    
    let response = host.process_message(message).await.expect("Should handle invalid route");
    
    assert!(!response.success, "Invalid route should fail");
    assert_eq!(response.request_id, "test-invalid-1");
    assert!(response.error.is_some(), "Should have error for invalid route");
    
    let error = response.error.as_ref().unwrap();
    assert!(
        error.message.to_lowercase().contains("route") || error.message.to_lowercase().contains("not found"),
        "Error should mention route issue, got: {}", error.message
    );
}

#[tokio::test]
async fn test_set_and_check_hf_token() {
    let host = create_test_host("hf_token_flow").await;
    
    // First, set a token
    let set_message = IncomingMessage {
        request_id: "test-hf-set-1".to_string(),
        route: "set_hf_token".to_string(),
        payload: json!({
            "token": "hf_test_token_12345"
        }),
    };
    
    let set_response = host.process_message(set_message).await.expect("Set token should work");
    assert!(set_response.success, "Setting HF token should succeed. Error: {:?}", set_response.error);
    
    // NOTE: HF tokens are stored in OS keyring or global config dir, not test temp dir
    // So we can't reliably test persistence across calls in integration tests
    // Just verify the routes work and don't crash
    let status_message = IncomingMessage {
        request_id: "test-hf-status-1".to_string(),
        route: "get_hf_token_status".to_string(),
        payload: json!(null),
    };
    
    let status_response = host.process_message(status_message).await.expect("Status check should work");
    assert!(status_response.success, "Status check should succeed. Error: {:?}", status_response.error);
    assert!(status_response.data.is_some(), "Should have status data");
    
    // We can't reliably assert token persistence in tests since HF tokens
    // are stored globally, not in test temp dirs
}

#[tokio::test]
async fn test_check_model_feasibility() {
    let host = create_test_host("feasibility").await;
    
    let message = IncomingMessage {
        request_id: "test-feasibility-1".to_string(),
        route: "check_model_feasibility".to_string(),
        payload: json!({
            "model_size_mb": 5000
        }),
    };
    
    let response = host.process_message(message).await.expect("Feasibility check should respond");
    
    assert!(response.success, "Feasibility check should succeed");
    assert!(response.data.is_some(), "Should have feasibility data");
}

#[tokio::test]
async fn test_get_recommended_models() {
    let host = create_test_host("recommended").await;
    
    let message = IncomingMessage {
        request_id: "test-recommended-1".to_string(),
        route: "get_recommended_models".to_string(),
        payload: json!(null),
    };
    
    let response = host.process_message(message).await.expect("Recommended models should respond");
    
    assert!(response.success, "Recommended models should succeed");
    assert!(response.data.is_some(), "Should have recommended models data");
}

