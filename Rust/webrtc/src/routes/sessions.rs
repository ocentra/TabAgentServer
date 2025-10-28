//! Sessions endpoint for WebRTC data channels.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tabagent_values::{RequestValue, Message};
use crate::{
    error::{WebRtcResult, WebRtcError},
    route_trait::{DataChannelRoute, RouteMetadata, TestCase},
    traits::RequestHandler,
};

/// Session management request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum SessionsRequest {
    /// Get chat history for a session
    GetHistory {
        /// Optional session ID (uses current if not provided)
        session_id: Option<String>,
        /// Maximum number of messages to return
        limit: Option<usize>,
    },
    /// Save a message to session history
    SaveMessage {
        /// Session identifier
        session_id: String,
        /// Message to save
        message: Message,
    },
}

/// Session management response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionsResponse {
    /// Whether the operation succeeded
    pub success: bool,
    /// Session identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Retrieved messages (for GetHistory)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages: Option<Vec<Message>>,
}

/// Session management route handler
pub struct SessionsRoute;

#[async_trait]
impl DataChannelRoute for SessionsRoute {
    type Request = SessionsRequest;
    type Response = SessionsResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "sessions",
            tags: &["Sessions", "History"],
            description: "Manage chat sessions - retrieve history and save messages",
            supports_streaming: false,
            supports_binary: false,
            requires_auth: true,
            rate_limit_tier: Some("standard"),
            max_payload_size: None,
            media_type: None,
        }
    }

    async fn validate_request(req: &Self::Request) -> WebRtcResult<()> {
        match req {
            SessionsRequest::SaveMessage { session_id, .. } => {
                if session_id.is_empty() {
                    return Err(WebRtcError::ValidationError {
                        field: "session_id".to_string(),
                        message: "session_id cannot be empty".to_string(),
                    });
                }
            }
            SessionsRequest::GetHistory { .. } => {}
        }
        Ok(())
    }

    async fn handle<H>(req: Self::Request, handler: &H) -> WebRtcResult<Self::Response>
    where
        H: RequestHandler + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        
        tracing::info!(request_id = %request_id, route = "sessions", action = ?req, "WebRTC sessions request");

        let request_value = match &req {
            SessionsRequest::GetHistory { session_id, limit } => {
                RequestValue::chat_history(session_id.clone(), *limit)
            }
            SessionsRequest::SaveMessage { session_id, message } => {
                RequestValue::save_message(session_id.clone(), message)
            }
        };

        let response = handler.handle_request(request_value).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Sessions request failed");
                WebRtcError::from(e)
            })?;

        let (session_id, messages) = response.as_chat_history()
            .map(|(sid, msgs)| (Some(sid.to_string()), Some(msgs.to_vec())))
            .unwrap_or((None, None));

        tracing::info!(request_id = %request_id, "Sessions request successful");

        Ok(SessionsResponse {
            success: true,
            session_id,
            messages,
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::success(
                "get_history",
                SessionsRequest::GetHistory {
                    session_id: Some("test-session".to_string()),
                    limit: Some(10),
                },
                SessionsResponse {
                    success: true,
                    session_id: Some("test-session".to_string()),
                    messages: Some(vec![]),
                },
            ),
            TestCase::error(
                "save_empty_session",
                SessionsRequest::SaveMessage {
                    session_id: "".to_string(),
                    message: Message {
                        role: tabagent_values::MessageRole::User,
                        content: "test".to_string(),
                        name: None,
                    },
                },
                "session_id cannot be empty",
            ),
        ]
    }
}

crate::enforce_data_channel_route!(SessionsRoute);
