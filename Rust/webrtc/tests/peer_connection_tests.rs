//! Integration tests for real WebRTC peer connections

use tabagent_webrtc::{WebRtcConfig, WebRtcManager, PeerConnectionHandler};
use tabagent_values::{RequestValue, ResponseValue, HealthStatus};
use std::sync::Arc;
use anyhow::Result;

/// Test basic peer connection creation
#[tokio::test]
async fn test_peer_connection_creation() -> Result<()> {
    let config = WebRtcConfig::default();
    
    let handler: Arc<dyn Fn(RequestValue) -> futures::future::BoxFuture<'static, Result<ResponseValue>> + Send + Sync> = 
        Arc::new(|_req| Box::pin(async { Ok(ResponseValue::health(HealthStatus::Healthy)) }));
    
    let pc_handler = PeerConnectionHandler::new(&config, handler).await?;
    
    // Should be able to create a peer connection
    assert!(true, "Peer connection created successfully");
    
    Ok(())
}

/// Test SDP offer generation
#[tokio::test]
async fn test_sdp_offer_generation() -> Result<()> {
    let config = WebRtcConfig::default();
    
    let handler: Arc<dyn Fn(RequestValue) -> futures::future::BoxFuture<'static, Result<ResponseValue>> + Send + Sync> = 
        Arc::new(|_req| Box::pin(async { Ok(ResponseValue::health(HealthStatus::Healthy)) }));
    
    let pc_handler = PeerConnectionHandler::new(&config, handler).await?;
    let offer_sdp = pc_handler.create_offer().await?;
    
    // Verify SDP format
    assert!(offer_sdp.contains("v=0"), "SDP should have version");
    assert!(offer_sdp.contains("m=application"), "SDP should include data channel");
    
    Ok(())
}

/// Test WebRtcManager with real peer connection
#[tokio::test]
async fn test_manager_with_real_peer_connection() -> Result<()> {
    let config = WebRtcConfig::default();
    
    let handler: Arc<dyn Fn(RequestValue) -> futures::future::BoxFuture<'static, Result<ResponseValue>> + Send + Sync> = 
        Arc::new(|_req| Box::pin(async { Ok(ResponseValue::health(HealthStatus::Healthy)) }));
    
    let manager = Arc::new(WebRtcManager::new(config, handler));
    
    // Create offer
    let session = manager.create_offer("test-client".to_string()).await?;
    
    // Verify session
    assert!(!session.id.is_empty(), "Session should have ID");
    assert!(!session.offer_sdp.is_empty(), "Session should have offer SDP");
    assert!(session.offer_sdp.contains("v=0"), "Offer SDP should be valid");
    
    // Verify we can retrieve the session
    let retrieved = manager.get_session(&session.id).await?;
    assert_eq!(retrieved.id, session.id);
    assert!(retrieved.has_offer);
    assert!(!retrieved.has_answer);
    
    Ok(())
}

/// Test full signaling flow (offer → answer → ICE)
#[tokio::test]
async fn test_full_signaling_flow() -> Result<()> {
    let config = WebRtcConfig::default();
    
    let handler: Arc<dyn Fn(RequestValue) -> futures::future::BoxFuture<'static, Result<ResponseValue>> + Send + Sync> = 
        Arc::new(|_req| Box::pin(async { Ok(ResponseValue::health(HealthStatus::Healthy)) }));
    
    let manager = Arc::new(WebRtcManager::new(config, handler));
    
    // Step 1: Create offer
    let session = manager.create_offer("test-client".to_string()).await?;
    assert!(session.offer_sdp.contains("v=0"));
    
    // Step 2: Submit answer (mock SDP)
    let mock_answer = format!(
        "v=0\r\n\
         o=- {} 0 IN IP4 127.0.0.1\r\n\
         s=TabAgent Client\r\n\
         t=0 0\r\n\
         m=application 9 UDP/DTLS/SCTP webrtc-datachannel\r\n\
         c=IN IP4 0.0.0.0\r\n\
         a=ice-ufrag:client-ufrag\r\n\
         a=ice-pwd:client-pwd\r\n\
         a=fingerprint:sha-256 AA:BB:CC:DD:EE:FF\r\n\
         a=setup:active\r\n",
        session.id
    );
    
    manager.submit_answer(&session.id, mock_answer).await?;
    
    // Verify session state updated
    let updated_session = manager.get_session(&session.id).await?;
    assert!(updated_session.has_answer, "Session should have answer");
    
    // Step 3: Add ICE candidate
    use chrono::Utc;
    let ice_candidate = tabagent_webrtc::IceCandidate {
        candidate: "candidate:1 1 UDP 2130706431 127.0.0.1 50000 typ host".to_string(),
        sdp_mid: Some("0".to_string()),
        sdp_mline_index: Some(0),
        added_at: Utc::now(),
    };
    
    manager.add_ice_candidate(&session.id, ice_candidate).await?;
    
    // Verify ICE candidate added
    let final_session = manager.get_session(&session.id).await?;
    assert!(final_session.ice_candidate_count > 0, "Should have ICE candidates");
    
    Ok(())
}

/// Test session cleanup
#[tokio::test]
async fn test_session_removal() -> Result<()> {
    let config = WebRtcConfig::default();
    
    let handler: Arc<dyn Fn(RequestValue) -> futures::future::BoxFuture<'static, Result<ResponseValue>> + Send + Sync> = 
        Arc::new(|_req| Box::pin(async { Ok(ResponseValue::health(HealthStatus::Healthy)) }));
    
    let manager = Arc::new(WebRtcManager::new(config, handler));
    
    // Create session
    let session = manager.create_offer("test-client".to_string()).await?;
    let session_id = session.id.clone();
    
    // Remove session
    manager.remove_session(&session_id).await?;
    
    // Verify session is gone
    let result = manager.get_session(&session_id).await;
    assert!(result.is_err(), "Session should not exist after removal");
    
    Ok(())
}

