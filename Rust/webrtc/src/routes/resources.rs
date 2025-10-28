//! Resource management endpoints for WebRTC data channels.
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

// ==================== GET RESOURCES ====================

/// Get resources request (no parameters).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetResourcesRequest;

/// Get resources response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetResourcesResponse {
    /// CPU usage percentage
    pub cpu_usage: f32,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Total memory in bytes
    pub memory_total: u64,
    /// Optional GPU usage percentage
    pub gpu_usage: Option<f32>,
    /// Optional GPU memory usage in bytes
    pub gpu_memory_usage: Option<u64>,
    /// Optional total GPU memory in bytes
    pub gpu_memory_total: Option<u64>,
    /// Disk usage in bytes
    pub disk_usage: u64,
    /// Total disk space in bytes
    pub disk_total: u64,
}

/// Get resources route handler.
pub struct GetResourcesRoute;

#[async_trait]
impl DataChannelRoute for GetResourcesRoute {
    type Request = GetResourcesRequest;
    type Response = GetResourcesResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "get_resources",
            tags: &["System", "Resources", "Monitoring"],
            description: "Get current system resource usage including CPU, memory, GPU, and disk",
            supports_streaming: false,
            supports_binary: false,
            requires_auth: false,
            rate_limit_tier: Some("standard"),
            max_payload_size: None,
            media_type: None,
        }
    }

    async fn validate_request(_req: &Self::Request) -> WebRtcResult<()> {
        Ok(()) // No validation needed
    }

    async fn handle<H>(_req: Self::Request, handler: &H) -> WebRtcResult<Self::Response>
    where
        H: RequestHandler + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(request_id = %request_id, route = "get_resources", "WebRTC get resources request");

        let request_value = RequestValue::get_resources();

        let response = handler.handle_request(request_value).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Get resources request failed");
                WebRtcError::from(e)
            })?;

        let (cpu_usage, memory_usage, memory_total, gpu_usage, gpu_memory_usage, gpu_memory_total, disk_usage, disk_total) = response.as_resources()
            .ok_or_else(|| WebRtcError::InternalError("Invalid response type".to_string()))?;

        tracing::info!(
            request_id = %request_id,
            cpu_usage = cpu_usage,
            memory_usage = memory_usage,
            "Get resources successful"
        );

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
            TestCase::success(
                "get_resources",
                GetResourcesRequest,
                GetResourcesResponse {
                    cpu_usage: 45.5,
                    memory_usage: 8_000_000_000,
                    memory_total: 16_000_000_000,
                    gpu_usage: Some(30.2),
                    gpu_memory_usage: Some(4_000_000_000),
                    gpu_memory_total: Some(8_000_000_000),
                    disk_usage: 100_000_000_000,
                    disk_total: 500_000_000_000,
                },
            ),
        ]
    }
}

crate::enforce_data_channel_route!(GetResourcesRoute);

// ==================== ESTIMATE MEMORY ====================

/// Estimate memory request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EstimateMemoryRequest {
    /// Model identifier
    pub model: String,
    /// Optional context length
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_length: Option<u32>,
}

/// Estimate memory response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EstimateMemoryResponse {
    /// Model identifier
    pub model: String,
    /// Estimated memory in MB
    pub estimated_memory_mb: u64,
    /// Optional estimated VRAM in MB
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_vram_mb: Option<u64>,
    /// Whether the model can be loaded
    pub can_load: bool,
    /// Optional reason if model cannot be loaded
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Estimate memory route handler.
pub struct EstimateMemoryRoute;

#[async_trait]
impl DataChannelRoute for EstimateMemoryRoute {
    type Request = EstimateMemoryRequest;
    type Response = EstimateMemoryResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "estimate_memory",
            tags: &["Resources", "Models"],
            description: "Estimate memory requirements for loading a model",
            supports_streaming: false,
            supports_binary: false,
            requires_auth: false,
            rate_limit_tier: Some("standard"),
            max_payload_size: None,
            media_type: None,
        }
    }

    async fn validate_request(req: &Self::Request) -> WebRtcResult<()> {
        if req.model.is_empty() {
            return Err(WebRtcError::ValidationError {
                field: "model".to_string(),
                message: "model cannot be empty".to_string(),
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
            route = "estimate_memory",
            model = %req.model,
            "WebRTC estimate memory request"
        );

        let request_value = RequestValue::estimate_memory(&req.model, None);

        let response = handler.handle_request(request_value).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Estimate memory request failed");
                WebRtcError::from(e)
            })?;

        let (estimated_memory_mb, estimated_vram_mb, can_load, reason) = response.as_memory_estimate()
            .ok_or_else(|| WebRtcError::InternalError("Invalid response type".to_string()))?;

        tracing::info!(
            request_id = %request_id,
            estimated_memory_mb = estimated_memory_mb,
            "Estimate memory successful"
        );

        Ok(EstimateMemoryResponse {
            model: req.model,
            estimated_memory_mb,
            estimated_vram_mb,
            can_load,
            reason: reason.clone(),
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::success(
                "estimate_memory",
                EstimateMemoryRequest {
                    model: "gpt-3.5-turbo".to_string(),
                    context_length: Some(4096),
                },
                EstimateMemoryResponse {
                    model: "gpt-3.5-turbo".to_string(),
                    estimated_memory_mb: 2048,
                    estimated_vram_mb: Some(1024),
                    can_load: true,
                    reason: None,
                },
            ),
            TestCase::error(
                "estimate_empty_model",
                EstimateMemoryRequest {
                    model: "".to_string(),
                    context_length: None,
                },
                "model cannot be empty",
            ),
        ]
    }
}

crate::enforce_data_channel_route!(EstimateMemoryRoute);

// ==================== COMPATIBILITY ====================

/// Compatibility check request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityRequest {
    /// Model identifier
    pub model: String,
}

/// Compatibility check response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityResponse {
    /// Model identifier
    pub model: String,
    /// Whether the model is compatible
    pub compatible: bool,
    /// List of requirements
    pub requirements: Vec<String>,
    /// List of missing requirements
    pub missing_requirements: Vec<String>,
    /// Recommendations for compatibility
    pub recommendations: Vec<String>,
}

/// Compatibility check route handler.
pub struct CompatibilityRoute;

#[async_trait]
impl DataChannelRoute for CompatibilityRoute {
    type Request = CompatibilityRequest;
    type Response = CompatibilityResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "compatibility",
            tags: &["Resources", "Models"],
            description: "Check if a model is compatible with the current system",
            supports_streaming: false,
            supports_binary: false,
            requires_auth: false,
            rate_limit_tier: Some("standard"),
            max_payload_size: None,
            media_type: None,
        }
    }

    async fn validate_request(req: &Self::Request) -> WebRtcResult<()> {
        if req.model.is_empty() {
            return Err(WebRtcError::ValidationError {
                field: "model".to_string(),
                message: "model cannot be empty".to_string(),
            });
        }
        Ok(())
    }

    async fn handle<H>(req: Self::Request, _handler: &H) -> WebRtcResult<Self::Response>
    where
        H: RequestHandler + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(
            request_id = %request_id,
            route = "compatibility",
            model = %req.model,
            "WebRTC compatibility request"
        );

        // TODO: TIER 2 - Implement check_compatibility in RequestValue
        // For now, return placeholder response
        tracing::warn!(request_id = %request_id, "Compatibility check not fully implemented");

        Ok(CompatibilityResponse {
            model: req.model,
            compatible: true,
            requirements: vec!["CUDA 11.8+".to_string()],
            missing_requirements: vec![],
            recommendations: vec![],
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::success(
                "compatibility_check",
                CompatibilityRequest {
                    model: "gpt-3.5-turbo".to_string(),
                },
                CompatibilityResponse {
                    model: "gpt-3.5-turbo".to_string(),
                    compatible: true,
                    requirements: vec!["CUDA 11.8+".to_string()],
                    missing_requirements: vec![],
                    recommendations: vec![],
                },
            ),
            TestCase::error(
                "compatibility_empty_model",
                CompatibilityRequest {
                    model: "".to_string(),
                },
                "model cannot be empty",
            ),
        ]
    }
}

crate::enforce_data_channel_route!(CompatibilityRoute);
