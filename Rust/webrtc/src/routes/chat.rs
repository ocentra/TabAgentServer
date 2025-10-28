//! Chat completions endpoint for WebRTC data channels.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tabagent_values::{RequestValue, Message, MessageRole};

use crate::{
    error::{WebRtcResult, WebRtcError},
    route_trait::{DataChannelRoute, RouteMetadata, TestCase},
    traits::RequestHandler,
};

/// Chat completion request (identical to API).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    /// Model identifier
    pub model: String,
    /// Conversation messages
    pub messages: Vec<ChatMessage>,
    /// Sampling temperature (0.0-2.0)
    #[serde(default)]
    pub temperature: Option<f32>,
    /// Maximum tokens to generate
    #[serde(default)]
    pub max_tokens: Option<u32>,
    /// Nucleus sampling parameter
    #[serde(default)]
    pub top_p: Option<f32>,
    /// Stop sequences
    #[serde(default)]
    pub stop: Option<Vec<String>>,
    /// Enable streaming response
    #[serde(default)]
    pub stream: bool,
}

/// Chat message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatMessage {
    /// Message role (system, user, assistant)
    pub role: String,
    /// Message content
    pub content: String,
    /// Optional speaker name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Chat completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    /// Generated response text
    pub text: String,
    /// Model that generated the response
    pub model: String,
    /// Token usage statistics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

/// Token usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    /// Tokens in the prompt
    pub prompt_tokens: u32,
    /// Tokens in the completion
    pub completion_tokens: u32,
    /// Total tokens used
    pub total_tokens: u32,
}

/// Chat route handler for WebRTC
pub struct ChatRoute;

#[async_trait]
impl DataChannelRoute for ChatRoute {
    type Request = ChatCompletionRequest;
    type Response = ChatCompletionResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "chat",
            tags: &["AI", "Chat", "WebRTC"],
            description: "Chat completion over WebRTC data channel with OpenAI-compatible interface",
            supports_streaming: true,
            supports_binary: false,
            requires_auth: true,
            rate_limit_tier: Some("standard"),
            max_payload_size: None,
            media_type: None,
        }
    }

    async fn validate_request(req: &Self::Request) -> WebRtcResult<()> {
        let request_id = uuid::Uuid::new_v4();
        tracing::debug!(
            request_id = %request_id,
            model = %req.model,
            message_count = req.messages.len(),
            "Validating chat request"
        );

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

    async fn handle<H>(req: Self::Request, handler: &H) -> WebRtcResult<Self::Response>
    where
        H: RequestHandler + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        
        tracing::info!(
            request_id = %request_id,
            route = "chat",
            model = %req.model,
            message_count = req.messages.len(),
            "WebRTC chat request received"
        );

        // Convert chat messages to tabagent_values::Message
        let messages: Vec<Message> = req.messages.iter().map(|m| {
            let role = match m.role.to_lowercase().as_str() {
                "system" => MessageRole::System,
                "user" => MessageRole::User,
                "assistant" => MessageRole::Assistant,
                "function" => MessageRole::Function,
                _ => MessageRole::User, // Default fallback
            };
            Message {
                role,
                content: m.content.clone(),
                name: m.name.clone(),
            }
        }).collect();

        // Convert to RequestValue for handler
        let request_value = RequestValue::chat_full(
            req.model.clone(),
            messages,
            req.temperature,
            req.max_tokens,
            req.top_p,
            req.stream,
        );

        let response = handler.handle_request(request_value).await
            .map_err(|e| {
                tracing::error!(
                    request_id = %request_id,
                    error = %e,
                    "WebRTC chat request failed"
                );
                WebRtcError::from(e)
            })?;

        // Extract response data (avoiding temporary borrow issues)
        let (response_text, model_name, prompt_tokens, completion_tokens, total_tokens) = 
            if let Some((text, model, usage)) = response.as_chat() {
                (text.to_string(), model.to_string(), usage.prompt_tokens, usage.completion_tokens, usage.total_tokens)
            } else {
                ("".to_string(), req.model.clone(), 0, 0, 0)
            };

        let chat_response = ChatCompletionResponse {
            text: response_text,
            model: model_name,
            usage: Some(Usage {
                prompt_tokens,
                completion_tokens,
                total_tokens,
            }),
        };

        tracing::info!(
            request_id = %request_id,
            response_len = chat_response.text.len(),
            "WebRTC chat request successful"
        );

        Ok(chat_response)
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::success(
                "valid_chat_request",
                ChatCompletionRequest {
                    model: "test-model".to_string(),
                    messages: vec![ChatMessage {
                        role: "user".to_string(),
                        content: "Hello".to_string(),
                        name: None,
                    }],
                    temperature: Some(0.7),
                    max_tokens: Some(100),
                    top_p: None,
                    stop: None,
                    stream: false,
                },
                ChatCompletionResponse {
                    text: "Hello! How can I help you?".to_string(),
                    model: "test-model".to_string(),
                    usage: Some(Usage {
                        prompt_tokens: 5,
                        completion_tokens: 7,
                        total_tokens: 12,
                    }),
                },
            ),
            TestCase::error(
                "empty_model",
                ChatCompletionRequest {
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
                },
                "model cannot be empty",
            ),
            TestCase::error(
                "empty_messages",
                ChatCompletionRequest {
                    model: "test-model".to_string(),
                    messages: vec![],
                    temperature: None,
                    max_tokens: None,
                    top_p: None,
                    stop: None,
                    stream: false,
                },
                "messages cannot be empty",
            ),
            TestCase::error(
                "invalid_temperature",
                ChatCompletionRequest {
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
                },
                "temperature must be between",
            ),
        ]
    }
}

// COMPILE-TIME ENFORCEMENT
crate::enforce_data_channel_route!(ChatRoute);

/// Alternative responses route handler
pub struct ResponsesRoute;

#[async_trait]
impl DataChannelRoute for ResponsesRoute {
    type Request = ChatCompletionRequest;
    type Response = ChatCompletionResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "responses",
            tags: &["AI", "Chat", "WebRTC"],
            description: "Alternative chat responses endpoint for WebRTC data channel",
            supports_streaming: false,
            supports_binary: false,
            requires_auth: true,
            rate_limit_tier: Some("standard"),
            max_payload_size: None,
            media_type: None,
        }
    }

    async fn validate_request(req: &Self::Request) -> WebRtcResult<()> {
        ChatRoute::validate_request(req).await
    }

    async fn handle<H>(req: Self::Request, handler: &H) -> WebRtcResult<Self::Response>
    where
        H: RequestHandler + Send + Sync,
    {
        ChatRoute::handle(req, handler).await
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::success(
                "valid_response",
                ChatCompletionRequest {
                    model: "test-model".to_string(),
                    messages: vec![ChatMessage {
                        role: "user".to_string(),
                        content: "Test".to_string(),
                        name: None,
                    }],
                    temperature: None,
                    max_tokens: None,
                    top_p: None,
                    stop: None,
                    stream: false,
                },
                ChatCompletionResponse {
                    text: "Response".to_string(),
                    model: "test-model".to_string(),
                    usage: None,
                },
            ),
        ]
    }
}

crate::enforce_data_channel_route!(ResponsesRoute);

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validation_empty_model() {
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

        let result = ChatRoute::validate_request(&req).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), WebRtcError::ValidationError { .. }));
    }

    #[tokio::test]
    async fn test_validation_empty_messages() {
        let req = ChatCompletionRequest {
            model: "test-model".to_string(),
            messages: vec![],
            temperature: None,
            max_tokens: None,
            top_p: None,
            stop: None,
            stream: false,
        };

        let result = ChatRoute::validate_request(&req).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validation_invalid_temperature() {
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

        let result = ChatRoute::validate_request(&req).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validation_valid_request() {
        let req = ChatCompletionRequest {
            model: "test-model".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
                name: None,
            }],
            temperature: Some(0.7),
            max_tokens: Some(100),
            top_p: Some(0.9),
            stop: None,
            stream: false,
        };

        let result = ChatRoute::validate_request(&req).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_metadata() {
        let metadata = ChatRoute::metadata();
        assert_eq!(metadata.route_id, "chat");
        assert!(!metadata.description.is_empty());
        assert!(metadata.tags.contains(&"Chat"));
    }

    #[test]
    fn test_has_test_cases() {
        let test_cases = ChatRoute::test_cases();
        assert!(!test_cases.is_empty());
        assert!(test_cases.len() >= 3);
    }
}

