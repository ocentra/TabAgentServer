//! Extended RAG endpoints for WebRTC data channels.
//!
//! ENFORCED RULES:
//! ✅ Documentation ✅ Tests ✅ tabagent-values ✅ Tracing ✅ Validation

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tabagent_values::RequestValue;
use crate::{
    error::{WebRtcResult, WebRtcError},
    route_trait::{DataChannelRoute, RouteMetadata, TestCase},
    traits::RequestHandler,
};

// ==================== SEMANTIC SEARCH ====================

/// Semantic search request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticSearchRequest {
    /// Query text
    pub query: String,
    /// Number of results to return
    #[serde(default = "default_k")]
    pub k: usize,
    /// Optional search filters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<serde_json::Value>,
}

fn default_k() -> usize {
    10
}

/// Semantic search response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticSearchResponse {
    /// Search results
    pub results: serde_json::Value,
}

/// Semantic search route handler.
pub struct SemanticSearchRoute;

#[async_trait]
impl DataChannelRoute for SemanticSearchRoute {
    type Request = SemanticSearchRequest;
    type Response = SemanticSearchResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "semantic_search",
            tags: &["AI", "RAG", "Search"],
            description: "Perform semantic search over documents using embeddings",
            supports_streaming: false,
            supports_binary: false,
            requires_auth: true,
            rate_limit_tier: Some("rag"),
            max_payload_size: Some(1024 * 1024), // 1MB
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
        if req.k == 0 {
            return Err(WebRtcError::ValidationError {
                field: "k".to_string(),
                message: "k must be greater than 0".to_string(),
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
            route = "semantic_search",
            query_length = req.query.len(),
            k = req.k,
            "WebRTC semantic search request"
        );

        let request_value = RequestValue::semantic_search(&req.query, req.k, req.filters.clone());

        let response = handler.handle_request(request_value).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Semantic search failed");
                WebRtcError::from(e)
            })?;

        let results = response.to_json_value();

        tracing::info!(request_id = %request_id, "Semantic search successful");

        Ok(SemanticSearchResponse { results })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::success(
                "semantic_search",
                SemanticSearchRequest {
                    query: "test query".to_string(),
                    k: 5,
                    filters: None,
                },
                SemanticSearchResponse {
                    results: serde_json::json!([]),
                },
            ),
            TestCase::error(
                "empty_query",
                SemanticSearchRequest {
                    query: "".to_string(),
                    k: 5,
                    filters: None,
                },
                "query cannot be empty",
            ),
        ]
    }
}

crate::enforce_data_channel_route!(SemanticSearchRoute);

// ==================== SIMILARITY ====================

/// Similarity calculation request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityRequest {
    /// First text
    pub text1: String,
    /// Second text
    pub text2: String,
    /// Optional model to use
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

/// Similarity calculation response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityResponse {
    /// Similarity score (0.0-1.0)
    pub similarity: f32,
}

/// Similarity calculation route handler.
pub struct SimilarityRoute;

#[async_trait]
impl DataChannelRoute for SimilarityRoute {
    type Request = SimilarityRequest;
    type Response = SimilarityResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "similarity",
            tags: &["AI", "RAG", "Embeddings"],
            description: "Calculate similarity between two texts using embeddings",
            supports_streaming: false,
            supports_binary: false,
            requires_auth: true,
            rate_limit_tier: Some("rag"),
            max_payload_size: Some(1024 * 1024), // 1MB
            media_type: None,
        }
    }

    async fn validate_request(req: &Self::Request) -> WebRtcResult<()> {
        if req.text1.is_empty() {
            return Err(WebRtcError::ValidationError {
                field: "text1".to_string(),
                message: "text1 cannot be empty".to_string(),
            });
        }
        if req.text2.is_empty() {
            return Err(WebRtcError::ValidationError {
                field: "text2".to_string(),
                message: "text2 cannot be empty".to_string(),
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
            route = "similarity",
            "WebRTC similarity calculation request"
        );

        let request_value = RequestValue::calculate_similarity(&req.text1, &req.text2, req.model.clone());

        let _response = handler.handle_request(request_value).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Similarity calculation failed");
                WebRtcError::from(e)
            })?;

        // TODO: Extract similarity from response once backend implements as_similarity()
        let similarity = 0.0; // Placeholder

        tracing::info!(request_id = %request_id, "Similarity calculation successful");

        Ok(SimilarityResponse { similarity })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::success(
                "similarity",
                SimilarityRequest {
                    text1: "Hello world".to_string(),
                    text2: "Hi world".to_string(),
                    model: None,
                },
                SimilarityResponse {
                    similarity: 0.85,
                },
            ),
            TestCase::error(
                "empty_text1",
                SimilarityRequest {
                    text1: "".to_string(),
                    text2: "Hi".to_string(),
                    model: None,
                },
                "text1 cannot be empty",
            ),
        ]
    }
}

crate::enforce_data_channel_route!(SimilarityRoute);

// ==================== EVALUATE EMBEDDINGS ====================

/// Evaluate embeddings request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluateEmbeddingsRequest {
    /// Texts to evaluate
    pub texts: Vec<String>,
    /// Optional model to use
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

/// Evaluate embeddings response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluateEmbeddingsResponse {
    /// Evaluation results
    pub results: serde_json::Value,
}

/// Evaluate embeddings route handler.
pub struct EvaluateEmbeddingsRoute;

#[async_trait]
impl DataChannelRoute for EvaluateEmbeddingsRoute {
    type Request = EvaluateEmbeddingsRequest;
    type Response = EvaluateEmbeddingsResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "evaluate_embeddings",
            tags: &["AI", "RAG", "Embeddings"],
            description: "Evaluate and analyze embeddings for multiple texts",
            supports_streaming: false,
            supports_binary: false,
            requires_auth: true,
            rate_limit_tier: Some("rag"),
            max_payload_size: Some(2 * 1024 * 1024), // 2MB
            media_type: None,
        }
    }

    async fn validate_request(req: &Self::Request) -> WebRtcResult<()> {
        if req.texts.is_empty() {
            return Err(WebRtcError::ValidationError {
                field: "texts".to_string(),
                message: "texts cannot be empty".to_string(),
            });
        }
        for (i, text) in req.texts.iter().enumerate() {
            if text.is_empty() {
                return Err(WebRtcError::ValidationError {
                    field: format!("texts[{}]", i),
                    message: "text cannot be empty".to_string(),
                });
            }
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
            route = "evaluate_embeddings",
            text_count = req.texts.len(),
            "WebRTC evaluate embeddings request"
        );

        // TODO: Fix signature once backend is implemented properly
        let request_value = RequestValue::evaluate_embeddings(
            req.model.clone().unwrap_or_else(|| "default".to_string()),
            vec![],  // queries - need to determine proper split
            req.texts.clone()  // documents
        );

        let response = handler.handle_request(request_value).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Evaluate embeddings failed");
                WebRtcError::from(e)
            })?;

        let results = response.to_json_value();

        tracing::info!(request_id = %request_id, "Evaluate embeddings successful");

        Ok(EvaluateEmbeddingsResponse { results })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::success(
                "evaluate_embeddings",
                EvaluateEmbeddingsRequest {
                    texts: vec!["text1".to_string(), "text2".to_string()],
                    model: None,
                },
                EvaluateEmbeddingsResponse {
                    results: serde_json::json!({}),
                },
            ),
            TestCase::error(
                "empty_texts",
                EvaluateEmbeddingsRequest {
                    texts: vec![],
                    model: None,
                },
                "texts cannot be empty",
            ),
        ]
    }
}

crate::enforce_data_channel_route!(EvaluateEmbeddingsRoute);

// ==================== CLUSTER ====================

/// Cluster documents request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterRequest {
    /// Texts to cluster
    pub texts: Vec<String>,
    /// Number of clusters
    #[serde(default = "default_clusters")]
    pub n_clusters: usize,
    /// Optional model to use
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

fn default_clusters() -> usize {
    5
}

/// Cluster documents response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterResponse {
    /// Cluster assignments
    pub clusters: Vec<usize>,
    /// Cluster centers
    pub centers: serde_json::Value,
}

/// Cluster documents route handler.
pub struct ClusterRoute;

#[async_trait]
impl DataChannelRoute for ClusterRoute {
    type Request = ClusterRequest;
    type Response = ClusterResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "cluster",
            tags: &["AI", "RAG", "Clustering"],
            description: "Cluster documents using embeddings",
            supports_streaming: false,
            supports_binary: false,
            requires_auth: true,
            rate_limit_tier: Some("rag"),
            max_payload_size: Some(2 * 1024 * 1024), // 2MB
            media_type: None,
        }
    }

    async fn validate_request(req: &Self::Request) -> WebRtcResult<()> {
        if req.texts.is_empty() {
            return Err(WebRtcError::ValidationError {
                field: "texts".to_string(),
                message: "texts cannot be empty".to_string(),
            });
        }
        if req.n_clusters == 0 {
            return Err(WebRtcError::ValidationError {
                field: "n_clusters".to_string(),
                message: "n_clusters must be greater than 0".to_string(),
            });
        }
        if req.n_clusters > req.texts.len() {
            return Err(WebRtcError::ValidationError {
                field: "n_clusters".to_string(),
                message: "n_clusters cannot be greater than number of texts".to_string(),
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
            route = "cluster",
            text_count = req.texts.len(),
            n_clusters = req.n_clusters,
            "WebRTC cluster documents request"
        );

        let request_value = RequestValue::cluster_documents(req.texts.clone(), req.n_clusters, req.model.clone());

        let _response = handler.handle_request(request_value).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Cluster documents failed");
                WebRtcError::from(e)
            })?;

        // TODO: Extract clusters from response once backend implements as_cluster_results()
        let clusters = vec![0; req.texts.len()]; // Placeholder
        let centers = serde_json::json!([]);

        tracing::info!(request_id = %request_id, "Cluster documents successful");

        Ok(ClusterResponse {
            clusters,
            centers,
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::success(
                "cluster",
                ClusterRequest {
                    texts: vec!["text1".to_string(), "text2".to_string(), "text3".to_string()],
                    n_clusters: 2,
                    model: None,
                },
                ClusterResponse {
                    clusters: vec![0, 0, 1],
                    centers: serde_json::json!([]),
                },
            ),
            TestCase::error(
                "empty_texts",
                ClusterRequest {
                    texts: vec![],
                    n_clusters: 2,
                    model: None,
                },
                "texts cannot be empty",
            ),
        ]
    }
}

crate::enforce_data_channel_route!(ClusterRoute);

// ==================== RECOMMEND ====================

/// Recommend content request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendRequest {
    /// Query text
    pub query: String,
    /// Candidate texts
    pub candidates: Vec<String>,
    /// Number of recommendations
    #[serde(default = "default_k")]
    pub k: usize,
    /// Optional model to use
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

/// Recommend content response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendResponse {
    /// Recommended indices
    pub recommendations: Vec<usize>,
    /// Similarity scores
    pub scores: Vec<f32>,
}

/// Recommend content route handler.
pub struct RecommendRoute;

#[async_trait]
impl DataChannelRoute for RecommendRoute {
    type Request = RecommendRequest;
    type Response = RecommendResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "recommend",
            tags: &["AI", "RAG", "Recommendations"],
            description: "Recommend content based on query similarity",
            supports_streaming: false,
            supports_binary: false,
            requires_auth: true,
            rate_limit_tier: Some("rag"),
            max_payload_size: Some(2 * 1024 * 1024), // 2MB
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
        if req.candidates.is_empty() {
            return Err(WebRtcError::ValidationError {
                field: "candidates".to_string(),
                message: "candidates cannot be empty".to_string(),
            });
        }
        if req.k == 0 {
            return Err(WebRtcError::ValidationError {
                field: "k".to_string(),
                message: "k must be greater than 0".to_string(),
            });
        }
        if req.k > req.candidates.len() {
            return Err(WebRtcError::ValidationError {
                field: "k".to_string(),
                message: "k cannot be greater than number of candidates".to_string(),
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
            route = "recommend",
            candidate_count = req.candidates.len(),
            k = req.k,
            "WebRTC recommend content request"
        );

        let request_value = RequestValue::recommend_content(&req.query, req.candidates.clone(), req.k, req.model.clone());

        let _response = handler.handle_request(request_value).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Recommend content failed");
                WebRtcError::from(e)
            })?;

        // TODO: Extract recommendations from response once backend implements as_recommendations()
        let recommendations = vec![0]; // Placeholder
        let scores = vec![0.0]; // Placeholder

        tracing::info!(request_id = %request_id, "Recommend content successful");

        Ok(RecommendResponse {
            recommendations,
            scores,
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::success(
                "recommend",
                RecommendRequest {
                    query: "test query".to_string(),
                    candidates: vec!["candidate1".to_string(), "candidate2".to_string()],
                    k: 1,
                    model: None,
                },
                RecommendResponse {
                    recommendations: vec![0],
                    scores: vec![0.85],
                },
            ),
            TestCase::error(
                "empty_query",
                RecommendRequest {
                    query: "".to_string(),
                    candidates: vec!["candidate1".to_string()],
                    k: 1,
                    model: None,
                },
                "query cannot be empty",
            ),
        ]
    }
}

crate::enforce_data_channel_route!(RecommendRoute);
