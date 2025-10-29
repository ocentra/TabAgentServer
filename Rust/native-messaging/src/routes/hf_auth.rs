//! HuggingFace Authentication Routes for Native Messaging
//!
//! Routes for managing HuggingFace API tokens via Chrome extension.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::route_trait::{NativeMessagingRoute, RouteMetadata};
use crate::error::{NativeMessagingError, NativeMessagingResult};
use crate::traits::AppStateProvider;

// ========== SET HF TOKEN ==========

#[derive(Debug, Deserialize, Serialize)]
pub struct SetHfTokenRequest {
    pub token: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SetHfTokenResponse {
    pub message: String,
}

pub struct SetHfTokenRoute;

#[async_trait]
impl NativeMessagingRoute for SetHfTokenRoute {
    type Request = SetHfTokenRequest;
    type Response = SetHfTokenResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "set_hf_token",
            tags: &["HuggingFace", "Auth"],
            description: "Store HuggingFace API token securely",
            openai_compatible: false,
            idempotent: false,
            requires_auth: false,
            rate_limit_tier: None,
            supports_streaming: false,
            supports_binary: false,
            max_payload_size: Some(1024),
        }
    }

    async fn validate_request(req: &Self::Request) -> NativeMessagingResult<()> {
        if !req.token.starts_with("hf_") {
            return Err(NativeMessagingError::validation(
                "token",
                "Token must start with 'hf_'"
            ));
        }
        Ok(())
    }

    async fn handle<S>(req: Self::Request, state: &S) -> NativeMessagingResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request_id = Uuid::new_v4();
        tracing::info!(request_id = %request_id, "Set HF token request");

        let request_value = tabagent_values::RequestValue::from_json(&serde_json::to_string(&serde_json::json!({
            "action": "set_hf_token",
            "token": req.token
        }))?)?;

        state.handle_request(request_value).await
            .map_err(|e| NativeMessagingError::Backend(e))?;

        Ok(SetHfTokenResponse {
            message: "HuggingFace token stored securely".to_string(),
        })
    }

    fn test_cases() -> Vec<crate::route_trait::TestCase<Self::Request, Self::Response>> {
        vec![
            crate::route_trait::TestCase {
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

crate::enforce_native_messaging_route!(SetHfTokenRoute);

// ========== GET HF TOKEN STATUS ==========

#[derive(Debug, Deserialize, Serialize)]
pub struct GetHfTokenStatusRequest;

#[derive(Debug, Deserialize, Serialize)]
pub struct GetHfTokenStatusResponse {
    pub has_token: bool,
    pub message: String,
}

pub struct GetHfTokenStatusRoute;

#[async_trait]
impl NativeMessagingRoute for GetHfTokenStatusRoute {
    type Request = GetHfTokenStatusRequest;
    type Response = GetHfTokenStatusResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "get_hf_token_status",
            tags: &["HuggingFace", "Auth"],
            description: "Check if HuggingFace token is stored",
            openai_compatible: false,
            idempotent: true,
            requires_auth: false,
            rate_limit_tier: None,
            supports_streaming: false,
            supports_binary: false,
            max_payload_size: None,
        }
    }

    async fn validate_request(_req: &Self::Request) -> NativeMessagingResult<()> {
        Ok(())
    }

    async fn handle<S>(_req: Self::Request, state: &S) -> NativeMessagingResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request_id = Uuid::new_v4();
        tracing::info!(request_id = %request_id, "Get HF token status request");

        let request_value = tabagent_values::RequestValue::from_json(r#"{"action":"get_hf_token_status"}"#)?;

        let response = state.handle_request(request_value).await
            .map_err(|e| NativeMessagingError::Backend(e))?;

        let json_str = response.to_json()?;
        let data: serde_json::Value = serde_json::from_str(&json_str)?;

        Ok(GetHfTokenStatusResponse {
            has_token: data["hasToken"].as_bool().unwrap_or(false),
            message: data["message"].as_str().unwrap_or("").to_string(),
        })
    }

    fn test_cases() -> Vec<crate::route_trait::TestCase<Self::Request, Self::Response>> {
        vec![
            crate::route_trait::TestCase {
                name: "get_status",
                request: GetHfTokenStatusRequest,
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_native_messaging_route!(GetHfTokenStatusRoute);

// ========== CLEAR HF TOKEN ==========

#[derive(Debug, Deserialize, Serialize)]
pub struct ClearHfTokenRequest;

#[derive(Debug, Deserialize, Serialize)]
pub struct ClearHfTokenResponse {
    pub message: String,
}

pub struct ClearHfTokenRoute;

#[async_trait]
impl NativeMessagingRoute for ClearHfTokenRoute {
    type Request = ClearHfTokenRequest;
    type Response = ClearHfTokenResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "clear_hf_token",
            tags: &["HuggingFace", "Auth"],
            description: "Remove stored HuggingFace token",
            openai_compatible: false,
            idempotent: true,
            requires_auth: false,
            rate_limit_tier: None,
            supports_streaming: false,
            supports_binary: false,
            max_payload_size: None,
        }
    }

    async fn validate_request(_req: &Self::Request) -> NativeMessagingResult<()> {
        Ok(())
    }

    async fn handle<S>(_req: Self::Request, state: &S) -> NativeMessagingResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request_id = Uuid::new_v4();
        tracing::info!(request_id = %request_id, "Clear HF token request");

        let request_value = tabagent_values::RequestValue::from_json(r#"{"action":"clear_hf_token"}"#)?;

        state.handle_request(request_value).await
            .map_err(|e| NativeMessagingError::Backend(e))?;

        Ok(ClearHfTokenResponse {
            message: "HuggingFace token removed".to_string(),
        })
    }

    fn test_cases() -> Vec<crate::route_trait::TestCase<Self::Request, Self::Response>> {
        vec![
            crate::route_trait::TestCase {
                name: "clear_token",
                request: ClearHfTokenRequest,
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_native_messaging_route!(ClearHfTokenRoute);

