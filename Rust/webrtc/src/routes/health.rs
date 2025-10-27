//! Health check endpoint for WebRTC data channels.

use async_trait::async_trait;
use tabagent_values::{RequestValue, ResponseValue};
use crate::{error::{WebRtcResult, WebRtcError}, routes::DataChannelRoute};

/// Health route handler for WebRTC.
pub struct HealthRoute;

#[async_trait]
impl DataChannelRoute for HealthRoute {
    fn route_id() -> &'static str {
        "health"
    }

    async fn handle<H>(request: RequestValue, handler: &H) -> WebRtcResult<ResponseValue>
    where
        H: crate::traits::RequestHandler,
    {
        let request_id = uuid::Uuid::new_v4();
        
        tracing::info!(
            request_id = %request_id,
            route = "health",
            "WebRTC health request received"
        );

        let response = handler.handle_request(request).await
            .map_err(|e| {
                tracing::error!(
                    request_id = %request_id,
                    error = %e,
                    "WebRTC health request failed"
                );
                WebRtcError::from(e)
            })?;

        tracing::info!(
            request_id = %request_id,
            "WebRTC health request successful"
        );

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_id() {
        assert_eq!(HealthRoute::route_id(), "health");
    }
}

