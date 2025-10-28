//! RAG (Retrieval-Augmented Generation) endpoint for WebRTC data channels.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tabagent_values::{RequestValue, response::RagResult};
use crate::{
    error::{WebRtcResult, WebRtcError},
    route_trait::{DataChannelRoute, RouteMetadata, TestCase},
    traits::RequestHandler,
};

/// RAG query request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagRequest {
    /// Query text
    pub query: String,
    /// Number of top results to return
    #[serde(default)]
    pub top_k: Option<usize>,
    /// Optional filters for search
    #[serde(default)]
    pub filters: Option<serde_json::Value>,
}

/// RAG query response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagResponse {
    /// Retrieved and ranked results
    pub results: Vec<RagResult>,
    /// Query execution time in milliseconds
    pub query_time_ms: u64,
}

/// RAG route handler
pub struct RagRoute;

#[async_trait]
impl DataChannelRoute for RagRoute {
    type Request = RagRequest;
    type Response = RagResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "rag",
            tags: &["AI", "RAG", "Search"],
            description: "Retrieval-Augmented Generation - query knowledge base and generate context-aware responses",
            supports_streaming: false,
            supports_binary: false,
            requires_auth: true,
            rate_limit_tier: Some("standard"),
            max_payload_size: None,
            media_type: None,
        }
    }

    async fn validate_request(req: &Self::Request) -> WebRtcResult<()> {
        if req.query.is_empty() {
            return Err(WebRtcError::ValidationError {
                field: "query".to_string(),
                message: "query cannot be empty".to_string(),
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
            route = "rag",
            query = %req.query,
            "WebRTC RAG request"
        );

        let request_value = RequestValue::rag_query(
            req.query.clone(),
            req.top_k,
            req.filters.clone(),
        );

        let response = handler.handle_request(request_value).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "RAG request failed");
                WebRtcError::from(e)
            })?;

        let (results, query_time_ms) = response.as_rag()
            .unwrap_or((&[], 0));

        let rag_results = results.to_vec();

        tracing::info!(request_id = %request_id, result_count = results.len(), "RAG request successful");

        Ok(RagResponse {
            results: rag_results,
            query_time_ms,
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::success(
                "rag_basic",
                RagRequest {
                    query: "test query".to_string(),
                    top_k: Some(5),
                    filters: None,
                },
                RagResponse {
                    results: vec![
                        RagResult {
                            id: "res1".to_string(),
                            content: "result 1".to_string(),
                            score: 0.9,
                            metadata: None,
                        },
                    ],
                    query_time_ms: 100,
                },
            ),
            TestCase::error(
                "empty_query",
                RagRequest {
                    query: "".to_string(),
                    top_k: None,
                    filters: None,
                },
                "query cannot be empty",
            ),
        ]
    }
}

crate::enforce_data_channel_route!(RagRoute);
