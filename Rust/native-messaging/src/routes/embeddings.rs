//! Embeddings endpoint for native messaging.

use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use tabagent_values::RequestValue;
use crate::{
    error::{NativeMessagingResult, NativeMessagingError},
    route_trait::{NativeMessagingRoute, RouteMetadata, TestCase},
    traits::AppStateProvider,
};

/// Embedding request (OpenAI-compatible).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingsRequest {
    /// Model identifier
    pub model: String,
    /// Input text(s)
    pub input: EmbeddingInputType,
}

/// Input can be single string or array.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EmbeddingInputType {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingsResponse {
    pub object: String,
    pub data: Vec<EmbeddingData>,
    pub model: String,
    pub usage: Usage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingData {
    pub object: String,
    pub embedding: Vec<f32>,
    pub index: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub total_tokens: u32,
}

pub struct EmbeddingsRoute;

#[async_trait]
impl NativeMessagingRoute for EmbeddingsRoute {
    type Request = EmbeddingsRequest;
    type Response = EmbeddingsResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "embeddings",
            tags: &["AI", "Embeddings"],
            description: "Generate text embeddings using AI models",
            openai_compatible: true,
            idempotent: true,
            requires_auth: false,
            rate_limit_tier: Some("inference"),
            supports_streaming: false,
            supports_binary: false,
            max_payload_size: Some(1024 * 1024),
        }
    }

    async fn validate_request(req: &Self::Request) -> NativeMessagingResult<()> {
        if req.model.is_empty() {
            return Err(NativeMessagingError::validation("model", "cannot be empty"));
        }
        
        // Validate input based on type
        match &req.input {
            EmbeddingInputType::Single(s) => {
                if s.is_empty() {
                    return Err(NativeMessagingError::validation("input", "cannot be empty"));
                }
            }
            EmbeddingInputType::Multiple(arr) => {
                if arr.is_empty() {
                    return Err(NativeMessagingError::validation("input", "cannot be empty array"));
                }
                for (i, s) in arr.iter().enumerate() {
                    if s.is_empty() {
                        return Err(NativeMessagingError::validation(
                            format!("input[{}]", i), 
                            "cannot be empty".to_string()
                        ));
                    }
                }
            }
        }
        
        Ok(())
    }

    async fn handle<S>(req: Self::Request, state: &S) -> NativeMessagingResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(request_id = %request_id, route = "embeddings", model = %req.model);

        let input = match req.input {
            EmbeddingInputType::Single(s) => tabagent_values::EmbeddingInput::Single(s),
            EmbeddingInputType::Multiple(arr) => tabagent_values::EmbeddingInput::Multiple(arr),
        };
        let request = RequestValue::embeddings(&req.model, input);
        let response = state.handle_request(request).await
            .map_err(|e| NativeMessagingError::Backend(e))?;

        let embeddings = response.as_embeddings()
            .ok_or_else(|| NativeMessagingError::internal("Invalid response type"))?;

        Ok(EmbeddingsResponse {
            object: "list".to_string(),
            data: embeddings.iter().enumerate().map(|(i, emb)| EmbeddingData {
                object: "embedding".to_string(),
                embedding: emb.clone(),
                index: i as u32,
            }).collect(),
            model: req.model,
            usage: Usage {
                prompt_tokens: 10,
                total_tokens: 10,
            },
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::error("empty_model", EmbeddingsRequest {
                model: "".to_string(),
                input: EmbeddingInputType::Single("test".to_string()),
            }, "model"),
            TestCase::error("empty_input", EmbeddingsRequest {
                model: "text-embedding-ada-002".to_string(),
                input: EmbeddingInputType::Single("".to_string()),
            }, "input"),
            TestCase {
                name: "basic_embedding",
                request: EmbeddingsRequest {
                    model: "text-embedding-ada-002".to_string(),
                    input: EmbeddingInputType::Single("Hello world".to_string()),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_native_messaging_route!(EmbeddingsRoute);