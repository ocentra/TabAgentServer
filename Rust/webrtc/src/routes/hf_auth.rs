//! HuggingFace Authentication Routes for WebRTC
//!
//! Routes for managing HuggingFace API tokens via WebRTC data channel.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::{
    error::WebRtcResult,
    route_trait::{DataChannelRoute, RouteMetadata, TestCase},
    traits::RequestHandler,
};

// ========== SET HF TOKEN ==========

/// Request to set HuggingFace authentication token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetHfTokenRequest {
    /// HuggingFace API token to store securely
    pub token: String,
}

/// Response after setting HuggingFace token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetHfTokenResponse {
    /// Confirmation message
    pub message: String,
}

/// Route handler for setting HuggingFace token
pub struct SetHfTokenRoute;

#[async_trait]
impl DataChannelRoute for SetHfTokenRoute {
    type Request = SetHfTokenRequest;
    type Response = SetHfTokenResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "set_hf_token",
            tags: &["HuggingFace", "Auth"],
            description: "Store HuggingFace API token securely for accessing gated models",
            supports_streaming: false,
            supports_binary: false,
            requires_auth: false,
            rate_limit_tier: None,
            max_payload_size: Some(1024),
            media_type: None,
        }
    }

    async fn validate_request(req: &Self::Request) -> WebRtcResult<()> {
        if !req.token.starts_with("hf_") {
            return Err(crate::error::WebRtcError::ValidationError {
                field: "token".to_string(),
                message: "Token must start with 'hf_'".to_string(),
            });
        }
        Ok(())
    }

    async fn handle<H>(req: Self::Request, handler: &H) -> WebRtcResult<Self::Response>
    where
        H: RequestHandler + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(request_id = %request_id, "Set HF token request");

        let request_value = tabagent_values::RequestValue::from_json(&serde_json::to_string(&serde_json::json!({
            "action": "set_hf_token",
            "token": req.token
        })).map_err(|e| crate::error::WebRtcError::InternalError(e.to_string()))?)
            .map_err(|e| crate::error::WebRtcError::InternalError(format!("Failed to create request: {}", e)))?;

        handler.handle_request(request_value).await
            .map_err(|e| crate::error::WebRtcError::from(e))?;

        Ok(SetHfTokenResponse {
            message: "HuggingFace token stored securely".to_string(),
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase {
                name: "set_token",
                request: SetHfTokenRequest {
                    token: "hf_test".to_string(),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_data_channel_route!(SetHfTokenRoute);

// ========== GET HF TOKEN STATUS ==========

/// Request to check HuggingFace token status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetHfTokenStatusRequest;

/// Response with HuggingFace token status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetHfTokenStatusResponse {
    /// Whether a token is currently stored
    pub has_token: bool,
    /// Status message
    pub message: String,
}

/// Route handler for checking HuggingFace token status
pub struct GetHfTokenStatusRoute;

#[async_trait]
impl DataChannelRoute for GetHfTokenStatusRoute {
    type Request = GetHfTokenStatusRequest;
    type Response = GetHfTokenStatusResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "get_hf_token_status",
            tags: &["HuggingFace", "Auth"],
            description: "Check if HuggingFace token is stored",
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
        tracing::info!(request_id = %request_id, "Get HF token status request");

        let request_value = tabagent_values::RequestValue::from_json(r#"{"action":"get_hf_token_status"}"#)
            .map_err(|e| crate::error::WebRtcError::InternalError(format!("Failed to create request: {}", e)))?;

        let response = handler.handle_request(request_value).await
            .map_err(|e| crate::error::WebRtcError::from(e))?;

        let json_str = response.to_json()
            .map_err(|e| crate::error::WebRtcError::InternalError(format!("Failed to serialize response: {}", e)))?;
        let data: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| crate::error::WebRtcError::InternalError(e.to_string()))?;

        Ok(GetHfTokenStatusResponse {
            has_token: data["hasToken"].as_bool().unwrap_or(false),
            message: data["message"].as_str().unwrap_or("").to_string(),
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase {
                name: "get_status",
                request: GetHfTokenStatusRequest,
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_data_channel_route!(GetHfTokenStatusRoute);

// ========== CLEAR HF TOKEN ==========

/// Request to clear stored HuggingFace token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClearHfTokenRequest;

/// Response after clearing HuggingFace token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClearHfTokenResponse {
    /// Confirmation message
    pub message: String,
}

/// Route handler for clearing HuggingFace token
pub struct ClearHfTokenRoute;

#[async_trait]
impl DataChannelRoute for ClearHfTokenRoute {
    type Request = ClearHfTokenRequest;
    type Response = ClearHfTokenResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "clear_hf_token",
            tags: &["HuggingFace", "Auth"],
            description: "Remove stored HuggingFace token",
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
        tracing::info!(request_id = %request_id, "Clear HF token request");

        let request_value = tabagent_values::RequestValue::from_json(r#"{"action":"clear_hf_token"}"#)
            .map_err(|e| crate::error::WebRtcError::InternalError(format!("Failed to create request: {}", e)))?;

        handler.handle_request(request_value).await
            .map_err(|e| crate::error::WebRtcError::from(e))?;

        Ok(ClearHfTokenResponse {
            message: "HuggingFace token removed".to_string(),
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase {
                name: "clear_token",
                request: ClearHfTokenRequest,
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_data_channel_route!(ClearHfTokenRoute);

