//! Integration tests for WebRTC manager

use tabagent_webrtc::{IceCandidate, WebRtcConfig, WebRtcManager};
use chrono::Utc;

#[tokio::test]
async fn test_create_offer() {
    let config = WebRtcConfig::default();
    let manager = WebRtcManager::new(config);
    
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
    let manager = WebRtcManager::new(config);
    
    // Create offer
    let session = manager.create_offer("test-client".to_string()).await.unwrap();
    let session_id = session.id.clone();
    
    // Submit answer
    let answer_sdp = "v=0\r\no=- 123 0 IN IP4 127.0.0.1\r\n".to_string();
    manager.submit_answer(&session_id, answer_sdp.clone()).await.unwrap();
    
    // Verify session state
    let session_info = manager.get_session(&session_id).await.unwrap();
    assert_eq!(session_info.state, "ice_gathering");
    assert!(session_info.has_answer);
}

#[tokio::test]
async fn test_add_ice_candidate() {
    let config = WebRtcConfig::default();
    let manager = WebRtcManager::new(config);
    
    // Create offer and submit answer
    let session = manager.create_offer("test-client".to_string()).await.unwrap();
    let session_id = session.id.clone();
    manager.submit_answer(&session_id, "answer".to_string()).await.unwrap();
    
    // Add ICE candidate
    let candidate = IceCandidate {
        candidate: "candidate:1 1 UDP 2130706431 192.168.1.1 54321 typ host".to_string(),
        sdp_mid: Some("0".to_string()),
        sdp_mline_index: Some(0),
        added_at: Utc::now(),
    };
    
    manager.add_ice_candidate(&session_id, candidate).await.unwrap();
    
    // Verify
    let session_info = manager.get_session(&session_id).await.unwrap();
    assert_eq!(session_info.ice_candidate_count, 1);
}

#[tokio::test]
async fn test_list_sessions() {
    let config = WebRtcConfig::default();
    let manager = WebRtcManager::new(config);
    
    // Create multiple sessions
    manager.create_offer("client-1".to_string()).await.unwrap();
    manager.create_offer("client-2".to_string()).await.unwrap();
    manager.create_offer("client-3".to_string()).await.unwrap();
    
    // List sessions
    let sessions = manager.list_sessions().await;
    assert_eq!(sessions.len(), 3);
}

#[tokio::test]
async fn test_session_limit() {
    let mut config = WebRtcConfig::default();
    config.max_sessions = 2;
    let manager = WebRtcManager::new(config);
    
    // Create 2 sessions (should succeed)
    manager.create_offer("client-1".to_string()).await.unwrap();
    manager.create_offer("client-2".to_string()).await.unwrap();
    
    // Try to create 3rd session (should fail)
    let result = manager.create_offer("client-3".to_string()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_per_client_limit() {
    let mut config = WebRtcConfig::default();
    config.max_sessions_per_client = 2;
    let manager = WebRtcManager::new(config);
    
    // Create 2 sessions for same client (should succeed)
    manager.create_offer("client-1".to_string()).await.unwrap();
    manager.create_offer("client-1".to_string()).await.unwrap();
    
    // Try to create 3rd session for same client (should fail)
    let result = manager.create_offer("client-1".to_string()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_mark_connected() {
    let config = WebRtcConfig::default();
    let manager = WebRtcManager::new(config);
    
    let session = manager.create_offer("test-client".to_string()).await.unwrap();
    let session_id = session.id.clone();
    
    // Mark as connected
    manager.mark_connected(&session_id).await.unwrap();
    
    // Verify
    let session_info = manager.get_session(&session_id).await.unwrap();
    assert_eq!(session_info.state, "connected");
    assert!(session_info.data_channel_connected);
    
    // Check stats
    let stats = manager.get_stats().await;
    assert_eq!(stats.connected_sessions, 1);
}

#[tokio::test]
async fn test_mark_disconnected() {
    let config = WebRtcConfig::default();
    let manager = WebRtcManager::new(config);
    
    let session = manager.create_offer("test-client".to_string()).await.unwrap();
    let session_id = session.id.clone();
    
    // Connect then disconnect
    manager.mark_connected(&session_id).await.unwrap();
    manager.mark_disconnected(&session_id).await.unwrap();
    
    // Verify
    let session_info = manager.get_session(&session_id).await.unwrap();
    assert_eq!(session_info.state, "disconnected");
    assert!(!session_info.data_channel_connected);
}

#[tokio::test]
async fn test_remove_session() {
    let config = WebRtcConfig::default();
    let manager = WebRtcManager::new(config);
    
    let session = manager.create_offer("test-client".to_string()).await.unwrap();
    let session_id = session.id.clone();
    
    // Remove session
    manager.remove_session(&session_id).await.unwrap();
    
    // Verify it's gone
    let result = manager.get_session(&session_id).await;
    assert!(result.is_err());
    
    // Check stats
    let stats = manager.get_stats().await;
    assert_eq!(stats.active_sessions, 0);
    assert_eq!(stats.sessions_cleaned, 1);
}

#[tokio::test]
async fn test_stats_tracking() {
    let config = WebRtcConfig::default();
    let manager = WebRtcManager::new(config);
    
    // Create sessions
    manager.create_offer("client-1".to_string()).await.unwrap();
    manager.create_offer("client-2".to_string()).await.unwrap();
    
    let stats = manager.get_stats().await;
    assert_eq!(stats.total_sessions, 2);
    assert_eq!(stats.active_sessions, 2);
    assert_eq!(stats.waiting_sessions, 2);
}

