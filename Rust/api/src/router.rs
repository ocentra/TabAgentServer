//! Router configuration and setup.

use axum::{
    Router,
    routing::{get, post},
};
use tower_http::{
    trace::{TraceLayer, DefaultMakeSpan, DefaultOnResponse},
    compression::CompressionLayer,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use utoipa_rapidoc::RapiDoc;
// Redoc served manually via HTML route

use crate::{
    config::ApiConfig,
    middleware,
    routes,
};

/// Configure routes and middleware with Axum 0.8 compatibility.
///
/// KEY FIX: Middleware layers applied BEFORE .with_state() for proper type inference.
/// Returns a MakeService ready to be passed to axum::serve().
pub fn configure_routes(
    state: crate::traits::AppStateWrapper,
    webrtc_manager: Option<std::sync::Arc<tabagent_webrtc::WebRtcManager>>,
    config: &ApiConfig,
) -> axum::routing::IntoMakeService<Router> {
    use crate::route_trait::RegisterableRoute;
    
    // Start with stateless router
    let mut router = Router::new();
    
    // Register trait-based routes (converted routes)
    router = routes::health::HealthRoute::register(router);
    router = routes::chat::ChatRoute::register(router);
    router = routes::generate::GenerateRoute::register(router);
    router = routes::embeddings::EmbeddingsRoute::register(router);
    router = routes::models::ListModelsRoute::register(router);
    router = routes::models::LoadModelRoute::register(router);
    router = routes::models::UnloadModelRoute::register(router);
    router = routes::models::ModelInfoRoute::register(router);
    router = routes::models::GetModelQuantsRoute::register(router);
    router = routes::models::GetInferenceSettingsRoute::register(router);
    router = routes::models::SaveInferenceSettingsRoute::register(router);
    router = routes::sessions::GetHistoryRoute::register(router);
    router = routes::sessions::SaveMessageRoute::register(router);
    router = routes::rag::RagRoute::register(router);
    router = routes::rerank::RerankRoute::register(router);
    router = routes::system::SystemRoute::register(router);
    router = routes::generation::StopGenerationRoute::register(router);
    router = routes::params::GetParamsRoute::register(router);
    router = routes::params::SetParamsRoute::register(router);
    router = routes::stats::GetStatsRoute::register(router);
    router = routes::resources::GetResourcesRoute::register(router);
    router = routes::resources::EstimateMemoryRoute::register(router);
    router = routes::management::PullModelRoute::register(router);
    router = routes::management::DeleteModelRoute::register(router);
    router = routes::management::GetLoadedModelsRoute::register(router);
    router = routes::management::SelectModelRoute::register(router);
    router = routes::management::GetEmbeddingModelsRoute::register(router);
    router = routes::management::GetRecipesRoute::register(router);
    router = routes::rag_extended::SemanticSearchRoute::register(router);
    router = routes::rag_extended::SimilarityRoute::register(router);
    router = routes::rag_extended::EvaluateEmbeddingsRoute::register(router);
    router = routes::rag_extended::ClusterRoute::register(router);
    router = routes::rag_extended::RecommendRoute::register(router);
    router = routes::chat::ResponsesRoute::register(router);
    router = routes::management::GetRegisteredModelsRoute::register(router);
    router = routes::generation::GetHaltStatusRoute::register(router);
    router = routes::resources::CompatibilityRoute::register(router);
    router = routes::discovery::DiscoveryRoute::register(router);
    
    // HuggingFace Auth routes
    router = router
        .route("/v1/hf/token", post(routes::hf_auth::set_hf_token::<crate::traits::AppStateWrapper>))
        .route("/v1/hf/token/status", get(routes::hf_auth::get_hf_token_status::<crate::traits::AppStateWrapper>))
        .route("/v1/hf/token", axum::routing::delete(routes::hf_auth::clear_hf_token::<crate::traits::AppStateWrapper>));
    
    // Hardware routes
    router = router
        .route("/v1/hardware/info", get(routes::hardware::get_hardware_info::<crate::traits::AppStateWrapper>))
        .route("/v1/hardware/feasibility", post(routes::hardware::check_model_feasibility::<crate::traits::AppStateWrapper>))
        .route("/v1/hardware/recommendations", get(routes::hardware::get_recommended_models::<crate::traits::AppStateWrapper>));
    
    // WebRTC signaling routes (manual - use WebRtcManager directly)
    if let Some(manager) = webrtc_manager {
        let manager_clone = manager.clone();
        router = router
            // POST /v1/webrtc/offer
            .route("/v1/webrtc/offer", post(move |
                axum::Json(req): axum::Json<routes::webrtc::CreateOfferRequest>| {
                let manager = manager_clone.clone();
                async move {
                    let request_id = uuid::Uuid::new_v4();
                    tracing::info!(request_id = %request_id, peer_id = ?req.peer_id, "WebRTC offer received");
                    
                    // Validate
                    if req.sdp.trim().is_empty() {
                        return Err(crate::error::ApiError::BadRequest("SDP offer cannot be empty".into()));
                    }
                    
                    // Create offer via WebRtcManager (NOT AppState!)
                    let client_id = req.peer_id.unwrap_or_else(|| format!("client-{}", request_id));
                    let session = manager.create_offer(client_id).await
                        .map_err(|e| crate::error::ApiError::InternalError(e.to_string()))?;
                    
                    let response = routes::webrtc::CreateOfferResponse {
                        session_id: session.id.clone(),
                        created_at: session.created_at.to_rfc3339(),
                    };
                    
                    tracing::info!(request_id = %request_id, session_id = %session.id, "WebRTC offer created");
                    Ok::<_, crate::error::ApiError>(axum::Json(response))
                }
            }));
        
        let manager_clone = manager.clone();
        router = router
            // POST /v1/webrtc/answer
            .route("/v1/webrtc/answer", post(move |
                axum::Json(req): axum::Json<routes::webrtc::SubmitAnswerRequest>| {
                let manager = manager_clone.clone();
                async move {
                    let request_id = uuid::Uuid::new_v4();
                    tracing::info!(request_id = %request_id, session_id = %req.session_id, "WebRTC answer received");
                    
                    // Validate
                    if req.session_id.trim().is_empty() {
                        return Err(crate::error::ApiError::BadRequest("Session ID cannot be empty".into()));
                    }
                    if req.sdp.trim().is_empty() {
                        return Err(crate::error::ApiError::BadRequest("SDP answer cannot be empty".into()));
                    }
                    
                    // Submit answer via WebRtcManager (NOT AppState!)
                    manager.submit_answer(&req.session_id, req.sdp.clone()).await
                        .map_err(|e| crate::error::ApiError::InternalError(e.to_string()))?;
                    
                    let response = routes::webrtc::SubmitAnswerResponse {
                        success: true,
                        session_id: req.session_id.clone(),
                    };
                    
                    tracing::info!(request_id = %request_id, session_id = %req.session_id, "WebRTC answer accepted");
                    Ok::<_, crate::error::ApiError>(axum::Json(response))
                }
            }));
        
        let manager_clone = manager.clone();
        router = router
            // POST /v1/webrtc/ice
            .route("/v1/webrtc/ice", post(move |
                axum::Json(req): axum::Json<routes::webrtc::AddIceCandidateRequest>| {
                let manager = manager_clone.clone();
                async move {
                    let request_id = uuid::Uuid::new_v4();
                    tracing::info!(request_id = %request_id, session_id = %req.session_id, "ICE candidate received");
                    
                    // Validate
                    if req.session_id.trim().is_empty() {
                        return Err(crate::error::ApiError::BadRequest("Session ID cannot be empty".into()));
                    }
                    
                    // Add ICE candidate via WebRtcManager (NOT AppState!)
                    let ice_candidate = tabagent_webrtc::IceCandidate {
                        candidate: req.candidate.clone(),
                        sdp_mid: None,
                        sdp_mline_index: None,
                        added_at: chrono::Utc::now(),
                    };
                    manager.add_ice_candidate(&req.session_id, ice_candidate).await
                        .map_err(|e| crate::error::ApiError::InternalError(e.to_string()))?;
                    
                    let response = routes::webrtc::AddIceCandidateResponse {
                        success: true,
                        session_id: req.session_id.clone(),
                    };
                    
                    tracing::info!(request_id = %request_id, session_id = %req.session_id, "ICE candidate added");
                    Ok::<_, crate::error::ApiError>(axum::Json(response))
                }
            }));
            
        let manager_clone = manager.clone();
        router = router
            // GET /v1/webrtc/session/{session_id}
            .route("/v1/webrtc/session/{session_id}", get(move |
                axum::extract::Path(session_id): axum::extract::Path<String>| {
                let manager = manager_clone.clone();
                async move {
                    let request_id = uuid::Uuid::new_v4();
                    tracing::info!(request_id = %request_id, session_id = %session_id, "Get WebRTC session");
                    
                    // Validate
                    if session_id.trim().is_empty() {
                        return Err(crate::error::ApiError::BadRequest("Session ID cannot be empty".into()));
                    }
                    
                    // Get session via WebRtcManager (NOT AppState!)
                    let session = manager.get_session(&session_id).await
                        .map_err(|e| {
                            // SessionNotFound error -> 404
                            if e.to_string().contains("not found") {
                                crate::error::ApiError::NotFound(format!("Session {} not found", session_id))
                            } else {
                                crate::error::ApiError::InternalError(e.to_string())
                            }
                        })?;
                    
                    let response = routes::webrtc::WebRtcSessionResponse {
                        session_id: session.id.clone(),
                        state: session.state.clone(),
                        offer: if session.has_offer { Some("SDP Offer (present)".to_string()) } else { None },
                        answer: if session.has_answer { Some("SDP Answer (present)".to_string()) } else { None },
                        ice_candidates: vec![], // SessionInfo doesn't include full candidate list
                    };
                    
                    tracing::info!(request_id = %request_id, session_id = %session_id, "WebRTC session retrieved");
                    Ok::<_, crate::error::ApiError>(axum::Json(response))
                }
            }));
    }
    
    // Manual route aliases (trait system only registers one path per route)
    
    // /v1/halt POST alias for /v1/generation/stop
    router = router
        .route("/v1/halt", post(|
            axum::extract::State(state): axum::extract::State<crate::traits::AppStateWrapper>,
            axum::Json(req): axum::Json<routes::generation::StopGenerationRequest>| async move {
            use crate::route_trait::RouteHandler;
            routes::generation::StopGenerationRoute::validate_request(&req).await?;
            let response = routes::generation::StopGenerationRoute::handle(req, &state).await?;
            Ok::<_, crate::error::ApiError>(axum::Json(response))
        }));
    
    // /v1/load alias for /v1/models/load
    router = router
        .route("/v1/load", post(|
            axum::extract::State(state): axum::extract::State<crate::traits::AppStateWrapper>,
            axum::Json(req): axum::Json<routes::models::LoadModelRequest>| async move {
            use crate::route_trait::RouteHandler;
            routes::models::LoadModelRoute::validate_request(&req).await?;
            let response = routes::models::LoadModelRoute::handle(req, &state).await?;
            Ok::<_, crate::error::ApiError>(axum::Json(response))
        }));
    
    // /v1/unload alias for /v1/models/unload
    router = router
        .route("/v1/unload", post(|
            axum::extract::State(state): axum::extract::State<crate::traits::AppStateWrapper>,
            axum::Json(req): axum::Json<routes::models::UnloadModelRequest>| async move {
            use crate::route_trait::RouteHandler;
            routes::models::UnloadModelRoute::validate_request(&req).await?;
            let response = routes::models::UnloadModelRoute::handle(req, &state).await?;
            Ok::<_, crate::error::ApiError>(axum::Json(response))
        }));
    
    // /v1/resources/loaded-models alias for /v1/models/loaded
    router = router
        .route("/v1/resources/loaded-models", get(|
            axum::extract::State(state): axum::extract::State<crate::traits::AppStateWrapper>| async move {
            use crate::route_trait::RouteHandler;
            let req = routes::management::GetLoadedModelsRequest;
            routes::management::GetLoadedModelsRoute::validate_request(&req).await?;
            let response = routes::management::GetLoadedModelsRoute::handle(req, &state).await?;
            Ok::<_, crate::error::ApiError>(axum::Json(response))
        }));

    // Add OpenAPI documentation if enabled
    router = if config.enable_swagger {
        let openapi = routes::ApiDoc::openapi();
        
        // Add ALL THREE documentation UIs to the existing stateful router
        router
            // Swagger UI - Classic interactive docs with "Try it out"
            .merge(SwaggerUi::new("/swagger-ui")
                .url("/api-doc/openapi.json", openapi.clone()))
            
            // RapiDoc - Modern, customizable UI with multiple themes
            .merge(RapiDoc::new("/api-doc/openapi.json")
                .path("/rapidoc"))
            
            // Redoc - Serve manually via route (SwaggerUi already registers /api-doc/openapi.json)
            .route("/redoc", get(|| async {
                axum::response::Html(r#"<!DOCTYPE html>
<html>
<head>
    <title>TabAgent API - ReDoc</title>
    <meta charset="utf-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <link href="https://fonts.googleapis.com/css?family=Montserrat:300,400,700|Roboto:300,400,700" rel="stylesheet">
    <style>body { margin: 0; padding: 0; }</style>
</head>
<body>
    <redoc spec-url='/api-doc/openapi.json'></redoc>
    <script src="https://cdn.redoc.ly/redoc/latest/bundles/redoc.standalone.js"></script>
</body>
</html>"#)
            }))
    } else {
        router
    };
    
    // AXUM 0.8 FIX: Apply middleware layers BEFORE .with_state()
    // Order matters: outer to inner
    router = router
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::new().include_headers(true))
            .on_response(DefaultOnResponse::new().include_headers(true)))
        .layer(middleware::cors_layer(config));
    
    // CRITICAL: Apply state LAST for Axum 0.8 + trait objects
    let router_with_state = router.with_state(state);
    
    // Convert to MakeService immediately
    router_with_state.into_make_service()
}

