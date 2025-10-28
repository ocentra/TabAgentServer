//! Document reranking endpoint.
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

/// Rerank request.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RerankRequest {
    /// Model identifier
    pub model: String,
    /// Query text
    pub query: String,
    /// Documents to rerank
    pub documents: Vec<String>,
    /// Number of top results to return
    #[serde(default = "default_top_n")]
    pub top_n: usize,
}

fn default_top_n() -> usize {
    10
}

/// Rerank route handler.
///
/// Reranks documents based on relevance to a query using a reranking model.
/// Returns documents ordered by relevance score.
pub struct RerankRoute;

#[async_trait]
impl RouteHandler for RerankRoute {
    type Request = RerankRequest;
    type Response = serde_json::Value;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            path: "/v1/rerank",
            method: Method::POST,
            tags: &["RAG", "Reranking"],
            description: "Rerank documents by relevance to query using specialized reranking models",
            openai_compatible: false,
            idempotent: true, // Same input = same output
            requires_auth: false,
            rate_limit_tier: Some("rerank"),
        }
    }

    async fn validate_request(req: &Self::Request) -> ApiResult<()> {
        NotEmpty.validate(&req.model)?;
        NotEmpty.validate(&req.query)?;
        VecNotEmpty::<String>::new().validate(&req.documents)?;
        
        if req.top_n == 0 || req.top_n > req.documents.len() {
            return Err(ApiError::BadRequest(
                format!("top_n must be between 1 and {} (document count)", req.documents.len())
            ));
        }
        
        // Check that documents are not empty
        for (i, doc) in req.documents.iter().enumerate() {
            if doc.is_empty() {
                return Err(ApiError::BadRequest(
                    format!("document at index {} cannot be empty", i)
                ));
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
            query_length = req.query.len(),
            document_count = req.documents.len(),
            top_n = req.top_n,
            "Rerank request received"
        );

        let request = RequestValue::rerank(
            &req.model,
            &req.query,
            req.documents.clone(),
            Some(req.top_n),
        );
        
        tracing::debug!(request_id = %request_id, "Dispatching to handler");
        let response = state.handle_request(request).await
            .map_err(|e| {
                tracing::error!(
                    request_id = %request_id,
                    model = %req.model,
                    error = %e,
                    "Rerank failed"
                );
                e
            })?;

        let results = response
            .as_rerank()
            .ok_or_else(|| {
                tracing::error!(
                    request_id = %request_id,
                    model = %req.model,
                    "Handler returned invalid response type (expected RerankResponse)"
                );
                ApiError::Internal(
                    format!("Handler returned invalid response type for rerank request (request_id: {})", request_id)
                )
            })?;

        tracing::info!(
            request_id = %request_id,
            model = %req.model,
            result_count = results.len(),
            "Rerank successful"
        );

        let result_data: Vec<_> = results.iter().map(|r| {
            serde_json::json!({
                "index": r.index,
                "score": r.score,
                "document": r.document
            })
        }).collect();

        Ok(serde_json::json!({
            "model": req.model,
            "results": result_data,
            "count": results.len()
        }))
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::error(
                "empty_model",
                RerankRequest {
                    model: "".to_string(),
                    query: "test".to_string(),
                    documents: vec!["doc1".to_string()],
                    top_n: 1,
                },
                "cannot be empty",
            ),
            TestCase::error(
                "empty_query",
                RerankRequest {
                    model: "rerank-model".to_string(),
                    query: "".to_string(),
                    documents: vec!["doc1".to_string()],
                    top_n: 1,
                },
                "cannot be empty",
            ),
            TestCase::error(
                "empty_documents",
                RerankRequest {
                    model: "rerank-model".to_string(),
                    query: "test".to_string(),
                    documents: vec![],
                    top_n: 1,
                },
                "cannot be empty",
            ),
            // === SUCCESS TEST CASES ===
            TestCase {
                name: "rerank_basic",
                request: RerankRequest {
                    model: "rerank-english-v2.0".to_string(),
                    query: "machine learning".to_string(),
                    documents: vec![
                        "Machine learning is a subset of AI".to_string(),
                        "Pizza is a popular Italian food".to_string(),
                        "Deep learning uses neural networks".to_string(),
                    ],
                    top_n: 2,
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "rerank_single_document",
                request: RerankRequest {
                    model: "rerank-multilingual-v2.0".to_string(),
                    query: "best restaurants".to_string(),
                    documents: vec!["This restaurant serves amazing food".to_string()],
                    top_n: 1,
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "rerank_many_documents",
                request: RerankRequest {
                    model: "rerank-english-v2.0".to_string(),
                    query: "climate change".to_string(),
                    documents: (0..50).map(|i| format!("Document {} about various topics", i)).collect(),
                    top_n: 10,
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "rerank_all_documents",
                request: RerankRequest {
                    model: "rerank-english-v2.0".to_string(),
                    query: "artificial intelligence".to_string(),
                    documents: vec![
                        "AI is transforming industries".to_string(),
                        "Cooking recipes".to_string(),
                        "ML algorithms are powerful".to_string(),
                    ],
                    top_n: 3, // Return all
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "rerank_long_query",
                request: RerankRequest {
                    model: "rerank-english-v2.0".to_string(),
                    query: "What are the best practices for implementing machine learning models in production environments?".to_string(),
                    documents: vec![
                        "ML deployment guide".to_string(),
                        "Cooking instructions".to_string(),
                        "Production ML systems".to_string(),
                    ],
                    top_n: 2,
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "rerank_special_chars",
                request: RerankRequest {
                    model: "rerank-multilingual-v2.0".to_string(),
                    query: "C++ programming".to_string(),
                    documents: vec![
                        "C++ is a powerful language".to_string(),
                        "Python is easy to learn".to_string(),
                        "Java for enterprise".to_string(),
                    ],
                    top_n: 1,
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_route_handler!(RerankRoute);

// Implement RegisterableRoute with Axum 0.8 compatible handler
crate::impl_registerable_route!(RerankRoute);

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validation() {
        let req = RerankRequest {
            model: "test-model".to_string(),
            query: "test query".to_string(),
            documents: vec!["doc1".to_string(), "doc2".to_string()],
            top_n: 2,
        };
        assert!(RerankRoute::validate_request(&req).await.is_ok());
    }

    #[tokio::test]
    async fn test_validation_empty_documents() {
        let req = RerankRequest {
            model: "test-model".to_string(),
            query: "test".to_string(),
            documents: vec![],
            top_n: 1,
        };
        assert!(RerankRoute::validate_request(&req).await.is_err());
    }

    #[test]
    fn test_metadata() {
        let meta = RerankRoute::metadata();
        assert_eq!(meta.path, "/v1/rerank");
        assert!(meta.idempotent);
    }
}
