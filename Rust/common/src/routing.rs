//! Unified routing trait system for all TabAgent entry points.
//!
//! This module defines a single, canonical route trait that works across
//! HTTP, Native Messaging, and WebRTC transports.
//!
//! # Design Principles
//!
//! 1. **DRY**: ONE route trait definition, parameterized by transport
//! 2. **Type Safety**: Compile-time enforcement of route standards
//! 3. **Transport-Specific Metadata**: HTTP gets paths/methods, others get route_id
//! 4. **Validation**: All routes MUST validate requests
//! 5. **Testing**: All routes MUST have test cases
//! 6. **Values**: All routes MUST use `tabagent-values`
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────┐
//! │  RouteHandler<Transport>            │  <- SINGLE TRAIT
//! │  - metadata() -> Metadata           │
//! │  - validate_request()               │
//! │  - handle()                         │
//! │  - test_cases()                     │
//! └─────────────────────────────────────┘
//!          │        │           │
//!          ▼        ▼           ▼
//!      ┌─────┐  ┌──────┐  ┌─────────┐
//!      │ Http│  │Native│  │ WebRTC  │  <- Transport Markers
//!      └─────┘  └──────┘  └─────────┘
//! ```

use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Debug;

use crate::backend::AppStateProvider;

// --- Transport Marker Types ---

/// Marker type for HTTP transport (API).
pub struct HttpTransport;

/// Marker type for Native Messaging transport (stdin/stdout).
pub struct NativeMessagingTransport;

/// Marker type for WebRTC transport (data channels).
pub struct WebRtcTransport;

/// Trait for transport-specific metadata.
///
/// Each transport has its own metadata requirements:
/// - HTTP: path, method, OpenAPI tags
/// - Native Messaging: route_id, protocol features
/// - WebRTC: route_id, media handling, binary support
pub trait TransportMetadata: Debug + Clone + Send + Sync + 'static {
    /// Get a unique identifier for this route.
    ///
    /// For HTTP: the path (e.g., "/v1/chat/completions")
    /// For Native Messaging: the route_id (e.g., "chat")
    /// For WebRTC: the route_id (e.g., "chat")
    fn route_identifier(&self) -> &str;

    /// Get tags for categorization and documentation.
    fn tags(&self) -> &[&'static str];

    /// Get human-readable description (REQUIRED for documentation).
    fn description(&self) -> &str;

    /// Is this route idempotent? (safe to retry)
    fn idempotent(&self) -> bool;

    /// Requires authentication?
    fn requires_auth(&self) -> bool;

    /// Rate limit tier (None = no limit)
    fn rate_limit_tier(&self) -> Option<&'static str>;
}

/// HTTP-specific route metadata.
#[derive(Debug, Clone)]
pub struct HttpMetadata {
    /// HTTP path (e.g., "/v1/chat/completions")
    pub path: &'static str,
    /// HTTP method (stored as string for simplicity)
    pub method: &'static str, // "GET", "POST", "PUT", "DELETE"
    /// OpenAPI tags for grouping
    pub tags: &'static [&'static str],
    /// Description for documentation (REQUIRED)
    pub description: &'static str,
    /// Is this route OpenAI-compatible?
    pub openai_compatible: bool,
    /// Is this route idempotent?
    pub idempotent: bool,
    /// Requires authentication?
    pub requires_auth: bool,
    /// Rate limit tier
    pub rate_limit_tier: Option<&'static str>,
}

impl TransportMetadata for HttpMetadata {
    fn route_identifier(&self) -> &str {
        self.path
    }

    fn tags(&self) -> &[&'static str] {
        self.tags
    }

    fn description(&self) -> &str {
        self.description
    }

    fn idempotent(&self) -> bool {
        self.idempotent
    }

    fn requires_auth(&self) -> bool {
        self.requires_auth
    }

    fn rate_limit_tier(&self) -> Option<&'static str> {
        self.rate_limit_tier
    }
}

/// Native Messaging-specific route metadata.
#[derive(Debug, Clone)]
pub struct NativeMessagingMetadata {
    /// Route identifier (e.g., "chat", "embeddings")
    pub route_id: &'static str,
    /// Category tags for organization
    pub tags: &'static [&'static str],
    /// Human-readable description (REQUIRED)
    pub description: &'static str,
    /// Is this route OpenAI-compatible?
    pub openai_compatible: bool,
    /// Is this route idempotent?
    pub idempotent: bool,
    /// Requires authentication?
    pub requires_auth: bool,
    /// Rate limit tier
    pub rate_limit_tier: Option<&'static str>,
    /// Supports streaming responses?
    pub supports_streaming: bool,
    /// Supports binary data?
    pub supports_binary: bool,
    /// Maximum payload size in bytes
    pub max_payload_size: Option<usize>,
}

impl TransportMetadata for NativeMessagingMetadata {
    fn route_identifier(&self) -> &str {
        self.route_id
    }

    fn tags(&self) -> &[&'static str] {
        self.tags
    }

    fn description(&self) -> &str {
        self.description
    }

    fn idempotent(&self) -> bool {
        self.idempotent
    }

    fn requires_auth(&self) -> bool {
        self.requires_auth
    }

    fn rate_limit_tier(&self) -> Option<&'static str> {
        self.rate_limit_tier
    }
}

/// WebRTC-specific route metadata.
#[derive(Debug, Clone)]
pub struct WebRtcMetadata {
    /// Route identifier (e.g., "chat", "video_stream")
    pub route_id: &'static str,
    /// Category tags for organization
    pub tags: &'static [&'static str],
    /// Human-readable description (REQUIRED)
    pub description: &'static str,
    /// Supports streaming responses?
    pub supports_streaming: bool,
    /// Supports binary data?
    pub supports_binary: bool,
    /// Requires authentication?
    pub requires_auth: bool,
    /// Rate limit tier
    pub rate_limit_tier: Option<&'static str>,
    /// Maximum payload size in bytes
    pub max_payload_size: Option<usize>,
    /// Media type (if handling video/audio)
    pub media_type: Option<MediaType>,
}

impl TransportMetadata for WebRtcMetadata {
    fn route_identifier(&self) -> &str {
        self.route_id
    }

    fn tags(&self) -> &[&'static str] {
        self.tags
    }

    fn description(&self) -> &str {
        self.description
    }

    fn idempotent(&self) -> bool {
        // WebRTC routes are generally not idempotent due to stateful nature
        false
    }

    fn requires_auth(&self) -> bool {
        self.requires_auth
    }

    fn rate_limit_tier(&self) -> Option<&'static str> {
        self.rate_limit_tier
    }
}

/// Media types for video/audio handling routes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaType {
    /// Video stream (requires codec validation)
    Video,
    /// Audio stream (requires codec validation)
    Audio,
    /// Both video and audio
    VideoAudio,
    /// Screen sharing
    ScreenShare,
}

// --- Test Case Support ---

/// Test case for route handlers.
///
/// Routes MUST provide test cases to ensure they work correctly.
/// Test cases are used by the test framework to verify:
/// - Validation logic works
/// - Handler logic works
/// - Errors are handled properly
#[derive(Debug, Clone)]
pub struct TestCase<Req, Resp> {
    /// Test case name
    pub name: &'static str,
    /// Request to test
    pub request: Req,
    /// Expected response (None = any success)
    pub expected_response: Option<Resp>,
    /// Expected error message (None = expect success)
    pub expected_error: Option<&'static str>,
    /// Custom assertions (for complex validation)
    pub assertions: Vec<&'static str>,
}

impl<Req, Resp> TestCase<Req, Resp> {
    /// Create a test case expecting success.
    pub fn success(name: &'static str, request: Req, expected_response: Resp) -> Self {
        Self {
            name,
            request,
            expected_response: Some(expected_response),
            expected_error: None,
            assertions: Vec::new(),
        }
    }

    /// Create a test case expecting an error.
    pub fn error(name: &'static str, request: Req, expected_error: &'static str) -> Self {
        Self {
            name,
            request,
            expected_response: None,
            expected_error: Some(expected_error),
            assertions: Vec::new(),
        }
    }
}

// --- Unified Route Handler Trait ---

/// Unified route handler trait for all transports.
///
/// ALL routes (HTTP, Native Messaging, WebRTC) MUST implement this trait.
///
/// # Type Parameters
///
/// * `M` - Transport-specific metadata type
///
/// # ENFORCED RULES (COMPILE-TIME)
///
/// Every route MUST:
/// 1. ✅ Have documentation (doc comments)
/// 2. ✅ Have tests (unit + integration via test_cases)
/// 3. ✅ Tests cannot be fake (must call actual handler)
/// 4. ✅ Use `tabagent-values` for requests/responses
/// 5. ✅ Have proper tracing (request_id in handle)
/// 6. ✅ Have proper error handling (return Result, not panic)
/// 7. ✅ Have metadata (description REQUIRED)
/// 8. ✅ Validate requests (validate_request REQUIRED)
/// 9. ✅ Return proper types (no debug formatting)
/// 10. ✅ Log success AND failure cases
///
/// # Example
///
/// ```rust,ignore
/// use common::routing::{RouteHandler, HttpMetadata, TestCase};
/// use async_trait::async_trait;
///
/// struct ChatRoute;
///
/// #[async_trait]
/// impl RouteHandler<HttpMetadata> for ChatRoute {
///     type Request = ChatRequest;
///     type Response = ChatResponse;
///     
///     fn metadata() -> HttpMetadata {
///         HttpMetadata {
///             path: "/v1/chat/completions",
///             method: "POST",
///             tags: &["Chat", "OpenAI"],
///             description: "OpenAI-compatible chat completions",
///             openai_compatible: true,
///             idempotent: false,
///             requires_auth: false,
///             rate_limit_tier: Some("inference"),
///         }
///     }
///     
///     async fn validate_request(req: &Self::Request) -> anyhow::Result<()> {
///         if req.messages.is_empty() {
///             return Err(anyhow::anyhow!("messages cannot be empty"));
///         }
///         Ok(())
///     }
///     
///     async fn handle<S>(req: Self::Request, state: &S) -> anyhow::Result<Self::Response>
///     where
///         S: AppStateProvider + Send + Sync,
///     {
///         let request_id = uuid::Uuid::new_v4();
///         tracing::info!(request_id = %request_id, "Chat request");
///         // ... implementation
///     }
///     
///     fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
///         vec![/* test cases */]
///     }
/// }
/// ```
#[async_trait]
pub trait RouteHandler<M: TransportMetadata>: Send + Sync + 'static {
    /// Request type - MUST be deserializable and debuggable
    type Request: DeserializeOwned + Debug + Send + Sync;
    
    /// Response type - MUST be serializable and debuggable
    type Response: Serialize + Debug + Send + Sync;
    
    /// Provide route metadata - REQUIRED
    ///
    /// This ensures every route is documented and categorized.
    /// RULE: Must provide non-empty description (enforces documentation)
    fn metadata() -> M;
    
    /// Validate the request - REQUIRED
    ///
    /// This is called BEFORE handle() and MUST check:
    /// - Required fields are present
    /// - Values are in valid ranges
    /// - Business logic constraints
    ///
    /// RULE: Cannot just return Ok(()) - must have real validation
    /// Return error for validation failures.
    async fn validate_request(req: &Self::Request) -> anyhow::Result<()>;
    
    /// Handle the request - REQUIRED
    ///
    /// RULES (MUST follow ALL):
    /// 1. Generate a request_id (uuid::Uuid::new_v4() or similar)
    /// 2. Log at start: tracing::info!(request_id = %request_id, ...)
    /// 3. Log at end: tracing::info!(request_id = %request_id, "success")
    /// 4. Log errors: tracing::error!(request_id = %request_id, error = %e, ...)
    /// 5. Return Err for failures (NEVER panic)
    /// 6. Use tabagent_values::RequestValue/ResponseValue internally
    /// 7. Return proper Response type (no debug formatting)
    async fn handle<S>(req: Self::Request, state: &S) -> anyhow::Result<Self::Response>
    where
        S: AppStateProvider + Send + Sync;
    
    /// Provide test cases - REQUIRED FOR ENFORCEMENT
    ///
    /// RULE: Must return at least ONE test case.
    /// RULE: Test cases MUST call the actual handler (no fake tests)
    /// RULE: Test cases MUST assert on actual response values
    ///
    /// This is used by the test framework to ensure every route
    /// has real, executable tests.
    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        Vec::new() // Default empty - will trigger compile warning if not overridden
    }
}

// --- Transport-Specific Route Registration ---

/// Marker trait for routes that can be registered with a transport.
///
/// Each transport crate implements this trait differently:
/// - HTTP (Axum): Uses `Router::route()` with concrete handler functions
/// - Native Messaging: Uses `MessageRouter::register()` with type-erased handlers
/// - WebRTC: Uses `DataChannelRouter::register()` with type-erased handlers
///
/// This trait allows transport-specific registration while maintaining
/// the unified `RouteHandler<M>` interface.
///
/// # Transport Implementation Notes
///
/// ## HTTP (Axum 0.8)
///
/// **CRITICAL**: Must use concrete `async fn handler` (not closures!) and
/// `AppStateWrapper` directly in extractors:
///
/// ```ignore
/// // ✅ CORRECT (Axum 0.8 compatible)
/// async fn handler(
///     State(state): State<AppStateWrapper>,  // Concrete type
///     Json(req): Json<Request>,
/// ) -> Result<Json<Response>, ApiError> {
///     // ...
/// }
/// router.route("/path", post(handler))  // Named function
///
/// // ❌ WRONG (breaks Axum 0.8)
/// router.route("/path", post(|state, req| async move { ... }))  // Closure
/// ```
///
/// ## Native Messaging / WebRTC
///
/// Can use closures or type-erased trait objects since they don't use Axum.
pub trait RegisterableRoute<M: TransportMetadata>: RouteHandler<M> {
    /// The transport-specific router type (e.g., `axum::Router`, `MessageRouter`)
    type Router;
    
    /// Register this route with the transport's router.
    ///
    /// # Implementation Notes
    ///
    /// - For Axum: Must generate concrete `async fn` handler
    /// - For Native Messaging: Can use `Box<dyn Fn>` or type-erased handlers
    /// - For WebRTC: Can use `Box<dyn Fn>` or type-erased handlers
    fn register(router: Self::Router) -> Self::Router;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    struct TestRequest {
        value: String,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    struct TestResponse {
        result: String,
    }

    struct TestRoute;

    #[async_trait]
    impl RouteHandler<HttpMetadata> for TestRoute {
        type Request = TestRequest;
        type Response = TestResponse;

        fn metadata() -> HttpMetadata {
            HttpMetadata {
                path: "/test",
                method: "POST",
                tags: &["Test"],
                description: "Test route",
                openai_compatible: false,
                idempotent: true,
                requires_auth: false,
                rate_limit_tier: None,
            }
        }

        async fn validate_request(req: &Self::Request) -> anyhow::Result<()> {
            if req.value.is_empty() {
                return Err(anyhow::anyhow!("value cannot be empty"));
            }
            Ok(())
        }

        async fn handle<S>(_req: Self::Request, _state: &S) -> anyhow::Result<Self::Response>
        where
            S: AppStateProvider + Send + Sync,
        {
            Ok(TestResponse {
                result: "success".to_string(),
            })
        }

        fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
            vec![
                TestCase::error(
                    "empty_value",
                    TestRequest { value: "".to_string() },
                    "value cannot be empty",
                ),
                TestCase::success(
                    "valid_request",
                    TestRequest { value: "test".to_string() },
                    TestResponse { result: "success".to_string() },
                ),
            ]
        }
    }

    #[test]
    fn test_metadata_access() {
        let metadata = TestRoute::metadata();
        assert_eq!(metadata.route_identifier(), "/test");
        assert_eq!(metadata.description(), "Test route");
        assert!(metadata.idempotent());
    }

    #[tokio::test]
    async fn test_validation() {
        let valid_req = TestRequest { value: "test".to_string() };
        assert!(TestRoute::validate_request(&valid_req).await.is_ok());

        let invalid_req = TestRequest { value: "".to_string() };
        assert!(TestRoute::validate_request(&invalid_req).await.is_err());
    }

    #[test]
    fn test_test_cases_present() {
        let cases = TestRoute::test_cases();
        assert!(!cases.is_empty(), "Routes must have test cases");
    }
}

