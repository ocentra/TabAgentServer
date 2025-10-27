//! Type definitions for WebRTC operations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// ICE candidate for WebRTC connection
///
/// Represents a discovered ICE (Interactive Connectivity Establishment) candidate
/// used to establish peer-to-peer connections.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IceCandidate {
    /// Candidate string (SDP format)
    pub candidate: String,
    
    /// SDP media stream ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sdp_mid: Option<String>,
    
    /// SDP media stream index
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sdp_mline_index: Option<u16>,
    
    /// When this candidate was added
    pub added_at: DateTime<Utc>,
}

/// Session information for API responses
///
/// Lightweight representation of a WebRTC session for status queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    /// Session ID
    pub id: String,
    
    /// Client ID who created this session
    pub client_id: String,
    
    /// Current session state
    pub state: String,
    
    /// When the session was created
    pub created_at: DateTime<Utc>,
    
    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,
    
    /// Whether data channel is connected
    pub data_channel_connected: bool,
    
    /// Number of ICE candidates
    pub ice_candidate_count: usize,
    
    /// Whether offer has been created
    pub has_offer: bool,
    
    /// Whether answer has been received
    pub has_answer: bool,
}

/// Statistics for WebRTC manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebRtcStats {
    /// Total sessions created
    pub total_sessions: usize,
    
    /// Currently active sessions
    pub active_sessions: usize,
    
    /// Sessions in waiting state
    pub waiting_sessions: usize,
    
    /// Connected sessions (data channel open)
    pub connected_sessions: usize,
    
    /// Disconnected sessions (pending cleanup)
    pub disconnected_sessions: usize,
    
    /// Total messages sent
    pub messages_sent: u64,
    
    /// Total messages received
    pub messages_received: u64,
    
    /// Total bytes sent
    pub bytes_sent: u64,
    
    /// Total bytes received
    pub bytes_received: u64,
    
    /// Sessions cleaned up (timeout/error)
    pub sessions_cleaned: usize,
}

impl Default for WebRtcStats {
    fn default() -> Self {
        Self {
            total_sessions: 0,
            active_sessions: 0,
            waiting_sessions: 0,
            connected_sessions: 0,
            disconnected_sessions: 0,
            messages_sent: 0,
            messages_received: 0,
            bytes_sent: 0,
            bytes_received: 0,
            sessions_cleaned: 0,
        }
    }
}

