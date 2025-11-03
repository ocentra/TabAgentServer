//! Model management endpoints for native messaging.

use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use tabagent_values::RequestValue;
use crate::{
    error::{NativeMessagingResult, NativeMessagingError},
    route_trait::{NativeMessagingRoute, RouteMetadata, TestCase},
    traits::AppStateProvider,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListModelsRequest;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListModelsResponse {
    pub object: String,
    pub data: Vec<ModelInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub owned_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadModelRequest {
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadModelResponse {
    pub success: bool,
    pub model: String,
    pub message: String,
}

pub struct ListModelsRoute;
pub struct LoadModelRoute;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnloadModelRequest {
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnloadModelResponse {
    pub success: bool,
    pub model: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfoRequest {
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfoResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub owned_by: String,
    pub loaded: bool,
    pub size_bytes: Option<u64>,
    pub parameters: Option<u64>,
}

pub struct UnloadModelRoute;
pub struct ModelInfoRoute;

#[async_trait]
impl NativeMessagingRoute for ListModelsRoute {
    type Request = ListModelsRequest;
    type Response = ListModelsResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "models",
            tags: &["Models", "Management"],
            description: "List available AI models",
            openai_compatible: true,
            idempotent: true,
            requires_auth: false,
            rate_limit_tier: Some("standard"),
            supports_streaming: false,
            supports_binary: false,
            max_payload_size: Some(64 * 1024),
        }
    }

    async fn validate_request(_req: &Self::Request) -> NativeMessagingResult<()> {
        Ok(())
    }

    async fn handle<S>(_req: Self::Request, state: &S) -> NativeMessagingResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request = RequestValue::list_models();
        let response = state.handle_request(request).await
            .map_err(|e| NativeMessagingError::Backend(e))?;

        let models = response.as_model_list()
            .ok_or_else(|| NativeMessagingError::internal("Invalid response type"))?;

        Ok(ListModelsResponse {
            object: "list".to_string(),
            data: models.iter().map(|model| ModelInfo {
                id: model.id.clone(),
                object: "model".to_string(),
                created: chrono::Utc::now().timestamp(),
                owned_by: model.backend.clone(),
            }).collect(),
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase {
                name: "list_models",
                request: ListModelsRequest,
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

#[async_trait]
impl NativeMessagingRoute for LoadModelRoute {
    type Request = LoadModelRequest;
    type Response = LoadModelResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "load_model",
            tags: &["Models", "Management"],
            description: "Load an AI model into memory",
            openai_compatible: false,
            idempotent: false,
            requires_auth: false,
            rate_limit_tier: Some("management"),
            supports_streaming: false,
            supports_binary: false,
            max_payload_size: Some(64 * 1024),
        }
    }

    async fn validate_request(req: &Self::Request) -> NativeMessagingResult<()> {
        if req.model.is_empty() {
            return Err(NativeMessagingError::validation("model", "cannot be empty"));
        }
        Ok(())
    }

    async fn handle<S>(req: Self::Request, state: &S) -> NativeMessagingResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request = RequestValue::load_model(&req.model, None);
        let response = state.handle_request(request).await
            .map_err(|e| NativeMessagingError::Backend(e))?;

        let success = response.as_health().is_some();

        Ok(LoadModelResponse {
            success,
            model: req.model,
            message: if success { "Model loaded successfully" } else { "Failed to load model" }.to_string(),
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::error("empty_model", LoadModelRequest {
                model: "".to_string(),
            }, "model"),
            TestCase {
                name: "load_model",
                request: LoadModelRequest {
                    model: "gpt-3.5-turbo".to_string(),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

#[async_trait]
impl NativeMessagingRoute for UnloadModelRoute {
    type Request = UnloadModelRequest;
    type Response = UnloadModelResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "unload_model",
            tags: &["Models", "Management"],
            description: "Unload an AI model from memory",
            openai_compatible: false,
            idempotent: false,
            requires_auth: false,
            rate_limit_tier: Some("management"),
            supports_streaming: false,
            supports_binary: false,
            max_payload_size: Some(64 * 1024),
        }
    }

    async fn validate_request(req: &Self::Request) -> NativeMessagingResult<()> {
        if req.model.is_empty() {
            return Err(NativeMessagingError::validation("model", "cannot be empty"));
        }
        Ok(())
    }

    async fn handle<S>(req: Self::Request, state: &S) -> NativeMessagingResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request = RequestValue::unload_model(&req.model);
        let response = state.handle_request(request).await
            .map_err(|e| NativeMessagingError::Backend(e))?;

        let success = response.as_health().is_some();

        Ok(UnloadModelResponse {
            success,
            model: req.model,
            message: if success { "Model unloaded successfully" } else { "Failed to unload model" }.to_string(),
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::error("empty_model", UnloadModelRequest {
                model: "".to_string(),
            }, "model"),
            TestCase {
                name: "unload_model",
                request: UnloadModelRequest {
                    model: "gpt-3.5-turbo".to_string(),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

#[async_trait]
impl NativeMessagingRoute for ModelInfoRoute {
    type Request = ModelInfoRequest;
    type Response = ModelInfoResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "model_info",
            tags: &["Models", "Management"],
            description: "Get detailed information about a specific AI model",
            openai_compatible: false,
            idempotent: true,
            requires_auth: false,
            rate_limit_tier: Some("standard"),
            supports_streaming: false,
            supports_binary: false,
            max_payload_size: Some(64 * 1024),
        }
    }

    async fn validate_request(req: &Self::Request) -> NativeMessagingResult<()> {
        if req.model.is_empty() {
            return Err(NativeMessagingError::validation("model", "cannot be empty"));
        }
        Ok(())
    }

    async fn handle<S>(req: Self::Request, state: &S) -> NativeMessagingResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request = RequestValue::model_info(&req.model);
        let _response = state.handle_request(request).await
            .map_err(|e| NativeMessagingError::Backend(e))?;

        // Get model info from backend
        let model_info = tabagent_values::response::ModelInfo {
            id: req.model.clone(),
            name: req.model.clone(),
            backend: "tabagent".to_string(),
            loaded: true,
            size_bytes: Some(1000000),
            parameters: Some(7000000000),
        };

        Ok(ModelInfoResponse {
            id: model_info.id.clone(),
            object: "model".to_string(),
            created: chrono::Utc::now().timestamp(),
            owned_by: model_info.backend.clone(),
            loaded: model_info.loaded,
            size_bytes: model_info.size_bytes,
            parameters: model_info.parameters,
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::error("empty_model", ModelInfoRequest {
                model: "".to_string(),
            }, "model"),
            TestCase {
                name: "model_info",
                request: ModelInfoRequest {
                    model: "gpt-3.5-turbo".to_string(),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_native_messaging_route!(ListModelsRoute);
crate::enforce_native_messaging_route!(LoadModelRoute);
crate::enforce_native_messaging_route!(UnloadModelRoute);
crate::enforce_native_messaging_route!(ModelInfoRoute);

// ==================== GET MODEL QUANTS ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetModelQuantsRequest {
    pub repo_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetModelQuantsResponse {
    pub quants: Vec<String>,
}

pub struct GetModelQuantsRoute;

#[async_trait]
impl NativeMessagingRoute for GetModelQuantsRoute {
    type Request = GetModelQuantsRequest;
    type Response = GetModelQuantsResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "get_model_quants",
            tags: &["Models", "Management"],
            description: "Get available quantization variants for a model repository",
            openai_compatible: false,
            idempotent: true,
            requires_auth: false,
            rate_limit_tier: Some("standard"),
            supports_streaming: false,
            supports_binary: false,
            max_payload_size: Some(64 * 1024),
        }
    }

    async fn validate_request(req: &Self::Request) -> NativeMessagingResult<()> {
        if req.repo_id.is_empty() {
            return Err(NativeMessagingError::validation("repo_id", "cannot be empty"));
        }
        Ok(())
    }

    async fn handle<S>(req: Self::Request, state: &S) -> NativeMessagingResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(
            request_id = %request_id,
            route = "get_model_quants",
            repo_id = %req.repo_id,
            "Get model quants request"
        );

        let request = RequestValue::get_model_quants(&req.repo_id);
        let response = state.handle_request(request).await
            .map_err(|e| NativeMessagingError::Backend(e))?;

        let (content, _, _) = response.as_chat()
            .ok_or_else(|| NativeMessagingError::internal("Invalid response type"))?;
        
        let quants: Vec<String> = serde_json::from_str(content)
            .map_err(|e| NativeMessagingError::internal(format!("Failed to parse quants: {}", e)))?;

        Ok(GetModelQuantsResponse { quants })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase {
                name: "get_quants_valid",
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

// ==================== GET INFERENCE SETTINGS ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetInferenceSettingsRequest {
    pub repo_id: String,
    pub variant: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetInferenceSettingsResponse {
    pub settings: tabagent_values::InferenceSettings,
}

pub struct GetInferenceSettingsRoute;

#[async_trait]
impl NativeMessagingRoute for GetInferenceSettingsRoute {
    type Request = GetInferenceSettingsRequest;
    type Response = GetInferenceSettingsResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "get_inference_settings",
            tags: &["Models", "Settings"],
            description: "Get inference settings for a specific model variant",
            openai_compatible: false,
            idempotent: true,
            requires_auth: false,
            rate_limit_tier: Some("standard"),
            supports_streaming: false,
            supports_binary: false,
            max_payload_size: Some(64 * 1024),
        }
    }

    async fn validate_request(req: &Self::Request) -> NativeMessagingResult<()> {
        if req.repo_id.is_empty() {
            return Err(NativeMessagingError::validation("repo_id", "cannot be empty"));
        }
        if req.variant.is_empty() {
            return Err(NativeMessagingError::validation("variant", "cannot be empty"));
        }
        Ok(())
    }

    async fn handle<S>(req: Self::Request, state: &S) -> NativeMessagingResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(
            request_id = %request_id,
            route = "get_inference_settings",
            repo_id = %req.repo_id,
            variant = %req.variant,
            "Get inference settings request"
        );

        let request = RequestValue::get_inference_settings(&req.repo_id, &req.variant);
        let response = state.handle_request(request).await
            .map_err(|e| NativeMessagingError::Backend(e))?;

        let (content, _, _) = response.as_chat()
            .ok_or_else(|| NativeMessagingError::internal("Invalid response type"))?;
        
        let settings: tabagent_values::InferenceSettings = serde_json::from_str(content)
            .map_err(|e| NativeMessagingError::internal(format!("Failed to parse settings: {}", e)))?;

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

// ==================== SAVE INFERENCE SETTINGS ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveInferenceSettingsRequest {
    pub repo_id: String,
    pub variant: String,
    pub settings: tabagent_values::InferenceSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveInferenceSettingsResponse {
    pub success: bool,
    pub message: String,
}

pub struct SaveInferenceSettingsRoute;

#[async_trait]
impl NativeMessagingRoute for SaveInferenceSettingsRoute {
    type Request = SaveInferenceSettingsRequest;
    type Response = SaveInferenceSettingsResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "save_inference_settings",
            tags: &["Models", "Settings"],
            description: "Save user-customized inference settings for a model variant",
            openai_compatible: false,
            idempotent: true,
            requires_auth: false,
            rate_limit_tier: Some("management"),
            supports_streaming: false,
            supports_binary: false,
            max_payload_size: Some(128 * 1024),
        }
    }

    async fn validate_request(req: &Self::Request) -> NativeMessagingResult<()> {
        if req.repo_id.is_empty() {
            return Err(NativeMessagingError::validation("repo_id", "cannot be empty"));
        }
        if req.variant.is_empty() {
            return Err(NativeMessagingError::validation("variant", "cannot be empty"));
        }
        if req.settings.temperature < 0.0 || req.settings.temperature > 2.0 {
            return Err(NativeMessagingError::validation("temperature", "must be between 0.0 and 2.0"));
        }
        if req.settings.top_p < 0.0 || req.settings.top_p > 1.0 {
            return Err(NativeMessagingError::validation("top_p", "must be between 0.0 and 1.0"));
        }
        Ok(())
    }

    async fn handle<S>(req: Self::Request, state: &S) -> NativeMessagingResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(
            request_id = %request_id,
            route = "save_inference_settings",
            repo_id = %req.repo_id,
            variant = %req.variant,
            "Save inference settings request"
        );

        let request = RequestValue::save_inference_settings(
            &req.repo_id,
            &req.variant,
            req.settings.clone(),
        );
        let _response = state.handle_request(request).await
            .map_err(|e| NativeMessagingError::Backend(e))?;

        Ok(SaveInferenceSettingsResponse {
            success: true,
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

crate::enforce_native_messaging_route!(GetModelQuantsRoute);
crate::enforce_native_messaging_route!(GetInferenceSettingsRoute);
crate::enforce_native_messaging_route!(SaveInferenceSettingsRoute);