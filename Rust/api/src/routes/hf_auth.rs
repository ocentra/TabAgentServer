//! HuggingFace Authentication Routes
//!
//! Routes for managing HuggingFace API tokens (set, get status, clear).

use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::error::{ApiError, ApiResult};
use common::backend::AppStateProvider;

// ========== SET HF TOKEN ==========

#[derive(Debug, Deserialize, ToSchema)]
pub struct SetHfTokenRequest {
    /// HuggingFace API token (must start with 'hf_')
    pub token: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SetHfTokenResponse {
    pub message: String,
}

/// Set HuggingFace API token
#[utoipa::path(
    post,
    path = "/v1/hf/token",
    request_body = SetHfTokenRequest,
    responses(
        (status = 200, description = "Token stored successfully", body = SetHfTokenResponse),
        (status = 400, description = "Invalid token format"),
    ),
    tag = "HuggingFace Auth"
)]
pub async fn set_hf_token<S>(
    State(state): State<S>,
    Json(payload): Json<SetHfTokenRequest>,
) -> ApiResult<Json<SetHfTokenResponse>>
where
    S: AppStateProvider + Send + Sync + 'static,
{
    let request = tabagent_values::RequestValue::from_json(&serde_json::to_string(&serde_json::json!({
        "action": "set_hf_token",
        "token": payload.token
    }))?)?;
    
    state.handle_request(request).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    
    Ok(Json(SetHfTokenResponse {
        message: "HuggingFace token stored securely".to_string(),
    }))
}

// ========== GET HF TOKEN STATUS ==========

#[derive(Debug, Serialize, ToSchema)]
pub struct GetHfTokenStatusResponse {
    pub has_token: bool,
    pub message: String,
}

/// Get HuggingFace token status
#[utoipa::path(
    get,
    path = "/v1/hf/token/status",
    responses(
        (status = 200, description = "Token status retrieved", body = GetHfTokenStatusResponse),
    ),
    tag = "HuggingFace Auth"
)]
pub async fn get_hf_token_status<S>(
    State(state): State<S>,
) -> ApiResult<Json<GetHfTokenStatusResponse>>
where
    S: AppStateProvider + Send + Sync + 'static,
{
    let request = tabagent_values::RequestValue::from_json(r#"{"action":"get_hf_token_status"}"#)?;
    
    let response = state.handle_request(request).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    
    let json_str = response.to_json()?;
    let data: serde_json::Value = serde_json::from_str(&json_str)?;
    
    Ok(Json(GetHfTokenStatusResponse {
        has_token: data["hasToken"].as_bool().unwrap_or(false),
        message: data["message"].as_str().unwrap_or("").to_string(),
    }))
}

// ========== CLEAR HF TOKEN ==========

#[derive(Debug, Serialize, ToSchema)]
pub struct ClearHfTokenResponse {
    pub message: String,
}

/// Clear HuggingFace token
#[utoipa::path(
    delete,
    path = "/v1/hf/token",
    responses(
        (status = 200, description = "Token cleared successfully", body = ClearHfTokenResponse),
    ),
    tag = "HuggingFace Auth"
)]
pub async fn clear_hf_token<S>(
    State(state): State<S>,
) -> ApiResult<Json<ClearHfTokenResponse>>
where
    S: AppStateProvider + Send + Sync + 'static,
{
    let request = tabagent_values::RequestValue::from_json(r#"{"action":"clear_hf_token"}"#)?;
    
    state.handle_request(request).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    
    Ok(Json(ClearHfTokenResponse {
        message: "HuggingFace token removed".to_string(),
    }))
}

