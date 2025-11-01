//! Integration tests for tabagent-mcp
//!
//! Tests the full MCP transport with real storage engine and AppState

use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;

use tabagent_mcp::McpManager;
use common::logging::{LogEntry, LogLevel, LogQuery, LogSource};
use storage::DatabaseCoordinator;
use appstate::{AppState, AppStateConfig};
use tabagent_values::{RequestValue, ResponseValue};

/// Create a test MCP manager with real storage and state
async fn create_test_manager() -> (McpManager, TempDir, Arc<AppState>) {
    // Create temporary directory for test databases
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Initialize AppState with temp directory
    let config = AppStateConfig {
        db_path: temp_path.join("test_db"),
        model_cache_path: temp_path.join("test_models"),
    };
    let state = Arc::new(AppState::new(config).await.unwrap());
    
    // Initialize storage coordinator with temp path
    let coordinator = Arc::new(
        DatabaseCoordinator::with_base_path(Some(temp_path.join("storage_db").into()))
            .unwrap()
    );
    
    // Create MCP manager
    let manager = McpManager::new(state.clone(), coordinator);
    
    (manager, temp_dir, state)
}

#[tokio::test]
async fn test_store_and_query_logs() {
    let (manager, _temp_dir, _state) = create_test_manager().await;
    
    // Store test logs
    let log1 = LogEntry::new(
        LogLevel::Error,
        "test_context".to_string(),
        "Test error message".to_string(),
        LogSource::Storage,
    );
    let log2 = LogEntry::new(
        LogLevel::Info,
        "test_context".to_string(),
        "Test info message".to_string(),
        LogSource::Server,
    );
    
    manager.store_log(log1.clone()).unwrap();
    manager.store_log(log2.clone()).unwrap();
    
    // Query all logs
    let query = LogQuery::default();
    let logs = manager.query_logs(query).unwrap();
    assert_eq!(logs.len(), 2);
    
    // Verify we can find both logs
    let ids: Vec<_> = logs.iter().map(|l| l.id.clone()).collect();
    assert!(ids.contains(&log1.id));
    assert!(ids.contains(&log2.id));
}

#[tokio::test]
async fn test_query_logs_by_level() {
    let (manager, _temp_dir, _state) = create_test_manager().await;
    
    // Store logs with different levels
    manager.store_log(LogEntry::new(
        LogLevel::Error,
        "context".to_string(),
        "Error message".to_string(),
        LogSource::Storage,
    )).unwrap();
    
    manager.store_log(LogEntry::new(
        LogLevel::Info,
        "context".to_string(),
        "Info message".to_string(),
        LogSource::Storage,
    )).unwrap();
    
    // Query only errors
    let mut query = LogQuery::default();
    query.level = Some("error".to_string());
    let logs = manager.query_logs(query).unwrap();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].level, LogLevel::Error);
}

#[tokio::test]
async fn test_query_logs_by_source() {
    let (manager, _temp_dir, _state) = create_test_manager().await;
    
    manager.store_log(LogEntry::new(
        LogLevel::Info,
        "context".to_string(),
        "Message 1".to_string(),
        LogSource::Storage,
    )).unwrap();
    
    manager.store_log(LogEntry::new(
        LogLevel::Info,
        "context".to_string(),
        "Message 2".to_string(),
        LogSource::Server,
    )).unwrap();
    
    // Query only storage logs
    let mut query = LogQuery::default();
    query.source = Some("storage".to_string());
    let logs = manager.query_logs(query).unwrap();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].source, LogSource::Storage);
}

#[tokio::test]
async fn test_query_logs_with_limit() {
    let (manager, _temp_dir, _state) = create_test_manager().await;
    
    // Store 5 logs
    for i in 0..5 {
        manager.store_log(LogEntry::new(
            LogLevel::Info,
            "context".to_string(),
            format!("Message {}", i),
            LogSource::Storage,
        )).unwrap();
    }
    
    // Query with limit of 2
    let mut query = LogQuery::default();
    query.limit = Some(2);
    let logs = manager.query_logs(query).unwrap();
    assert_eq!(logs.len(), 2);
}

#[tokio::test]
async fn test_query_logs_validation() {
    let (manager, _temp_dir, _state) = create_test_manager().await;
    
    // Test invalid limit (too large)
    let mut query = LogQuery::default();
    query.limit = Some(10001);
    assert!(manager.query_logs(query).is_err());
    
    // Test invalid limit (zero)
    let mut query = LogQuery::default();
    query.limit = Some(0);
    assert!(manager.query_logs(query).is_err());
    
    // Test invalid timestamp
    let mut query = LogQuery::default();
    query.since = Some("not-a-timestamp".to_string());
    assert!(manager.query_logs(query).is_err());
}

#[tokio::test]
async fn test_get_log_stats() {
    let (manager, _temp_dir, _state) = create_test_manager().await;
    
    // Store logs with different characteristics
    manager.store_log(LogEntry::new(
        LogLevel::Error,
        "context1".to_string(),
        "Error 1".to_string(),
        LogSource::Storage,
    )).unwrap();
    
    manager.store_log(LogEntry::new(
        LogLevel::Error,
        "context2".to_string(),
        "Error 2".to_string(),
        LogSource::Storage,
    )).unwrap();
    
    manager.store_log(LogEntry::new(
        LogLevel::Info,
        "context1".to_string(),
        "Info".to_string(),
        LogSource::Server,
    )).unwrap();
    
    let stats = manager.get_log_stats().unwrap();
    
    assert_eq!(stats.total_logs, 3);
    assert_eq!(*stats.by_level.get("error").unwrap(), 2);
    assert_eq!(*stats.by_level.get("info").unwrap(), 1);
    assert_eq!(*stats.by_source.get("Storage").unwrap(), 2);
    assert_eq!(*stats.by_source.get("Server").unwrap(), 1);
    assert_eq!(*stats.by_context.get("context1").unwrap(), 2);
    assert_eq!(*stats.by_context.get("context2").unwrap(), 1);
}

#[tokio::test]
async fn test_clear_logs() {
    let (manager, _temp_dir, _state) = create_test_manager().await;
    
    // Store some logs
    for _ in 0..3 {
        manager.store_log(LogEntry::new(
            LogLevel::Info,
            "context".to_string(),
            "Message".to_string(),
            LogSource::Storage,
        )).unwrap();
    }
    
    // Verify they exist
    let logs_before = manager.query_logs(LogQuery::default()).unwrap();
    assert_eq!(logs_before.len(), 3);
    
    // Clear all logs
    let cleared = manager.clear_logs().unwrap();
    assert_eq!(cleared, 3);
    
    // Verify they're gone
    let logs_after = manager.query_logs(LogQuery::default()).unwrap();
    assert_eq!(logs_after.len(), 0);
}

#[tokio::test]
async fn test_model_operations() {
    let (manager, _temp_dir, state) = create_test_manager().await;
    
    // Test list_models - should work with real AppState
    let result = manager.list_models().await;
    assert!(result.is_ok());
    
    // Test get_system_info
    let result = manager.get_system_info().await;
    assert!(result.is_ok());
    
    // Test health_check
    let result = manager.health_check().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_log_query_optimization() {
    let (manager, _temp_dir, _state) = create_test_manager().await;
    
    // Store many logs
    for i in 0..1000 {
        manager.store_log(LogEntry::new(
            if i % 2 == 0 { LogLevel::Info } else { LogLevel::Error },
            "context".to_string(),
            format!("Message {}", i),
            LogSource::Storage,
        )).unwrap();
    }
    
    // Query with limit - should stop early
    let mut query = LogQuery::default();
    query.level = Some("error".to_string());
    query.limit = Some(50);
    
    let start = std::time::Instant::now();
    let logs = manager.query_logs(query).unwrap();
    let duration = start.elapsed();
    
    // Should return exactly limit (50), not all 500 errors
    assert_eq!(logs.len(), 50);
    
    // Should be fast (early termination optimization)
    assert!(duration.as_millis() < 5000, "Query took too long: {:?}", duration);
}

