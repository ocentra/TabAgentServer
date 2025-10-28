//! Message routing and dispatch system.
//!
//! This module provides the core routing functionality that dispatches
//! incoming native messaging requests to appropriate route handlers.

use crate::{
    error::{NativeMessagingError, NativeMessagingResult, ErrorResponse},
    protocol::{IncomingMessage, OutgoingMessage},
    route_trait::NativeMessagingRoute,
    traits::AppStateProvider,
};
use std::{collections::HashMap, sync::Arc};
use async_trait::async_trait;

/// Type-erased route dispatcher trait.
///
/// This trait allows storing different route types in a single collection
/// while maintaining type safety for request/response handling.
#[async_trait]
pub trait RouteDispatcher: Send + Sync {
    /// Dispatch a request to this route.
    ///
    /// # Arguments
    ///
    /// * `payload` - The request payload as JSON
    /// * `state` - Application state provider
    ///
    /// # Returns
    ///
    /// * `Ok(serde_json::Value)` - Successful response data
    /// * `Err(NativeMessagingError)` - Processing error
    async fn dispatch(
        &self,
        payload: serde_json::Value,
        state: &Arc<dyn AppStateProvider>,
    ) -> NativeMessagingResult<serde_json::Value>;
    
    /// Get route metadata.
    fn metadata(&self) -> crate::route_trait::RouteMetadata;
}

/// Concrete route dispatcher implementation.
pub struct ConcreteRouteDispatcher<R: NativeMessagingRoute> {
    _phantom: std::marker::PhantomData<R>,
}

impl<R: NativeMessagingRoute> ConcreteRouteDispatcher<R> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<R: NativeMessagingRoute> RouteDispatcher for ConcreteRouteDispatcher<R> {
    async fn dispatch(
        &self,
        payload: serde_json::Value,
        state: &Arc<dyn AppStateProvider>,
    ) -> NativeMessagingResult<serde_json::Value> {
        let request_id = uuid::Uuid::new_v4();
        
        tracing::debug!(
            request_id = %request_id,
            route = R::metadata().route_id,
            "Dispatching to route handler"
        );
        
        // Deserialize request
        let request: R::Request = serde_json::from_value(payload)
            .map_err(|e| NativeMessagingError::validation(
                "payload", 
                &format!("Invalid request format: {}", e)
            ))?;
        
        // Validate request
        R::validate_request(&request).await?;
        
        // Handle request
        let response = R::handle(request, state).await?;
        
        // Serialize response
        let response_value = serde_json::to_value(response)
            .map_err(|e| NativeMessagingError::internal(
                format!("Failed to serialize response: {}", e)
            ))?;
        
        tracing::debug!(
            request_id = %request_id,
            route = R::metadata().route_id,
            "Route handler completed successfully"
        );
        
        Ok(response_value)
    }
    
    fn metadata(&self) -> crate::route_trait::RouteMetadata {
        R::metadata()
    }
}

/// Message router that dispatches requests to appropriate handlers.
pub struct MessageRouter {
    routes: HashMap<String, Box<dyn RouteDispatcher>>,
    state: Arc<dyn AppStateProvider>,
}

impl MessageRouter {
    /// Create a new message router.
    ///
    /// # Arguments
    ///
    /// * `state` - Application state provider
    pub fn new(state: Arc<dyn AppStateProvider>) -> Self {
        Self {
            routes: HashMap::new(),
            state,
        }
    }
    
    /// Register a route handler.
    ///
    /// # Type Parameters
    ///
    /// * `R` - Route type implementing NativeMessagingRoute
    pub fn register_route<R: NativeMessagingRoute>(&mut self) {
        let metadata = R::metadata();
        let dispatcher = Box::new(ConcreteRouteDispatcher::<R>::new());
        
        tracing::debug!(
            route_id = metadata.route_id,
            description = metadata.description,
            "Registering route handler"
        );
        
        self.routes.insert(metadata.route_id.to_string(), dispatcher);
    }
    
    /// Register all available routes.
    ///
    /// This method registers all route handlers that have been implemented
    /// and verified through the compile-time enforcement system.
    pub fn register_all_routes(&mut self) {
        // System routes (implemented)
        self.register_route::<crate::routes::health::HealthRoute>();
        self.register_route::<crate::routes::system::SystemRoute>();
        self.register_route::<crate::routes::stats::GetStatsRoute>();
        
        // AI/ML routes (implemented)
        self.register_route::<crate::routes::chat::ChatRoute>();
        self.register_route::<crate::routes::chat::ResponsesRoute>();
        
        self.register_route::<crate::routes::embeddings::EmbeddingsRoute>();
        self.register_route::<crate::routes::generate::GenerateRoute>();
        self.register_route::<crate::routes::models::ListModelsRoute>();
        self.register_route::<crate::routes::models::LoadModelRoute>();
        self.register_route::<crate::routes::models::UnloadModelRoute>();
        self.register_route::<crate::routes::models::ModelInfoRoute>();
        self.register_route::<crate::routes::rag::RagRoute>();
        self.register_route::<crate::routes::rerank::RerankRoute>();
        self.register_route::<crate::routes::sessions::GetHistoryRoute>();
        self.register_route::<crate::routes::sessions::SaveMessageRoute>();
        self.register_route::<crate::routes::params::GetParamsRoute>();
        self.register_route::<crate::routes::params::SetParamsRoute>();
        self.register_route::<crate::routes::generation::StopGenerationRoute>();
        self.register_route::<crate::routes::generation::GetHaltStatusRoute>();
        self.register_route::<crate::routes::resources::GetResourcesRoute>();
        self.register_route::<crate::routes::resources::EstimateMemoryRoute>();
        self.register_route::<crate::routes::resources::CompatibilityRoute>();
        self.register_route::<crate::routes::management::PullModelRoute>();
        self.register_route::<crate::routes::management::DeleteModelRoute>();
        self.register_route::<crate::routes::management::GetLoadedModelsRoute>();
        // self.register_route::<crate::routes::generate::GenerateRoute>();
        // ... etc
        
        tracing::info!(
            route_count = self.routes.len(),
            "Implemented routes registered successfully"
        );
    }
    
    /// Dispatch an incoming message to the appropriate route handler.
    ///
    /// # Arguments
    ///
    /// * `message` - Incoming message from Chrome extension
    ///
    /// # Returns
    ///
    /// * `Ok(OutgoingMessage)` - Successful response
    /// * `Err(NativeMessagingError)` - Processing error
    pub async fn dispatch(&self, message: IncomingMessage) -> NativeMessagingResult<OutgoingMessage> {
        let request_id = message.request_id.clone();
        let route = message.route.clone();
        
        tracing::info!(
            request_id = %request_id,
            route = %route,
            "Processing native messaging request"
        );
        
        // Find route handler
        let dispatcher = self.routes.get(&route)
            .ok_or_else(|| NativeMessagingError::route_not_found(&route))?;
        
        // Dispatch to handler
        match dispatcher.dispatch(message.payload, &self.state).await {
            Ok(response_data) => {
                tracing::info!(
                    request_id = %request_id,
                    route = %route,
                    "Request processed successfully"
                );
                
                Ok(OutgoingMessage::success(request_id, response_data))
            }
            Err(e) => {
                tracing::error!(
                    request_id = %request_id,
                    route = %route,
                    error = %e,
                    "Request processing failed"
                );
                
                let mut error_response: ErrorResponse = e.into();
                error_response.request_id = Some(request_id.clone());
                
                Ok(OutgoingMessage::error(request_id, error_response))
            }
        }
    }
    
    /// Get all registered routes metadata.
    pub fn get_routes(&self) -> Vec<crate::route_trait::RouteMetadata> {
        self.routes.values()
            .map(|dispatcher| dispatcher.metadata())
            .collect()
    }
    
    /// Get route count.
    pub fn route_count(&self) -> usize {
        self.routes.len()
    }
    
    /// Check if a route is registered.
    pub fn has_route(&self, route_id: &str) -> bool {
        self.routes.contains_key(route_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::AppStateProvider;
    use tabagent_values::{RequestValue, ResponseValue, HealthStatus};
    
    struct MockAppState;
    
    #[async_trait::async_trait]
    impl AppStateProvider for MockAppState {
        async fn handle_request(&self, request: RequestValue) -> anyhow::Result<ResponseValue> {
            match request.request_type() {
                tabagent_values::RequestType::Health => {
                    Ok(ResponseValue::health(HealthStatus::Healthy))
                }
                _ => Ok(ResponseValue::health(HealthStatus::Healthy)),
            }
        }
    }
    
    #[tokio::test]
    async fn test_router_creation() {
        let state = Arc::new(MockAppState);
        let router = MessageRouter::new(state);
        
        assert_eq!(router.route_count(), 0);
        assert!(!router.has_route("health"));
    }
    
    #[tokio::test]
    async fn test_route_registration() {
        let state = Arc::new(MockAppState);
        let mut router = MessageRouter::new(state);
        
        router.register_route::<crate::routes::health::HealthRoute>();
        
        assert_eq!(router.route_count(), 1);
        assert!(router.has_route("health"));
        
        let routes = router.get_routes();
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].route_id, "health");
    }
    
    #[tokio::test]
    async fn test_message_dispatch_success() {
        let state = Arc::new(MockAppState);
        let mut router = MessageRouter::new(state);
        router.register_route::<crate::routes::health::HealthRoute>();
        
        let message = IncomingMessage {
            route: "health".to_string(),
            request_id: "test-123".to_string(),
            payload: serde_json::json!({}),
        };
        
        let response = router.dispatch(message).await.unwrap();
        assert!(response.success);
        assert_eq!(response.request_id, "test-123");
        assert!(response.data.is_some());
        assert!(response.error.is_none());
    }
    
    #[tokio::test]
    async fn test_message_dispatch_route_not_found() {
        let state = Arc::new(MockAppState);
        let router = MessageRouter::new(state);
        
        let message = IncomingMessage {
            route: "nonexistent".to_string(),
            request_id: "test-456".to_string(),
            payload: serde_json::json!({}),
        };
        
        let response = router.dispatch(message).await.unwrap();
        assert!(!response.success);
        assert_eq!(response.request_id, "test-456");
        assert!(response.data.is_none());
        assert!(response.error.is_some());
        
        let error = response.error.unwrap();
        assert_eq!(error.code, "ROUTE_NOT_FOUND");
    }
    
    #[tokio::test]
    async fn test_all_routes_registration() {
        let state = Arc::new(MockAppState);
        let mut router = MessageRouter::new(state);
        
        router.register_all_routes();
        
        // Should have at least the implemented routes
        assert!(router.route_count() >= 5); // At least 5 routes expected
        
        // Check implemented routes
        assert!(router.has_route("health"));
        assert!(router.has_route("system"));
        assert!(router.has_route("stats"));
        assert!(router.has_route("chat"));
        assert!(router.has_route("responses"));
    }
}