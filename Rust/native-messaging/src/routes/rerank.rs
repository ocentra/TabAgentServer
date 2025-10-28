//! Reranking endpoint for native messaging.

use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use tabagent_values::{RequestValue, ResponseValue};
use crate::{
    error::{NativeMessagingResult, NativeMessagingError},
    route_trait::{NativeMessagingRoute, RouteMetadata, TestCase},
    traits::AppStateProvider,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankRequest {
    pub query: String,
    pub documents: Vec<String>,
    #[serde(default)]
    pub top_k: Option<u32>,
    #[serde(default)]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankResponse {
    pub results: Vec<RerankResult>,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankResult {
    pub index: u32,
    pub document: String,
    pub relevance_score: f32,
}

pub struct RerankRoute;

#[async_trait]
impl NativeMessagingRoute for RerankRoute {
    type Request = RerankRequest;
    type Response = RerankResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "rerank",
            tags: &["Rerank", "AI", "Search"],
            description: "Rerank documents based on relevance to a query",
            openai_compatible: false,
            idempotent: true,
            requires_auth: false,
            rate_limit_tier: Some("inference"),
            supports_streaming: false,
            supports_binary: false,
            max_payload_size: Some(2 * 1024 * 1024),
        }
    }

    async fn validate_request(req: &Self::Request) -> NativeMessagingResult<()> {
        if req.query.is_empty() {
            return Err(NativeMessagingError::validation("query", "cannot be empty"));
        }
        if req.documents.is_empty() {
            return Err(NativeMessagingError::validation("documents", "cannot be empty"));
        }
        if let Some(top_k) = req.top_k {
            if top_k == 0 || top_k > 100 {
                return Err(NativeMessagingError::validation("top_k", "must be between 1 and 100"));
            }
        }
        Ok(())
    }

    async fn handle<S>(req: Self::Request, state: &S) -> NativeMessagingResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(request_id = %request_id, route = "rerank", query = %req.query);

        let request = RequestValue::rerank(
            req.model.as_deref().unwrap_or("default"),
            &req.query,
            req.documents,
            req.top_k.map(|k| k as usize),
        );
        let response = state.handle_request(request).await
            .map_err(|e| NativeMessagingError::Backend(e))?;

        let results = response.as_rerank()
            .ok_or_else(|| NativeMessagingError::internal("Invalid response type"))?;

        Ok(RerankResponse {
            results: results.iter().enumerate().map(|(i, result)| RerankResult {
                index: i as u32,
                document: result.document.clone(),
                relevance_score: result.score,
            }).collect(),
            model: req.model.unwrap_or_else(|| "default".to_string()),
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::error("empty_query", RerankRequest {
                query: "".to_string(),
                documents: vec!["doc1".to_string()],
                top_k: None,
                model: None,
            }, "query"),
            TestCase::error("empty_documents", RerankRequest {
                query: "What is AI?".to_string(),
                documents: vec![],
                top_k: None,
                model: None,
            }, "documents"),
            TestCase {
                name: "basic_rerank",
                request: RerankRequest {
                    query: "machine learning algorithms".to_string(),
                    documents: vec![
                        "Deep learning neural networks".to_string(),
                        "Cooking recipes for pasta".to_string(),
                        "Supervised learning methods".to_string(),
                    ],
                    top_k: Some(2),
                    model: Some("rerank-model".to_string()),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_native_messaging_route!(RerankRoute);