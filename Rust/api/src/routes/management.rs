//! Extended model management endpoints (pull, delete, recipes).
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

// ==================== PULL MODEL ====================

/// Pull model request.
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct PullModelRequest {
    /// Model repository (e.g., "microsoft/phi-2")
    pub model: String,
    /// Quantization format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantization: Option<String>,
}

/// Pull model route handler.
pub struct PullModelRoute;

#[async_trait]
impl RouteHandler for PullModelRoute {
    type Request = PullModelRequest;
    type Response = serde_json::Value;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            path: "/v1/pull",
            method: Method::POST,
            tags: &["Management"],
            description: "Pull/download a model from repository with optional quantization",
            openai_compatible: false,
            idempotent: false, // Pulling might update the model
            requires_auth: false,
            rate_limit_tier: Some("management"),
        }
    }

    async fn validate_request(req: &Self::Request) -> ApiResult<()> {
        NotEmpty.validate(&req.model)?;
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
            quantization = ?req.quantization,
            "Pull model request received"
        );

        let request = RequestValue::pull_model(&req.model, req.quantization.clone());
        
        tracing::debug!(request_id = %request_id, "Dispatching to handler");
        let response = state.handle_request(request).await
            .map_err(|e| {
                tracing::error!(
                    request_id = %request_id,
                    model = %req.model,
                    error = %e,
                    "Pull model failed"
                );
                e
            })?;

        tracing::info!(
            request_id = %request_id,
            model = %req.model,
            "Pull model successful"
        );

        Ok(serde_json::json!({
            "model": req.model,
            "status": "started",
            "message": format!("{:?}", response)
        }))
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            // === VALIDATION ERROR TESTS ===
            TestCase::error(
                "empty_model",
                PullModelRequest {
                    model: "".to_string(),
                    quantization: None,
                },
                "cannot be empty",
            ),
            // === SUCCESS TEST CASES ===
            TestCase {
                name: "pull_model_basic",
                request: PullModelRequest {
                    model: "llama-2-7b".to_string(),
                    quantization: None,
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "pull_model_with_quantization",
                request: PullModelRequest {
                    model: "llama-2-13b".to_string(),
                    quantization: Some("Q4_K_M".to_string()),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "pull_gpt_style_model",
                request: PullModelRequest {
                    model: "gpt-3.5-turbo".to_string(),
                    quantization: None,
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_route_handler!(PullModelRoute);

// Implement RegisterableRoute with Axum 0.8 compatible handler
crate::impl_registerable_route!(PullModelRoute);

// ==================== DELETE MODEL ====================

/// Delete model request.
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct DeleteModelRequest {
    /// Model identifier
    pub model_id: String,
}

/// Delete model route handler.
pub struct DeleteModelRoute;

#[async_trait]
impl RouteHandler for DeleteModelRoute {
    type Request = DeleteModelRequest;
    type Response = serde_json::Value;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            path: "/v1/delete",
            method: Method::DELETE,
            tags: &["Management"],
            description: "Delete a model from local storage",
            openai_compatible: false,
            idempotent: true, // Deleting same model twice has same effect
            requires_auth: false,
            rate_limit_tier: Some("management"),
        }
    }

    async fn validate_request(req: &Self::Request) -> ApiResult<()> {
        NotEmpty.validate(&req.model_id)?;
        Ok(())
    }

    async fn handle<S>(req: Self::Request, state: &S) -> ApiResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(
            request_id = %request_id,
            model_id = %req.model_id,
            "Delete model request received"
        );

        let request = RequestValue::delete_model(&req.model_id);
        
        tracing::debug!(request_id = %request_id, "Dispatching to handler");
        let _response = state.handle_request(request).await
            .map_err(|e| {
                tracing::error!(
                    request_id = %request_id,
                    model_id = %req.model_id,
                    error = %e,
                    "Delete model failed"
                );
                e
            })?;

        tracing::info!(
            request_id = %request_id,
            model_id = %req.model_id,
            "Delete model successful"
        );

        Ok(serde_json::json!({
            "model_id": req.model_id,
            "status": "deleted"
        }))
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::error(
                "empty_model_id",
                DeleteModelRequest {
                    model_id: "".to_string(),
                },
                "cannot be empty",
            ),
        ]
    }
}

crate::enforce_route_handler!(DeleteModelRoute);

// Implement RegisterableRoute with Axum 0.8 compatible handler
crate::impl_registerable_route!(DeleteModelRoute);

// ==================== GET LOADED MODELS ====================

/// Get loaded models request (no parameters).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetLoadedModelsRequest;

/// Get loaded models route handler.
pub struct GetLoadedModelsRoute;

#[async_trait]
impl RouteHandler for GetLoadedModelsRoute {
    type Request = GetLoadedModelsRequest;
    type Response = serde_json::Value;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            path: "/v1/models/loaded",
            method: Method::GET,
            tags: &["Management"],
            description: "Get list of currently loaded models in memory",
            openai_compatible: false,
            idempotent: true,
            requires_auth: false,
            rate_limit_tier: Some("low"),
        }
    }

    async fn validate_request(_req: &Self::Request) -> ApiResult<()> {
        Ok(()) // No validation needed
    }

    async fn handle<S>(_req: Self::Request, state: &S) -> ApiResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(request_id = %request_id, "Get loaded models request received");

        let request = RequestValue::get_loaded_models();
        
        tracing::debug!(request_id = %request_id, "Dispatching to handler");
        let response = state.handle_request(request).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Get loaded models failed");
                e
            })?;

        let models_json = response.to_json_value();

        tracing::info!(request_id = %request_id, "Get loaded models successful");

        Ok(models_json)
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase {
                name: "get_loaded_models_success",
                request: GetLoadedModelsRequest,
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_route_handler!(GetLoadedModelsRoute);

// Implement RegisterableRoute with Axum 0.8 compatible handler
crate::impl_registerable_route!(GetLoadedModelsRoute);

// ==================== SELECT MODEL ====================

/// Select model request.
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct SelectModelRequest {
    /// Model identifier to set as active
    pub model_id: String,
}

/// Select model route handler.
pub struct SelectModelRoute;

#[async_trait]
impl RouteHandler for SelectModelRoute {
    type Request = SelectModelRequest;
    type Response = serde_json::Value;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            path: "/v1/models/select",
            method: Method::POST,
            tags: &["Management"],
            description: "Select a model as the active/default model for inference",
            openai_compatible: false,
            idempotent: true, // Selecting same model multiple times is idempotent
            requires_auth: false,
            rate_limit_tier: Some("management"),
        }
    }

    async fn validate_request(req: &Self::Request) -> ApiResult<()> {
        NotEmpty.validate(&req.model_id)?;
        Ok(())
    }

    async fn handle<S>(req: Self::Request, state: &S) -> ApiResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(
            request_id = %request_id,
            model_id = %req.model_id,
            "Select model request received"
        );

        let request = RequestValue::select_model(&req.model_id);
        
        tracing::debug!(request_id = %request_id, "Dispatching to handler");
        let _response = state.handle_request(request).await
            .map_err(|e| {
                tracing::error!(
                    request_id = %request_id,
                    model_id = %req.model_id,
                    error = %e,
                    "Select model failed"
                );
                e
            })?;

        tracing::info!(
            request_id = %request_id,
            model_id = %req.model_id,
            "Select model successful"
        );

        Ok(serde_json::json!({
            "model_id": req.model_id,
            "status": "selected"
        }))
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::error(
                "empty_model_id",
                SelectModelRequest {
                    model_id: "".to_string(),
                },
                "cannot be empty",
            ),
        ]
    }
}

crate::enforce_route_handler!(SelectModelRoute);

// Implement RegisterableRoute with Axum 0.8 compatible handler
crate::impl_registerable_route!(SelectModelRoute);

// ==================== GET EMBEDDING MODELS ====================

/// Get embedding models request (no parameters).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetEmbeddingModelsRequest;

/// Get embedding models route handler.
pub struct GetEmbeddingModelsRoute;

#[async_trait]
impl RouteHandler for GetEmbeddingModelsRoute {
    type Request = GetEmbeddingModelsRequest;
    type Response = serde_json::Value;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            path: "/v1/embedding-models",
            method: Method::GET,
            tags: &["Management"],
            description: "Get list of available embedding models",
            openai_compatible: false,
            idempotent: true,
            requires_auth: false,
            rate_limit_tier: Some("low"),
        }
    }

    async fn validate_request(_req: &Self::Request) -> ApiResult<()> {
        Ok(()) // No validation needed
    }

    async fn handle<S>(_req: Self::Request, state: &S) -> ApiResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(request_id = %request_id, "Get embedding models request received");

        let request = RequestValue::get_embedding_models();
        
        tracing::debug!(request_id = %request_id, "Dispatching to handler");
        let response = state.handle_request(request).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Get embedding models failed");
                e
            })?;

        let models_json = response.to_json_value();

        tracing::info!(request_id = %request_id, "Get embedding models successful");

        Ok(models_json)
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase {
                name: "get_embedding_models_success",
                request: GetEmbeddingModelsRequest,
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_route_handler!(GetEmbeddingModelsRoute);

// Implement RegisterableRoute with Axum 0.8 compatible handler
crate::impl_registerable_route!(GetEmbeddingModelsRoute);

// ==================== GET RECIPES ====================

/// Hardware recipe.
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, PartialEq)]
pub struct HardwareRecipe {
    /// Recipe identifier (e.g., "onnx-npu", "llama-cuda")
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Backend engine
    pub engine: String,
    /// Hardware requirements
    pub hardware: Vec<String>,
    /// Available on this system
    pub available: bool,
}

/// Get recipes request (no parameters).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetRecipesRequest;

/// Get recipes response.
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, PartialEq)]
pub struct GetRecipesResponse {
    /// Available hardware recipes
    pub recipes: Vec<HardwareRecipe>,
}

/// Get recipes route handler.
pub struct GetRecipesRoute;

#[async_trait]
impl RouteHandler for GetRecipesRoute {
    type Request = GetRecipesRequest;
    type Response = GetRecipesResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            path: "/v1/recipes",
            method: Method::GET,
            tags: &["Management"],
            description: "Get available hardware configuration recipes (ONNX, GGUF, CUDA, etc.)",
            openai_compatible: false,
            idempotent: true,
            requires_auth: false,
            rate_limit_tier: Some("low"),
        }
    }

    async fn validate_request(_req: &Self::Request) -> ApiResult<()> {
        Ok(()) // No validation needed
    }

    async fn handle<S>(_req: Self::Request, state: &S) -> ApiResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(request_id = %request_id, "Get recipes request received");

        let request = RequestValue::get_recipes();
        
        tracing::debug!(request_id = %request_id, "Dispatching to handler");
        let response = state.handle_request(request).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Get recipes failed");
                e
            })?;

        let recipes_json = response.to_json_value();
        let recipes: GetRecipesResponse = serde_json::from_value(recipes_json)
            .map_err(|e| ApiError::Internal(format!("Failed to parse recipes: {}", e)))?;

        tracing::info!(
            request_id = %request_id,
            recipe_count = recipes.recipes.len(),
            "Get recipes successful"
        );

        Ok(recipes)
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase {
                name: "get_recipes_success",
                request: GetRecipesRequest,
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_route_handler!(GetRecipesRoute);

// Implement RegisterableRoute with Axum 0.8 compatible handler
crate::impl_registerable_route!(GetRecipesRoute);

// ==================== GET REGISTERED MODELS ====================

/// Get registered models request (no parameters).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetRegisteredModelsRequest;

/// Get registered models route handler.
pub struct GetRegisteredModelsRoute;

#[async_trait]
impl RouteHandler for GetRegisteredModelsRoute {
    type Request = GetRegisteredModelsRequest;
    type Response = serde_json::Value;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            path: "/v1/models/registered",
            method: Method::GET,
            tags: &["Management"],
            description: "Get list of registered/available models in the model registry",
            openai_compatible: false,
            idempotent: true,
            requires_auth: false,
            rate_limit_tier: Some("low"),
        }
    }

    async fn validate_request(_req: &Self::Request) -> ApiResult<()> {
        Ok(()) // No validation needed
    }

    async fn handle<S>(_req: Self::Request, state: &S) -> ApiResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(request_id = %request_id, "Get registered models request received");

        // Use list_models to get all available models
        let request = RequestValue::list_models();
        
        tracing::debug!(request_id = %request_id, "Dispatching to handler");
        let response = state.handle_request(request).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Get registered models failed");
                e
            })?;

        let models_json = response.to_json_value();

        tracing::info!(request_id = %request_id, "Get registered models successful");

        Ok(models_json)
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase {
                name: "get_registered_models_success",
                request: GetRegisteredModelsRequest,
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_route_handler!(GetRegisteredModelsRoute);

// Implement RegisterableRoute with Axum 0.8 compatible handler
crate::impl_registerable_route!(GetRegisteredModelsRoute);

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pull_model_validation() {
        let req = PullModelRequest {
            model: "test-model".to_string(),
            quantization: Some("q4_0".to_string()),
        };
        assert!(PullModelRoute::validate_request(&req).await.is_ok());
    }

    #[tokio::test]
    async fn test_pull_model_validation_empty() {
        let req = PullModelRequest {
            model: "".to_string(),
            quantization: None,
        };
        assert!(PullModelRoute::validate_request(&req).await.is_err());
    }

    #[tokio::test]
    async fn test_delete_model_validation() {
        let req = DeleteModelRequest {
            model_id: "test-model".to_string(),
        };
        assert!(DeleteModelRoute::validate_request(&req).await.is_ok());
    }

    #[tokio::test]
    async fn test_select_model_validation() {
        let req = SelectModelRequest {
            model_id: "test-model".to_string(),
        };
        assert!(SelectModelRoute::validate_request(&req).await.is_ok());
    }

    #[test]
    fn test_metadata() {
        let meta = PullModelRoute::metadata();
        assert!(!meta.idempotent); // Pull can update
        
        let meta2 = DeleteModelRoute::metadata();
        assert!(meta2.idempotent); // Delete is idempotent
        
        let meta3 = SelectModelRoute::metadata();
        assert!(meta3.idempotent); // Select is idempotent
        
        let meta4 = GetRecipesRoute::metadata();
        assert!(meta4.idempotent); // Get is idempotent
    }
}
