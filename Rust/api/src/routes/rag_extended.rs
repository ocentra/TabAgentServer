//! Extended RAG endpoints (semantic search, similarity, clustering, recommendations).
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

// ==================== SEMANTIC SEARCH ====================

/// Semantic search request.
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct SemanticSearchRequest {
    /// Query text
    pub query: String,
    /// Number of results
    #[serde(default = "default_k")]
    pub k: usize,
    /// Optional filters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<serde_json::Value>,
}

fn default_k() -> usize {
    10
}

fn default_clusters() -> usize {
    5
}

/// Semantic search route handler.
pub struct SemanticSearchRoute;

#[async_trait]
impl RouteHandler for SemanticSearchRoute {
    type Request = SemanticSearchRequest;
    type Response = serde_json::Value;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            path: "/v1/semantic-search",
            method: Method::POST,
            tags: &["RAG"],
            description: "Perform semantic search over documents using embeddings",
            openai_compatible: false,
            idempotent: false,
            requires_auth: false,
            rate_limit_tier: Some("rag"),
        }
    }

    async fn validate_request(req: &Self::Request) -> ApiResult<()> {
        NotEmpty.validate(&req.query)?;
        if req.k == 0 {
            return Err(ApiError::BadRequest("k must be greater than 0".into()));
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
            "Semantic search request received"
        );

        let request = RequestValue::semantic_search(&req.query, req.k, req.filters.clone());
        
        tracing::debug!(request_id = %request_id, "Dispatching to handler");
        let response = state.handle_request(request).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Semantic search failed");
                e
            })?;

        let results_json = response.to_json_value();

        tracing::info!(request_id = %request_id, "Semantic search successful");
        Ok(results_json)
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::error(
                "empty_query",
                SemanticSearchRequest {
                    query: "".to_string(),
                    k: 10,
                    filters: None,
                },
                "cannot be empty",
            ),
            TestCase::error(
                "zero_k",
                SemanticSearchRequest {
                    query: "test query".to_string(),
                    k: 0,
                    filters: None,
                },
                "must be greater than 0",
            ),
        ]
    }
}

crate::enforce_route_handler!(SemanticSearchRoute);

// Implement RegisterableRoute with Axum 0.8 compatible handler
crate::impl_registerable_route!(SemanticSearchRoute);

// ==================== SIMILARITY ====================

/// Similarity request.
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct SimilarityRequest {
    /// First text
    pub text1: String,
    /// Second text
    pub text2: String,
    /// Model for embeddings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

/// Similarity route handler.
pub struct SimilarityRoute;

#[async_trait]
impl RouteHandler for SimilarityRoute {
    type Request = SimilarityRequest;
    type Response = serde_json::Value;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            path: "/v1/similarity",
            method: Method::POST,
            tags: &["RAG"],
            description: "Calculate semantic similarity between two texts",
            openai_compatible: false,
            idempotent: true, // Same inputs = same output
            requires_auth: false,
            rate_limit_tier: Some("rag"),
        }
    }

    async fn validate_request(req: &Self::Request) -> ApiResult<()> {
        NotEmpty.validate(&req.text1)?;
        NotEmpty.validate(&req.text2)?;
        Ok(())
    }

    async fn handle<S>(req: Self::Request, state: &S) -> ApiResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(
            request_id = %request_id,
            text1_length = req.text1.len(),
            text2_length = req.text2.len(),
            model = ?req.model,
            "Similarity request received"
        );

        let request = RequestValue::calculate_similarity(&req.text1, &req.text2, req.model.clone());
        
        tracing::debug!(request_id = %request_id, "Dispatching to handler");
        let response = state.handle_request(request).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Similarity calculation failed");
                e
            })?;

        let similarity_json = response.to_json_value();

        tracing::info!(request_id = %request_id, "Similarity calculation successful");
        Ok(similarity_json)
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::error(
                "empty_text1",
                SimilarityRequest {
                    text1: "".to_string(),
                    text2: "test".to_string(),
                    model: None,
                },
                "cannot be empty",
            ),
            TestCase::error(
                "empty_text2",
                SimilarityRequest {
                    text1: "test".to_string(),
                    text2: "".to_string(),
                    model: None,
                },
                "cannot be empty",
            ),
        ]
    }
}

crate::enforce_route_handler!(SimilarityRoute);

// Implement RegisterableRoute with Axum 0.8 compatible handler
crate::impl_registerable_route!(SimilarityRoute);

// ==================== EVALUATE EMBEDDINGS ====================

/// Evaluate embeddings request.
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct EvaluateEmbeddingsRequest {
    /// Model identifier
    pub model: String,
    /// Test queries
    pub queries: Vec<String>,
    /// Ground truth documents
    pub documents: Vec<String>,
}

/// Evaluate embeddings route handler.
pub struct EvaluateEmbeddingsRoute;

#[async_trait]
impl RouteHandler for EvaluateEmbeddingsRoute {
    type Request = EvaluateEmbeddingsRequest;
    type Response = serde_json::Value;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            path: "/v1/evaluate-embeddings",
            method: Method::POST,
            tags: &["RAG"],
            description: "Evaluate embedding model quality using queries and ground truth documents",
            openai_compatible: false,
            idempotent: true, // Same inputs = same metrics
            requires_auth: false,
            rate_limit_tier: Some("rag"),
        }
    }

    async fn validate_request(req: &Self::Request) -> ApiResult<()> {
        NotEmpty.validate(&req.model)?;
        VecNotEmpty::<String>::new().validate(&req.queries)?;
        VecNotEmpty::<String>::new().validate(&req.documents)?;
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
            query_count = req.queries.len(),
            document_count = req.documents.len(),
            "Evaluate embeddings request received"
        );

        let request = RequestValue::evaluate_embeddings(&req.model, req.queries.clone(), req.documents.clone());
        
        tracing::debug!(request_id = %request_id, "Dispatching to handler");
        let response = state.handle_request(request).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Evaluate embeddings failed");
                e
            })?;

        let eval_json = response.to_json_value();

        tracing::info!(request_id = %request_id, "Evaluate embeddings successful");
        Ok(eval_json)
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::error(
                "empty_model",
                EvaluateEmbeddingsRequest {
                    model: "".to_string(),
                    queries: vec!["query".to_string()],
                    documents: vec!["doc".to_string()],
                },
                "cannot be empty",
            ),
            TestCase::error(
                "empty_queries",
                EvaluateEmbeddingsRequest {
                    model: "test-model".to_string(),
                    queries: vec![],
                    documents: vec!["doc".to_string()],
                },
                "Array cannot be empty",
            ),
        ]
    }
}

crate::enforce_route_handler!(EvaluateEmbeddingsRoute);

// Implement RegisterableRoute with Axum 0.8 compatible handler
crate::impl_registerable_route!(EvaluateEmbeddingsRoute);

// ==================== CLUSTER ====================

/// Cluster request.
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct ClusterRequest {
    /// Documents to cluster
    pub documents: Vec<String>,
    /// Number of clusters
    #[serde(default = "default_clusters")]
    pub n_clusters: usize,
    /// Model for embeddings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

/// Cluster route handler.
pub struct ClusterRoute;

#[async_trait]
impl RouteHandler for ClusterRoute {
    type Request = ClusterRequest;
    type Response = serde_json::Value;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            path: "/v1/cluster",
            method: Method::POST,
            tags: &["RAG"],
            description: "Cluster documents using embedding-based k-means clustering",
            openai_compatible: false,
            idempotent: true, // Same inputs = same clusters
            requires_auth: false,
            rate_limit_tier: Some("rag"),
        }
    }

    async fn validate_request(req: &Self::Request) -> ApiResult<()> {
        VecNotEmpty::<String>::new().validate(&req.documents)?;
        if req.n_clusters == 0 {
            return Err(ApiError::BadRequest("n_clusters must be greater than 0".into()));
        }
        if req.n_clusters > req.documents.len() {
            return Err(ApiError::BadRequest(
                format!("n_clusters ({}) cannot exceed document count ({})", req.n_clusters, req.documents.len())
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
            document_count = req.documents.len(),
            n_clusters = req.n_clusters,
            model = ?req.model,
            "Cluster request received"
        );

        let request = RequestValue::cluster_documents(req.documents.clone(), req.n_clusters, req.model.clone());
        
        tracing::debug!(request_id = %request_id, "Dispatching to handler");
        let response = state.handle_request(request).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Clustering failed");
                e
            })?;

        let cluster_json = response.to_json_value();

        tracing::info!(request_id = %request_id, "Clustering successful");
        Ok(cluster_json)
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::error(
                "empty_documents",
                ClusterRequest {
                    documents: vec![],
                    n_clusters: 5,
                    model: None,
                },
                "Array cannot be empty",
            ),
            TestCase::error(
                "zero_clusters",
                ClusterRequest {
                    documents: vec!["doc1".to_string(), "doc2".to_string()],
                    n_clusters: 0,
                    model: None,
                },
                "must be greater than 0",
            ),
        ]
    }
}

crate::enforce_route_handler!(ClusterRoute);

// Implement RegisterableRoute with Axum 0.8 compatible handler
crate::impl_registerable_route!(ClusterRoute);

// ==================== RECOMMEND ====================

/// Recommend request.
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct RecommendRequest {
    /// Query or seed document
    pub query: String,
    /// Candidate documents
    pub candidates: Vec<String>,
    /// Number of recommendations
    #[serde(default = "default_k")]
    pub top_n: usize,
    /// Model for embeddings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

/// Recommend route handler.
pub struct RecommendRoute;

#[async_trait]
impl RouteHandler for RecommendRoute {
    type Request = RecommendRequest;
    type Response = serde_json::Value;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            path: "/v1/recommend",
            method: Method::POST,
            tags: &["RAG"],
            description: "Get document recommendations based on semantic similarity to a query",
            openai_compatible: false,
            idempotent: true, // Same inputs = same recommendations
            requires_auth: false,
            rate_limit_tier: Some("rag"),
        }
    }

    async fn validate_request(req: &Self::Request) -> ApiResult<()> {
        NotEmpty.validate(&req.query)?;
        VecNotEmpty::<String>::new().validate(&req.candidates)?;
        if req.top_n == 0 {
            return Err(ApiError::BadRequest("top_n must be greater than 0".into()));
        }
        if req.top_n > req.candidates.len() {
            return Err(ApiError::BadRequest(
                format!("top_n ({}) cannot exceed candidate count ({})", req.top_n, req.candidates.len())
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
            candidate_count = req.candidates.len(),
            top_n = req.top_n,
            model = ?req.model,
            "Recommend request received"
        );

        let request = RequestValue::recommend_content(&req.query, req.candidates.clone(), req.top_n, req.model.clone());
        
        tracing::debug!(request_id = %request_id, "Dispatching to handler");
        let response = state.handle_request(request).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Recommendation failed");
                e
            })?;

        let recommend_json = response.to_json_value();

        tracing::info!(request_id = %request_id, "Recommendation successful");
        Ok(recommend_json)
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::error(
                "empty_query",
                RecommendRequest {
                    query: "".to_string(),
                    candidates: vec!["doc".to_string()],
                    top_n: 1,
                    model: None,
                },
                "cannot be empty",
            ),
            TestCase::error(
                "empty_candidates",
                RecommendRequest {
                    query: "query".to_string(),
                    candidates: vec![],
                    top_n: 1,
                    model: None,
                },
                "Array cannot be empty",
            ),
        ]
    }
}

crate::enforce_route_handler!(RecommendRoute);

// Implement RegisterableRoute with Axum 0.8 compatible handler
crate::impl_registerable_route!(RecommendRoute);

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_semantic_search_validation() {
        let req = SemanticSearchRequest {
            query: "test query".to_string(),
            k: 10,
            filters: None,
        };
        assert!(SemanticSearchRoute::validate_request(&req).await.is_ok());
    }

    #[tokio::test]
    async fn test_similarity_validation() {
        let req = SimilarityRequest {
            text1: "text1".to_string(),
            text2: "text2".to_string(),
            model: None,
        };
        assert!(SimilarityRoute::validate_request(&req).await.is_ok());
    }

    #[tokio::test]
    async fn test_cluster_validation() {
        let req = ClusterRequest {
            documents: vec!["doc1".to_string(), "doc2".to_string()],
            n_clusters: 2,
            model: None,
        };
        assert!(ClusterRoute::validate_request(&req).await.is_ok());
    }

    #[test]
    fn test_metadata() {
        let meta = SemanticSearchRoute::metadata();
        assert!(!meta.idempotent); // Search may change with index updates
        
        let meta2 = SimilarityRoute::metadata();
        assert!(meta2.idempotent); // Similarity is deterministic
        
        let meta3 = ClusterRoute::metadata();
        assert!(meta3.idempotent); // Clustering is deterministic
    }
}
