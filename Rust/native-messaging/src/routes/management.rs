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
pub struct PullModelRequest {
    pub model: String,
    #[serde(default)]
    pub variant: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullModelResponse {
    pub success: bool,
    pub model: String,
    pub message: String,
    pub progress: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteModelRequest {
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteModelResponse {
    pub success: bool,
    pub model: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetLoadedModelsRequest;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetLoadedModelsResponse {
    pub models: Vec<LoadedModelInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadedModelInfo {
    pub id: String,
    pub name: String,
    pub memory_usage: u64,
    pub load_time: i64,
}

pub struct PullModelRoute;
pub struct DeleteModelRoute;
pub struct GetLoadedModelsRoute;
// TODO: TIER 2 - Implement these routes
#[allow(dead_code)]
pub struct SelectModelRoute;
#[allow(dead_code)]
pub struct GetEmbeddingModelsRoute;
#[allow(dead_code)]
pub struct GetRecipesRoute;
#[allow(dead_code)]
pub struct GetRegisteredModelsRoute;

#[async_trait]
impl NativeMessagingRoute for PullModelRoute {
    type Request = PullModelRequest;
    type Response = PullModelResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "pull_model",
            tags: &["Models", "Management"],
            description: "Download and install a model",
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
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(request_id = %request_id, route = "pull_model", model = %req.model);

        let request = RequestValue::pull_model(&req.model, req.variant);
        let response = state.handle_request(request).await
            .map_err(|e| NativeMessagingError::Backend(e))?;

        let (success, progress) = response.as_pull_result()
            .ok_or_else(|| NativeMessagingError::internal("Invalid response type"))?;

        Ok(PullModelResponse {
            success: *success,
            model: req.model,
            message: if *success { "Model pulled successfully" } else { "Failed to pull model" }.to_string(),
            progress: *progress,
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::error("empty_model", PullModelRequest {
                model: "".to_string(),
                variant: None,
            }, "model"),
            TestCase {
                name: "pull_model",
                request: PullModelRequest {
                    model: "gpt-3.5-turbo".to_string(),
                    variant: Some("4bit".to_string()),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

#[async_trait]
impl NativeMessagingRoute for DeleteModelRoute {
    type Request = DeleteModelRequest;
    type Response = DeleteModelResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "delete_model",
            tags: &["Models", "Management"],
            description: "Delete a model from storage",
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
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(request_id = %request_id, route = "delete_model", model = %req.model);

        let request = RequestValue::delete_model(&req.model);
        let response = state.handle_request(request).await
            .map_err(|e| NativeMessagingError::Backend(e))?;

        let success = response.as_delete_result()
            .ok_or_else(|| NativeMessagingError::internal("Invalid response type"))?;

        Ok(DeleteModelResponse {
            success: *success,
            model: req.model,
            message: if *success { "Model deleted successfully" } else { "Failed to delete model" }.to_string(),
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::error("empty_model", DeleteModelRequest {
                model: "".to_string(),
            }, "model"),
            TestCase {
                name: "delete_model",
                request: DeleteModelRequest {
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
impl NativeMessagingRoute for GetLoadedModelsRoute {
    type Request = GetLoadedModelsRequest;
    type Response = GetLoadedModelsResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "get_loaded_models",
            tags: &["Models", "Management"],
            description: "Get list of currently loaded models",
            openai_compatible: false,
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
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(request_id = %request_id, route = "get_loaded_models");

        let request = RequestValue::get_loaded_models();
        let response = state.handle_request(request).await
            .map_err(|e| NativeMessagingError::Backend(e))?;

        let models = response.as_loaded_models()
            .ok_or_else(|| NativeMessagingError::internal("Invalid response type"))?;

        Ok(GetLoadedModelsResponse {
            models: models.iter().map(|model| LoadedModelInfo {
                id: model.id.clone(),
                name: model.name.clone(),
                memory_usage: model.memory_usage,
                load_time: model.load_time,
            }).collect(),
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase {
                name: "get_loaded_models",
                request: GetLoadedModelsRequest,
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_native_messaging_route!(PullModelRoute);
crate::enforce_native_messaging_route!(DeleteModelRoute);
crate::enforce_native_messaging_route!(GetLoadedModelsRoute);