//! Extended RAG operations endpoint for WebRTC data channels.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tabagent_values::RequestValue;
use crate::{
    error::{WebRtcResult, WebRtcError},
    route_trait::{DataChannelRoute, RouteMetadata, TestCase},
    traits::RequestHandler,
};

/// Extended RAG operations request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum RagExtendedRequest {
    /// Perform semantic search
    SemanticSearch {
        /// Search query
        query: String,
        /// Number of results to return
        k: usize,
        /// Optional search filters
        #[serde(default)]
        filters: Option<serde_json::Value>,
    },
    /// Calculate similarity between two texts
    CalculateSimilarity {
        /// First text
        text1: String,
        /// Second text
        text2: String,
        /// Optional model to use
        #[serde(default)]
        model: Option<String>,
    },
    /// Estimate memory requirements
    EstimateMemory {
        /// Model identifier
        model: String,
        /// Optional quantization setting
        #[serde(default)]
        quantization: Option<String>,
    },
}

/// Extended RAG operations response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagExtendedResponse {
    /// Operation that was performed
    pub operation: String,
    /// Operation result data
    pub result: serde_json::Value,
}

/// Extended RAG operations route handler
pub struct RagExtendedRoute;

#[async_trait]
impl DataChannelRoute for RagExtendedRoute {
    type Request = RagExtendedRequest;
    type Response = RagExtendedResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "rag_extended",
            tags: &["AI", "RAG", "Search", "Advanced"],
            description: "Extended RAG operations - semantic search, similarity calculation, and memory estimation",
            supports_streaming: false,
            supports_binary: false,
            requires_auth: true,
            rate_limit_tier: Some("standard"),
            max_payload_size: None,
            media_type: None,
        }
    }

    async fn validate_request(req: &Self::Request) -> WebRtcResult<()> {
        match req {
            RagExtendedRequest::SemanticSearch { query, k, .. } => {
                if query.is_empty() {
                    return Err(WebRtcError::ValidationError {
                        field: "query".to_string(),
                        message: "query cannot be empty".to_string(),
                    });
                }
                if *k == 0 {
                    return Err(WebRtcError::ValidationError {
                        field: "k".to_string(),
                        message: "k must be greater than 0".to_string(),
                    });
                }
            }
            RagExtendedRequest::CalculateSimilarity { text1, text2, .. } => {
                if text1.is_empty() || text2.is_empty() {
                    return Err(WebRtcError::ValidationError {
                        field: "text".to_string(),
                        message: "both texts must be non-empty".to_string(),
                    });
                }
            }
            RagExtendedRequest::EstimateMemory { model, .. } => {
                if model.is_empty() {
                    return Err(WebRtcError::ValidationError {
                        field: "model".to_string(),
                        message: "model cannot be empty".to_string(),
                    });
                }
            }
        }
        Ok(())
    }

    async fn handle<H>(req: Self::Request, handler: &H) -> WebRtcResult<Self::Response>
    where
        H: RequestHandler + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        
        let operation = match &req {
            RagExtendedRequest::SemanticSearch { .. } => "semantic_search",
            RagExtendedRequest::CalculateSimilarity { .. } => "calculate_similarity",
            RagExtendedRequest::EstimateMemory { .. } => "estimate_memory",
        };

        tracing::info!(
            request_id = %request_id,
            route = "rag_extended",
            operation = operation,
            "WebRTC RAG extended request"
        );

        let request_value = match &req {
            RagExtendedRequest::SemanticSearch { query, k, filters } => {
                RequestValue::semantic_search(query.clone(), *k, filters.clone())
            }
            RagExtendedRequest::CalculateSimilarity { text1, text2, model } => {
                RequestValue::calculate_similarity(text1.clone(), text2.clone(), model.clone())
            }
            RagExtendedRequest::EstimateMemory { model, quantization } => {
                RequestValue::estimate_memory(model.clone(), quantization.clone())
            }
        };

        let response = handler.handle_request(request_value).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "RAG extended request failed");
                WebRtcError::from(e)
            })?;

        let result = response.to_json_value();

        tracing::info!(request_id = %request_id, "RAG extended request successful");

        Ok(RagExtendedResponse {
            operation: operation.to_string(),
            result,
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::success(
                "semantic_search",
                RagExtendedRequest::SemanticSearch {
                    query: "test query".to_string(),
                    k: 5,
                    filters: None,
                },
                RagExtendedResponse {
                    operation: "semantic_search".to_string(),
                    result: serde_json::json!({"results": []}),
                },
            ),
            TestCase::error(
                "empty_query",
                RagExtendedRequest::SemanticSearch {
                    query: "".to_string(),
                    k: 5,
                    filters: None,
                },
                "query cannot be empty",
            ),
        ]
    }
}

crate::enforce_data_channel_route!(RagExtendedRoute);
