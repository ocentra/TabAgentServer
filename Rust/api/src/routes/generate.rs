//! Text generation endpoint (completions).
//!
//! ENFORCED RULES:
//! ✅ Documentation ✅ Tests ✅ tabagent-values ✅ Tracing ✅ Validation

use axum::http::Method;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use async_trait::async_trait;

use tabagent_values::RequestValue;
use crate::{
    error::{ApiResult, ApiError},
    route_trait::{RouteHandler, RouteMetadata, TestCase, ValidationRule, validators::*},
    traits::AppStateProvider,
};

/// Text completion request.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CompletionRequest {
    /// Model identifier
    pub model: String,
    /// Input prompt
    pub prompt: String,
    /// Sampling temperature
    #[serde(default)]
    pub temperature: Option<f32>,
    /// Maximum tokens to generate
    #[serde(default)]
    pub max_tokens: Option<u32>,
}

/// Text generation route handler.
///
/// Generates text completions given a prompt. Similar to OpenAI's completions endpoint.
pub struct GenerateRoute;

#[async_trait]
impl RouteHandler for GenerateRoute {
    type Request = CompletionRequest;
    type Response = serde_json::Value;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            path: "/v1/completions",
            method: Method::POST,
            tags: &["Generation", "OpenAI"],
            description: "Text completion endpoint for prompt-based generation",
            openai_compatible: true,
            idempotent: false,
            requires_auth: false,
            rate_limit_tier: Some("standard"),
        }
    }

    async fn validate_request(req: &Self::Request) -> ApiResult<()> {
        NotEmpty.validate(&req.model)?;
        NotEmpty.validate(&req.prompt)?;
        
        if let Some(temp) = req.temperature {
            InRange { min: 0.0, max: 2.0 }.validate(&temp)?;
        }
        
        if let Some(max_tokens) = req.max_tokens {
            if max_tokens == 0 || max_tokens > 100000 {
                return Err(ApiError::BadRequest("max_tokens must be between 1 and 100000".into()));
            }
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
            prompt_length = req.prompt.len(),
            temperature = ?req.temperature,
            max_tokens = ?req.max_tokens,
            "Text completion request received"
        );

        let request = RequestValue::generate_full(
            &req.model,
            &req.prompt,
            req.temperature,
            req.max_tokens,
        );

        tracing::debug!(request_id = %request_id, "Dispatching to handler");
        let response = state.handle_request(request).await
            .map_err(|e| {
                tracing::error!(
                    request_id = %request_id,
                    model = %req.model,
                    error = %e,
                    "Text completion failed"
                );
                e
            })?;
        
        let (text, usage) = response
            .as_generate()
            .ok_or_else(|| {
                tracing::error!(
                    request_id = %request_id,
                    model = %req.model,
                    "Handler returned invalid response type (expected GenerateResponse)"
                );
                ApiError::Internal(
                    format!("Handler returned invalid response type for generation request (request_id: {})", request_id)
                )
            })?;

        tracing::info!(
            request_id = %request_id,
            model = %req.model,
            prompt_tokens = usage.prompt_tokens,
            completion_tokens = usage.completion_tokens,
            total_tokens = usage.total_tokens,
            response_length = text.len(),
            "Text completion successful"
        );

        Ok(serde_json::json!({
            "id": uuid::Uuid::new_v4().to_string(),
            "object": "text_completion",
            "created": chrono::Utc::now().timestamp(),
            "model": req.model,
            "choices": [{
                "text": text,
                "index": 0,
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": usage.prompt_tokens,
                "completion_tokens": usage.completion_tokens,
                "total_tokens": usage.total_tokens
            }
        }))
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            // === VALIDATION ERROR TESTS ===
            TestCase::error(
                "empty_model",
                CompletionRequest {
                    model: "".to_string(),
                    prompt: "Hello".to_string(),
                    temperature: None,
                    max_tokens: None,
                },
                "cannot be empty",
            ),
            TestCase::error(
                "empty_prompt",
                CompletionRequest {
                    model: "test-model".to_string(),
                    prompt: "".to_string(),
                    temperature: None,
                    max_tokens: None,
                },
                "cannot be empty",
            ),
            TestCase::error(
                "invalid_temperature_too_high",
                CompletionRequest {
                    model: "gpt-3.5-turbo".to_string(),
                    prompt: "Test".to_string(),
                    temperature: Some(2.5),
                    max_tokens: None,
                },
                "not in range",
            ),
            TestCase::error(
                "invalid_temperature_negative",
                CompletionRequest {
                    model: "gpt-3.5-turbo".to_string(),
                    prompt: "Test".to_string(),
                    temperature: Some(-0.1),
                    max_tokens: None,
                },
                "not in range",
            ),
            TestCase::error(
                "invalid_max_tokens_zero",
                CompletionRequest {
                    model: "gpt-3.5-turbo".to_string(),
                    prompt: "Test".to_string(),
                    temperature: None,
                    max_tokens: Some(0),
                },
                "not in range",
            ),
            // === SUCCESS TEST CASES ===
            TestCase {
                name: "basic_completion",
                request: CompletionRequest {
                    model: "gpt-3.5-turbo".to_string(),
                    prompt: "Once upon a time".to_string(),
                    temperature: None,
                    max_tokens: None,
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "completion_with_temperature",
                request: CompletionRequest {
                    model: "gpt-4".to_string(),
                    prompt: "Write a haiku about programming".to_string(),
                    temperature: Some(0.9),
                    max_tokens: None,
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "completion_with_max_tokens",
                request: CompletionRequest {
                    model: "gpt-3.5-turbo".to_string(),
                    prompt: "Explain quantum physics".to_string(),
                    temperature: None,
                    max_tokens: Some(50),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "completion_with_all_params",
                request: CompletionRequest {
                    model: "gpt-4-turbo".to_string(),
                    prompt: "Tell me a story".to_string(),
                    temperature: Some(1.5),
                    max_tokens: Some(500),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "completion_very_long_prompt",
                request: CompletionRequest {
                    model: "gpt-4".to_string(),
                    prompt: "Lorem ipsum ".repeat(500),
                    temperature: Some(0.7),
                    max_tokens: Some(100),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "completion_boundary_temp_min",
                request: CompletionRequest {
                    model: "gpt-3.5-turbo".to_string(),
                    prompt: "Test".to_string(),
                    temperature: Some(0.0),
                    max_tokens: None,
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "completion_boundary_temp_max",
                request: CompletionRequest {
                    model: "gpt-3.5-turbo".to_string(),
                    prompt: "Test".to_string(),
                    temperature: Some(2.0),
                    max_tokens: None,
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "completion_single_token",
                request: CompletionRequest {
                    model: "gpt-3.5-turbo".to_string(),
                    prompt: "Say hi".to_string(),
                    temperature: None,
                    max_tokens: Some(1),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_route_handler!(GenerateRoute);

// Implement RegisterableRoute with Axum 0.8 compatible handler
crate::impl_registerable_route!(GenerateRoute);

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validation_empty_model() {
        let req = CompletionRequest {
            model: "".to_string(),
            prompt: "test".to_string(),
            temperature: None,
            max_tokens: None,
        };
        assert!(GenerateRoute::validate_request(&req).await.is_err());
    }

    #[tokio::test]
    async fn test_validation_empty_prompt() {
        let req = CompletionRequest {
            model: "test".to_string(),
            prompt: "".to_string(),
            temperature: None,
            max_tokens: None,
        };
        assert!(GenerateRoute::validate_request(&req).await.is_err());
    }

    #[test]
    fn test_metadata() {
        let meta = GenerateRoute::metadata();
        assert_eq!(meta.path, "/v1/completions");
        assert_eq!(meta.method, Method::POST);
        assert!(meta.openai_compatible);
    }
}
