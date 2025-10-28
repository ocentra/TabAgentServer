//! TabAgent WebRTC Crate
//!
//! WebRTC signaling and data channel handler for TabAgent Server.
//!
//! # Architecture
//!
//! This crate provides a browser-only alternative to Native Messaging with:
//! - WebRTC signaling (offer/answer/ICE) via REST API
//! - Data channel message routing using `tabagent-values`
//! - Session lifecycle management (cleanup, timeouts, limits)
//! - Full API parity with HTTP routes (all 36 endpoints)
//!
//! # Usage
//!
//! ```rust,no_run
//! use tabagent_webrtc::{WebRtcManager, WebRtcConfig};
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create WebRTC manager
//!     let config = WebRtcConfig::default();
//!     let manager = Arc::new(WebRtcManager::new(config));
//!     
//!     // Create signaling session
//!     let session = manager.create_offer("client-123".to_string()).await.unwrap();
//!     println!("Session ID: {}", session.id);
//!     println!("Offer SDP: {}", session.offer_sdp);
//!     
//!     // Submit answer from client
//!     manager.submit_answer(&session.id, "answer sdp...".to_string()).await.unwrap();
//!     
//!     // Data channel messages route to same handler as HTTP API
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(unsafe_code)]

mod config;
mod error;
mod manager;
mod session;
mod types;
mod data_channel;
pub mod routes;       // Public module for route handlers
pub mod traits;       // Public module for trait definitions
pub mod route_trait;  // Public module for compile-time enforcement system

// Re-exports
pub use config::{WebRtcConfig, TurnServer};
pub use error::{WebRtcError, WebRtcResult, ProblemDetails};
pub use manager::WebRtcManager;
pub use session::{WebRtcSession, SessionState};
pub use types::{IceCandidate, SessionInfo, WebRtcStats};
pub use data_channel::DataChannelHandler;
pub use routes::DataChannelRoute;
pub use route_trait::{
    RouteMetadata, MediaType, ValidationRule, TestCase, RouteCollection,
};

