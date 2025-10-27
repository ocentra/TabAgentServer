//! RAG Extended endpoint for WebRTC data channels.

use async_trait::async_trait;
use tabagent_values::{RequestValue, ResponseValue};
use crate::{error::{WebRtcResult, WebRtcError}, routes::DataChannelRoute};

pub struct RagExtendedRoute;

#[async_trait]
impl DataChannelRoute for RagExtendedRoute {
    fn route_id() -> &'static str {
        "rag_extended"
    }
    
    async fn handle<H>(request: RequestValue, handler: &H) -> WebRtcResult<ResponseValue>
    where
        H: crate::traits::RequestHandler,
    {
        let request_id = uuid::Uuid::new_v4();
        
        tracing::info!(
            request_id = %request_id,
            route = "rag_extended",
            "WebRTC request received"
        );
        
        let response = handler.handle_request(request).await
            .map_err(|e| {
                tracing::error!(
                    request_id = %request_id,
                    error = %e,
                    "WebRTC request failed"
                );
                WebRtcError::from(e)
            })?;
        
        tracing::info!(
            request_id = %request_id,
            "WebRTC request successful"
        );
        
        Ok(response)
    }
}
