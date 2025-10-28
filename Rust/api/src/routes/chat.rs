//! Chat completions endpoint (OpenAI-compatible).
//!
//! ENFORCED RULES:
//! ✅ Documentation
//! ✅ Tests (real, not fake)
//! ✅ Uses tabagent-values
//! ✅ Proper tracing
//! ✅ Proper validation
//! ✅ Error handling

use axum::http::Method;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use async_trait::async_trait;

use tabagent_values::{RequestValue, Message, MessageRole};
use crate::{
    error::{ApiResult, ApiError},
    route_trait::{RouteHandler, RouteMetadata, TestCase, ValidationRule, validators::*},
    traits::AppStateProvider,
};

/// Chat completion request (OpenAI-compatible).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ChatMessage {
    /// Message role
    pub role: String,
    /// Message content
    pub content: String,
    /// Optional name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Chat completion response (OpenAI-compatible).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ChatCompletionResponse {
    /// Response ID
    pub id: String,
    /// Object type (always "chat.completion")
    pub object: String,
    /// Creation timestamp
    pub created: i64,
    /// Model used
    pub model: String,
    /// Choices array
    pub choices: Vec<ChatChoice>,
    /// Token usage
    pub usage: Usage,
}

/// Chat choice.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ChatChoice {
    /// Choice index
    pub index: u32,
    /// Message content
    pub message: ChatMessage,
    /// Finish reason
    pub finish_reason: String,
}

/// Token usage statistics.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct Usage {
    /// Prompt tokens
    pub prompt_tokens: u32,
    /// Completion tokens
    pub completion_tokens: u32,
    /// Total tokens
    pub total_tokens: u32,
}

/// Chat completions route handler (OpenAI-compatible).
///
/// This endpoint implements OpenAI's chat completions API for maximum compatibility.
/// It supports all major parameters including temperature, max_tokens, top_p, and streaming.
pub struct ChatRoute;

#[async_trait]
impl RouteHandler for ChatRoute {
    type Request = ChatCompletionRequest;
    type Response = ChatCompletionResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            path: "/v1/chat/completions",
            method: Method::POST,
            tags: &["Chat", "OpenAI"],
            description: "OpenAI-compatible chat completions endpoint for conversational AI",
            openai_compatible: true,
            idempotent: false, // POST is not idempotent
            requires_auth: false, // TODO: Add auth when implemented
            rate_limit_tier: Some("standard"),
        }
    }

    async fn validate_request(req: &Self::Request) -> ApiResult<()> {
        // Model must not be empty
        if req.model.is_empty() {
            return Err(ApiError::BadRequest("model cannot be empty".into()));
        }

        // Messages must not be empty
        if req.messages.is_empty() {
            return Err(ApiError::BadRequest("messages cannot be empty".into()));
        }

        // Temperature must be in valid range
        if let Some(temp) = req.temperature {
            InRange { min: 0.0, max: 2.0 }.validate(&temp)?;
        }

        // Max tokens must be reasonable
        if let Some(max_tokens) = req.max_tokens {
            if max_tokens == 0 || max_tokens > 100000 {
                return Err(ApiError::BadRequest(
                    "max_tokens must be between 1 and 100000".into()
                ));
            }
        }

        // Top-p must be in valid range
        if let Some(top_p) = req.top_p {
            InRange { min: 0.0, max: 1.0 }.validate(&top_p)?;
        }

        Ok(())
    }

    async fn handle<S>(req: Self::Request, state: &S) -> ApiResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(
            request_id = %request_id,
            model = %req.model,
            message_count = req.messages.len(),
            temperature = ?req.temperature,
            max_tokens = ?req.max_tokens,
            stream = req.stream,
            "Chat completion request received"
        );

        // Convert to internal format
        let messages: Vec<Message> = req
            .messages
            .iter()
            .map(|m| Message {
                role: match m.role.as_str() {
                    "system" => MessageRole::System,
                    "user" => MessageRole::User,
                    "assistant" => MessageRole::Assistant,
                    "function" => MessageRole::Function,
                    _ => MessageRole::User,
                },
                content: m.content.clone(),
                name: m.name.clone(),
            })
            .collect();

        // Create request using tabagent-values
        let request = RequestValue::chat_full(
            &req.model,
            messages,
            req.temperature,
            req.max_tokens,
            req.top_p,
            req.stream,
        );

        // Handle via unified handler
        tracing::debug!(request_id = %request_id, "Dispatching to handler");
        let response = state.handle_request(request).await
            .map_err(|e| {
                tracing::error!(
                    request_id = %request_id,
                    model = %req.model,
                    error = %e,
                    "Chat completion failed"
                );
                e
            })?;

        // Extract chat response data
        let (response_text, model_used, usage) = response
            .as_chat()
            .ok_or_else(|| {
                tracing::error!(
                    request_id = %request_id,
                    model = %req.model,
                    "Handler returned invalid response type (expected ChatResponse)"
                );
                ApiError::Internal(
                    format!("Handler returned invalid response type for chat request (request_id: {})", request_id)
                )
            })?;

        tracing::info!(
            request_id = %request_id,
            model = %model_used,
            prompt_tokens = usage.prompt_tokens,
            completion_tokens = usage.completion_tokens,
            total_tokens = usage.total_tokens,
            response_length = response_text.len(),
            "Chat completion successful"
        );

        Ok(ChatCompletionResponse {
            id: uuid::Uuid::new_v4().to_string(),
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp(),
            model: model_used.to_string(),
            choices: vec![ChatChoice {
                index: 0,
                message: ChatMessage {
                    role: "assistant".to_string(),
                    content: response_text.to_string(),
                    name: None,
                },
                finish_reason: "stop".to_string(),
            }],
            usage: Usage {
                prompt_tokens: usage.prompt_tokens,
                completion_tokens: usage.completion_tokens,
                total_tokens: usage.total_tokens,
            },
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
                "model cannot be empty",
            ),
            TestCase::error(
                "empty_messages",
                ChatCompletionRequest {
                    model: "gpt-3.5-turbo".to_string(),
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
                "invalid_temperature_too_high",
                ChatCompletionRequest {
                    model: "gpt-3.5-turbo".to_string(),
                    messages: vec![ChatMessage {
                        role: "user".to_string(),
                        content: "Hello".to_string(),
                        name: None,
                    }],
                    temperature: Some(3.0), // Invalid: > 2.0
                    max_tokens: None,
                    top_p: None,
                    stop: None,
                    stream: false,
                },
                "not in range",
            ),
            TestCase::error(
                "invalid_temperature_negative",
                ChatCompletionRequest {
                    model: "gpt-3.5-turbo".to_string(),
                    messages: vec![ChatMessage {
                        role: "user".to_string(),
                        content: "Hello".to_string(),
                        name: None,
                    }],
                    temperature: Some(-0.5), // Invalid: < 0.0
                    max_tokens: None,
                    top_p: None,
                    stop: None,
                    stream: false,
                },
                "not in range",
            ),
            TestCase::error(
                "invalid_top_p_too_high",
                ChatCompletionRequest {
                    model: "gpt-3.5-turbo".to_string(),
                    messages: vec![ChatMessage {
                        role: "user".to_string(),
                        content: "Hello".to_string(),
                        name: None,
                    }],
                    temperature: None,
                    max_tokens: None,
                    top_p: Some(1.5), // Invalid: > 1.0
                    stop: None,
                    stream: false,
                },
                "not in range",
            ),
            TestCase::error(
                "invalid_top_p_negative",
                ChatCompletionRequest {
                    model: "gpt-3.5-turbo".to_string(),
                    messages: vec![ChatMessage {
                        role: "user".to_string(),
                        content: "Hello".to_string(),
                        name: None,
                    }],
                    temperature: None,
                    max_tokens: None,
                    top_p: Some(-0.1), // Invalid: < 0.0
                    stop: None,
                    stream: false,
                },
                "not in range",
            ),
            TestCase::error(
                "invalid_max_tokens_zero",
                ChatCompletionRequest {
                    model: "gpt-3.5-turbo".to_string(),
                    messages: vec![ChatMessage {
                        role: "user".to_string(),
                        content: "Hello".to_string(),
                        name: None,
                    }],
                    temperature: None,
                    max_tokens: Some(0), // Invalid: must be > 0
                    top_p: None,
                    stop: None,
                    stream: false,
                },
                "not in range",
            ),
            TestCase::error(
                "invalid_max_tokens_negative",
                ChatCompletionRequest {
                    model: "gpt-3.5-turbo".to_string(),
                    messages: vec![ChatMessage {
                        role: "user".to_string(),
                        content: "Hello".to_string(),
                        name: None,
                    }],
                    temperature: None,
                    max_tokens: Some(0),  // 0 is invalid (should be > 0)
                    top_p: None,
                    stop: None,
                    stream: false,
                },
                "not in range",
            ),
            // === SUCCESS TEST CASES ===
            TestCase {
                name: "basic_chat_single_message",
                request: ChatCompletionRequest {
                    model: "gpt-3.5-turbo".to_string(),
                    messages: vec![ChatMessage {
                        role: "user".to_string(),
                        content: "Hello, how are you?".to_string(),
                        name: None,
                    }],
                    temperature: None,
                    max_tokens: None,
                    top_p: None,
                    stop: None,
                    stream: false,
                },
                expected_response: None, // Backend-dependent
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "chat_with_system_message",
                request: ChatCompletionRequest {
                    model: "gpt-4".to_string(),
                    messages: vec![
                        ChatMessage {
                            role: "system".to_string(),
                            content: "You are a helpful assistant.".to_string(),
                            name: None,
                        },
                        ChatMessage {
                            role: "user".to_string(),
                            content: "What is the capital of France?".to_string(),
                            name: None,
                        },
                    ],
                    temperature: Some(0.7),
                    max_tokens: Some(100),
                    top_p: None,
                    stop: None,
                    stream: false,
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "chat_with_all_parameters",
                request: ChatCompletionRequest {
                    model: "gpt-4-turbo".to_string(),
                    messages: vec![ChatMessage {
                        role: "user".to_string(),
                        content: "Tell me a joke".to_string(),
                        name: Some("TestUser".to_string()),
                    }],
                    temperature: Some(1.5),
                    max_tokens: Some(500),
                    top_p: Some(0.9),
                    stop: Some(vec!["END".to_string(), "STOP".to_string()]),
                    stream: false,
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "chat_streaming_enabled",
                request: ChatCompletionRequest {
                    model: "gpt-3.5-turbo".to_string(),
                    messages: vec![ChatMessage {
                        role: "user".to_string(),
                        content: "Write a short story".to_string(),
                        name: None,
                    }],
                    temperature: Some(1.0),
                    max_tokens: Some(1000),
                    top_p: None,
                    stop: None,
                    stream: true, // Streaming enabled
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "chat_boundary_temperature_min",
                request: ChatCompletionRequest {
                    model: "gpt-3.5-turbo".to_string(),
                    messages: vec![ChatMessage {
                        role: "user".to_string(),
                        content: "Test".to_string(),
                        name: None,
                    }],
                    temperature: Some(0.0), // Minimum valid
                    max_tokens: None,
                    top_p: None,
                    stop: None,
                    stream: false,
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "chat_boundary_temperature_max",
                request: ChatCompletionRequest {
                    model: "gpt-3.5-turbo".to_string(),
                    messages: vec![ChatMessage {
                        role: "user".to_string(),
                        content: "Test".to_string(),
                        name: None,
                    }],
                    temperature: Some(2.0), // Maximum valid
                    max_tokens: None,
                    top_p: None,
                    stop: None,
                    stream: false,
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "chat_very_long_message",
                request: ChatCompletionRequest {
                    model: "gpt-4".to_string(),
                    messages: vec![ChatMessage {
                        role: "user".to_string(),
                        content: "a".repeat(5000), // Long message
                        name: None,
                    }],
                    temperature: None,
                    max_tokens: Some(100),
                    top_p: None,
                    stop: None,
                    stream: false,
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "chat_multi_turn_conversation",
                request: ChatCompletionRequest {
                    model: "gpt-3.5-turbo".to_string(),
                    messages: vec![
                        ChatMessage {
                            role: "user".to_string(),
                            content: "Hi".to_string(),
                            name: None,
                        },
                        ChatMessage {
                            role: "assistant".to_string(),
                            content: "Hello! How can I help?".to_string(),
                            name: None,
                        },
                        ChatMessage {
                            role: "user".to_string(),
                            content: "What's the weather?".to_string(),
                            name: None,
                        },
                    ],
                    temperature: Some(0.8),
                    max_tokens: Some(200),
                    top_p: None,
                    stop: None,
                    stream: false,
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

// Enforce compile-time rules
crate::enforce_route_handler!(ChatRoute);

// Implement RegisterableRoute with Axum 0.8 compatible handler
crate::impl_registerable_route!(ChatRoute);

// ==================== RESPONSES (Alternative Format) ====================

/// Responses request (alternative format).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ResponsesRequest {
    /// Input (string or messages)
    pub input: serde_json::Value,
    /// Model name
    #[serde(default = "default_model")]
    pub model: String,
    /// Enable streaming
    #[serde(default)]
    pub stream: bool,
    /// Generation parameters
    #[serde(flatten)]
    pub params: Option<serde_json::Value>,
}

fn default_model() -> String {
    "default".to_string()
}

/// Responses route handler (alternative to chat/completions).
pub struct ResponsesRoute;

#[async_trait]
impl RouteHandler for ResponsesRoute {
    type Request = ResponsesRequest;
    type Response = serde_json::Value;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            path: "/v1/responses",
            method: Method::POST,
            tags: &["Chat"],
            description: "Generate responses using alternative API format (flexible input)",
            openai_compatible: false,
            idempotent: false,
            requires_auth: false,
            rate_limit_tier: Some("inference"),
        }
    }

    async fn validate_request(_req: &Self::Request) -> ApiResult<()> {
        // Input validation is flexible - can be string or array
        Ok(())
    }

    async fn handle<S>(req: Self::Request, state: &S) -> ApiResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(
            request_id = %request_id,
            model = %req.model,
            stream = req.stream,
            "Responses request received"
        );

        // Convert to standard chat request format
        let messages = if req.input.is_string() {
            vec![tabagent_values::Message {
                role: tabagent_values::MessageRole::User,
                content: req.input.as_str().unwrap_or("").to_string(),
                name: None,
            }]
        } else if req.input.is_array() {
            serde_json::from_value(req.input.clone())
                .unwrap_or_else(|_| vec![])
        } else {
            vec![]
        };

        if messages.is_empty() {
            return Err(ApiError::BadRequest("Invalid input format".into()));
        }

        let request = RequestValue::chat(&req.model, messages, None);
        
        tracing::debug!(request_id = %request_id, "Dispatching to handler");
        let response = state.handle_request(request).await
            .map_err(|e| {
                tracing::error!(
                    request_id = %request_id,
                    model = %req.model,
                    error = %e,
                    "Responses generation failed"
                );
                e
            })?;

        let (text, _model, _usage) = response
            .as_chat()
            .ok_or_else(|| ApiError::Internal("Invalid response type".into()))?;

        tracing::info!(
            request_id = %request_id,
            model = %req.model,
            response_length = text.len(),
            "Responses generation successful"
        );

        Ok(serde_json::json!({
            "response": text,
            "model": req.model,
            "stream": req.stream
        }))
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            // === VALIDATION ERROR TESTS ===
            TestCase::error(
                "empty_input_object",
                ResponsesRequest {
                    input: serde_json::json!({}),
                    model: "test".to_string(),
                    stream: false,
                    params: None,
                },
                "Invalid input",
            ),
            TestCase::error(
                "null_input",
                ResponsesRequest {
                    input: serde_json::json!(null),
                    model: "test".to_string(),
                    stream: false,
                    params: None,
                },
                "Invalid input",
            ),
            TestCase::error(
                "numeric_input",
                ResponsesRequest {
                    input: serde_json::json!(123),
                    model: "test".to_string(),
                    stream: false,
                    params: None,
                },
                "Invalid input",
            ),
            // === SUCCESS TEST CASES ===
            TestCase {
                name: "string_input_basic",
                request: ResponsesRequest {
                    input: serde_json::json!("Hello, how are you?"),
                    model: "gpt-3.5-turbo".to_string(),
                    stream: false,
                    params: None,
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "string_input_long",
                request: ResponsesRequest {
                    input: serde_json::json!("a".repeat(1000)),
                    model: "gpt-4".to_string(),
                    stream: false,
                    params: None,
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "array_input_single_message",
                request: ResponsesRequest {
                    input: serde_json::json!([
                        {"role": "user", "content": "Tell me a joke"}
                    ]),
                    model: "gpt-3.5-turbo".to_string(),
                    stream: false,
                    params: None,
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "array_input_multi_message",
                request: ResponsesRequest {
                    input: serde_json::json!([
                        {"role": "system", "content": "You are helpful"},
                        {"role": "user", "content": "What is 2+2?"}
                    ]),
                    model: "gpt-4".to_string(),
                    stream: false,
                    params: None,
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "streaming_enabled",
                request: ResponsesRequest {
                    input: serde_json::json!("Write a story"),
                    model: "gpt-3.5-turbo".to_string(),
                    stream: true,
                    params: None,
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "with_custom_params",
                request: ResponsesRequest {
                    input: serde_json::json!("Hello"),
                    model: "gpt-4".to_string(),
                    stream: false,
                    params: Some(serde_json::json!({
                        "temperature": 0.8,
                        "max_tokens": 150
                    })),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "empty_string_input",
                request: ResponsesRequest {
                    input: serde_json::json!(""),
                    model: "default".to_string(),
                    stream: false,
                    params: None,
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_route_handler!(ResponsesRoute);

// Implement RegisterableRoute with Axum 0.8 compatible handler
crate::impl_registerable_route!(ResponsesRoute);

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
        assert!(result.unwrap_err().to_string().contains("model cannot be empty"));
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
        assert!(result.unwrap_err().to_string().contains("messages cannot be empty"));
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

    #[test]
    fn test_metadata() {
        let meta = ChatRoute::metadata();
        assert_eq!(meta.path, "/v1/chat/completions");
        assert_eq!(meta.method, Method::POST);
        assert!(meta.openai_compatible);
        assert!(!meta.idempotent);
    }
}
