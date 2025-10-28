//! RAG query endpoint for native messaging.

use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use tabagent_values::RequestValue;
use crate::{
    error::{NativeMessagingResult, NativeMessagingError},
    route_trait::{NativeMessagingRoute, RouteMetadata, TestCase},
    traits::AppStateProvider,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagRequest {
    pub query: String,
    pub documents: Vec<String>,
    #[serde(default)]
    pub top_k: Option<u32>,
    #[serde(default)]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagResponse {
    pub answer: String,
    pub sources: Vec<RagSource>,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagSource {
    pub content: String,
    pub score: f32,
    pub index: u32,
}

pub struct RagRoute;

#[async_trait]
impl NativeMessagingRoute for RagRoute {
    type Request = RagRequest;
    type Response = RagResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "rag",
            tags: &["RAG", "AI", "Search"],
            description: "Perform retrieval-augmented generation queries",
            openai_compatible: false,
            idempotent: false,
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
        tracing::info!(request_id = %request_id, route = "rag", query = %req.query);

        let request = RequestValue::rag_query(&req.query, req.top_k.map(|k| k as usize), None);
        let response = state.handle_request(request).await
            .map_err(|e| NativeMessagingError::Backend(e))?;

        let (results, _query_time) = response.as_rag()
            .ok_or_else(|| NativeMessagingError::internal("Invalid response type"))?;

        Ok(RagResponse {
            answer: if !results.is_empty() { results[0].content.clone() } else { "No results found".to_string() },
            sources: results.iter().enumerate().map(|(i, result)| RagSource {
                content: result.content.clone(),
                score: result.score,
                index: i as u32,
            }).collect(),
            model: req.model.unwrap_or_else(|| "default".to_string()),
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::error("empty_query", RagRequest {
                query: "".to_string(),
                documents: vec!["doc1".to_string()],
                top_k: None,
                model: None,
            }, "query"),
            TestCase::error("empty_documents", RagRequest {
                query: "What is AI?".to_string(),
                documents: vec![],
                top_k: None,
                model: None,
            }, "documents"),
            TestCase {
                name: "basic_rag",
                request: RagRequest {
                    query: "What is artificial intelligence?".to_string(),
                    documents: vec![
                        "AI is machine intelligence".to_string(),
                        "Machine learning is a subset of AI".to_string(),
                    ],
                    top_k: Some(5),
                    model: Some("gpt-3.5-turbo".to_string()),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_native_messaging_route!(RagRoute);