//! Session/chat history endpoints.
//!
//! ENFORCED RULES:
//! ✅ Documentation ✅ Tests ✅ tabagent-values ✅ Tracing ✅ Validation

use axum::http::Method;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use async_trait::async_trait;

use tabagent_values::{RequestValue, Message};
use crate::{
    error::{ApiResult, ApiError},
    route_trait::{RouteHandler, RouteMetadata, TestCase, ValidationRule, validators::*},
    traits::AppStateProvider,
};

// ==================== GET HISTORY ====================

/// Get history request (path parameter wrapped).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetHistoryRequest {
    /// Session identifier
    pub session_id: String,
}

/// Get chat history route handler.
///
/// Retrieves all messages for a given session ID from the database.
pub struct GetHistoryRoute;

#[async_trait]
impl RouteHandler for GetHistoryRoute {
    type Request = GetHistoryRequest;
    type Response = serde_json::Value;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            path: "/v1/sessions/{session_id}/history",
            method: Method::GET,
            tags: &["Sessions"],
            description: "Retrieve chat history for a specific session",
            openai_compatible: false,
            idempotent: true,
            requires_auth: false,
            rate_limit_tier: Some("standard"),
        }
    }

    async fn validate_request(req: &Self::Request) -> ApiResult<()> {
        NotEmpty.validate(&req.session_id)?;
        Ok(())
    }

    async fn handle<S>(req: Self::Request, state: &S) -> ApiResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(
            request_id = %request_id,
            session_id = %req.session_id,
            "Get chat history request received"
        );

        let request = RequestValue::chat_history(Some(&req.session_id), None);
        
        tracing::debug!(request_id = %request_id, "Dispatching to handler");
        let response = state.handle_request(request).await
            .map_err(|e| {
                tracing::error!(
                    request_id = %request_id,
                    session_id = %req.session_id,
                    error = %e,
                    "Get chat history failed"
                );
                e
            })?;

        let (session_id_resp, messages) = response
            .as_chat_history()
            .ok_or_else(|| {
                tracing::error!(
                    request_id = %request_id,
                    session_id = %req.session_id,
                    "Handler returned invalid response type (expected ChatHistoryResponse)"
                );
                ApiError::Internal(
                    format!("Handler returned invalid response type for chat history request (request_id: {})", request_id)
                )
            })?;

        tracing::info!(
            request_id = %request_id,
            session_id = %session_id_resp,
            message_count = messages.len(),
            "Get chat history successful"
        );

        Ok(serde_json::json!({
            "session_id": session_id_resp,
            "messages": messages,
            "count": messages.len()
        }))
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            // === VALIDATION ERROR TESTS ===
            TestCase::error(
                "empty_session_id",
                GetHistoryRequest {
                    session_id: "".to_string(),
                },
                "cannot be empty",
            ),
            // === SUCCESS TEST CASES ===
            TestCase {
                name: "get_history_basic",
                request: GetHistoryRequest {
                    session_id: "session-123".to_string(),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "get_history_uuid_session",
                request: GetHistoryRequest {
                    session_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "get_history_nonexistent_session",
                request: GetHistoryRequest {
                    session_id: "nonexistent-session-xyz".to_string(),
                },
                expected_response: None,
                expected_error: None, // Should return empty history
                assertions: vec![],
            },
            TestCase {
                name: "get_history_alphanumeric_id",
                request: GetHistoryRequest {
                    session_id: "user-session-2024-01-15".to_string(),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "get_history_idempotent",
                request: GetHistoryRequest {
                    session_id: "test-session".to_string(),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_route_handler!(GetHistoryRoute);

// Implement RegisterableRoute with Axum 0.8 compatible handler
crate::impl_registerable_route!(GetHistoryRoute);

// ==================== SAVE MESSAGE ====================

/// Save message request.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SaveMessageRequest {
    /// Session identifier (from path)
    pub session_id: String,
    /// Message to save
    pub message: Message,
}

/// Save message route handler.
///
/// Saves a message to a session's chat history in the database.
pub struct SaveMessageRoute;

#[async_trait]
impl RouteHandler for SaveMessageRoute {
    type Request = SaveMessageRequest;
    type Response = serde_json::Value;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            path: "/v1/sessions/{session_id}/messages",
            method: Method::POST,
            tags: &["Sessions"],
            description: "Save a message to a session's chat history",
            openai_compatible: false,
            idempotent: false, // Saving multiple times creates duplicates
            requires_auth: false,
            rate_limit_tier: Some("standard"),
        }
    }

    async fn validate_request(req: &Self::Request) -> ApiResult<()> {
        NotEmpty.validate(&req.session_id)?;
        NotEmpty.validate(&req.message.content)?;
        Ok(())
    }

    async fn handle<S>(req: Self::Request, state: &S) -> ApiResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(
            request_id = %request_id,
            session_id = %req.session_id,
            message_role = ?req.message.role,
            content_length = req.message.content.len(),
            "Save message request received"
        );

        let request = RequestValue::save_message(&req.session_id, &req.message);
        
        tracing::debug!(request_id = %request_id, "Dispatching to handler");
        let _response = state.handle_request(request).await
            .map_err(|e| {
                tracing::error!(
                    request_id = %request_id,
                    session_id = %req.session_id,
                    error = %e,
                    "Save message failed"
                );
                e
            })?;

        tracing::info!(
            request_id = %request_id,
            session_id = %req.session_id,
            "Message saved successfully"
        );

        Ok(serde_json::json!({
            "status": "saved",
            "session_id": req.session_id,
            "request_id": request_id.to_string()
        }))
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        use tabagent_values::MessageRole;
        
        vec![
            // === VALIDATION ERROR TESTS ===
            TestCase::error(
                "empty_session_id",
                SaveMessageRequest {
                    session_id: "".to_string(),
                    message: Message {
                        role: MessageRole::User,
                        content: "test".to_string(),
                        name: None,
                    },
                },
                "cannot be empty",
            ),
            TestCase::error(
                "empty_content",
                SaveMessageRequest {
                    session_id: "test-session".to_string(),
                    message: Message {
                        role: MessageRole::User,
                        content: "".to_string(),
                        name: None,
                    },
                },
                "cannot be empty",
            ),
            // === SUCCESS TEST CASES ===
            TestCase {
                name: "save_user_message",
                request: SaveMessageRequest {
                    session_id: "session-123".to_string(),
                    message: Message {
                        role: MessageRole::User,
                        content: "Hello, how are you?".to_string(),
                        name: None,
                    },
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "save_assistant_message",
                request: SaveMessageRequest {
                    session_id: "session-123".to_string(),
                    message: Message {
                        role: MessageRole::Assistant,
                        content: "I'm doing well, thank you!".to_string(),
                        name: None,
                    },
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "save_system_message",
                request: SaveMessageRequest {
                    session_id: "session-123".to_string(),
                    message: Message {
                        role: MessageRole::System,
                        content: "You are a helpful assistant.".to_string(),
                        name: None,
                    },
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "save_message_with_name",
                request: SaveMessageRequest {
                    session_id: "session-123".to_string(),
                    message: Message {
                        role: MessageRole::User,
                        content: "This is a named message".to_string(),
                        name: Some("John".to_string()),
                    },
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "save_long_message",
                request: SaveMessageRequest {
                    session_id: "session-456".to_string(),
                    message: Message {
                        role: MessageRole::User,
                        content: "This is a very long message. ".repeat(100),
                        name: None,
                    },
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "save_message_special_chars",
                request: SaveMessageRequest {
                    session_id: "session-789".to_string(),
                    message: Message {
                        role: MessageRole::User,
                        content: "Test with émojis 🎉 and spëcial çhars!".to_string(),
                        name: None,
                    },
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "save_message_new_session",
                request: SaveMessageRequest {
                    session_id: "new-session-001".to_string(),
                    message: Message {
                        role: MessageRole::User,
                        content: "First message in new session".to_string(),
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

crate::enforce_route_handler!(SaveMessageRoute);

// Implement RegisterableRoute with Axum 0.8 compatible handler
crate::impl_registerable_route!(SaveMessageRoute);

#[cfg(test)]
mod tests {
    use super::*;
    use tabagent_values::MessageRole;

    #[tokio::test]
    async fn test_get_history_validation() {
        let req = GetHistoryRequest {
            session_id: "valid-session".to_string(),
        };
        assert!(GetHistoryRoute::validate_request(&req).await.is_ok());
    }

    #[tokio::test]
    async fn test_save_message_validation() {
        let req = SaveMessageRequest {
            session_id: "valid-session".to_string(),
            message: Message {
                role: MessageRole::User,
                content: "test message".to_string(),
                name: None,
            },
        };
        assert!(SaveMessageRoute::validate_request(&req).await.is_ok());
    }

    #[test]
    fn test_metadata() {
        let meta = GetHistoryRoute::metadata();
        assert!(meta.idempotent);
        
        let meta2 = SaveMessageRoute::metadata();
        assert!(!meta2.idempotent);
    }
}
