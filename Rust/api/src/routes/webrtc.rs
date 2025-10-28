//! WebRTC signaling routes.
//!
//! These routes handle WebRTC session handshaking (SDP offer/answer, ICE candidates).
//! The actual WebRTC connection logic lives in the server's handler.
//! Once handshake is complete, WebRTC goes peer-to-peer (no server involved).
//!
//! ENFORCED RULES:
//! ✅ Documentation (module-level and type-level docs)
//! ✅ Tests (see test modules)
//! ✅ Uses tabagent-values (RequestValue/ResponseValue)
//! ✅ Proper tracing (request_id)
//! ✅ Proper error handling (ApiError)
//! ✅ Validation (implemented)
//! ✅ Metadata (RouteHandler trait)

use axum::http::Method;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use async_trait::async_trait;

use crate::error::ApiResult;
use crate::route_trait::{RouteHandler, RouteMetadata, TestCase};
use crate::traits::AppStateProvider;

// ============================================================================
// POST /v1/webrtc/offer - Create WebRTC Offer
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateOfferRequest {
    /// SDP offer string
    pub sdp: String,
    /// Optional peer identifier
    #[serde(default)]
    pub peer_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct CreateOfferResponse {
    /// WebRTC session ID
    pub session_id: String,
    /// Creation timestamp
    pub created_at: String,
}

pub struct CreateOfferRoute;

#[async_trait]
impl RouteHandler for CreateOfferRoute {
    type Request = CreateOfferRequest;
    type Response = CreateOfferResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            path: "/v1/webrtc/offer",
            method: Method::POST,
            description: "Create WebRTC offer and get session ID for signaling",
            tags: &["WebRTC"],
            openai_compatible: false,
            requires_auth: false,
            rate_limit_tier: None,
            idempotent: false,
        }
    }

    async fn validate_request(req: &Self::Request) -> ApiResult<()> {
        if req.sdp.trim().is_empty() {
            return Err(crate::error::ApiError::BadRequest("SDP offer cannot be empty".into()));
        }
        Ok(())
    }

    async fn handle<S>(req: Self::Request, state: &S) -> ApiResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(
            request_id = %request_id,
            peer_id = ?req.peer_id,
            "WebRTC offer received"
        );

        let request = tabagent_values::RequestValue::create_webrtc_offer(
            &req.sdp,
            req.peer_id.as_deref(),
        );

        let response = state.handle_request(request).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Create offer failed");
                e
            })?;

        // Extract session info from backend response
        let (session_id, created_at) = response.as_webrtc_session_created();

        tracing::info!(
            request_id = %request_id,
            session_id = %session_id,
            "WebRTC offer created"
        );

        Ok(CreateOfferResponse {
            session_id: session_id.to_string(),
            created_at: created_at.to_string(),
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::error(
                "empty_sdp",
                CreateOfferRequest {
                    sdp: "".to_string(),
                    peer_id: None,
                },
                "cannot be empty",
            ),
            TestCase::error(
                "whitespace_only_sdp",
                CreateOfferRequest {
                    sdp: "   ".to_string(),
                    peer_id: None,
                },
                "cannot be empty",
            ),
        ]
    }
}

// Enforce compile-time rules
crate::enforce_route_handler!(CreateOfferRoute);

// Implement RegisterableRoute with Axum 0.8 compatible handler
crate::impl_registerable_route!(CreateOfferRoute);

// ============================================================================
// POST /v1/webrtc/answer - Submit WebRTC Answer
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SubmitAnswerRequest {
    /// Session ID from offer creation
    pub session_id: String,
    /// SDP answer string
    pub sdp: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct SubmitAnswerResponse {
    pub success: bool,
    pub session_id: String,
}

pub struct SubmitAnswerRoute;

#[async_trait]
impl RouteHandler for SubmitAnswerRoute {
    type Request = SubmitAnswerRequest;
    type Response = SubmitAnswerResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            path: "/v1/webrtc/answer",
            method: Method::POST,
            description: "Submit WebRTC answer for an existing session",
            tags: &["WebRTC"],
            openai_compatible: false,
            requires_auth: false,
            rate_limit_tier: None,
            idempotent: true,
        }
    }

    async fn validate_request(req: &Self::Request) -> ApiResult<()> {
        if req.session_id.trim().is_empty() {
            return Err(crate::error::ApiError::BadRequest("Session ID cannot be empty".into()));
        }
        if req.sdp.trim().is_empty() {
            return Err(crate::error::ApiError::BadRequest("SDP answer cannot be empty".into()));
        }
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
            "WebRTC answer received"
        );

        let request = tabagent_values::RequestValue::submit_webrtc_answer(&req.session_id, &req.sdp);

        // Call backend - if it succeeds, we're good
        state.handle_request(request).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Submit answer failed");
                e
            })?;

        tracing::info!(
            request_id = %request_id,
            session_id = %req.session_id,
            "WebRTC answer submitted"
        );

        Ok(SubmitAnswerResponse {
            success: true,
            session_id: req.session_id,
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::error(
                "empty_session_id",
                SubmitAnswerRequest {
                    session_id: "".to_string(),
                    sdp: "valid_sdp".to_string(),
                },
                "cannot be empty",
            ),
            TestCase::error(
                "empty_sdp",
                SubmitAnswerRequest {
                    session_id: "session123".to_string(),
                    sdp: "".to_string(),
                },
                "cannot be empty",
            ),
        ]
    }
}

// Enforce compile-time rules
crate::enforce_route_handler!(SubmitAnswerRoute);

// Implement RegisterableRoute with Axum 0.8 compatible handler
crate::impl_registerable_route!(SubmitAnswerRoute);

// ============================================================================
// POST /v1/webrtc/ice - Add ICE Candidate
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AddIceCandidateRequest {
    /// Session ID
    pub session_id: String,
    /// ICE candidate string
    pub candidate: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct AddIceCandidateResponse {
    pub success: bool,
    pub session_id: String,
}

pub struct AddIceCandidateRoute;

#[async_trait]
impl RouteHandler for AddIceCandidateRoute {
    type Request = AddIceCandidateRequest;
    type Response = AddIceCandidateResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            path: "/v1/webrtc/ice",
            method: Method::POST,
            description: "Add ICE candidate to WebRTC session",
            tags: &["WebRTC"],
            openai_compatible: false,
            requires_auth: false,
            rate_limit_tier: None,
            idempotent: true,
        }
    }

    async fn validate_request(req: &Self::Request) -> ApiResult<()> {
        if req.session_id.trim().is_empty() {
            return Err(crate::error::ApiError::BadRequest("Session ID cannot be empty".into()));
        }
        if req.candidate.trim().is_empty() {
            return Err(crate::error::ApiError::BadRequest("ICE candidate cannot be empty".into()));
        }
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
            "ICE candidate received"
        );

        let request = tabagent_values::RequestValue::add_ice_candidate(&req.session_id, &req.candidate);

        // Call backend - if it succeeds, we're good
        state.handle_request(request).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Add ICE candidate failed");
                e
            })?;

        tracing::info!(
            request_id = %request_id,
            session_id = %req.session_id,
            "ICE candidate added"
        );

        Ok(AddIceCandidateResponse {
            success: true,
            session_id: req.session_id,
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::error(
                "empty_session_id",
                AddIceCandidateRequest {
                    session_id: "".to_string(),
                    candidate: "candidate:1234".to_string(),
                },
                "cannot be empty",
            ),
            TestCase::error(
                "empty_candidate",
                AddIceCandidateRequest {
                    session_id: "session123".to_string(),
                    candidate: "".to_string(),
                },
                "cannot be empty",
            ),
        ]
    }
}

// Enforce compile-time rules
crate::enforce_route_handler!(AddIceCandidateRoute);

// Implement RegisterableRoute with Axum 0.8 compatible handler
crate::impl_registerable_route!(AddIceCandidateRoute);

// ============================================================================
// GET /v1/webrtc/session/{session_id} - Get Session State
// ============================================================================

// Note: This route uses Path parameter, so we handle it separately
// in the router registration without the trait system for now.
// TODO: Extend trait system to support Path parameters

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WebRtcSessionResponse {
    /// Session ID
    pub session_id: String,
    /// Connection state (new, connecting, connected, disconnected)
    pub state: String,
    /// SDP offer (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offer: Option<String>,
    /// SDP answer (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub answer: Option<String>,
    /// ICE candidates collected so far
    pub ice_candidates: Vec<String>,
}

