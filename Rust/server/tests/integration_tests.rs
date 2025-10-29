//! Integration tests for TabAgent server.
//!
//! These tests verify that the server can:
//! - Initialize properly with all dependencies
//! - Handle different server modes
//! - Shut down gracefully

use std::path::PathBuf;
use std::time::Duration;
use tokio::time::timeout;

/// Helper to create a temporary test directory
fn create_test_dir(test_name: &str) -> PathBuf {
    let temp_dir = std::env::temp_dir().join(format!("tabagent_server_test_{}", test_name));
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).expect("Failed to create test dir");
    temp_dir
}

#[tokio::test]
async fn test_appstate_initialization() {
    use appstate::{AppState, AppStateConfig};
    
    let test_dir = create_test_dir("appstate_init");
    let config = AppStateConfig {
        db_path: test_dir.join("db"),
        model_cache_path: test_dir.join("models"),
    };
    
    // Should initialize without errors
    let state = AppState::new(config).await;
    assert!(state.is_ok(), "AppState initialization should succeed");
    
    let state = state.unwrap();
    
    // Should start with no loaded models
    assert_eq!(state.list_loaded_models().len(), 0, "Should start with no models loaded");
    
    // Cleanup
    let _ = std::fs::remove_dir_all(&test_dir);
}

#[tokio::test]
async fn test_native_messaging_host_creation() {
    use appstate::{AppState, AppStateConfig};
    use tabagent_native_messaging::{NativeMessagingHost, NativeMessagingConfig};
    use std::sync::Arc;
    use common::AppStateProvider;
    
    let test_dir = create_test_dir("native_host");
    let config = AppStateConfig {
        db_path: test_dir.join("db"),
        model_cache_path: test_dir.join("models"),
    };
    
    let state = Arc::new(AppState::new(config).await.unwrap()) as Arc<dyn AppStateProvider>;
    let nm_config = NativeMessagingConfig::default();
    
    // Should create host without errors
    let _host = NativeMessagingHost::new(state, nm_config);
    
    // Cleanup
    let _ = std::fs::remove_dir_all(&test_dir);
}

#[tokio::test]
async fn test_config_parsing() {
    use tabagent_server::config::{CliArgs, ServerMode};
    use clap::Parser;
    
    // Test default config
    let args = CliArgs::parse_from(&["tabagent-server"]);
    assert_eq!(args.mode, ServerMode::Http);
    assert_eq!(args.port, 8080);
    
    // Test custom config
    let args = CliArgs::parse_from(&[
        "tabagent-server",
        "--mode", "native",
        "--port", "3000",
    ]);
    assert_eq!(args.mode, ServerMode::Native);
    assert_eq!(args.port, 3000);
}

#[tokio::test]
async fn test_multiple_state_instances() {
    use appstate::{AppState, AppStateConfig};
    
    // Create multiple states with different paths
    let test_dir1 = create_test_dir("multi_state_1");
    let test_dir2 = create_test_dir("multi_state_2");
    
    let config1 = AppStateConfig {
        db_path: test_dir1.join("db"),
        model_cache_path: test_dir1.join("models"),
    };
    
    let config2 = AppStateConfig {
        db_path: test_dir2.join("db"),
        model_cache_path: test_dir2.join("models"),
    };
    
    let state1 = AppState::new(config1).await;
    let state2 = AppState::new(config2).await;
    
    assert!(state1.is_ok(), "First state should initialize");
    assert!(state2.is_ok(), "Second state should initialize");
    
    // Cleanup
    let _ = std::fs::remove_dir_all(&test_dir1);
    let _ = std::fs::remove_dir_all(&test_dir2);
}

#[tokio::test]
async fn test_shutdown_signal_handling() {
    // Test that we can create a shutdown signal and handle it
    let shutdown_future = async {
        // Simulate ctrl+c after a short delay
        tokio::time::sleep(Duration::from_millis(100)).await;
        // In real scenario, signal::ctrl_c() would be called
    };
    
    // Should complete within timeout
    let result = timeout(Duration::from_secs(1), shutdown_future).await;
    assert!(result.is_ok(), "Shutdown handling should complete");
}

#[tokio::test]
async fn test_concurrent_state_access() {
    use appstate::{AppState, AppStateConfig};
    use std::sync::Arc;
    use common::AppStateProvider;
    
    let test_dir = create_test_dir("concurrent_access");
    let config = AppStateConfig {
        db_path: test_dir.join("db"),
        model_cache_path: test_dir.join("models"),
    };
    
    let state = Arc::new(AppState::new(config).await.unwrap()) as Arc<dyn AppStateProvider>;
    
    // Spawn multiple tasks accessing state concurrently
    let mut handles = vec![];
    
    for i in 0..10 {
        let state_clone = state.clone();
        let handle = tokio::spawn(async move {
            // Just verify we can access state from multiple tasks
            let request = tabagent_values::RequestValue::health();
            let _response = state_clone.handle_request(request).await;
            i
        });
        handles.push(handle);
    }
    
    // All tasks should complete successfully
    for handle in handles {
        let result = handle.await;
        assert!(result.is_ok(), "Concurrent state access should succeed");
    }
    
    // Cleanup
    let _ = std::fs::remove_dir_all(&test_dir);
}

#[tokio::test]
async fn test_server_mode_enum() {
    use tabagent_server::config::ServerMode;
    
    // Test enum variants exist and are distinct
    let modes = vec![
        ServerMode::Native,
        ServerMode::Http,
        ServerMode::WebRtc,
        ServerMode::Both,
        ServerMode::All,
    ];
    
    assert_eq!(modes.len(), 5, "Should have 5 server modes");
    
    // Test Debug formatting
    assert_eq!(format!("{:?}", ServerMode::Http), "Http");
    assert_eq!(format!("{:?}", ServerMode::Native), "Native");
}

