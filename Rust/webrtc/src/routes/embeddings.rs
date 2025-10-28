//! Embeddings generation endpoint for WebRTC data channels.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tabagent_values::{RequestValue, EmbeddingInput};
use crate::{
    error::{WebRtcResult, WebRtcError},
    route_trait::{DataChannelRoute, RouteMetadata, TestCase},
    traits::RequestHandler,
};

/// Embeddings generation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingsRequest {
    /// Model identifier
    pub model: String,
    /// Input texts to embed
    pub input: Vec<String>,
}

/// Embeddings generation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingsResponse {
    /// Model that generated embeddings
    pub model: String,
    /// Generated embedding vectors
    pub embeddings: Vec<Vec<f32>>,
}

/// Embeddings route handler
pub struct EmbeddingsRoute;

#[async_trait]
impl DataChannelRoute for EmbeddingsRoute {
    type Request = EmbeddingsRequest;
    type Response = EmbeddingsResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "embeddings",
            tags: &["AI", "Embeddings"],
            description: "Generate text embeddings for semantic search and similarity calculations",
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
        if req.input.is_empty() {
            return Err(WebRtcError::ValidationError {
                field: "input".to_string(),
                message: "input cannot be empty".to_string(),
            });
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
            route = "embeddings",
            model = %req.model,
            input_count = req.input.len(),
            "WebRTC embeddings request"
        );

        let request_value = RequestValue::embeddings(
            req.model.clone(),
            EmbeddingInput::Multiple(req.input.clone()),
        );

        let response = handler.handle_request(request_value).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Embeddings request failed");
                WebRtcError::from(e)
            })?;

        let embeddings = response.as_embeddings()
            .map(|e| e.to_vec())
            .unwrap_or_default();

        tracing::info!(request_id = %request_id, embedding_count = embeddings.len(), "Embeddings request successful");

        Ok(EmbeddingsResponse {
            model: req.model,
            embeddings,
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::success(
                "embeddings_single",
                EmbeddingsRequest {
                    model: "test-model".to_string(),
                    input: vec!["test text".to_string()],
                },
                EmbeddingsResponse {
                    model: "test-model".to_string(),
                    embeddings: vec![vec![0.1, 0.2, 0.3]],
                },
            ),
            TestCase::error(
                "empty_model",
                EmbeddingsRequest {
                    model: "".to_string(),
                    input: vec!["test".to_string()],
                },
                "model cannot be empty",
            ),
            TestCase::error(
                "empty_input",
                EmbeddingsRequest {
                    model: "test-model".to_string(),
                    input: vec![],
                },
                "input cannot be empty",
            ),
        ]
    }
}

crate::enforce_data_channel_route!(EmbeddingsRoute);
