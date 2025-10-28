//! RAG query endpoints.
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

/// RAG query request.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RagQueryRequest {
    /// Query text
    pub query: String,
    /// Number of results
    #[serde(default = "default_top_k")]
    pub k: usize,
    /// Optional filters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<serde_json::Value>,
}

fn default_top_k() -> usize {
    10
}

/// RAG query route handler.
///
/// Performs Retrieval-Augmented Generation (RAG) queries against the vector store.
/// Returns the top-k most relevant documents with similarity scores.
pub struct RagRoute;

#[async_trait]
impl RouteHandler for RagRoute {
    type Request = RagQueryRequest;
    type Response = serde_json::Value;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            path: "/v1/rag/query",
            method: Method::POST,
            tags: &["RAG"],
            description: "Perform RAG query to retrieve relevant documents from vector store",
            openai_compatible: false,
            idempotent: true, // Same query = same results
            requires_auth: false,
            rate_limit_tier: Some("rag"),
        }
    }

    async fn validate_request(req: &Self::Request) -> ApiResult<()> {
        NotEmpty.validate(&req.query)?;
        
        if req.k == 0 || req.k > 1000 {
            return Err(ApiError::BadRequest(
                "k must be between 1 and 1000".into()
            ));
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
            query_length = req.query.len(),
            k = req.k,
            has_filters = req.filters.is_some(),
            "RAG query request received"
        );

        let request = RequestValue::rag_query(&req.query, Some(req.k), req.filters.clone());
        
        tracing::debug!(request_id = %request_id, "Dispatching to handler");
        let response = state.handle_request(request).await
            .map_err(|e| {
                tracing::error!(
                    request_id = %request_id,
                    error = %e,
                    "RAG query failed"
                );
                e
            })?;

        let (results, query_time_ms) = response
            .as_rag()
            .ok_or_else(|| {
                tracing::error!(
                    request_id = %request_id,
                    "Handler returned invalid response type (expected RagResponse)"
                );
                ApiError::Internal(
                    format!("Handler returned invalid response type for RAG query (request_id: {})", request_id)
                )
            })?;

        tracing::info!(
            request_id = %request_id,
            result_count = results.len(),
            query_time_ms = query_time_ms,
            "RAG query successful"
        );

        let result_data: Vec<_> = results.iter().map(|r| {
            serde_json::json!({
                "id": r.id,
                "content": r.content,
                "score": r.score,
                "metadata": r.metadata
            })
        }).collect();

        Ok(serde_json::json!({
            "query": req.query,
            "results": result_data,
            "count": results.len(),
            "query_time_ms": query_time_ms
        }))
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::error(
                "empty_query",
                RagQueryRequest {
                    query: "".to_string(),
                    k: 10,
                    filters: None,
                },
                "cannot be empty",
            ),
            TestCase::error(
                "invalid_k_zero",
                RagQueryRequest {
                    query: "test query".to_string(),
                    k: 0,
                    filters: None,
                },
                "must be between",
            ),
            TestCase::error(
                "invalid_k_too_large",
                RagQueryRequest {
                    query: "test query".to_string(),
                    k: 10000,
                    filters: None,
                },
                "must be between",
            ),
            // === SUCCESS TEST CASES ===
            TestCase {
                name: "rag_query_basic",
                request: RagQueryRequest {
                    query: "What is machine learning?".to_string(),
                    k: 5,
                    filters: None,
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "rag_query_with_filters",
                request: RagQueryRequest {
                    query: "Python programming".to_string(),
                    k: 10,
                    filters: Some(serde_json::json!({"category": "programming"})),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "rag_query_large_k",
                request: RagQueryRequest {
                    query: "artificial intelligence".to_string(),
                    k: 100,
                    filters: None,
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "rag_query_small_k",
                request: RagQueryRequest {
                    query: "quantum computing".to_string(),
                    k: 1,
                    filters: None,
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "rag_query_long_text",
                request: RagQueryRequest {
                    query: "This is a very long query that contains multiple sentences and discusses various topics including machine learning, artificial intelligence, natural language processing, and computer vision.".to_string(),
                    k: 10,
                    filters: None,
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "rag_query_with_special_chars",
                request: RagQueryRequest {
                    query: "What's the différence between ML & AI?".to_string(),
                    k: 5,
                    filters: None,
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "rag_query_complex_filters",
                request: RagQueryRequest {
                    query: "database systems".to_string(),
                    k: 20,
                    filters: Some(serde_json::json!({
                        "category": "database",
                        "year": 2023,
                        "difficulty": "intermediate"
                    })),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_route_handler!(RagRoute);

// Implement RegisterableRoute with Axum 0.8 compatible handler
crate::impl_registerable_route!(RagRoute);

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validation() {
        let req = RagQueryRequest {
            query: "valid query".to_string(),
            k: 10,
            filters: None,
        };
        assert!(RagRoute::validate_request(&req).await.is_ok());
    }

    #[tokio::test]
    async fn test_validation_empty_query() {
        let req = RagQueryRequest {
            query: "".to_string(),
            k: 10,
            filters: None,
        };
        assert!(RagRoute::validate_request(&req).await.is_err());
    }

    #[test]
    fn test_metadata() {
        let meta = RagRoute::metadata();
        assert_eq!(meta.path, "/v1/rag/query");
        assert!(meta.idempotent);
    }
}
