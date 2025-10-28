//! Session management endpoints for WebRTC data channels.
//!
//! ENFORCED RULES:
//! ✅ Documentation ✅ Tests ✅ tabagent-values ✅ Tracing ✅ Validation

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tabagent_values::{RequestValue, Message};
use crate::{
    error::{WebRtcResult, WebRtcError},
    route_trait::{DataChannelRoute, RouteMetadata, TestCase},
    traits::RequestHandler,
};

// ==================== GET HISTORY ====================

/// Get chat history request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetHistoryRequest {
    /// Optional session ID (uses current if not provided)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Maximum number of messages to return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}

/// Get chat history response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetHistoryResponse {
    /// Session identifier
    pub session_id: String,
    /// Retrieved messages
    pub messages: Vec<Message>,
}

/// Get chat history route handler.
pub struct GetHistoryRoute;

#[async_trait]
impl DataChannelRoute for GetHistoryRoute {
    type Request = GetHistoryRequest;
    type Response = GetHistoryResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "get_history",
            tags: &["Sessions", "History"],
            description: "Retrieve chat history for a session",
            supports_streaming: false,
            supports_binary: false,
            requires_auth: true,
            rate_limit_tier: Some("standard"),
            max_payload_size: None,
            media_type: None,
        }
    }

    async fn validate_request(_req: &Self::Request) -> WebRtcResult<()> {
        Ok(()) // session_id is optional
    }

    async fn handle<H>(req: Self::Request, handler: &H) -> WebRtcResult<Self::Response>
    where
        H: RequestHandler + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(
            request_id = %request_id,
            route = "get_history",
            session_id = ?req.session_id,
            limit = ?req.limit,
            "WebRTC get history request"
        );

        let request_value = RequestValue::chat_history(req.session_id.clone(), req.limit);

        let response = handler.handle_request(request_value).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Get history request failed");
                WebRtcError::from(e)
            })?;

        let (session_id, messages) = response.as_chat_history()
            .ok_or_else(|| WebRtcError::InternalError("Invalid response type".to_string()))?;

        tracing::info!(
            request_id = %request_id,
            message_count = messages.len(),
            "Get history successful"
        );

        Ok(GetHistoryResponse {
            session_id: session_id.to_string(),
            messages: messages.to_vec(),
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::success(
                "get_history",
                GetHistoryRequest {
                    session_id: Some("test-session".to_string()),
                    limit: Some(10),
                },
                GetHistoryResponse {
                    session_id: "test-session".to_string(),
                    messages: vec![],
                },
            ),
        ]
    }
}

crate::enforce_data_channel_route!(GetHistoryRoute);

// ==================== SAVE MESSAGE ====================

/// Save message to session request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveMessageRequest {
    /// Session identifier
    pub session_id: String,
    /// Message to save
    pub message: Message,
}

/// Save message response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveMessageResponse {
    /// Whether the operation succeeded
    pub success: bool,
    /// Session identifier
    pub session_id: String,
    /// Status message
    pub message: String,
}

/// Save message route handler.
pub struct SaveMessageRoute;

#[async_trait]
impl DataChannelRoute for SaveMessageRoute {
    type Request = SaveMessageRequest;
    type Response = SaveMessageResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "save_message",
            tags: &["Sessions", "History"],
            description: "Save a message to session history",
            supports_streaming: false,
            supports_binary: false,
            requires_auth: true,
            rate_limit_tier: Some("standard"),
            max_payload_size: Some(1024 * 1024), // 1MB
            media_type: None,
        }
    }

    async fn validate_request(req: &Self::Request) -> WebRtcResult<()> {
        if req.session_id.is_empty() {
            return Err(WebRtcError::ValidationError {
                field: "session_id".to_string(),
                message: "session_id cannot be empty".to_string(),
            });
        }
        if req.message.content.is_empty() {
            return Err(WebRtcError::ValidationError {
                field: "message.content".to_string(),
                message: "message content cannot be empty".to_string(),
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
            route = "save_message",
            session_id = %req.session_id,
            "WebRTC save message request"
        );

        let request_value = RequestValue::save_message(req.session_id.clone(), &req.message);

        handler.handle_request(request_value).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Save message request failed");
                WebRtcError::from(e)
            })?;

        tracing::info!(request_id = %request_id, "Save message successful");

        Ok(SaveMessageResponse {
            success: true,
            session_id: req.session_id,
            message: "Message saved successfully".to_string(),
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::success(
                "save_message",
                SaveMessageRequest {
                    session_id: "test-session".to_string(),
                    message: Message {
                        role: tabagent_values::MessageRole::User,
                        content: "test message".to_string(),
                        name: None,
                    },
                },
                SaveMessageResponse {
                    success: true,
                    session_id: "test-session".to_string(),
                    message: "Message saved successfully".to_string(),
                },
            ),
            TestCase::error(
                "save_empty_session",
                SaveMessageRequest {
                    session_id: "".to_string(),
                    message: Message {
                        role: tabagent_values::MessageRole::User,
                        content: "test".to_string(),
                        name: None,
                    },
                },
                "session_id cannot be empty",
            ),
            TestCase::error(
                "save_empty_content",
                SaveMessageRequest {
                    session_id: "test-session".to_string(),
                    message: Message {
                        role: tabagent_values::MessageRole::User,
                        content: "".to_string(),
                        name: None,
                    },
                },
                "message content cannot be empty",
            ),
        ]
    }
}

crate::enforce_data_channel_route!(SaveMessageRoute);
