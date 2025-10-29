//! Hardware Detection Routes for WebRTC
//!
//! Routes for querying system hardware and model feasibility via WebRTC data channel.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::{
    error::WebRtcResult,
    route_trait::{DataChannelRoute, RouteMetadata, TestCase},
    traits::RequestHandler,
};

// ========== GET HARDWARE INFO ==========

/// Request to get hardware information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetHardwareInfoRequest;

/// Response with detailed hardware information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetHardwareInfoResponse {
    /// CPU information
    pub cpu: serde_json::Value,
    /// List of GPU information
    pub gpus: Vec<serde_json::Value>,
    /// RAM information
    pub ram: serde_json::Value,
}

/// Route handler for getting hardware information
pub struct GetHardwareInfoRoute;

#[async_trait]
impl DataChannelRoute for GetHardwareInfoRoute {
    type Request = GetHardwareInfoRequest;
    type Response = GetHardwareInfoResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "get_hardware_info",
            tags: &["Hardware", "System"],
            description: "Get detailed hardware information (CPU, GPU, RAM)",
            supports_streaming: false,
            supports_binary: false,
            requires_auth: false,
            rate_limit_tier: None,
            max_payload_size: None,
            media_type: None,
        }
    }

    async fn validate_request(_req: &Self::Request) -> WebRtcResult<()> {
        Ok(())
    }

    async fn handle<H>(_req: Self::Request, handler: &H) -> WebRtcResult<Self::Response>
    where
        H: RequestHandler + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(request_id = %request_id, "Get hardware info request");

        let request_value = tabagent_values::RequestValue::from_json(r#"{"action":"get_hardware_info"}"#)
            .map_err(|e| crate::error::WebRtcError::InternalError(format!("Failed to create request: {}", e)))?;

        let response = handler.handle_request(request_value).await
            .map_err(|e| crate::error::WebRtcError::from(e))?;

        let json_str = response.to_json()
            .map_err(|e| crate::error::WebRtcError::InternalError(format!("Failed to serialize response: {}", e)))?;
        let data: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| crate::error::WebRtcError::InternalError(e.to_string()))?;

        Ok(GetHardwareInfoResponse {
            cpu: data["cpu"].clone(),
            gpus: data["gpus"].as_array().cloned().unwrap_or_default(),
            ram: data["ram"].clone(),
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase {
                name: "get_hardware_info",
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

/// Request to check if a model can be loaded given hardware constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckModelFeasibilityRequest {
    /// Model size in megabytes
    pub model_size_mb: u64,
}

/// Response with model feasibility assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckModelFeasibilityResponse {
    /// Whether the model can be loaded
    pub feasible: bool,
    /// Detailed feasibility message
    pub message: String,
    /// Hardware recommendations
    pub recommendations: Vec<String>,
}

/// Route handler for checking model feasibility
pub struct CheckModelFeasibilityRoute;

#[async_trait]
impl DataChannelRoute for CheckModelFeasibilityRoute {
    type Request = CheckModelFeasibilityRequest;
    type Response = CheckModelFeasibilityResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "check_model_feasibility",
            tags: &["Hardware", "Models"],
            description: "Check if a model of given size can run on current hardware",
            supports_streaming: false,
            supports_binary: false,
            requires_auth: false,
            rate_limit_tier: None,
            max_payload_size: None,
            media_type: None,
        }
    }

    async fn validate_request(req: &Self::Request) -> WebRtcResult<()> {
        if req.model_size_mb == 0 {
            return Err(crate::error::WebRtcError::ValidationError {
                field: "model_size_mb".to_string(),
                message: "Model size must be greater than 0".to_string(),
            });
        }
        Ok(())
    }

    async fn handle<H>(req: Self::Request, handler: &H) -> WebRtcResult<Self::Response>
    where
        H: RequestHandler + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(request_id = %request_id, model_size_mb = req.model_size_mb, "Check model feasibility request");

        let request_value = tabagent_values::RequestValue::from_json(&serde_json::to_string(&serde_json::json!({
            "action": "check_model_feasibility",
            "model_size_mb": req.model_size_mb
        })).map_err(|e| crate::error::WebRtcError::InternalError(e.to_string()))?)
            .map_err(|e| crate::error::WebRtcError::InternalError(format!("Failed to create request: {}", e)))?;

        let response = handler.handle_request(request_value).await
            .map_err(|e| crate::error::WebRtcError::from(e))?;

        let json_str = response.to_json()
            .map_err(|e| crate::error::WebRtcError::InternalError(format!("Failed to serialize response: {}", e)))?;
        let data: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| crate::error::WebRtcError::InternalError(e.to_string()))?;

        Ok(CheckModelFeasibilityResponse {
            feasible: data["feasible"].as_bool().unwrap_or(false),
            message: data["message"].as_str().unwrap_or("").to_string(),
            recommendations: data["recommendations"].as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default(),
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase {
                name: "check_feasibility",
                request: CheckModelFeasibilityRequest {
                    model_size_mb: 7000,
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

/// Request to get recommended model sizes for current hardware
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetRecommendedModelsRequest;

/// Response with recommended model sizes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetRecommendedModelsResponse {
    /// List of recommended model configurations
    pub models: Vec<serde_json::Value>,
}

/// Route handler for getting recommended models
pub struct GetRecommendedModelsRoute;

#[async_trait]
impl DataChannelRoute for GetRecommendedModelsRoute {
    type Request = GetRecommendedModelsRequest;
    type Response = GetRecommendedModelsResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "get_recommended_models",
            tags: &["Hardware", "Models"],
            description: "Get list of models recommended for current hardware",
            supports_streaming: false,
            supports_binary: false,
            requires_auth: false,
            rate_limit_tier: None,
            max_payload_size: None,
            media_type: None,
        }
    }

    async fn validate_request(_req: &Self::Request) -> WebRtcResult<()> {
        Ok(())
    }

    async fn handle<H>(_req: Self::Request, handler: &H) -> WebRtcResult<Self::Response>
    where
        H: RequestHandler + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(request_id = %request_id, "Get recommended models request");

        let request_value = tabagent_values::RequestValue::from_json(r#"{"action":"get_recommended_models"}"#)
            .map_err(|e| crate::error::WebRtcError::InternalError(format!("Failed to create request: {}", e)))?;

        let response = handler.handle_request(request_value).await
            .map_err(|e| crate::error::WebRtcError::from(e))?;

        let json_str = response.to_json()
            .map_err(|e| crate::error::WebRtcError::InternalError(format!("Failed to serialize response: {}", e)))?;
        let data: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| crate::error::WebRtcError::InternalError(e.to_string()))?;

        Ok(GetRecommendedModelsResponse {
            models: data["models"].as_array().cloned().unwrap_or_default(),
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase {
                name: "get_recommended_models",
                request: GetRecommendedModelsRequest,
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_data_channel_route!(GetRecommendedModelsRoute);

