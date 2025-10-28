//! Parameter management endpoints for native messaging.

use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use tabagent_values::{RequestValue, ResponseValue};
use crate::{
    error::{NativeMessagingResult, NativeMessagingError},
    route_trait::{NativeMessagingRoute, RouteMetadata, TestCase},
    traits::AppStateProvider,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetParamsRequest {
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetParamsResponse {
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
    pub stop: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetParamsRequest {
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub max_tokens: Option<u32>,
    #[serde(default)]
    pub top_p: Option<f32>,
    #[serde(default)]
    pub frequency_penalty: Option<f32>,
    #[serde(default)]
    pub presence_penalty: Option<f32>,
    #[serde(default)]
    pub stop: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetParamsResponse {
    pub success: bool,
    pub message: String,
}

pub struct GetParamsRoute;
pub struct SetParamsRoute;

#[async_trait]
impl NativeMessagingRoute for GetParamsRoute {
    type Request = GetParamsRequest;
    type Response = GetParamsResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "get_params",
            tags: &["Parameters", "Configuration"],
            description: "Get current generation parameters for a session",
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

    async fn handle<S>(req: Self::Request, state: &S) -> NativeMessagingResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(request_id = %request_id, route = "get_params", session_id = ?req.session_id);

        let request = RequestValue::get_params();
        let response = state.handle_request(request).await
            .map_err(|e| NativeMessagingError::Backend(e))?;

        let (temperature, max_tokens, top_p, frequency_penalty, presence_penalty, stop) = response.as_params()
            .ok_or_else(|| NativeMessagingError::internal("Invalid response type"))?;

        Ok(GetParamsResponse {
            temperature,
            max_tokens,
            top_p,
            frequency_penalty,
            presence_penalty,
            stop: stop.clone(),
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase {
                name: "get_params_no_session",
                request: GetParamsRequest {
                    session_id: None,
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "get_params_with_session",
                request: GetParamsRequest {
                    session_id: Some("session-123".to_string()),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

#[async_trait]
impl NativeMessagingRoute for SetParamsRoute {
    type Request = SetParamsRequest;
    type Response = SetParamsResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "set_params",
            tags: &["Parameters", "Configuration"],
            description: "Set generation parameters for a session",
            openai_compatible: false,
            idempotent: false,
            requires_auth: false,
            rate_limit_tier: Some("standard"),
            supports_streaming: false,
            supports_binary: false,
            max_payload_size: Some(64 * 1024),
        }
    }

    async fn validate_request(req: &Self::Request) -> NativeMessagingResult<()> {
        if let Some(temp) = req.temperature {
            if temp < 0.0 || temp > 2.0 {
                return Err(NativeMessagingError::validation("temperature", "must be between 0.0 and 2.0"));
            }
        }
        if let Some(top_p) = req.top_p {
            if top_p < 0.0 || top_p > 1.0 {
                return Err(NativeMessagingError::validation("top_p", "must be between 0.0 and 1.0"));
            }
        }
        if let Some(max_tokens) = req.max_tokens {
            if max_tokens == 0 || max_tokens > 100000 {
                return Err(NativeMessagingError::validation("max_tokens", "must be between 1 and 100000"));
            }
        }
        Ok(())
    }

    async fn handle<S>(req: Self::Request, state: &S) -> NativeMessagingResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(request_id = %request_id, route = "set_params", session_id = ?req.session_id);

        let params = serde_json::json!({
            "temperature": req.temperature,
            "max_tokens": req.max_tokens,
            "top_p": req.top_p,
            "frequency_penalty": req.frequency_penalty,
            "presence_penalty": req.presence_penalty,
            "stop": req.stop
        });
        let request = RequestValue::set_params(params);
        let _response = state.handle_request(request).await
            .map_err(|e| NativeMessagingError::Backend(e))?;

        Ok(SetParamsResponse {
            success: true,
            message: "Parameters updated successfully".to_string(),
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::error("invalid_temperature", SetParamsRequest {
                session_id: None,
                temperature: Some(3.0),
                max_tokens: None,
                top_p: None,
                frequency_penalty: None,
                presence_penalty: None,
                stop: None,
            }, "temperature"),
            TestCase {
                name: "set_params_basic",
                request: SetParamsRequest {
                    session_id: Some("session-123".to_string()),
                    temperature: Some(0.7),
                    max_tokens: Some(1000),
                    top_p: Some(0.9),
                    frequency_penalty: Some(0.1),
                    presence_penalty: Some(0.1),
                    stop: Some(vec!["END".to_string()]),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_native_messaging_route!(GetParamsRoute);
crate::enforce_native_messaging_route!(SetParamsRoute);