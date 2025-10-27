//! Data channel message handler
//!
//! Routes messages from WebRTC data channels to tabagent-server handlers,
//! using the same Request/Response protocol as Native Messaging and HTTP API.

use crate::error::{WebRtcError, WebRtcResult};
use serde_json::Value as JsonValue;
use tabagent_values::{RequestValue, ResponseValue};

/// Data channel message handler
///
/// This handler processes JSON messages received over WebRTC data channels,
/// parses them as `RequestValue`, routes to the appropriate handler, and
/// serializes the `ResponseValue` back to JSON.
pub struct DataChannelHandler {
    /// Maximum message size (bytes)
    max_message_size: usize,
}

impl DataChannelHandler {
    /// Create a new data channel handler
    pub fn new(max_message_size: usize) -> Self {
        Self { max_message_size }
    }
    
    /// Handle incoming message from data channel
    ///
    /// This is the main entry point for processing WebRTC data channel messages.
    /// It follows the same flow as Native Messaging:
    /// 1. Parse JSON → RequestValue
    /// 2. Route to server handler
    /// 3. Serialize ResponseValue → JSON
    ///
    /// # Arguments
    /// * `message` - Raw message bytes from data channel
    /// * `handler` - Closure that processes RequestValue → ResponseValue
    ///
    /// # Returns
    /// JSON response bytes to send back over data channel
    pub async fn handle_message<F, Fut>(
        &self,
        message: &[u8],
        handler: F,
    ) -> WebRtcResult<Vec<u8>>
    where
        F: FnOnce(RequestValue) -> Fut,
        Fut: std::future::Future<Output = anyhow::Result<ResponseValue>>,
    {
        // Check message size
        if message.len() > self.max_message_size {
            return Err(WebRtcError::BadRequest(format!(
                "Message too large: {} bytes (max: {})",
                message.len(),
                self.max_message_size
            )));
        }
        
        // Parse JSON string
        let json_str = std::str::from_utf8(message).map_err(|e| {
            WebRtcError::BadRequest(format!("Invalid UTF-8: {}", e))
        })?;
        
        // Parse as RequestValue directly from JSON string
        let request = RequestValue::from_json(json_str).map_err(|e| {
            WebRtcError::BadRequest(format!("Invalid request format: {}", e))
        })?;
        
        tracing::debug!(
            "Received WebRTC message: {:?}",
            request.request_type()
        );
        
        // Call handler
        let response = handler(request).await.map_err(|e| {
            tracing::error!("Handler error: {}", e);
            WebRtcError::Other(e.to_string())
        })?;
        
        // Serialize response to JSON
        let response_json = response.to_json_value();
        let response_bytes = serde_json::to_vec(&response_json).map_err(|e| {
            WebRtcError::InternalError(format!("Failed to serialize response: {}", e))
        })?;
        
        tracing::debug!(
            "Sending WebRTC response: {:?} ({} bytes)",
            response.response_type(),
            response_bytes.len()
        );
        
        Ok(response_bytes)
    }
    
    /// Handle message with error recovery
    ///
    /// If handler fails, returns an error response instead of propagating the error.
    /// This ensures the client always gets a response, even on failures.
    pub async fn handle_message_safe<F, Fut>(
        &self,
        message: &[u8],
        handler: F,
    ) -> Vec<u8>
    where
        F: FnOnce(RequestValue) -> Fut,
        Fut: std::future::Future<Output = anyhow::Result<ResponseValue>>,
    {
        match self.handle_message(message, handler).await {
            Ok(response_bytes) => response_bytes,
            Err(e) => {
                // Create error response
                let error_response = self.create_error_response(&e);
                serde_json::to_vec(&error_response).unwrap_or_else(|_| {
                    // Fallback if serialization fails
                    br#"{"error":"Internal error"}"#.to_vec()
                })
            }
        }
    }
    
    /// Create RFC 7807 Problem Details error response
    fn create_error_response(&self, error: &WebRtcError) -> JsonValue {
        error.to_json_response(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tabagent_values::{RequestValue, HealthStatus};
    
    #[tokio::test]
    async fn test_handle_valid_message() {
        let handler = DataChannelHandler::new(1024 * 1024);
        
        // Create a test request as JSON
        let request_json = r#"{"action":"system_info"}"#;
        let request_bytes = request_json.as_bytes();
        
        // Mock handler that returns success
        let mock_handler = |_req: RequestValue| async {
            Ok(ResponseValue::health(HealthStatus::Healthy))
        };
        
        let response_bytes = handler.handle_message(request_bytes, mock_handler).await.unwrap();
        
        // Verify response
        let response_json: JsonValue = serde_json::from_slice(&response_bytes).unwrap();
        assert!(response_json.is_object());
    }
    
    #[tokio::test]
    async fn test_handle_message_too_large() {
        let handler = DataChannelHandler::new(100); // Small limit
        
        let large_message = vec![b'x'; 200];
        
        let mock_handler = |_req: RequestValue| async {
            Ok(ResponseValue::health(HealthStatus::Healthy))
        };
        
        let result = handler.handle_message(&large_message[..], mock_handler).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), WebRtcError::BadRequest(_)));
    }
    
    #[tokio::test]
    async fn test_handle_invalid_json() {
        let handler = DataChannelHandler::new(1024 * 1024);
        
        let invalid_json = b"{ this is not valid json }";
        
        let mock_handler = |_req: RequestValue| async {
            Ok(ResponseValue::health(HealthStatus::Healthy))
        };
        
        let result = handler.handle_message(&invalid_json[..], mock_handler).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_handle_message_safe_returns_error_response() {
        let handler = DataChannelHandler::new(1024 * 1024);
        
        let invalid_json = b"not json";
        
        let mock_handler = |_req: RequestValue| async {
            Ok(ResponseValue::health(HealthStatus::Healthy))
        };
        
        let response_bytes = handler.handle_message_safe(&invalid_json[..], mock_handler).await;
        
        // Should return valid JSON error
        let response_json: JsonValue = serde_json::from_slice(&response_bytes).unwrap();
        assert!(response_json.get("type").is_some());
        assert!(response_json.get("title").is_some());
    }
}

