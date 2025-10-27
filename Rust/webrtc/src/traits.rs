//! Traits for WebRTC handlers (mirrors API traits).

use tabagent_values::{RequestValue, ResponseValue};
use async_trait::async_trait;

/// Request handler trait for WebRTC (mirrors AppStateProvider from API).
#[async_trait]
pub trait RequestHandler: Send + Sync {
    /// Handle a request value and return a response value.
    async fn handle_request(&self, request: RequestValue) -> anyhow::Result<ResponseValue>;
}

