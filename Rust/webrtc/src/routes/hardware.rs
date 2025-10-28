//! Hardware Detection Routes for WebRTC
//!
//! Routes for querying hardware capabilities via WebRTC data channels.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::route_trait::{DataChannelRoute, RouteMetadata};
use crate::error::{WebRtcError, WebRtcResult};
use common::backend::AppStateProvider;

// ========== GET HARDWARE INFO ==========

#[derive(Debug, Deserialize, Serialize)]
pub struct GetHardwareInfoRequest;

#[derive(Debug, Deserialize, Serialize)]
pub struct GetHardwareInfoResponse {
    pub cpu: serde_json::Value,
    pub memory: serde_json::Value,
    pub gpus: Vec<serde_json::Value>,
    pub vram: serde_json::Value,
    pub execution_provider: String,
}

pub struct GetHardwareInfoRoute;

#[async_trait]
impl DataChannelRoute for GetHardwareInfoRoute {
    type Request = GetHardwareInfoRequest;
    type Response = GetHardwareInfoResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "get_hardware_info",
            tags: &["Hardware", "System"],
            description: "Get comprehensive hardware information (CPU, GPU, RAM, VRAM)",
            openai_compatible: false,
            idempotent: true,
            requires_auth: false,
            rate_limit_tier: None,
            supports_streaming: false,
            supports_binary: false,
            max_payload_size: None,
        }
    }

    async fn validate_request(_req: &Self::Request) -> WebRtcResult<()> {
        Ok(())
    }

    async fn handle<S>(_req: Self::Request, state: &S) -> WebRtcResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request_id = Uuid::new_v4();
        tracing::info!(request_id = %request_id, "WebRTC: Get hardware info request");

        let request_value = tabagent_values::RequestValue::from_json(r#"{"action":"get_hardware_info"}"#)?;

        let response = state.handle_request(request_value).await
            .map_err(|e| WebRtcError::InternalError(e.to_string()))?;

        let json_str = response.to_json()?;
        let data: serde_json::Value = serde_json::from_str(&json_str)?;

        Ok(GetHardwareInfoResponse {
            cpu: data["cpu"].clone(),
            memory: data["memory"].clone(),
            gpus: data["gpus"].as_array().unwrap_or(&vec![]).clone(),
            vram: data["vram"].clone(),
            execution_provider: data["execution_provider"].as_str().unwrap_or("").to_string(),
        })
    }

    fn test_cases() -> Vec<crate::route_trait::TestCase<Self::Request, Self::Response>> {
        vec![
            crate::route_trait::TestCase {
                name: "get_hardware",
                request: GetHardwareInfoRequest,
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_data_channel_route!(GetHardwareInfoRoute);

// ========== CHECK MODEL FEASIBILITY ==========

#[derive(Debug, Deserialize, Serialize)]
pub struct CheckModelFeasibilityRequest {
    pub model_size_mb: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CheckModelFeasibilityResponse {
    pub can_load: bool,
    pub model_size_mb: u64,
    pub available_ram_mb: u64,
    pub available_vram_mb: u64,
    pub recommendation: String,
}

pub struct CheckModelFeasibilityRoute;

#[async_trait]
impl DataChannelRoute for CheckModelFeasibilityRoute {
    type Request = CheckModelFeasibilityRequest;
    type Response = CheckModelFeasibilityResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "check_model_feasibility",
            tags: &["Hardware", "Models"],
            description: "Check if a model can be loaded given current hardware",
            openai_compatible: false,
            idempotent: true,
            requires_auth: false,
            rate_limit_tier: None,
            supports_streaming: false,
            supports_binary: false,
            max_payload_size: Some(1024),
        }
    }

    async fn validate_request(req: &Self::Request) -> WebRtcResult<()> {
        if req.model_size_mb == 0 {
            return Err(WebRtcError::ValidationError(
                "Model size must be greater than 0".to_string()
            ));
        }
        Ok(())
    }

    async fn handle<S>(req: Self::Request, state: &S) -> WebRtcResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request_id = Uuid::new_v4();
        tracing::info!(request_id = %request_id, model_size = req.model_size_mb, "WebRTC: Check model feasibility");

        let request_value = tabagent_values::RequestValue::from_json(&serde_json::to_string(&serde_json::json!({
            "action": "check_model_feasibility",
            "model_size_mb": req.model_size_mb
        }))?)?;

        let response = state.handle_request(request_value).await
            .map_err(|e| WebRtcError::InternalError(e.to_string()))?;

        let json_str = response.to_json()?;
        let data: serde_json::Value = serde_json::from_str(&json_str)?;

        Ok(CheckModelFeasibilityResponse {
            can_load: data["can_load"].as_bool().unwrap_or(false),
            model_size_mb: data["model_size_mb"].as_u64().unwrap_or(0),
            available_ram_mb: data["available_ram_mb"].as_u64().unwrap_or(0),
            available_vram_mb: data["available_vram_mb"].as_u64().unwrap_or(0),
            recommendation: data["recommendation"].as_str().unwrap_or("").to_string(),
        })
    }

    fn test_cases() -> Vec<crate::route_trait::TestCase<Self::Request, Self::Response>> {
        vec![
            crate::route_trait::TestCase {
                name: "check_feasibility",
                request: CheckModelFeasibilityRequest {
                    model_size_mb: 1024,
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_data_channel_route!(CheckModelFeasibilityRoute);

// ========== GET RECOMMENDED MODELS ==========

#[derive(Debug, Deserialize, Serialize)]
pub struct GetRecommendedModelsRequest;

#[derive(Debug, Deserialize, Serialize)]
pub struct GetRecommendedModelsResponse {
    pub available_ram_mb: u64,
    pub available_vram_mb: u64,
    pub safe_ram_mb: u64,
    pub safe_vram_mb: u64,
    pub recommended_sizes: Vec<String>,
    pub recommendation: String,
}

pub struct GetRecommendedModelsRoute;

#[async_trait]
impl DataChannelRoute for GetRecommendedModelsRoute {
    type Request = GetRecommendedModelsRequest;
    type Response = GetRecommendedModelsResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "get_recommended_models",
            tags: &["Hardware", "Models"],
            description: "Get recommended model sizes for current hardware",
            openai_compatible: false,
            idempotent: true,
            requires_auth: false,
            rate_limit_tier: None,
            supports_streaming: false,
            supports_binary: false,
            max_payload_size: None,
        }
    }

    async fn validate_request(_req: &Self::Request) -> WebRtcResult<()> {
        Ok(())
    }

    async fn handle<S>(_req: Self::Request, state: &S) -> WebRtcResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request_id = Uuid::new_v4();
        tracing::info!(request_id = %request_id, "WebRTC: Get recommended models request");

        let request_value = tabagent_values::RequestValue::from_json(r#"{"action":"get_recommended_models"}"#)?;

        let response = state.handle_request(request_value).await
            .map_err(|e| WebRtcError::InternalError(e.to_string()))?;

        let json_str = response.to_json()?;
        let data: serde_json::Value = serde_json::from_str(&json_str)?;

        let sizes: Vec<String> = data["recommended_sizes"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|v| v.as_str().unwrap_or("").to_string())
            .collect();

        Ok(GetRecommendedModelsResponse {
            available_ram_mb: data["available_ram_mb"].as_u64().unwrap_or(0),
            available_vram_mb: data["available_vram_mb"].as_u64().unwrap_or(0),
            safe_ram_mb: data["safe_ram_mb"].as_u64().unwrap_or(0),
            safe_vram_mb: data["safe_vram_mb"].as_u64().unwrap_or(0),
            recommended_sizes: sizes,
            recommendation: data["recommendation"].as_str().unwrap_or("").to_string(),
        })
    }

    fn test_cases() -> Vec<crate::route_trait::TestCase<Self::Request, Self::Response>> {
        vec![
            crate::route_trait::TestCase {
                name: "get_recommendations",
                request: GetRecommendedModelsRequest,
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_data_channel_route!(GetRecommendedModelsRoute);

