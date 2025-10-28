//! Generate endpoint for WebRTC data channels.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tabagent_values::RequestValue;
use crate::{
    error::{WebRtcResult, WebRtcError},
    route_trait::{DataChannelRoute, RouteMetadata, TestCase},
    traits::RequestHandler,
};

/// Text generation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateRequest {
    /// Model identifier
    pub model: String,
    /// Input prompt
    pub prompt: String,
    /// Sampling temperature (0.0-2.0)
    #[serde(default)]
    pub temperature: Option<f32>,
    /// Maximum tokens to generate
    #[serde(default)]
    pub max_tokens: Option<u32>,
}

/// Text generation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateResponse {
    /// Generated text
    pub text: String,
    /// Model that generated the text
    pub model: String,
}

/// Text generation route handler
pub struct GenerateRoute;

#[async_trait]
impl DataChannelRoute for GenerateRoute {
    type Request = GenerateRequest;
    type Response = GenerateResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "generate",
            tags: &["AI", "Generation"],
            description: "Generate text completion from a prompt using language models",
            supports_streaming: false,
            supports_binary: false,
            requires_auth: true,
            rate_limit_tier: Some("standard"),
            max_payload_size: None,
            media_type: None,
        }
    }

    async fn validate_request(req: &Self::Request) -> WebRtcResult<()> {
        if req.model.is_empty() {
            return Err(WebRtcError::ValidationError {
                field: "model".to_string(),
                message: "model cannot be empty".to_string(),
            });
        }
        if req.prompt.is_empty() {
            return Err(WebRtcError::ValidationError {
                field: "prompt".to_string(),
                message: "prompt cannot be empty".to_string(),
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
        Ok(())
    }

    async fn handle<H>(req: Self::Request, handler: &H) -> WebRtcResult<Self::Response>
    where
        H: RequestHandler + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        
        tracing::info!(
            request_id = %request_id,
            route = "generate",
            model = %req.model,
            prompt_len = req.prompt.len(),
            "WebRTC generate request"
        );

        let request_value = RequestValue::generate_full(
            req.model.clone(),
            req.prompt.clone(),
            req.temperature,
            req.max_tokens,
        );

        let response = handler.handle_request(request_value).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Generate request failed");
                WebRtcError::from(e)
            })?;

        let (text, _usage) = response.as_generate()
            .unwrap_or(("", &tabagent_values::TokenUsage::zero()));

        tracing::info!(request_id = %request_id, response_len = text.len(), "Generate request successful");

        Ok(GenerateResponse {
            text: text.to_string(),
            model: req.model,
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::success(
                "generate_basic",
                GenerateRequest {
                    model: "test-model".to_string(),
                    prompt: "Hello".to_string(),
                    temperature: Some(0.7),
                    max_tokens: Some(100),
                },
                GenerateResponse {
                    text: "Hello, world!".to_string(),
                    model: "test-model".to_string(),
                },
            ),
            TestCase::error(
                "empty_model",
                GenerateRequest {
                    model: "".to_string(),
                    prompt: "Hello".to_string(),
                    temperature: None,
                    max_tokens: None,
                },
                "model cannot be empty",
            ),
            TestCase::error(
                "empty_prompt",
                GenerateRequest {
                    model: "test-model".to_string(),
                    prompt: "".to_string(),
                    temperature: None,
                    max_tokens: None,
                },
                "prompt cannot be empty",
            ),
        ]
    }
}

crate::enforce_data_channel_route!(GenerateRoute);
