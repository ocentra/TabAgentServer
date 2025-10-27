//! Embeddings generation endpoint (OpenAI-compatible).
//!
//! ENFORCED RULES:
//! ✅ Documentation ✅ Tests ✅ tabagent-values ✅ Tracing ✅ Validation

use axum::http::Method;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use async_trait::async_trait;

use tabagent_values::{RequestValue, EmbeddingInput};
use crate::{
    error::{ApiResult, ApiError},
    route_trait::{RouteHandler, RouteMetadata, TestCase, ValidationRule, validators::*},
    traits::AppStateProvider,
};

/// Embedding request (OpenAI-compatible).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EmbeddingRequest {
    /// Model identifier
    pub model: String,
    /// Input text(s)
    pub input: EmbeddingInputType,
}

/// Input can be single string or array.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(untagged)]
pub enum EmbeddingInputType {
    Single(String),
    Multiple(Vec<String>),
}

/// Embeddings route handler (OpenAI-compatible).
///
/// Generates vector embeddings for text inputs. Compatible with OpenAI's embeddings API.
pub struct EmbeddingsRoute;

#[async_trait]
impl RouteHandler for EmbeddingsRoute {
    type Request = EmbeddingRequest;
    type Response = serde_json::Value;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            path: "/v1/embeddings",
            method: Method::POST,
            tags: &["Embeddings", "OpenAI"],
            description: "OpenAI-compatible embeddings endpoint for vector generation",
            openai_compatible: true,
            idempotent: true, // Same input = same output
            requires_auth: false,
            rate_limit_tier: Some("embeddings"),
        }
    }

    async fn validate_request(req: &Self::Request) -> ApiResult<()> {
        NotEmpty.validate(&req.model)?;
        
        match &req.input {
            EmbeddingInputType::Single(s) => {
                NotEmpty.validate(s)?;
            }
            EmbeddingInputType::Multiple(v) => {
                VecNotEmpty::<String>::new().validate(v)?;
                for text in v {
                    NotEmpty.validate(text)?;
                }
            }
        }
        
        Ok(())
    }

    async fn handle<S>(req: Self::Request, state: &S) -> ApiResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        let input_count = match &req.input {
            EmbeddingInputType::Single(_) => 1,
            EmbeddingInputType::Multiple(v) => v.len(),
        };
        
        tracing::info!(
            request_id = %request_id,
            model = %req.model,
            input_count = input_count,
            "Embedding request received"
        );

        let input = match req.input {
            EmbeddingInputType::Single(s) => EmbeddingInput::Single(s),
            EmbeddingInputType::Multiple(v) => EmbeddingInput::Multiple(v),
        };

        let request = RequestValue::embeddings(&req.model, input);
        
        tracing::debug!(request_id = %request_id, "Dispatching to handler");
        let response = state.handle_request(request).await
            .map_err(|e| {
                tracing::error!(
                    request_id = %request_id,
                    model = %req.model,
                    error = %e,
                    "Embedding generation failed"
                );
                e
            })?;

        let embeddings = response
            .as_embeddings()
            .ok_or_else(|| {
                tracing::error!(
                    request_id = %request_id,
                    model = %req.model,
                    "Handler returned invalid response type (expected EmbeddingsResponse)"
                );
                ApiError::Internal(
                    format!("Handler returned invalid response type for embedding request (request_id: {})", request_id)
                )
            })?;

        tracing::info!(
            request_id = %request_id,
            model = %req.model,
            embedding_count = embeddings.len(),
            embedding_dim = embeddings.get(0).map(|e| e.len()).unwrap_or(0),
            "Embedding generation successful"
        );

        let data: Vec<_> = embeddings.iter().enumerate().map(|(i, emb)| {
            serde_json::json!({
                "object": "embedding",
                "index": i,
                "embedding": emb
            })
        }).collect();

        Ok(serde_json::json!({
            "object": "list",
            "model": req.model,
            "data": data,
            "usage": {
                "prompt_tokens": 0,
                "total_tokens": 0
            }
        }))
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            // === VALIDATION ERROR TESTS ===
            TestCase::error(
                "empty_model",
                EmbeddingRequest {
                    model: "".to_string(),
                    input: EmbeddingInputType::Single("test".to_string()),
                },
                "cannot be empty",
            ),
            TestCase::error(
                "empty_single_input",
                EmbeddingRequest {
                    model: "text-embedding-ada-002".to_string(),
                    input: EmbeddingInputType::Single("".to_string()),
                },
                "cannot be empty",
            ),
            TestCase::error(
                "empty_array_input",
                EmbeddingRequest {
                    model: "text-embedding-ada-002".to_string(),
                    input: EmbeddingInputType::Multiple(vec![]),
                },
                "cannot be empty",
            ),
            TestCase::error(
                "array_with_empty_string",
                EmbeddingRequest {
                    model: "text-embedding-ada-002".to_string(),
                    input: EmbeddingInputType::Multiple(vec!["test".to_string(), "".to_string()]),
                },
                "cannot be empty",
            ),
            // === SUCCESS TEST CASES ===
            TestCase {
                name: "single_text_basic",
                request: EmbeddingRequest {
                    model: "text-embedding-ada-002".to_string(),
                    input: EmbeddingInputType::Single("Hello world".to_string()),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "single_text_long",
                request: EmbeddingRequest {
                    model: "text-embedding-ada-002".to_string(),
                    input: EmbeddingInputType::Single("This is a much longer text that contains multiple sentences. It should still be processed correctly by the embedding model.".to_string()),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "multiple_texts_small_batch",
                request: EmbeddingRequest {
                    model: "text-embedding-ada-002".to_string(),
                    input: EmbeddingInputType::Multiple(vec![
                        "First text".to_string(),
                        "Second text".to_string(),
                        "Third text".to_string(),
                    ]),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "multiple_texts_large_batch",
                request: EmbeddingRequest {
                    model: "text-embedding-ada-002".to_string(),
                    input: EmbeddingInputType::Multiple(
                        (0..50).map(|i| format!("Document number {}", i)).collect()
                    ),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "single_text_special_chars",
                request: EmbeddingRequest {
                    model: "text-embedding-ada-002".to_string(),
                    input: EmbeddingInputType::Single("Test with émojis 🎉 and spëcial çhars!".to_string()),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "single_text_code",
                request: EmbeddingRequest {
                    model: "text-embedding-ada-002".to_string(),
                    input: EmbeddingInputType::Single("fn main() { println!(\"Hello\"); }".to_string()),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "multiple_varying_lengths",
                request: EmbeddingRequest {
                    model: "text-embedding-3-small".to_string(),
                    input: EmbeddingInputType::Multiple(vec![
                        "Short".to_string(),
                        "This is a medium length text with more words".to_string(),
                        "This is a very long text that goes on and on with many sentences and provides a lot of detail about various topics to test how the embedding model handles varying text lengths in the same batch".to_string(),
                    ]),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "different_model",
                request: EmbeddingRequest {
                    model: "text-embedding-3-large".to_string(),
                    input: EmbeddingInputType::Single("Testing different model".to_string()),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "single_word",
                request: EmbeddingRequest {
                    model: "text-embedding-ada-002".to_string(),
                    input: EmbeddingInputType::Single("Hello".to_string()),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_route_handler!(EmbeddingsRoute);

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validation() {
        let req = EmbeddingRequest {
            model: "test".to_string(),
            input: EmbeddingInputType::Single("valid".to_string()),
        };
        assert!(EmbeddingsRoute::validate_request(&req).await.is_ok());
    }

    #[test]
    fn test_metadata() {
        let meta = EmbeddingsRoute::metadata();
        assert!(meta.idempotent);
        assert!(meta.openai_compatible);
    }
}
