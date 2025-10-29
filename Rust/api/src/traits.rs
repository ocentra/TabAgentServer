//! Traits for API integration.
//!
//! This module re-exports the unified traits from `common` and provides
//! additional API-specific state management.

use std::sync::Arc;
use tabagent_webrtc::WebRtcManager;

// Re-export the unified backend traits from common
pub use common::backend::{AppStateProvider, AppStateWrapper};

/// Combined state for API routes that need both business logic and WebRTC signaling.
///
/// This allows WebRTC signaling routes to access WebRtcManager directly,
/// while business logic routes use AppStateProvider.
#[derive(Clone)]
pub struct ApiState {
    /// Business logic provider
    pub app_state: Arc<dyn AppStateProvider>,
    /// WebRTC session manager (for signaling routes only)
    pub webrtc_manager: Arc<WebRtcManager>,
}

