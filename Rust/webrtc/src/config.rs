//! Configuration types for WebRTC manager

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for WebRTC manager
///
/// Defines session limits, timeouts, and STUN/TURN servers for WebRTC connections.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebRtcConfig {
    /// Maximum number of concurrent sessions
    pub max_sessions: usize,
    
    /// Maximum sessions per client ID
    pub max_sessions_per_client: usize,
    
    /// Session timeout (inactive sessions are cleaned up)
    pub session_timeout: Duration,
    
    /// Cleanup interval for stale sessions
    pub cleanup_interval: Duration,
    
    /// Maximum message size for data channels (bytes)
    pub max_message_size: usize,
    
    /// STUN server URLs
    pub stun_servers: Vec<String>,
    
    /// TURN server URLs (optional, for NAT traversal)
    pub turn_servers: Vec<TurnServer>,
    
    /// Enable verbose logging
    pub verbose_logging: bool,
}

impl Default for WebRtcConfig {
    fn default() -> Self {
        Self {
            max_sessions: 100,
            max_sessions_per_client: 5,
            session_timeout: Duration::from_secs(300), // 5 minutes
            cleanup_interval: Duration::from_secs(60), // 1 minute
            max_message_size: 1024 * 1024, // 1 MB
            stun_servers: vec![
                "stun:stun.l.google.com:19302".to_string(),
                "stun:stun1.l.google.com:19302".to_string(),
            ],
            turn_servers: vec![],
            verbose_logging: false,
        }
    }
}

/// TURN server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnServer {
    pub urls: Vec<String>,
    pub username: String,
    pub credential: String,
}

impl WebRtcConfig {
    /// Create config from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();
        
        if let Ok(max_sessions) = std::env::var("WEBRTC_MAX_SESSIONS") {
            if let Ok(val) = max_sessions.parse() {
                config.max_sessions = val;
            }
        }
        
        if let Ok(timeout_secs) = std::env::var("WEBRTC_SESSION_TIMEOUT") {
            if let Ok(val) = timeout_secs.parse() {
                config.session_timeout = Duration::from_secs(val);
            }
        }
        
        if let Ok(verbose) = std::env::var("WEBRTC_VERBOSE") {
            config.verbose_logging = verbose == "1" || verbose.to_lowercase() == "true";
        }
        
        config
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.max_sessions == 0 {
            return Err("max_sessions must be > 0".to_string());
        }
        
        if self.max_sessions_per_client == 0 {
            return Err("max_sessions_per_client must be > 0".to_string());
        }
        
        if self.max_message_size == 0 {
            return Err("max_message_size must be > 0".to_string());
        }
        
        if self.stun_servers.is_empty() && self.turn_servers.is_empty() {
            return Err("At least one STUN or TURN server is required".to_string());
        }
        
        Ok(())
    }
}

