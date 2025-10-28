//! Hardware Detection Routes
//!
//! Routes for querying hardware capabilities and recommendations.

use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;
use crate::error::{ApiError, ApiResult};
use common::backend::AppStateProvider;

// ========== GET HARDWARE INFO ==========

#[derive(Debug, Serialize, ToSchema)]
pub struct GetHardwareInfoResponse {
    pub cpu: serde_json::Value,
    pub memory: serde_json::Value,
    pub gpus: Vec<serde_json::Value>,
    pub vram: serde_json::Value,
    pub execution_provider: String,
}

/// Get hardware information
#[utoipa::path(
    get,
    path = "/v1/hardware/info",
    responses(
        (status = 200, description = "Hardware information retrieved", body = GetHardwareInfoResponse),
    ),
    tag = "Hardware"
)]
pub async fn get_hardware_info<S>(
    State(state): State<Arc<S>>,
) -> ApiResult<Json<serde_json::Value>>
where
    S: AppStateProvider + Send + Sync + 'static,
{
    let request = tabagent_values::RequestValue::from_json(r#"{"action":"get_hardware_info"}"#)?;
    
    let response = state.handle_request(request).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    
    let json_str = response.to_json()?;
    let data: serde_json::Value = serde_json::from_str(&json_str)?;
    
    Ok(Json(data))
}

// ========== CHECK MODEL FEASIBILITY ==========

#[derive(Debug, Deserialize, ToSchema)]
pub struct CheckModelFeasibilityRequest {
    /// Model size in megabytes
    pub model_size_mb: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CheckModelFeasibilityResponse {
    pub can_load: bool,
    pub model_size_mb: u64,
    pub available_ram_mb: u64,
    pub available_vram_mb: u64,
    pub recommendation: String,
}

/// Check if model can be loaded
#[utoipa::path(
    post,
    path = "/v1/hardware/feasibility",
    request_body = CheckModelFeasibilityRequest,
    responses(
        (status = 200, description = "Feasibility check completed", body = CheckModelFeasibilityResponse),
    ),
    tag = "Hardware"
)]
pub async fn check_model_feasibility<S>(
    State(state): State<Arc<S>>,
    Json(payload): Json<CheckModelFeasibilityRequest>,
) -> ApiResult<Json<serde_json::Value>>
where
    S: AppStateProvider + Send + Sync + 'static,
{
    let request = tabagent_values::RequestValue::from_json(&serde_json::to_string(&serde_json::json!({
        "action": "check_model_feasibility",
        "model_size_mb": payload.model_size_mb
    }))?)?;
    
    let response = state.handle_request(request).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    
    let json_str = response.to_json()?;
    let data: serde_json::Value = serde_json::from_str(&json_str)?;
    
    Ok(Json(data))
}

// ========== GET RECOMMENDED MODELS ==========

#[derive(Debug, Serialize, ToSchema)]
pub struct GetRecommendedModelsResponse {
    pub available_ram_mb: u64,
    pub available_vram_mb: u64,
    pub safe_ram_mb: u64,
    pub safe_vram_mb: u64,
    pub recommended_sizes: Vec<String>,
    pub recommendation: String,
}

/// Get recommended model sizes
#[utoipa::path(
    get,
    path = "/v1/hardware/recommendations",
    responses(
        (status = 200, description = "Recommendations retrieved", body = GetRecommendedModelsResponse),
    ),
    tag = "Hardware"
)]
pub async fn get_recommended_models<S>(
    State(state): State<Arc<S>>,
) -> ApiResult<Json<serde_json::Value>>
where
    S: AppStateProvider + Send + Sync + 'static,
{
    let request = tabagent_values::RequestValue::from_json(r#"{"action":"get_recommended_models"}"#)?;
    
    let response = state.handle_request(request).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    
    let json_str = response.to_json()?;
    let data: serde_json::Value = serde_json::from_str(&json_str)?;
    
    Ok(Json(data))
}

