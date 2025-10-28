//! Session management endpoints for native messaging.

use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use tabagent_values::{RequestValue, Message};
use crate::{
    error::{NativeMessagingResult, NativeMessagingError},
    route_trait::{NativeMessagingRoute, RouteMetadata, TestCase},
    traits::AppStateProvider,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetHistoryRequest {
    pub session_id: String,
    #[serde(default)]
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetHistoryResponse {
    pub session_id: String,
    pub messages: Vec<Message>,
    pub total_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveMessageRequest {
    pub session_id: String,
    pub message: Message,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveMessageResponse {
    pub success: bool,
    pub session_id: String,
    pub message_id: String,
}

pub struct GetHistoryRoute;
pub struct SaveMessageRoute;

#[async_trait]
impl NativeMessagingRoute for GetHistoryRoute {
    type Request = GetHistoryRequest;
    type Response = GetHistoryResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "get_history",
            tags: &["Sessions", "History"],
            description: "Retrieve conversation history for a session",
            openai_compatible: false,
            idempotent: true,
            requires_auth: false,
            rate_limit_tier: Some("standard"),
            supports_streaming: false,
            supports_binary: false,
            max_payload_size: Some(1024 * 1024),
        }
    }

    async fn validate_request(req: &Self::Request) -> NativeMessagingResult<()> {
        if req.session_id.is_empty() {
            return Err(NativeMessagingError::validation("session_id", "cannot be empty"));
        }
        if let Some(limit) = req.limit {
            if limit == 0 || limit > 1000 {
                return Err(NativeMessagingError::validation("limit", "must be between 1 and 1000"));
            }
        }
        Ok(())
    }

    async fn handle<S>(req: Self::Request, state: &S) -> NativeMessagingResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(request_id = %request_id, route = "get_history", session_id = %req.session_id);

        let request = RequestValue::chat_history(Some(&req.session_id), req.limit.map(|l| l as usize));
        let response = state.handle_request(request).await
            .map_err(|e| NativeMessagingError::Backend(e))?;

        let (session_id, messages) = response.as_chat_history()
            .ok_or_else(|| NativeMessagingError::internal("Invalid response type"))?;

        Ok(GetHistoryResponse {
            session_id: session_id.to_string(),
            messages: messages.to_vec(),
            total_count: messages.len() as u32,
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::error("empty_session_id", GetHistoryRequest {
                session_id: "".to_string(),
                limit: None,
            }, "session_id"),
            TestCase {
                name: "get_history",
                request: GetHistoryRequest {
                    session_id: "session-123".to_string(),
                    limit: Some(50),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

#[async_trait]
impl NativeMessagingRoute for SaveMessageRoute {
    type Request = SaveMessageRequest;
    type Response = SaveMessageResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "save_message",
            tags: &["Sessions", "History"],
            description: "Save a message to conversation history",
            openai_compatible: false,
            idempotent: false,
            requires_auth: false,
            rate_limit_tier: Some("standard"),
            supports_streaming: false,
            supports_binary: false,
            max_payload_size: Some(1024 * 1024),
        }
    }

    async fn validate_request(req: &Self::Request) -> NativeMessagingResult<()> {
        if req.session_id.is_empty() {
            return Err(NativeMessagingError::validation("session_id", "cannot be empty"));
        }
        if req.message.content.is_empty() {
            return Err(NativeMessagingError::validation("message.content", "cannot be empty"));
        }
        Ok(())
    }

    async fn handle<S>(req: Self::Request, state: &S) -> NativeMessagingResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(request_id = %request_id, route = "save_message", session_id = %req.session_id);

        let request = RequestValue::save_message(&req.session_id, &req.message);
        let _response = state.handle_request(request).await
            .map_err(|e| NativeMessagingError::Backend(e))?;

        let message_id = uuid::Uuid::new_v4().to_string();

        Ok(SaveMessageResponse {
            success: true,
            session_id: req.session_id,
            message_id,
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::error("empty_session_id", SaveMessageRequest {
                session_id: "".to_string(),
                message: Message {
                    role: tabagent_values::MessageRole::User,
                    content: "Hello".to_string(),
                    name: None,
                },
            }, "session_id"),
            TestCase {
                name: "save_message",
                request: SaveMessageRequest {
                    session_id: "session-123".to_string(),
                    message: Message {
                        role: tabagent_values::MessageRole::User,
                        content: "Hello, how are you?".to_string(),
                        name: None,
                    },
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_native_messaging_route!(GetHistoryRoute);
crate::enforce_native_messaging_route!(SaveMessageRoute);