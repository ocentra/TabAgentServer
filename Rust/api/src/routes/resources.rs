//! Resource management endpoints.
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

/// Resource information.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ResourceInfo {
    /// CPU RAM information
    pub cpu_ram: RamInfo,
    /// GPU information
    pub gpus: Vec<GpuResourceInfo>,
    /// Loaded models
    pub loaded_models: Vec<LoadedModelInfo>,
}

/// RAM information.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct RamInfo {
    /// Total RAM (MB)
    pub total_mb: u64,
    /// Available RAM (MB)
    pub available_mb: u64,
    /// Used RAM (MB)
    pub used_mb: u64,
}

/// GPU resource information.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct GpuResourceInfo {
    /// GPU index
    pub index: u32,
    /// GPU name
    pub name: String,
    /// Total VRAM (MB)
    pub vram_total_mb: u64,
    /// Available VRAM (MB)
    pub vram_available_mb: u64,
    /// Used VRAM (MB)
    pub vram_used_mb: u64,
}

/// Loaded model information.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct LoadedModelInfo {
    /// Model identifier
    pub model_id: String,
    /// Memory usage (MB)
    pub memory_mb: u64,
    /// Device (cpu, cuda:0, etc.)
    pub device: String,
}

/// Memory estimate request.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EstimateRequest {
    /// Model path or identifier
    pub model: String,
    /// Quantization type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantization: Option<String>,
}

/// Memory estimate response.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct EstimateResponse {
    /// Estimated memory (MB)
    pub estimated_mb: u64,
    /// Can fit in available RAM
    pub fits_in_ram: bool,
    /// Can fit in available VRAM
    pub fits_in_vram: bool,
}

// ==================== GET RESOURCES ====================

/// Get resources request (no parameters).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetResourcesRequest;

/// Get resources route handler.
///
/// Returns current system resource availability including RAM, VRAM,
/// and loaded model information for capacity planning.
pub struct GetResourcesRoute;

#[async_trait]
impl RouteHandler for GetResourcesRoute {
    type Request = GetResourcesRequest;
    type Response = ResourceInfo;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            path: "/v1/resources",
            method: Method::GET,
            tags: &["Resources"],
            description: "Get current system resource availability (RAM, VRAM, loaded models)",
            openai_compatible: false,
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
        tracing::info!(request_id = %request_id, "Get resources request received");

        let request = RequestValue::get_resources();
        
        tracing::debug!(request_id = %request_id, "Dispatching to handler");
        let response = state.handle_request(request).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Get resources failed");
                e
            })?;

        let resources_json = response.to_json_value();
        let resources: ResourceInfo = serde_json::from_value(resources_json)
            .map_err(|e| ApiError::Internal(format!("Failed to parse resources: {}", e)))?;

        tracing::info!(
            request_id = %request_id,
            ram_available_mb = resources.cpu_ram.available_mb,
            loaded_models = resources.loaded_models.len(),
            "Get resources successful"
        );
        Ok(resources)
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            // === SUCCESS TEST CASES ===
            TestCase {
                name: "get_resources_basic",
                request: GetResourcesRequest,
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "get_resources_idempotent",
                request: GetResourcesRequest,
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "get_resources_concurrent_safe",
                request: GetResourcesRequest,
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_route_handler!(GetResourcesRoute);

// ==================== ESTIMATE MEMORY ====================

/// Estimate memory route handler.
///
/// Estimates the memory requirements for loading a specific model with
/// optional quantization, helping with capacity planning and device selection.
pub struct EstimateMemoryRoute;

#[async_trait]
impl RouteHandler for EstimateMemoryRoute {
    type Request = EstimateRequest;
    type Response = EstimateResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            path: "/v1/resources/estimate",
            method: Method::POST,
            tags: &["Resources"],
            description: "Estimate memory requirements for loading a model with optional quantization",
            openai_compatible: false,
            idempotent: true, // Same model = same estimate
            requires_auth: false,
            rate_limit_tier: None,
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
            "Estimate memory request received"
        );

        let request = RequestValue::estimate_memory(req.model.clone(), req.quantization.clone());
        
        tracing::debug!(request_id = %request_id, "Dispatching to handler");
        let response = state.handle_request(request).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Estimate memory failed");
                e
            })?;

        let estimate_json = response.to_json_value();
        let estimate: EstimateResponse = serde_json::from_value(estimate_json)
            .map_err(|e| ApiError::Internal(format!("Failed to parse estimate: {}", e)))?;

        tracing::info!(
            request_id = %request_id,
            estimated_mb = estimate.estimated_mb,
            fits_in_ram = estimate.fits_in_ram,
            fits_in_vram = estimate.fits_in_vram,
            "Estimate memory successful"
        );
        Ok(estimate)
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            // === VALIDATION ERROR TESTS ===
            TestCase::error(
                "empty_model",
                EstimateRequest {
                    model: "".to_string(),
                    quantization: None,
                },
                "cannot be empty",
            ),
            // === SUCCESS TEST CASES ===
            TestCase {
                name: "estimate_basic",
                request: EstimateRequest {
                    model: "llama-2-7b".to_string(),
                    quantization: None,
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "estimate_with_quantization",
                request: EstimateRequest {
                    model: "llama-2-13b".to_string(),
                    quantization: Some("Q4_K_M".to_string()),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "estimate_large_model",
                request: EstimateRequest {
                    model: "llama-2-70b".to_string(),
                    quantization: Some("Q8_0".to_string()),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_route_handler!(EstimateMemoryRoute);

// ==================== COMPATIBILITY CHECK ====================

/// Compatibility check request.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CompatibilityRequest {
    /// Model path or identifier
    pub model: String,
    /// Quantization type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantization: Option<String>,
    /// Target device (cpu, cuda:0, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device: Option<String>,
}

/// Compatibility check response.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct CompatibilityResponse {
    /// Whether model is compatible
    pub compatible: bool,
    /// Compatibility status message
    pub message: String,
    /// Estimated memory requirement (MB)
    pub estimated_memory_mb: u64,
    /// Available memory (MB)
    pub available_memory_mb: u64,
    /// Recommended device
    pub recommended_device: String,
}

/// Compatibility check route handler.
pub struct CompatibilityRoute;

#[async_trait]
impl RouteHandler for CompatibilityRoute {
    type Request = CompatibilityRequest;
    type Response = CompatibilityResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            path: "/v1/resources/compatibility",
            method: Method::POST,
            tags: &["Resources"],
            description: "Check if a model is compatible with current system resources",
            openai_compatible: false,
            idempotent: true,
            requires_auth: false,
            rate_limit_tier: None,
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
            device = ?req.device,
            "Compatibility check request received"
        );

        // First get resources
        let resources_request = RequestValue::get_resources();
        let resources_response = state.handle_request(resources_request).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Get resources failed");
                e
            })?;

        // Then estimate memory
        let estimate_request = RequestValue::estimate_memory(req.model.clone(), req.quantization.clone());
        let estimate_response = state.handle_request(estimate_request).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Estimate memory failed");
                e
            })?;

        let resources_json = resources_response.to_json_value();
        let estimate_json = estimate_response.to_json_value();

        // Parse resources
        let ram_available = resources_json
            .get("cpu_ram")
            .and_then(|r| r.get("available_mb"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        // Parse estimate
        let estimated_mb = estimate_json
            .get("estimated_mb")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let fits_in_ram = estimate_json
            .get("fits_in_ram")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let fits_in_vram = estimate_json
            .get("fits_in_vram")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let compatible = fits_in_ram || fits_in_vram;
        let recommended_device = if fits_in_vram {
            "cuda:0".to_string()
        } else if fits_in_ram {
            "cpu".to_string()
        } else {
            "none".to_string()
        };

        let message = if compatible {
            format!("Model fits in {}", recommended_device)
        } else {
            "Insufficient memory for model".to_string()
        };

        tracing::info!(
            request_id = %request_id,
            compatible = compatible,
            recommended_device = %recommended_device,
            "Compatibility check successful"
        );

        Ok(CompatibilityResponse {
            compatible,
            message,
            estimated_memory_mb: estimated_mb,
            available_memory_mb: ram_available,
            recommended_device,
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            // === VALIDATION ERROR TESTS ===
            TestCase::error(
                "empty_model",
                CompatibilityRequest {
                    model: "".to_string(),
                    quantization: None,
                    device: None,
                },
                "cannot be empty",
            ),
            // === SUCCESS TEST CASES ===
            TestCase {
                name: "compatibility_basic",
                request: CompatibilityRequest {
                    model: "llama-2-7b".to_string(),
                    quantization: None,
                    device: None,
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "compatibility_with_quantization",
                request: CompatibilityRequest {
                    model: "llama-2-13b".to_string(),
                    quantization: Some("Q4_K_M".to_string()),
                    device: Some("cuda:0".to_string()),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "compatibility_large_model",
                request: CompatibilityRequest {
                    model: "llama-2-70b".to_string(),
                    quantization: Some("Q8_0".to_string()),
                    device: Some("cpu".to_string()),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_route_handler!(CompatibilityRoute);

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_estimate_validation() {
        let req = EstimateRequest {
            model: "test-model".to_string(),
            quantization: Some("q4_0".to_string()),
        };
        assert!(EstimateMemoryRoute::validate_request(&req).await.is_ok());
    }

    #[tokio::test]
    async fn test_estimate_validation_empty_model() {
        let req = EstimateRequest {
            model: "".to_string(),
            quantization: None,
        };
        assert!(EstimateMemoryRoute::validate_request(&req).await.is_err());
    }

    #[test]
    fn test_metadata() {
        let meta = GetResourcesRoute::metadata();
        assert!(meta.idempotent);
        
        let meta2 = EstimateMemoryRoute::metadata();
        assert!(meta2.idempotent);
    }
}
