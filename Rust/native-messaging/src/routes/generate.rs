//! Text generation endpoint for native messaging.

use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use tabagent_values::RequestValue;
use crate::{
    error::{NativeMessagingResult, NativeMessagingError},
    route_trait::{NativeMessagingRoute, RouteMetadata, TestCase},
    traits::AppStateProvider,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateRequest {
    pub model: String,
    pub prompt: String,
    #[serde(default)]
    pub max_tokens: Option<u32>,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub stream: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<GenerateChoice>,
    pub usage: GenerateUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateChoice {
    pub text: String,
    pub index: u32,
    pub finish_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

pub struct GenerateRoute;

#[async_trait]
impl NativeMessagingRoute for GenerateRoute {
    type Request = GenerateRequest;
    type Response = GenerateResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "generate",
            tags: &["AI", "Generation"],
            description: "Generate text completions using AI models",
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
        if req.prompt.is_empty() {
            return Err(NativeMessagingError::validation("prompt", "cannot be empty"));
        }
        if let Some(temp) = req.temperature {
            if temp < 0.0 || temp > 2.0 {
                return Err(NativeMessagingError::validation("temperature", "must be between 0.0 and 2.0"));
            }
        }
        Ok(())
    }

    async fn handle<S>(req: Self::Request, state: &S) -> NativeMessagingResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(request_id = %request_id, route = "generate", model = %req.model);

        let request = RequestValue::generate(&req.model, &req.prompt, req.temperature);
        let response = state.handle_request(request).await
            .map_err(|e| NativeMessagingError::Backend(e))?;

        let (text, usage) = response.as_generate()
            .ok_or_else(|| NativeMessagingError::internal("Invalid response type"))?;

        Ok(GenerateResponse {
            id: uuid::Uuid::new_v4().to_string(),
            object: "text_completion".to_string(),
            created: chrono::Utc::now().timestamp(),
            model: req.model,
            choices: vec![GenerateChoice {
                text: text.to_string(),
                index: 0,
                finish_reason: "stop".to_string(),
            }],
            usage: GenerateUsage {
                prompt_tokens: usage.prompt_tokens,
                completion_tokens: usage.completion_tokens,
                total_tokens: usage.total_tokens,
            },
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::error("empty_model", GenerateRequest {
                model: "".to_string(),
                prompt: "Hello".to_string(),
                max_tokens: None,
                temperature: None,
                stream: false,
            }, "model"),
            TestCase::error("empty_prompt", GenerateRequest {
                model: "gpt-3.5-turbo".to_string(),
                prompt: "".to_string(),
                max_tokens: None,
                temperature: None,
                stream: false,
            }, "prompt"),
            TestCase {
                name: "basic_generation",
                request: GenerateRequest {
                    model: "gpt-3.5-turbo".to_string(),
                    prompt: "Once upon a time".to_string(),
                    max_tokens: Some(100),
                    temperature: Some(0.7),
                    stream: false,
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_native_messaging_route!(GenerateRoute);