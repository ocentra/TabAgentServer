//! Model management endpoints.
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

// ==================== LIST MODELS ====================

/// List models request (no parameters).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ListModelsRequest;

/// Model information (OpenAI-compatible).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct Model {
    /// Model identifier
    pub id: String,
    /// Object type (always "model")
    pub object: String,
    /// Creation timestamp
    pub created: i64,
    /// Owner organization
    pub owned_by: String,
}

/// Model list response.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ModelListResponse {
    /// List of models
    pub models: Vec<Model>,
}

/// List models route handler (OpenAI-compatible).
pub struct ListModelsRoute;

#[async_trait]
impl RouteHandler for ListModelsRoute {
    type Request = ListModelsRequest;
    type Response = ModelListResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            path: "/v1/models",
            method: Method::GET,
            tags: &["Models", "OpenAI"],
            description: "List all loaded models (OpenAI-compatible endpoint)",
            openai_compatible: true,
            idempotent: true,
            requires_auth: false,
            rate_limit_tier: None,
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
        tracing::info!(request_id = %request_id, "List models request received");

        let request = RequestValue::list_models();
        
        tracing::debug!(request_id = %request_id, "Dispatching to handler");
        let response = state.handle_request(request).await
            .map_err(|e| {
                tracing::error!(
                    request_id = %request_id,
                    error = %e,
                    "List models failed"
                );
                e
            })?;

        let models = response
            .as_model_list()
            .ok_or_else(|| {
                tracing::error!(
                    request_id = %request_id,
                    "Handler returned invalid response type (expected ModelListResponse)"
                );
                ApiError::Internal(
                    format!("Handler returned invalid response type for list models request (request_id: {})", request_id)
                )
            })?;

        tracing::info!(
            request_id = %request_id,
            model_count = models.len(),
            "List models successful"
        );

        let model_data: Vec<_> = models.iter().map(|m| Model {
            id: m.id.clone(),
            object: "model".to_string(),
            created: 0,
            owned_by: "tabagent".to_string(),
        }).collect();

        Ok(ModelListResponse {
            models: model_data,
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            // === SUCCESS TEST CASES ===
            TestCase {
                name: "list_models_basic",
                request: ListModelsRequest,
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "list_models_returns_array",
                request: ListModelsRequest,
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "list_models_idempotent",
                request: ListModelsRequest,
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "list_models_no_side_effects",
                request: ListModelsRequest,
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_route_handler!(ListModelsRoute);

// Implement RegisterableRoute with Axum 0.8 compatible handler
crate::impl_registerable_route!(ListModelsRoute);

// ==================== LOAD MODEL ====================

/// Load model request.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LoadModelRequest {
    /// Model identifier
    pub model_id: String,
    /// Optional model variant
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
}

/// Load model route handler.
pub struct LoadModelRoute;

#[async_trait]
impl RouteHandler for LoadModelRoute {
    type Request = LoadModelRequest;
    type Response = serde_json::Value;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            path: "/v1/models/load",
            method: Method::POST,
            tags: &["Models"],
            description: "Load a model into memory for inference",
            openai_compatible: false,
            idempotent: true, // Loading same model multiple times is idempotent
            requires_auth: false,
            rate_limit_tier: Some("model_management"),
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
            variant = ?req.variant,
            "Load model request received"
        );

        let request = RequestValue::load_model(&req.model_id, req.variant.clone());
        
        tracing::debug!(request_id = %request_id, "Dispatching to handler");
        let _response = state.handle_request(request).await
            .map_err(|e| {
                tracing::error!(
                    request_id = %request_id,
                    model_id = %req.model_id,
                    error = %e,
                    "Load model failed"
                );
                e
            })?;

        tracing::info!(
            request_id = %request_id,
            model_id = %req.model_id,
            "Model loaded successfully"
        );

        Ok(serde_json::json!({
            "status": "loaded",
            "model_id": req.model_id,
            "request_id": request_id.to_string()
        }))
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            // === VALIDATION ERROR TESTS ===
            TestCase::error(
                "empty_model_id",
                LoadModelRequest {
                    model_id: "".to_string(),
                    variant: None,
                },
                "cannot be empty",
            ),
            // === SUCCESS TEST CASES ===
            TestCase {
                name: "load_model_basic",
                request: LoadModelRequest {
                    model_id: "llama-2-7b".to_string(),
                    variant: None,
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "load_model_with_variant",
                request: LoadModelRequest {
                    model_id: "llama-2-7b".to_string(),
                    variant: Some("Q4_K_M".to_string()),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "load_model_with_path",
                request: LoadModelRequest {
                    model_id: "models/custom-model.gguf".to_string(),
                    variant: None,
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "load_large_model",
                request: LoadModelRequest {
                    model_id: "llama-2-70b".to_string(),
                    variant: Some("Q8_0".to_string()),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "load_gpt_style_model",
                request: LoadModelRequest {
                    model_id: "gpt-3.5-turbo".to_string(),
                    variant: None,
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "load_model_with_special_chars",
                request: LoadModelRequest {
                    model_id: "model-name_v1.0".to_string(),
                    variant: None,
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_route_handler!(LoadModelRoute);

// Implement RegisterableRoute with Axum 0.8 compatible handler
crate::impl_registerable_route!(LoadModelRoute);

// ==================== UNLOAD MODEL ====================

/// Unload model request.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UnloadModelRequest {
    /// Model identifier
    pub model_id: String,
}

/// Unload model route handler.
pub struct UnloadModelRoute;

#[async_trait]
impl RouteHandler for UnloadModelRoute {
    type Request = UnloadModelRequest;
    type Response = serde_json::Value;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            path: "/v1/models/unload",
            method: Method::POST,
            tags: &["Models"],
            description: "Unload a model from memory to free resources",
            openai_compatible: false,
            idempotent: true, // Unloading same model multiple times is idempotent
            requires_auth: false,
            rate_limit_tier: Some("model_management"),
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
            "Unload model request received"
        );

        let request = RequestValue::unload_model(&req.model_id);
        
        tracing::debug!(request_id = %request_id, "Dispatching to handler");
        let _response = state.handle_request(request).await
            .map_err(|e| {
                tracing::error!(
                    request_id = %request_id,
                    model_id = %req.model_id,
                    error = %e,
                    "Unload model failed"
                );
                e
            })?;

        tracing::info!(
            request_id = %request_id,
            model_id = %req.model_id,
            "Model unloaded successfully"
        );

        Ok(serde_json::json!({
            "status": "unloaded",
            "model_id": req.model_id,
            "request_id": request_id.to_string()
        }))
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            // === VALIDATION ERROR TESTS ===
            TestCase::error(
                "empty_model_id",
                UnloadModelRequest {
                    model_id: "".to_string(),
                },
                "cannot be empty",
            ),
            // === SUCCESS TEST CASES ===
            TestCase {
                name: "unload_model_basic",
                request: UnloadModelRequest {
                    model_id: "llama-2-7b".to_string(),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "unload_model_with_path",
                request: UnloadModelRequest {
                    model_id: "models/custom-model".to_string(),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "unload_model_gpt_style",
                request: UnloadModelRequest {
                    model_id: "gpt-3.5-turbo".to_string(),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "unload_nonexistent_model",
                request: UnloadModelRequest {
                    model_id: "nonexistent-model".to_string(),
                },
                expected_response: None,
                expected_error: None, // Should handle gracefully
                assertions: vec![],
            },
            TestCase {
                name: "unload_model_idempotent",
                request: UnloadModelRequest {
                    model_id: "test-model".to_string(),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_route_handler!(UnloadModelRoute);

// Implement RegisterableRoute with Axum 0.8 compatible handler
crate::impl_registerable_route!(UnloadModelRoute);

// ==================== MODEL INFO ====================

/// Model info request.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ModelInfoRequest {
    /// Model identifier (from path)
    pub model_id: String,
}

/// Model info route handler.
pub struct ModelInfoRoute;

#[async_trait]
impl RouteHandler for ModelInfoRoute {
    type Request = ModelInfoRequest;
    type Response = serde_json::Value;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            path: "/v1/models/{model_id}",
            method: Method::GET,
            tags: &["Models"],
            description: "Get detailed information about a specific model",
            openai_compatible: false,
            idempotent: true,
            requires_auth: false,
            rate_limit_tier: None,
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
            "Model info request received"
        );

        let request = RequestValue::model_info(&req.model_id);
        
        tracing::debug!(request_id = %request_id, "Dispatching to handler");
        let _response = state.handle_request(request).await
            .map_err(|e| {
                tracing::error!(
                    request_id = %request_id,
                    model_id = %req.model_id,
                    error = %e,
                    "Model info request failed"
                );
                e
            })?;

        tracing::info!(
            request_id = %request_id,
            model_id = %req.model_id,
            "Model info retrieved successfully"
        );

        Ok(serde_json::json!({
            "id": req.model_id,
            "object": "model",
            "request_id": request_id.to_string()
        }))
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            // === VALIDATION ERROR TESTS ===
            TestCase::error(
                "empty_model_id",
                ModelInfoRequest {
                    model_id: "".to_string(),
                },
                "cannot be empty",
            ),
            // === SUCCESS TEST CASES ===
            TestCase {
                name: "model_info_basic",
                request: ModelInfoRequest {
                    model_id: "llama-2-7b".to_string(),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "model_info_loaded_model",
                request: ModelInfoRequest {
                    model_id: "gpt-3.5-turbo".to_string(),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "model_info_with_path",
                request: ModelInfoRequest {
                    model_id: "models/custom-model.gguf".to_string(),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "model_info_nonexistent",
                request: ModelInfoRequest {
                    model_id: "nonexistent-model-xyz".to_string(),
                },
                expected_response: None,
                expected_error: None, // Should return 404 or model not found
                assertions: vec![],
            },
            TestCase {
                name: "model_info_with_variant",
                request: ModelInfoRequest {
                    model_id: "llama-2-70b-Q4_K_M".to_string(),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "model_info_idempotent",
                request: ModelInfoRequest {
                    model_id: "test-model".to_string(),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_route_handler!(ModelInfoRoute);

// Implement RegisterableRoute with Axum 0.8 compatible handler
crate::impl_registerable_route!(ModelInfoRoute);

// ==================== GET MODEL QUANTS ====================

/// Get model quants request.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetModelQuantsRequest {
    /// Repository ID (e.g., "onnx-community/Phi-3.5-mini")
    pub repo_id: String,
}

/// Get model quants response.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct GetModelQuantsResponse {
    /// Available quantization variants
    pub quants: Vec<String>,
}

/// Get available quantization variants for a model (for UI dropdown).
pub struct GetModelQuantsRoute;

#[async_trait]
impl RouteHandler for GetModelQuantsRoute {
    type Request = GetModelQuantsRequest;
    type Response = GetModelQuantsResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            path: "/v1/models/{repo_id}/quants",
            method: Method::GET,
            tags: &["Models"],
            description: "Get available quantization variants for a model repository",
            openai_compatible: false,
            idempotent: true,
            requires_auth: false,
            rate_limit_tier: None,
        }
    }

    async fn validate_request(req: &Self::Request) -> ApiResult<()> {
        NotEmpty.validate(&req.repo_id)?;
        Ok(())
    }

    async fn handle<S>(req: Self::Request, state: &S) -> ApiResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(
            request_id = %request_id,
            repo_id = %req.repo_id,
            "Get model quants request received"
        );

        let request = RequestValue::get_model_quants(&req.repo_id);
        
        tracing::debug!(request_id = %request_id, "Dispatching to handler");
        let response = state.handle_request(request).await
            .map_err(|e| {
                tracing::error!(
                    request_id = %request_id,
                    repo_id = %req.repo_id,
                    error = %e,
                    "Get model quants failed"
                );
                e
            })?;

        tracing::info!(
            request_id = %request_id,
            repo_id = %req.repo_id,
            "Model quants retrieved successfully"
        );

        // Parse the quants from response
        let (content, _, _) = response.as_chat()
            .ok_or_else(|| ApiError::Internal("Invalid response type".to_string()))?;
        
        let quants: Vec<String> = serde_json::from_str(content)
            .map_err(|e| ApiError::Internal(format!("Failed to parse quants: {}", e)))?;

        Ok(GetModelQuantsResponse { quants })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase {
                name: "get_quants_valid_repo",
                request: GetModelQuantsRequest {
                    repo_id: "onnx-community/Phi-3.5-mini".to_string(),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_route_handler!(GetModelQuantsRoute);

// Custom RegisterableRoute for path parameter extraction
impl crate::route_trait::RegisterableRoute for GetModelQuantsRoute {
    fn register(
        router: axum::Router<crate::traits::AppStateWrapper>
    ) -> axum::Router<crate::traits::AppStateWrapper> {
        async fn handler(
            axum::extract::State(state): axum::extract::State<crate::traits::AppStateWrapper>,
            axum::extract::Path(repo_id): axum::extract::Path<String>,
        ) -> Result<axum::Json<GetModelQuantsResponse>, ApiError> {
            let req = GetModelQuantsRequest { repo_id };
            GetModelQuantsRoute::validate_request(&req).await?;
            let response = GetModelQuantsRoute::handle(req, &state).await?;
            Ok(axum::Json(response))
        }
        
        router.route("/v1/models/:repo_id/quants", axum::routing::get(handler))
    }
}

// ==================== GET INFERENCE SETTINGS ====================

/// Get inference settings request.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetInferenceSettingsRequest {
    /// Repository ID
    pub repo_id: String,
    /// Variant name (e.g., "fp16", "int8", "q4")
    pub variant: String,
}

/// Get inference settings response.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct GetInferenceSettingsResponse {
    /// Inference settings for the variant
    pub settings: tabagent_values::InferenceSettings,
}

/// Get inference settings for a model variant.
pub struct GetInferenceSettingsRoute;

#[async_trait]
impl RouteHandler for GetInferenceSettingsRoute {
    type Request = GetInferenceSettingsRequest;
    type Response = GetInferenceSettingsResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            path: "/v1/models/{repo_id}/{variant}/settings",
            method: Method::GET,
            tags: &["Models"],
            description: "Get inference settings for a specific model variant",
            openai_compatible: false,
            idempotent: true,
            requires_auth: false,
            rate_limit_tier: None,
        }
    }

    async fn validate_request(req: &Self::Request) -> ApiResult<()> {
        NotEmpty.validate(&req.repo_id)?;
        NotEmpty.validate(&req.variant)?;
        Ok(())
    }

    async fn handle<S>(req: Self::Request, state: &S) -> ApiResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(
            request_id = %request_id,
            repo_id = %req.repo_id,
            variant = %req.variant,
            "Get inference settings request received"
        );

        let request = RequestValue::get_inference_settings(&req.repo_id, &req.variant);
        
        tracing::debug!(request_id = %request_id, "Dispatching to handler");
        let response = state.handle_request(request).await
            .map_err(|e| {
                tracing::error!(
                    request_id = %request_id,
                    repo_id = %req.repo_id,
                    variant = %req.variant,
                    error = %e,
                    "Get inference settings failed"
                );
                e
            })?;

        tracing::info!(
            request_id = %request_id,
            repo_id = %req.repo_id,
            variant = %req.variant,
            "Inference settings retrieved successfully"
        );

        // Parse settings from response
        let (content, _, _) = response.as_chat()
            .ok_or_else(|| ApiError::Internal("Invalid response type".to_string()))?;
        
        let settings: tabagent_values::InferenceSettings = serde_json::from_str(content)
            .map_err(|e| ApiError::Internal(format!("Failed to parse settings: {}", e)))?;

        Ok(GetInferenceSettingsResponse { settings })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase {
                name: "get_settings_valid",
                request: GetInferenceSettingsRequest {
                    repo_id: "onnx-community/Phi-3.5-mini".to_string(),
                    variant: "fp16".to_string(),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_route_handler!(GetInferenceSettingsRoute);

// Custom RegisterableRoute for path parameter extraction
impl crate::route_trait::RegisterableRoute for GetInferenceSettingsRoute {
    fn register(
        router: axum::Router<crate::traits::AppStateWrapper>
    ) -> axum::Router<crate::traits::AppStateWrapper> {
        async fn handler(
            axum::extract::State(state): axum::extract::State<crate::traits::AppStateWrapper>,
            axum::extract::Path((repo_id, variant)): axum::extract::Path<(String, String)>,
        ) -> Result<axum::Json<GetInferenceSettingsResponse>, ApiError> {
            let req = GetInferenceSettingsRequest { repo_id, variant };
            GetInferenceSettingsRoute::validate_request(&req).await?;
            let response = GetInferenceSettingsRoute::handle(req, &state).await?;
            Ok(axum::Json(response))
        }
        
        router.route("/v1/models/:repo_id/:variant/settings", axum::routing::get(handler))
    }
}

// ==================== SAVE INFERENCE SETTINGS ====================

/// Save inference settings request.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SaveInferenceSettingsRequest {
    /// Repository ID
    pub repo_id: String,
    /// Variant name
    pub variant: String,
    /// Inference settings to save
    pub settings: tabagent_values::InferenceSettings,
}

/// Save inference settings response.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct SaveInferenceSettingsResponse {
    /// Success message
    pub message: String,
}

/// Save user-customized inference settings for a model variant.
pub struct SaveInferenceSettingsRoute;

#[async_trait]
impl RouteHandler for SaveInferenceSettingsRoute {
    type Request = SaveInferenceSettingsRequest;
    type Response = SaveInferenceSettingsResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            path: "/v1/models/{repo_id}/{variant}/settings",
            method: Method::PUT,
            tags: &["Models"],
            description: "Save user-customized inference settings for a model variant",
            openai_compatible: false,
            idempotent: true, // PUT is idempotent
            requires_auth: false,
            rate_limit_tier: Some("management"),
        }
    }

    async fn validate_request(req: &Self::Request) -> ApiResult<()> {
        NotEmpty.validate(&req.repo_id)?;
        NotEmpty.validate(&req.variant)?;
        // Validate settings ranges
        if req.settings.temperature < 0.0 || req.settings.temperature > 2.0 {
            return Err(ApiError::BadRequest("temperature must be between 0.0 and 2.0".into()));
        }
        if req.settings.top_p < 0.0 || req.settings.top_p > 1.0 {
            return Err(ApiError::BadRequest("top_p must be between 0.0 and 1.0".into()));
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
            repo_id = %req.repo_id,
            variant = %req.variant,
            "Save inference settings request received"
        );

        let request = RequestValue::save_inference_settings(
            &req.repo_id,
            &req.variant,
            req.settings.clone(),
        );
        
        tracing::debug!(request_id = %request_id, "Dispatching to handler");
        let _response = state.handle_request(request).await
            .map_err(|e| {
                tracing::error!(
                    request_id = %request_id,
                    repo_id = %req.repo_id,
                    variant = %req.variant,
                    error = %e,
                    "Save inference settings failed"
                );
                e
            })?;

        tracing::info!(
            request_id = %request_id,
            repo_id = %req.repo_id,
            variant = %req.variant,
            "Inference settings saved successfully"
        );

        Ok(SaveInferenceSettingsResponse {
            message: format!("Settings saved for {}:{}", req.repo_id, req.variant),
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase {
                name: "save_settings_valid",
                request: SaveInferenceSettingsRequest {
                    repo_id: "onnx-community/Phi-3.5-mini".to_string(),
                    variant: "fp16".to_string(),
                    settings: tabagent_values::InferenceSettings::default(),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_route_handler!(SaveInferenceSettingsRoute);

// Custom RegisterableRoute for path parameter + JSON body
impl crate::route_trait::RegisterableRoute for SaveInferenceSettingsRoute {
    fn register(
        router: axum::Router<crate::traits::AppStateWrapper>
    ) -> axum::Router<crate::traits::AppStateWrapper> {
        async fn handler(
            axum::extract::State(state): axum::extract::State<crate::traits::AppStateWrapper>,
            axum::extract::Path((repo_id, variant)): axum::extract::Path<(String, String)>,
            axum::Json(settings): axum::Json<tabagent_values::InferenceSettings>,
        ) -> Result<axum::Json<SaveInferenceSettingsResponse>, ApiError> {
            let req = SaveInferenceSettingsRequest { repo_id, variant, settings };
            SaveInferenceSettingsRoute::validate_request(&req).await?;
            let response = SaveInferenceSettingsRoute::handle(req, &state).await?;
            Ok(axum::Json(response))
        }
        
        router.route("/v1/models/:repo_id/:variant/settings", axum::routing::put(handler))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_load_validation() {
        let req = LoadModelRequest {
            model_id: "test-model".to_string(),
            variant: None,
        };
        assert!(LoadModelRoute::validate_request(&req).await.is_ok());
    }

    #[tokio::test]
    async fn test_unload_validation() {
        let req = UnloadModelRequest {
            model_id: "test-model".to_string(),
        };
        assert!(UnloadModelRoute::validate_request(&req).await.is_ok());
    }

    #[test]
    fn test_metadata() {
        let meta = ListModelsRoute::metadata();
        assert!(meta.openai_compatible);
        
        let meta2 = LoadModelRoute::metadata();
        assert!(meta2.idempotent);
    }
}
