//! WebRTC session state management

use crate::{
    peer_connection::PeerConnectionHandler,
    types::IceCandidate,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// WebRTC session state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionState {
    /// Offer created, waiting for answer from client
    WaitingForAnswer,
    
    /// Answer received, ICE gathering in progress
    IceGathering,
    
    /// Data channel connected and ready
    Connected,
    
    /// Connection failed or was closed
    Disconnected,
}

impl SessionState {
    /// Convert session state to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::WaitingForAnswer => "waiting_for_answer",
            Self::IceGathering => "ice_gathering",
            Self::Connected => "connected",
            Self::Disconnected => "disconnected",
        }
    }
}

/// A WebRTC session representing a connection to a Chrome extension
pub struct WebRtcSession {
    /// Unique session ID
    pub id: String,
    
    /// Client ID (from extension)
    pub client_id: String,
    
    /// Current session state
    pub state: SessionState,
    
    /// When this session was created
    pub created_at: DateTime<Utc>,
    
    /// Last activity timestamp (updated on any operation)
    pub last_activity: DateTime<Utc>,
    
    /// SDP offer from server
    pub offer_sdp: String,
    
    /// SDP answer from client (None until received)
    pub answer_sdp: Option<String>,
    
    /// Collected ICE candidates
    pub ice_candidates: Vec<IceCandidate>,
    
    /// Whether data channel is connected
    pub data_channel_connected: bool,
    
    /// Real WebRTC peer connection handler
    pub peer_connection: Option<Arc<PeerConnectionHandler>>,
}

impl WebRtcSession {
    /// Create a new session with an offer
    pub fn new(id: String, client_id: String, offer_sdp: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            client_id,
            state: SessionState::WaitingForAnswer,
            created_at: now,
            last_activity: now,
            offer_sdp,
            answer_sdp: None,
            ice_candidates: Vec::new(),
            data_channel_connected: false,
            peer_connection: None,
        }
    }
    
    /// Create a new session with a real peer connection
    pub fn with_peer_connection(
        id: String,
        client_id: String,
        offer_sdp: String,
        peer_connection: Arc<PeerConnectionHandler>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            client_id,
            state: SessionState::WaitingForAnswer,
            created_at: now,
            last_activity: now,
            offer_sdp,
            answer_sdp: None,
            ice_candidates: Vec::new(),
            data_channel_connected: false,
            peer_connection: Some(peer_connection),
        }
    }
    
    /// Update last activity timestamp
    pub fn touch(&mut self) {
        self.last_activity = Utc::now();
    }
    
    /// Set answer and transition to ICE gathering
    pub fn set_answer(&mut self, answer_sdp: String) {
        self.answer_sdp = Some(answer_sdp);
        self.state = SessionState::IceGathering;
        self.touch();
    }
    
    /// Add an ICE candidate
    pub fn add_ice_candidate(&mut self, candidate: IceCandidate) {
        self.ice_candidates.push(candidate);
        self.touch();
    }
    
    /// Mark data channel as connected
    pub fn mark_connected(&mut self) {
        self.state = SessionState::Connected;
        self.data_channel_connected = true;
        self.touch();
    }
    
    /// Mark as disconnected
    pub fn mark_disconnected(&mut self) {
        self.state = SessionState::Disconnected;
        self.data_channel_connected = false;
        self.touch();
    }
    
    /// Check if session is stale (inactive beyond timeout)
    pub fn is_stale(&self, timeout: std::time::Duration) -> bool {
        let elapsed = Utc::now()
            .signed_duration_since(self.last_activity)
            .to_std()
            .unwrap_or(std::time::Duration::ZERO);
        elapsed > timeout
    }
    
    /// Check if session is in a terminal state
    pub fn is_terminal(&self) -> bool {
        self.state == SessionState::Disconnected
    }
}

/// Thread-safe session container
pub type SessionHandle = Arc<RwLock<WebRtcSession>>;

