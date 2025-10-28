//! Rerank endpoint for WebRTC data channels.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tabagent_values::{RequestValue, response::RerankResult};
use crate::{
    error::{WebRtcResult, WebRtcError},
    route_trait::{DataChannelRoute, RouteMetadata, TestCase},
    traits::RequestHandler,
};

/// Reranking request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankRequest {
    /// Model identifier for reranking
    pub model: String,
    /// Query text
    pub query: String,
    /// Documents to rerank
    pub documents: Vec<String>,
    /// Number of top results to return
    #[serde(default)]
    pub top_n: Option<usize>,
}

/// Reranking response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankResponse {
    /// Model used for reranking
    pub model: String,
    /// Reranked results with relevance scores
    pub results: Vec<RerankResult>,
}

/// Reranking route handler
pub struct RerankRoute;

#[async_trait]
impl DataChannelRoute for RerankRoute {
    type Request = RerankRequest;
    type Response = RerankResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "rerank",
            tags: &["AI", "Search", "Rerank"],
            description: "Rerank documents based on relevance to a query using AI models",
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
        if req.query.is_empty() {
            return Err(WebRtcError::ValidationError {
                field: "query".to_string(),
                message: "query cannot be empty".to_string(),
            });
        }
        if req.documents.is_empty() {
            return Err(WebRtcError::ValidationError {
                field: "documents".to_string(),
                message: "documents cannot be empty".to_string(),
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
            route = "rerank",
            model = %req.model,
            doc_count = req.documents.len(),
            "WebRTC rerank request"
        );

        let request_value = RequestValue::rerank(
            req.model.clone(),
            req.query.clone(),
            req.documents.clone(),
            req.top_n,
        );

        let response = handler.handle_request(request_value).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Rerank request failed");
                WebRtcError::from(e)
            })?;

        let results: Vec<RerankResult> = response.as_rerank()
            .map(|r| r.to_vec())
            .unwrap_or_default();

        tracing::info!(request_id = %request_id, result_count = results.len(), "Rerank request successful");

        Ok(RerankResponse {
            model: req.model,
            results,
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::success(
                "rerank_basic",
                RerankRequest {
                    model: "test-model".to_string(),
                    query: "search query".to_string(),
                    documents: vec!["doc1".to_string(), "doc2".to_string()],
                    top_n: Some(2),
                },
                RerankResponse {
                    model: "test-model".to_string(),
                    results: vec![
                        RerankResult { index: 0, score: 0.9, document: "doc1".to_string() },
                        RerankResult { index: 1, score: 0.7, document: "doc2".to_string() },
                    ],
                },
            ),
            TestCase::error(
                "empty_query",
                RerankRequest {
                    model: "test-model".to_string(),
                    query: "".to_string(),
                    documents: vec!["doc1".to_string()],
                    top_n: None,
                },
                "query cannot be empty",
            ),
        ]
    }
}

crate::enforce_data_channel_route!(RerankRoute);
