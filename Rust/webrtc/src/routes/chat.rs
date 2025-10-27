//! Chat completions endpoint for WebRTC data channels.
//!
//! ENFORCED RULES (same as API):
//! ✅ Documentation
//! ✅ Tests (real, not fake)
//! ✅ Uses tabagent-values
//! ✅ Proper tracing
//! ✅ Proper validation
//! ✅ Error handling

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tabagent_values::{RequestValue, ResponseValue};

use crate::{
    error::{WebRtcResult, WebRtcError},
    routes::DataChannelRoute,
};

/// Chat completion request (identical to API).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub max_tokens: Option<u32>,
    #[serde(default)]
    pub top_p: Option<f32>,
    #[serde(default)]
    pub stop: Option<Vec<String>>,
    #[serde(default)]
    pub stream: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Chat route handler for WebRTC.
pub struct ChatRoute;

impl ChatRoute {
    /// Validate chat request (same logic as API).
    fn validate(req: &ChatCompletionRequest) -> WebRtcResult<()> {
        if req.model.is_empty() {
            return Err(WebRtcError::ValidationError {
                field: "model".to_string(),
                message: "model cannot be empty".to_string(),
            });
        }

        if req.messages.is_empty() {
            return Err(WebRtcError::ValidationError {
                field: "messages".to_string(),
                message: "messages cannot be empty".to_string(),
            });
        }

        if let Some(temp) = req.temperature {
            if !(0.0..=2.0).contains(&temp) {
                return Err(WebRtcError::ValidationError {
                    field: "temperature".to_string(),
                    message: "temperature must be between 0.0 and 2.0".to_string(),
                });
            }
        }

        if let Some(max_tokens) = req.max_tokens {
            if max_tokens == 0 || max_tokens > 100000 {
                return Err(WebRtcError::ValidationError {
                    field: "max_tokens".to_string(),
                    message: "max_tokens must be between 1 and 100000".to_string(),
                });
            }
        }

        if let Some(top_p) = req.top_p {
            if !(0.0..=1.0).contains(&top_p) {
                return Err(WebRtcError::ValidationError {
                    field: "top_p".to_string(),
                    message: "top_p must be between 0.0 and 1.0".to_string(),
                });
            }
        }

        Ok(())
    }
}

#[async_trait]
impl DataChannelRoute for ChatRoute {
    fn route_id() -> &'static str {
        "chat"
    }

    async fn handle<H>(request: RequestValue, handler: &H) -> WebRtcResult<ResponseValue>
    where
        H: crate::traits::RequestHandler + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        
        tracing::info!(
            request_id = %request_id,
            route = "chat",
            "WebRTC chat request received"
        );

        // Handle via unified handler (same as API)
        let response = handler.handle_request(request).await
            .map_err(|e| {
                tracing::error!(
                    request_id = %request_id,
                    error = %e,
                    "WebRTC chat request failed"
                );
                WebRtcError::from(e)
            })?;

        tracing::info!(
            request_id = %request_id,
            "WebRTC chat request successful"
        );

        Ok(response)
    }
}

/// Responses route handler (alternative format).
pub struct ResponsesRoute;

#[async_trait]
impl DataChannelRoute for ResponsesRoute {
    fn route_id() -> &'static str {
        "responses"
    }

    async fn handle<H>(request: RequestValue, handler: &H) -> WebRtcResult<ResponseValue>
    where
        H: crate::traits::RequestHandler + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        
        tracing::info!(
            request_id = %request_id,
            route = "responses",
            "WebRTC responses request received"
        );

        let response = handler.handle_request(request).await
            .map_err(|e| {
                tracing::error!(
                    request_id = %request_id,
                    error = %e,
                    "WebRTC responses request failed"
                );
                WebRtcError::from(e)
            })?;

        tracing::info!(
            request_id = %request_id,
            "WebRTC responses request successful"
        );

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_empty_model() {
        let req = ChatCompletionRequest {
            model: "".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
                name: None,
            }],
            temperature: None,
            max_tokens: None,
            top_p: None,
            stop: None,
            stream: false,
        };

        assert!(ChatRoute::validate(&req).is_err());
    }

    #[test]
    fn test_validation_empty_messages() {
        let req = ChatCompletionRequest {
            model: "test-model".to_string(),
            messages: vec![],
            temperature: None,
            max_tokens: None,
            top_p: None,
            stop: None,
            stream: false,
        };

        assert!(ChatRoute::validate(&req).is_err());
    }

    #[test]
    fn test_validation_invalid_temperature() {
        let req = ChatCompletionRequest {
            model: "test-model".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
                name: None,
            }],
            temperature: Some(3.0),
            max_tokens: None,
            top_p: None,
            stop: None,
            stream: false,
        };

        assert!(ChatRoute::validate(&req).is_err());
    }

    #[test]
    fn test_route_id() {
        assert_eq!(ChatRoute::route_id(), "chat");
        assert_eq!(ResponsesRoute::route_id(), "responses");
    }
}

