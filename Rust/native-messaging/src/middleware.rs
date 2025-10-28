//! Middleware for request/response processing.
//!
//! This module provides middleware components for cross-cutting concerns
//! like logging, authentication, rate limiting, and error handling.

use crate::{
    error::{NativeMessagingError, NativeMessagingResult},
    traits::{Middleware, RequestHandler},
    config::NativeMessagingConfig,
};
use tabagent_values::{RequestValue, ResponseValue};
use async_trait::async_trait;
use std::{collections::HashMap, sync::Arc, time::{Duration, Instant}};
use tokio::sync::RwLock;

/// Logging middleware that adds request tracing.
pub struct LoggingMiddleware {
    enabled: bool,
}

impl LoggingMiddleware {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }
}

#[async_trait]
impl Middleware for LoggingMiddleware {
    async fn process(
        &self,
        request: RequestValue,
        handler: &dyn RequestHandler,
    ) -> anyhow::Result<ResponseValue> {
        if !self.enabled {
            return handler.handle(request).await;
        }
        
        let request_id = uuid::Uuid::new_v4();
        let start_time = Instant::now();
        
        tracing::info!(
            request_id = %request_id,
            request_type = ?request.request_type(),
            "Processing request"
        );
        
        let result = handler.handle(request).await;
        let duration = start_time.elapsed();
        
        match &result {
            Ok(_) => {
                tracing::info!(
                    request_id = %request_id,
                    duration_ms = duration.as_millis(),
                    "Request completed successfully"
                );
            }
            Err(e) => {
                tracing::error!(
                    request_id = %request_id,
                    duration_ms = duration.as_millis(),
                    error = %e,
                    "Request failed"
                );
            }
        }
        
        result
    }
}

/// Rate limiting middleware.
pub struct RateLimitMiddleware {
    config: NativeMessagingConfig,
    request_counts: Arc<RwLock<HashMap<String, (Instant, u32)>>>,
}

impl RateLimitMiddleware {
    pub fn new(config: NativeMessagingConfig) -> Self {
        Self {
            config,
            request_counts: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    async fn check_rate_limit(&self, client_id: &str, tier: &str) -> NativeMessagingResult<()> {
        if !self.config.rate_limiting.enabled {
            return Ok(());
        }
        
        let limit = match tier {
            "standard" => self.config.rate_limiting.standard_rpm,
            "inference" => self.config.rate_limiting.inference_rpm,
            "premium" => self.config.rate_limiting.premium_rpm,
            _ => self.config.rate_limiting.standard_rpm,
        };
        
        let mut counts = self.request_counts.write().await;
        let now = Instant::now();
        
        // Clean up old entries (older than 1 minute)
        counts.retain(|_, (timestamp, _)| now.duration_since(*timestamp) < Duration::from_secs(60));
        
        // Check current count for this client
        let (last_reset, count) = counts.entry(client_id.to_string())
            .or_insert((now, 0));
        
        // Reset count if more than 1 minute has passed
        if now.duration_since(*last_reset) >= Duration::from_secs(60) {
            *last_reset = now;
            *count = 0;
        }
        
        // Check if limit exceeded
        if *count >= limit {
            let retry_after = 60 - now.duration_since(*last_reset).as_secs();
            return Err(NativeMessagingError::rate_limit_exceeded(
                format!("Rate limit exceeded for tier '{}'. Limit: {} requests per minute", tier, limit),
                Some(retry_after),
            ));
        }
        
        // Increment count
        *count += 1;
        
        Ok(())
    }
}

#[async_trait]
impl Middleware for RateLimitMiddleware {
    async fn process(
        &self,
        request: RequestValue,
        handler: &dyn RequestHandler,
    ) -> anyhow::Result<ResponseValue> {
        // Extract client ID (for now, use a default - in real implementation,
        // this would come from Chrome extension origin validation)
        let client_id = "default_client";
        
        // Determine rate limit tier based on request type
        let tier = match request.request_type() {
            tabagent_values::RequestType::Chat { .. } 
            | tabagent_values::RequestType::Generate { .. } 
            | tabagent_values::RequestType::Embeddings { .. } => "inference",
            _ => "standard",
        };
        
        // Check rate limit
        self.check_rate_limit(client_id, tier).await?;
        
        // Process request
        handler.handle(request).await
    }
}

/// Authentication middleware.
pub struct AuthMiddleware {
    enabled: bool,
    allowed_origins: Vec<String>,
}

impl AuthMiddleware {
    pub fn new(enabled: bool, allowed_origins: Vec<String>) -> Self {
        Self {
            enabled,
            allowed_origins,
        }
    }
    
    fn validate_origin(&self, _origin: &str) -> NativeMessagingResult<()> {
        if !self.enabled {
            return Ok(());
        }
        
        // TODO: Implement actual Chrome extension origin validation
        // For now, allow all origins
        Ok(())
    }
}

#[async_trait]
impl Middleware for AuthMiddleware {
    async fn process(
        &self,
        request: RequestValue,
        handler: &dyn RequestHandler,
    ) -> anyhow::Result<ResponseValue> {
        if self.enabled {
            // TODO: Extract origin from request context
            let origin = "chrome-extension://unknown";
            self.validate_origin(origin)?;
        }
        
        handler.handle(request).await
    }
}

/// Error handling middleware that converts errors to proper responses.
pub struct ErrorHandlingMiddleware;

impl ErrorHandlingMiddleware {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Middleware for ErrorHandlingMiddleware {
    async fn process(
        &self,
        request: RequestValue,
        handler: &dyn RequestHandler,
    ) -> anyhow::Result<ResponseValue> {
        match handler.handle(request).await {
            Ok(response) => Ok(response),
            Err(e) => {
                // Convert error to appropriate response
                // For now, just propagate the error
                Err(e)
            }
        }
    }
}

/// Middleware chain builder.
pub struct MiddlewareChain {
    middlewares: Vec<Box<dyn Middleware>>,
}

impl MiddlewareChain {
    pub fn new() -> Self {
        Self {
            middlewares: Vec::new(),
        }
    }
    
    pub fn add<M: Middleware + 'static>(mut self, middleware: M) -> Self {
        self.middlewares.push(Box::new(middleware));
        self
    }
    
    pub fn build(self) -> Vec<Box<dyn Middleware>> {
        self.middlewares
    }
}

impl Default for MiddlewareChain {
    fn default() -> Self {
        Self::new()
    }
}

/// Create default middleware chain.
pub fn create_default_middleware(config: &NativeMessagingConfig) -> Vec<Box<dyn Middleware>> {
    MiddlewareChain::new()
        .add(LoggingMiddleware::new(config.enable_logging))
        .add(RateLimitMiddleware::new(config.clone()))
        .add(AuthMiddleware::new(
            config.security.enable_auth,
            config.security.allowed_extension_ids.clone(),
        ))
        .add(ErrorHandlingMiddleware::new())
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tabagent_values::{HealthStatus, TokenUsage};
    
    struct MockHandler;
    
    #[async_trait]
    impl RequestHandler for MockHandler {
        async fn handle(
            &self,
            _request: RequestValue,
        ) -> anyhow::Result<ResponseValue> {
            // Return a simple health response for all requests in tests
            Ok(ResponseValue::health(HealthStatus::Healthy))
        }
    }
    
    #[tokio::test]
    async fn test_logging_middleware() {
        let middleware = LoggingMiddleware::new(true);
        let handler = MockHandler;
        let request = RequestValue::system_info();
        
        let result = middleware.process(request, &handler).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_rate_limit_middleware() {
        let config = NativeMessagingConfig::default();
        let middleware = RateLimitMiddleware::new(config);
        let handler = MockHandler;
        let request = RequestValue::system_info();
        
        // First request should succeed
        let result = middleware.process(request.clone(), &handler).await;
        assert!(result.is_ok());
        
        // Many requests in quick succession should eventually hit rate limit
        // (This test is simplified - in practice, we'd need to test with actual limits)
    }
    
    #[tokio::test]
    async fn test_auth_middleware() {
        let middleware = AuthMiddleware::new(false, vec![]);
        let handler = MockHandler;
        let request = RequestValue::system_info();
        
        let result = middleware.process(request, &handler).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_middleware_chain() {
        let config = NativeMessagingConfig::default();
        let middlewares = create_default_middleware(&config);
        
        assert_eq!(middlewares.len(), 4); // Logging, RateLimit, Auth, ErrorHandling
    }
}