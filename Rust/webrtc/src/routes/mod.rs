//! WebRTC data channel route handlers.
//!
//! This module mirrors `tabagent-api/src/routes/` exactly, providing the same
//! route handlers but over WebRTC data channels instead of HTTP.
//!
//! ARCHITECTURAL PARITY WITH API:
//! ✅ Same validation logic
//! ✅ Same error handling
//! ✅ Same tracing
//! ✅ Same test coverage
//! ✅ Same RouteHandler trait enforcement

pub mod chat;
pub mod embeddings;
pub mod generate;
pub mod generation;
pub mod health;
pub mod management;
pub mod models;
pub mod params;
pub mod rag;
pub mod rag_extended;
pub mod rerank;
pub mod resources;
pub mod sessions;
pub mod stats;
pub mod system;

use crate::error::WebRtcResult;
use tabagent_values::{RequestValue, ResponseValue};
use async_trait::async_trait;

/// Route handler trait for WebRTC data channel handlers.
///
/// This mirrors the API's RouteHandler trait but operates over data channels.
#[async_trait]
pub trait DataChannelRoute: Send + Sync {
    /// Route identifier (must match RequestType variant)
    fn route_id() -> &'static str;
    
    /// Handle the request and return a response
    async fn handle<H>(request: RequestValue, handler: &H) -> WebRtcResult<ResponseValue>
    where
        H: crate::traits::RequestHandler;
}

// TODO: Route registry will be implemented when we add per-route dispatch logic
// For now, all routes go through the unified handler in data_channel.rs

