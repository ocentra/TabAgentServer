//! Chat completions endpoint for native messaging.

use serde::{Deserialize, Serialize};
use async_trait::async_trait;

use tabagent_values::{RequestValue, Message, MessageRole};
use crate::{
    error::{NativeMessagingResult, NativeMessagingError},
    route_trait::{NativeMessagingRoute, RouteMetadata, TestCase},
    traits::AppStateProvider,
};

/// Chat completion request (OpenAI-compatible).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    /// Model identifier
    pub model: String,
    /// Array of messages
    pub messages: Vec<ChatMessage>,
    /// Sampling temperature (0.0 to 2.0)
    #[serde(default)]
    pub temperature: Option<f32>,
    /// Maximum tokens to generate
    #[serde(default)]
    pub max_tokens: Option<u32>,
    /// Top-p sampling
    #[serde(default)]
    pub top_p: Option<f32>,
    /// Stop sequences
    #[serde(default)]
    pub stop: Option<Vec<String>>,
    /// Enable streaming (not yet supported)
    #[serde(default)]
    pub stream: bool,
}

/// Chat message (OpenAI-compatible).
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

/// Chat completion response (OpenAI-compatible).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub model: String,
}

/// Chat completions route handler (OpenAI-compatible).
pub struct ChatRoute;

#[async_trait]
impl NativeMessagingRoute for ChatRoute {
    type Request = ChatCompletionRequest;
    type Response = ChatCompletionResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "chat",
            tags: &["Chat", "OpenAI", "AI"],
            description: "OpenAI-compatible chat completions endpoint",
            openai_compatible: true,
            idempotent: false,
            requires_auth: false,
            rate_limit_tier: Some("inference"),
            supports_streaming: true,
            supports_binary: false,
            max_payload_size: Some(1024 * 1024),
        }
    }

    async fn validate_request(req: &Self::Request) -> NativeMessagingResult<()> {
        if req.model.is_empty() {
            return Err(NativeMessagingError::validation("model", "cannot be empty"));
        }
        if req.messages.is_empty() {
            return Err(NativeMessagingError::validation("messages", "cannot be empty"));
        }
        
        // Validate temperature range
        if let Some(temp) = req.temperature {
            if temp < 0.0 || temp > 2.0 {
                return Err(NativeMessagingError::validation("temperature", "must be between 0.0 and 2.0"));
            }
        }
        
        // Validate max_tokens
        if let Some(max_tokens) = req.max_tokens {
            if max_tokens == 0 {
                return Err(NativeMessagingError::validation("max_tokens", "must be greater than 0"));
            }
        }
        
        // Validate top_p range
        if let Some(top_p) = req.top_p {
            if top_p < 0.0 || top_p > 1.0 {
                return Err(NativeMessagingError::validation("top_p", "must be between 0.0 and 1.0"));
            }
        }
        
        Ok(())
    }

    async fn handle<S>(req: Self::Request, state: &S) -> NativeMessagingResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(request_id = %request_id, route = "chat", model = %req.model);

        // Convert to tabagent-values format
        let messages: Vec<Message> = req.messages.into_iter().map(|msg| {
            let role = match msg.role.as_str() {
                "system" => MessageRole::System,
                "user" => MessageRole::User,
                "assistant" => MessageRole::Assistant,
                _ => MessageRole::User, // Default fallback
            };
            Message {
                role,
                content: msg.content,
                name: msg.name,
            }
        }).collect();

        let request = RequestValue::chat_full(
            req.model.clone(),
            messages,
            req.temperature,
            req.max_tokens,
            req.top_p,
            req.stream,
        );

        let response = state.handle_request(request).await
            .map_err(|e| NativeMessagingError::Backend(e))?;

        // Extract response data
        let response_json = response.to_json_value();
        let id = response_json.get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        tracing::info!(request_id = %request_id, response_id = %id, "Chat completion successful");

        Ok(ChatCompletionResponse {
            id,
            object: "chat.completion".to_string(),
            model: req.model,
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
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
                "model",
            ),
        ]
    }
}

/// Responses route handler (alternative format).
pub struct ResponsesRoute;

#[async_trait]
impl NativeMessagingRoute for ResponsesRoute {
    type Request = serde_json::Value;
    type Response = serde_json::Value;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "responses",
            tags: &["Chat", "Flexible"],
            description: "Generate responses using alternative API format",
            openai_compatible: false,
            idempotent: false,
            requires_auth: false,
            rate_limit_tier: Some("inference"),
            supports_streaming: true,
            supports_binary: false,
            max_payload_size: Some(1024 * 1024),
        }
    }

    async fn validate_request(_req: &Self::Request) -> NativeMessagingResult<()> {
        Ok(())
    }

    async fn handle<S>(req: Self::Request, _state: &S) -> NativeMessagingResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        Ok(serde_json::json!({"response": "test", "input": req}))
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase {
                name: "basic_test",
                request: serde_json::json!({"test": "value"}),
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

// Enforce compile-time rules
crate::enforce_native_messaging_route!(ChatRoute);
crate::enforce_native_messaging_route!(ResponsesRoute);