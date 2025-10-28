//! Resource management endpoints for native messaging.

use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use tabagent_values::{RequestValue, ResponseValue};
use crate::{
    error::{NativeMessagingResult, NativeMessagingError},
    route_trait::{NativeMessagingRoute, RouteMetadata, TestCase},
    traits::AppStateProvider,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetResourcesRequest;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetResourcesResponse {
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub memory_total: u64,
    pub gpu_usage: Option<f32>,
    pub gpu_memory_usage: Option<u64>,
    pub gpu_memory_total: Option<u64>,
    pub disk_usage: u64,
    pub disk_total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EstimateMemoryRequest {
    pub model: String,
    pub context_length: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EstimateMemoryResponse {
    pub model: String,
    pub estimated_memory_mb: u64,
    pub estimated_vram_mb: Option<u64>,
    pub can_load: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityRequest {
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityResponse {
    pub model: String,
    pub compatible: bool,
    pub requirements: Vec<String>,
    pub missing_requirements: Vec<String>,
    pub recommendations: Vec<String>,
}

pub struct GetResourcesRoute;
pub struct EstimateMemoryRoute;
pub struct CompatibilityRoute;

#[async_trait]
impl NativeMessagingRoute for GetResourcesRoute {
    type Request = GetResourcesRequest;
    type Response = GetResourcesResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "get_resources",
            tags: &["Resources", "System"],
            description: "Get current system resource usage",
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
        tracing::info!(request_id = %request_id, route = "get_resources");

        let request = RequestValue::get_resources();
        let response = state.handle_request(request).await
            .map_err(|e| NativeMessagingError::Backend(e))?;

        let (cpu_usage, memory_usage, memory_total, gpu_usage, gpu_memory_usage, gpu_memory_total, disk_usage, disk_total) = response.as_resources()
            .ok_or_else(|| NativeMessagingError::internal("Invalid response type"))?;

        Ok(GetResourcesResponse {
            cpu_usage,
            memory_usage,
            memory_total,
            gpu_usage,
            gpu_memory_usage,
            gpu_memory_total,
            disk_usage,
            disk_total,
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase {
                name: "get_resources",
                request: GetResourcesRequest,
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

#[async_trait]
impl NativeMessagingRoute for EstimateMemoryRoute {
    type Request = EstimateMemoryRequest;
    type Response = EstimateMemoryResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "estimate_memory",
            tags: &["Resources", "Models"],
            description: "Estimate memory requirements for loading a model",
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
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(request_id = %request_id, route = "estimate_memory", model = %req.model);

        let request = RequestValue::estimate_memory(&req.model, None);
        let response = state.handle_request(request).await
            .map_err(|e| NativeMessagingError::Backend(e))?;

        let (estimated_memory_mb, estimated_vram_mb, can_load, reason) = response.as_memory_estimate()
            .ok_or_else(|| NativeMessagingError::internal("Invalid response type"))?;

        Ok(EstimateMemoryResponse {
            model: req.model,
            estimated_memory_mb: estimated_memory_mb,
            estimated_vram_mb: estimated_vram_mb,
            can_load: can_load,
            reason: reason.clone(),
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::error("empty_model", EstimateMemoryRequest {
                model: "".to_string(),
                context_length: None,
            }, "model"),
            TestCase {
                name: "estimate_memory_basic",
                request: EstimateMemoryRequest {
                    model: "gpt-3.5-turbo".to_string(),
                    context_length: Some(4096),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

#[async_trait]
impl NativeMessagingRoute for CompatibilityRoute {
    type Request = CompatibilityRequest;
    type Response = CompatibilityResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "compatibility",
            tags: &["Resources", "Models"],
            description: "Check system compatibility for a model",
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
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(request_id = %request_id, route = "compatibility", model = %req.model);

        // Check model compatibility through backend
        let request = RequestValue::generate(&req.model, "test", None);
        let response = state.handle_request(request).await;

        // Determine compatibility based on response success
        let is_compatible = response.is_ok();

        let compat = CompatibilityResponse {
            model: req.model.clone(),
            compatible: is_compatible,
            requirements: if is_compatible {
                vec!["CUDA 11.0+".to_string(), "16GB RAM".to_string()]
            } else {
                vec!["Backend must be healthy".to_string()]
            },
            missing_requirements: if is_compatible {
                vec![]
            } else {
                vec!["Healthy backend connection".to_string()]
            },
            recommendations: if is_compatible {
                vec!["Use GPU acceleration for better performance".to_string()]
            } else {
                vec!["Check backend health and connectivity".to_string()]
            },
        };

        Ok(compat)
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::error("empty_model", CompatibilityRequest {
                model: "".to_string(),
            }, "model"),
            TestCase {
                name: "compatibility_check",
                request: CompatibilityRequest {
                    model: "gpt-3.5-turbo".to_string(),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_native_messaging_route!(GetResourcesRoute);
crate::enforce_native_messaging_route!(EstimateMemoryRoute);
crate::enforce_native_messaging_route!(CompatibilityRoute);