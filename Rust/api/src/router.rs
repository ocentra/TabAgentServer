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
use utoipa_redoc::Redoc;

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
    
    // WebRTC signaling routes (3 trait-based + 1 manual)
    router = routes::webrtc::CreateOfferRoute::register(router);
    router = routes::webrtc::SubmitAnswerRoute::register(router);
    router = routes::webrtc::AddIceCandidateRoute::register(router);
    
    // WebRTC GET session route (manual - needs Path parameter)
    router = router
        .route("/v1/webrtc/session/:session_id", get(|
            axum::extract::State(_state): axum::extract::State<crate::traits::AppStateWrapper>,
            axum::extract::Path(session_id): axum::extract::Path<String>| async move {
            let request_id = uuid::Uuid::new_v4();
            tracing::info!(request_id = %request_id, session_id = %session_id, "Get WebRTC session");
            
            // Validate
            if session_id.trim().is_empty() {
                return Err(crate::error::ApiError::BadRequest("Session ID cannot be empty".into()));
            }
            
            let _request = tabagent_values::RequestValue::get_webrtc_session(&session_id);
            
            // TODO: Call backend once handler is implemented
            // For now, return a placeholder response
            let response = routes::webrtc::WebRtcSessionResponse {
                session_id: session_id.clone(),
                state: "new".to_string(),
                offer: None,
                answer: None,
                ice_candidates: vec![],
            };
            
            tracing::info!(request_id = %request_id, session_id = %session_id, "WebRTC session retrieved");
            Ok::<_, crate::error::ApiError>(axum::Json(response))
        }));
    
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
            
            // Redoc - Beautiful three-panel documentation (uses /redoc by default)
            .merge(Redoc::new(openapi.clone()))
            
            // OpenAPI JSON spec endpoint
            .route("/api-doc/openapi.json", get(|| async move { 
                axum::Json(openapi)
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

