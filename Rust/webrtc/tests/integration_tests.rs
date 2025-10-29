//! Integration tests for WebRTC manager

use tabagent_webrtc::{IceCandidate, WebRtcConfig, WebRtcManager};
use tabagent_values::{RequestValue, ResponseValue, HealthStatus};
use chrono::Utc;
use std::sync::Arc;
use anyhow::Result;

fn create_test_handler() -> Arc<dyn Fn(RequestValue) -> futures::future::BoxFuture<'static, Result<ResponseValue>> + Send + Sync> {
    Arc::new(|_req| Box::pin(async { Ok(ResponseValue::health(HealthStatus::Healthy)) }))
}

#[tokio::test]
async fn test_create_offer() {
    let config = WebRtcConfig::default();
    let handler = create_test_handler();
    let manager = WebRtcManager::new(config, handler);
    
    let session = manager.create_offer("test-client".to_string()).await.unwrap();
    
    assert!(!session.id.is_empty());
    assert_eq!(session.client_id, "test-client");
    assert!(!session.offer_sdp.is_empty());
    assert!(session.answer_sdp.is_none());
    assert_eq!(session.ice_candidates.len(), 0);
}

#[tokio::test]
async fn test_submit_answer() {
    let config = WebRtcConfig::default();
    let handler = create_test_handler();
    let manager = WebRtcManager::new(config, handler);
    
    // Create offer
    let session = manager.create_offer("test-client".to_string()).await.unwrap();
    let session_id = session.id.clone();
    
    // Submit answer
    let answer_sdp = "v=0\r\no=- 123 0 IN IP4 127.0.0.1\r\n".to_string();
    let result = manager.submit_answer(&session_id, answer_sdp).await;
    
    assert!(result.is_ok());
    
    // Verify session state updated
    let session_info = manager.get_session(&session_id).await.unwrap();
    assert!(session_info.has_answer);
}

#[tokio::test]
async fn test_add_ice_candidate() {
    let config = WebRtcConfig::default();
    let handler = create_test_handler();
    let manager = WebRtcManager::new(config, handler);
    
    // Create offer and submit answer first
    let session = manager.create_offer("test-client".to_string()).await.unwrap();
    let session_id = session.id.clone();
    
    let answer_sdp = "v=0\r\no=- 123 0 IN IP4 127.0.0.1\r\n".to_string();
    manager.submit_answer(&session_id, answer_sdp).await.unwrap();
    
    // Add ICE candidate
    let candidate = IceCandidate {
        candidate: "candidate:1 1 UDP 2130706431 127.0.0.1 50000 typ host".to_string(),
        sdp_mid: Some("0".to_string()),
        sdp_mline_index: Some(0),
        added_at: Utc::now(),
    };
    
    let result = manager.add_ice_candidate(&session_id, candidate).await;
    assert!(result.is_ok());
    
    // Verify ICE candidate was added
    let session_info = manager.get_session(&session_id).await.unwrap();
    assert!(session_info.ice_candidate_count > 0);
}

#[tokio::test]
async fn test_get_session() {
    let config = WebRtcConfig::default();
    let handler = create_test_handler();
    let manager = WebRtcManager::new(config, handler);
    
    let session = manager.create_offer("test-client".to_string()).await.unwrap();
    let session_id = session.id.clone();
    
    let session_info = manager.get_session(&session_id).await.unwrap();
    
    assert_eq!(session_info.id, session_id);
    assert_eq!(session_info.client_id, "test-client");
    assert!(session_info.has_offer);
    assert!(!session_info.has_answer);
}

#[tokio::test]
async fn test_list_sessions() {
    let config = WebRtcConfig::default();
    let handler = create_test_handler();
    let manager = WebRtcManager::new(config, handler);
    
    // Create multiple sessions
    manager.create_offer("client1".to_string()).await.unwrap();
    manager.create_offer("client2".to_string()).await.unwrap();
    
    let sessions = manager.list_sessions().await;
    
    assert_eq!(sessions.len(), 2);
}

#[tokio::test]
async fn test_remove_session() {
    let config = WebRtcConfig::default();
    let handler = create_test_handler();
    let manager = WebRtcManager::new(config, handler);
    
    let session = manager.create_offer("test-client".to_string()).await.unwrap();
    let session_id = session.id.clone();
    
    // Remove session
    let result = manager.remove_session(&session_id).await;
    assert!(result.is_ok());
    
    // Verify session is gone
    let result = manager.get_session(&session_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_session_limits() {
    let config = WebRtcConfig {
        max_sessions: 2,
        max_sessions_per_client: 1,
        ..Default::default()
    };
    let handler = create_test_handler();
    let manager = WebRtcManager::new(config, handler);
    
    // First session should succeed
    manager.create_offer("client1".to_string()).await.unwrap();
    
    // Second session with same client should fail
    let result = manager.create_offer("client1".to_string()).await;
    assert!(result.is_err());
    
    // Session with different client should succeed
    manager.create_offer("client2".to_string()).await.unwrap();
    
    // Third session should fail (max_sessions = 2)
    let result = manager.create_offer("client3".to_string()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_mark_connected() {
    let config = WebRtcConfig::default();
    let handler = create_test_handler();
    let manager = WebRtcManager::new(config, handler);
    
    let session = manager.create_offer("test-client".to_string()).await.unwrap();
    let session_id = session.id.clone();
    
    // Mark as connected
    let result = manager.mark_connected(&session_id).await;
    assert!(result.is_ok());
    
    // Verify connection status
    let session_info = manager.get_session(&session_id).await.unwrap();
    assert!(session_info.data_channel_connected);
}

#[tokio::test]
async fn test_get_stats() {
    let config = WebRtcConfig::default();
    let handler = create_test_handler();
    let manager = WebRtcManager::new(config, handler);
    
    let stats = manager.get_stats().await;
    
    assert_eq!(stats.active_sessions, 0);
    assert_eq!(stats.total_sessions, 0);
    
    // Create a session
    manager.create_offer("test-client".to_string()).await.unwrap();
    
    let stats = manager.get_stats().await;
    assert_eq!(stats.active_sessions, 1);
    assert_eq!(stats.total_sessions, 1);
}
