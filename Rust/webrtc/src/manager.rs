//! WebRTC session manager - orchestrates signaling and session lifecycle

use crate::{
    config::WebRtcConfig,
    error::{WebRtcError, WebRtcResult},
    peer_connection::PeerConnectionHandler,
    session::{SessionHandle, SessionState, WebRtcSession},
    types::{IceCandidate, SessionInfo, WebRtcStats},
};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// WebRTC session manager
///
/// Handles WebRTC signaling (offer/answer/ICE) and session lifecycle management.
/// This is the main entry point for WebRTC operations.
pub struct WebRtcManager {
    /// Configuration
    config: WebRtcConfig,
    
    /// Active sessions (session_id → session)
    sessions: Arc<RwLock<HashMap<String, SessionHandle>>>,
    
    /// Sessions by client ID (client_id → session_ids)
    sessions_by_client: Arc<RwLock<HashMap<String, Vec<String>>>>,
    
    /// Statistics
    stats: Arc<RwLock<WebRtcStats>>,
    
    /// AppState handler for processing requests
    app_state_handler: Arc<dyn Fn(tabagent_values::RequestValue) -> futures::future::BoxFuture<'static, Result<tabagent_values::ResponseValue>> + Send + Sync>,
}

impl WebRtcManager {
    /// Create a new WebRTC manager
    pub fn new(
        config: WebRtcConfig,
        app_state_handler: Arc<dyn Fn(tabagent_values::RequestValue) -> futures::future::BoxFuture<'static, Result<tabagent_values::ResponseValue>> + Send + Sync>,
    ) -> Self {
        let manager = Self {
            config,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            sessions_by_client: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(WebRtcStats::default())),
            app_state_handler,
        };
        
        // Start cleanup task
        manager.start_cleanup_task();
        
        manager
    }
    
    /// Create a new WebRTC manager with default config (for testing)
    #[cfg(test)]
    pub fn new_test() -> Self {
        use tabagent_values::{RequestValue, ResponseValue, HealthStatus};
        let handler: Arc<dyn Fn(RequestValue) -> futures::future::BoxFuture<'static, Result<ResponseValue>> + Send + Sync> = 
            Arc::new(|_req| Box::pin(async { Ok(ResponseValue::health(HealthStatus::Healthy)) }));
        Self::new(WebRtcConfig::default(), handler)
    }
    
    /// Create a new WebRTC offer and session
    ///
    /// This is Step 1 of the signaling flow. The client will receive the offer SDP
    /// and create an answer.
    pub async fn create_offer(&self, client_id: String) -> WebRtcResult<WebRtcSession> {
        // Check session limits
        let sessions = self.sessions.read().await;
        if sessions.len() >= self.config.max_sessions {
            return Err(WebRtcError::SessionLimitReached {
                current: sessions.len(),
                max: self.config.max_sessions,
            });
        }
        
        // Check per-client limit
        let sessions_by_client = self.sessions_by_client.read().await;
        if let Some(client_sessions) = sessions_by_client.get(&client_id) {
            if client_sessions.len() >= self.config.max_sessions_per_client {
                return Err(WebRtcError::SessionLimitReached {
                    current: client_sessions.len(),
                    max: self.config.max_sessions_per_client,
                });
            }
        }
        drop(sessions);
        drop(sessions_by_client);
        
        // Generate session ID
        let session_id = Uuid::new_v4().to_string();
        
        // Create REAL peer connection
        let peer_connection = PeerConnectionHandler::new(&self.config, self.app_state_handler.clone())
            .await
            .map_err(|e| WebRtcError::InternalError(format!("Failed to create peer connection: {}", e)))?;
        
        let peer_connection = Arc::new(peer_connection);
        
        // Generate REAL SDP offer
        let offer_sdp = peer_connection.create_offer().await?;
        
        // Create session with real peer connection
        let session = WebRtcSession::with_peer_connection(
            session_id.clone(),
            client_id.clone(),
            offer_sdp.clone(),
            peer_connection,
        );
        let session_handle = Arc::new(RwLock::new(session));
        
        // Store session
        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), session_handle);
        
        // Track by client
        let mut sessions_by_client = self.sessions_by_client.write().await;
        sessions_by_client
            .entry(client_id.clone())
            .or_insert_with(Vec::new)
            .push(session_id.clone());
        
        // Update stats
        let mut stats = self.stats.write().await;
        stats.total_sessions += 1;
        stats.active_sessions += 1;
        stats.waiting_sessions += 1;
        
        // Save IDs for return (we need to clone before moving)
        let return_session_id = session_id.clone();
        let return_client_id = client_id.clone();
        let return_offer_sdp = offer_sdp.clone();
        
        tracing::info!("Created WebRTC offer for session {}", session_id);
        
        // Return a simplified session info for the response
        Ok(WebRtcSession::new(return_session_id, return_client_id, return_offer_sdp))
    }
    
    /// Submit answer from client
    ///
    /// This is Step 2 of the signaling flow. The client has created an answer
    /// to our offer.
    pub async fn submit_answer(
        &self,
        session_id: &str,
        answer_sdp: String,
    ) -> WebRtcResult<()> {
        let sessions = self.sessions.read().await;
        let session_handle = sessions
            .get(session_id)
            .ok_or_else(|| WebRtcError::SessionNotFound(session_id.to_string()))?;
        
        let mut session = session_handle.write().await;
        
        // Validate state
        if session.state != SessionState::WaitingForAnswer {
            return Err(WebRtcError::InvalidState {
                session_id: session_id.to_string(),
                expected: "waiting_for_answer".to_string(),
                actual: session.state.as_str().to_string(),
            });
        }
        
        // Set answer on REAL peer connection
        if let Some(peer_connection) = &session.peer_connection {
            peer_connection.set_answer(answer_sdp.clone()).await?;
        }
        
        // Store answer and transition to ICE gathering
        session.set_answer(answer_sdp);
        
        // Update stats
        let mut stats = self.stats.write().await;
        stats.waiting_sessions = stats.waiting_sessions.saturating_sub(1);
        
        tracing::info!("Received answer for session {}", session_id);
        
        Ok(())
    }
    
    /// Add ICE candidate
    ///
    /// This is Step 3 of the signaling flow (repeated multiple times).
    /// The client sends ICE candidates as they are discovered.
    pub async fn add_ice_candidate(
        &self,
        session_id: &str,
        candidate: IceCandidate,
    ) -> WebRtcResult<()> {
        let sessions = self.sessions.read().await;
        let session_handle = sessions
            .get(session_id)
            .ok_or_else(|| WebRtcError::SessionNotFound(session_id.to_string()))?;
        
        let mut session = session_handle.write().await;
        
        // Can only add ICE candidates after answer
        if session.answer_sdp.is_none() {
            return Err(WebRtcError::InvalidState {
                session_id: session_id.to_string(),
                expected: "has_answer".to_string(),
                actual: "no_answer".to_string(),
            });
        }
        
        // Add to REAL peer connection
        if let Some(peer_connection) = &session.peer_connection {
            peer_connection.add_ice_candidate(candidate.candidate.clone()).await?;
        }
        
        session.add_ice_candidate(candidate);
        
        tracing::debug!(
            "Added ICE candidate for session {} (total: {})",
            session_id,
            session.ice_candidates.len()
        );
        
        Ok(())
    }
    
    /// Get session information
    pub async fn get_session(&self, session_id: &str) -> WebRtcResult<SessionInfo> {
        let sessions = self.sessions.read().await;
        let session_handle = sessions
            .get(session_id)
            .ok_or_else(|| WebRtcError::SessionNotFound(session_id.to_string()))?;
        
        let session = session_handle.read().await;
        
        Ok(SessionInfo {
            id: session.id.clone(),
            client_id: session.client_id.clone(),
            state: session.state.as_str().to_string(),
            created_at: session.created_at,
            last_activity: session.last_activity,
            data_channel_connected: session.data_channel_connected,
            ice_candidate_count: session.ice_candidates.len(),
            has_offer: true,
            has_answer: session.answer_sdp.is_some(),
        })
    }
    
    /// List all sessions
    pub async fn list_sessions(&self) -> Vec<SessionInfo> {
        let sessions = self.sessions.read().await;
        let mut infos = Vec::new();
        
        for session_handle in sessions.values() {
            let session = session_handle.read().await;
            infos.push(SessionInfo {
                id: session.id.clone(),
                client_id: session.client_id.clone(),
                state: session.state.as_str().to_string(),
                created_at: session.created_at,
                last_activity: session.last_activity,
                data_channel_connected: session.data_channel_connected,
                ice_candidate_count: session.ice_candidates.len(),
                has_offer: true,
                has_answer: session.answer_sdp.is_some(),
            });
        }
        
        infos
    }
    
    /// Mark session as connected (data channel opened)
    pub async fn mark_connected(&self, session_id: &str) -> WebRtcResult<()> {
        let sessions = self.sessions.read().await;
        let session_handle = sessions
            .get(session_id)
            .ok_or_else(|| WebRtcError::SessionNotFound(session_id.to_string()))?;
        
        let mut session = session_handle.write().await;
        session.mark_connected();
        
        // Update stats
        let mut stats = self.stats.write().await;
        stats.connected_sessions += 1;
        
        tracing::info!("Session {} connected", session_id);
        
        Ok(())
    }
    
    /// Mark session as disconnected
    pub async fn mark_disconnected(&self, session_id: &str) -> WebRtcResult<()> {
        let sessions = self.sessions.read().await;
        let session_handle = sessions
            .get(session_id)
            .ok_or_else(|| WebRtcError::SessionNotFound(session_id.to_string()))?;
        
        let mut session = session_handle.write().await;
        
        let was_connected = session.data_channel_connected;
        session.mark_disconnected();
        
        // Update stats
        if was_connected {
            let mut stats = self.stats.write().await;
            stats.connected_sessions = stats.connected_sessions.saturating_sub(1);
            stats.disconnected_sessions += 1;
        }
        
        tracing::info!("Session {} disconnected", session_id);
        
        Ok(())
    }
    
    /// Remove a session
    pub async fn remove_session(&self, session_id: &str) -> WebRtcResult<()> {
        let mut sessions = self.sessions.write().await;
        let session_handle = sessions
            .remove(session_id)
            .ok_or_else(|| WebRtcError::SessionNotFound(session_id.to_string()))?;
        
        let session = session_handle.read().await;
        let client_id = session.client_id.clone();
        drop(session);
        
        // Remove from client tracking
        let mut sessions_by_client = self.sessions_by_client.write().await;
        if let Some(client_sessions) = sessions_by_client.get_mut(&client_id) {
            client_sessions.retain(|id| id != session_id);
            if client_sessions.is_empty() {
                sessions_by_client.remove(&client_id);
            }
        }
        
        // Update stats
        let mut stats = self.stats.write().await;
        stats.active_sessions = stats.active_sessions.saturating_sub(1);
        stats.sessions_cleaned += 1;
        
        tracing::info!("Removed session {}", session_id);
        
        Ok(())
    }
    
    /// Get statistics
    pub async fn get_stats(&self) -> WebRtcStats {
        self.stats.read().await.clone()
    }
    
    
    /// Start background cleanup task
    fn start_cleanup_task(&self) {
        let sessions = self.sessions.clone();
        let sessions_by_client = self.sessions_by_client.clone();
        let stats = self.stats.clone();
        let timeout = self.config.session_timeout;
        let interval = self.config.cleanup_interval;
        
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            
            loop {
                interval_timer.tick().await;
                
                // Find stale sessions
                let mut to_remove = Vec::new();
                {
                    let sessions_guard = sessions.read().await;
                    for (session_id, session_handle) in sessions_guard.iter() {
                        let session = session_handle.read().await;
                        if session.is_stale(timeout) || session.is_terminal() {
                            to_remove.push(session_id.clone());
                        }
                    }
                }
                
                // Remove stale sessions
                if !to_remove.is_empty() {
                    tracing::info!("Cleaning up {} stale sessions", to_remove.len());
                    
                    let mut sessions_guard = sessions.write().await;
                    let mut sessions_by_client_guard = sessions_by_client.write().await;
                    let mut stats_guard = stats.write().await;
                    
                    for session_id in to_remove {
                        if let Some(session_handle) = sessions_guard.remove(&session_id) {
                            let session = session_handle.read().await;
                            let client_id = session.client_id.clone();
                            drop(session);
                            
                            // Remove from client tracking
                            if let Some(client_sessions) = sessions_by_client_guard.get_mut(&client_id) {
                                client_sessions.retain(|id| id != &session_id);
                                if client_sessions.is_empty() {
                                    sessions_by_client_guard.remove(&client_id);
                                }
                            }
                            
                            stats_guard.active_sessions = stats_guard.active_sessions.saturating_sub(1);
                            stats_guard.sessions_cleaned += 1;
                        }
                    }
                }
            }
        });
    }
}

