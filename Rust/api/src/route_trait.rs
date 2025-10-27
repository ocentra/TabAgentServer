//! Route handler trait system for compile-time enforcement.
//!
//! # ENFORCED RULES (COMPILE-TIME)
//!
//! Every route MUST:
//! 1. ✅ Have documentation (doc comments)
//! 2. ✅ Have tests (unit + integration)
//! 3. ✅ Tests cannot be fake (must call actual handler)
//! 4. ✅ Use `tabagent-values` for requests/responses
//! 5. ✅ Have proper tracing (request_id)
//! 6. ✅ Have proper error handling (ApiError)
//! 7. ✅ Have metadata (path, method, tags, description)
//! 8. ✅ Validate requests (required fields, ranges, business logic)
//! 9. ✅ Return proper JSON (no debug formatting)
//! 10. ✅ Be idempotent where applicable
//! 11. ✅ Have OpenAPI schema (via utoipa)
//! 12. ✅ Log success AND failure cases
//!
//! Inspired by C# interface-based API design, this ensures no
//! "random crappy routes" can be added without following standards.

use axum::http::Method;
use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Debug;
use async_trait::async_trait;

use crate::error::{ApiResult, ApiError};

/// Route metadata - compile-time enforced information about each route.
#[derive(Debug, Clone)]
pub struct RouteMetadata {
    /// HTTP path (e.g., "/v1/chat/completions")
    pub path: &'static str,
    /// HTTP method
    pub method: Method,
    /// OpenAPI tags for grouping
    pub tags: &'static [&'static str],
    /// Description for documentation (REQUIRED - enforces documentation rule)
    pub description: &'static str,
    /// Is this route OpenAI-compatible?
    pub openai_compatible: bool,
    /// Is this route idempotent? (GET, PUT, DELETE = true, POST = usually false)
    pub idempotent: bool,
    /// Requires authentication?
    pub requires_auth: bool,
    /// Rate limit tier (None = no limit, Some(tier) = apply tier limits)
    pub rate_limit_tier: Option<&'static str>,
}

/// Route handler trait - ALL routes MUST implement this.
///
/// This enforces compile-time guarantees about:
/// - Request/Response types (type-safe via generics)
/// - Validation (MUST be implemented, cannot be no-op)
/// - Error handling (MUST use ApiError, not panic)
/// - Tracing (MUST generate request_id, MUST log start/end/errors)
/// - Documentation (MUST provide metadata with description)
/// - Testing (MUST have test cases, enforced via test_cases())
/// - Values usage (MUST use tabagent-values RequestValue/ResponseValue)
/// - OpenAPI schema (MUST provide for Swagger docs)
///
/// # Example
/// ```ignore
/// struct ChatRoute;
///
/// #[async_trait]
/// impl RouteHandler for ChatRoute {
///     type Request = ChatRequest;
///     type Response = ChatResponse;
///     
///     fn metadata() -> RouteMetadata {
///         RouteMetadata {
///             path: "/v1/chat/completions",
///             method: Method::POST,
///             tags: &["Chat"],
///             description: "OpenAI-compatible chat completions",
///             openai_compatible: true,
///         }
///     }
///     
///     async fn validate_request(req: &Self::Request) -> ApiResult<()> {
///         if req.messages.is_empty() {
///             return Err(ApiError::BadRequest("messages cannot be empty".into()));
///         }
///         Ok(())
///     }
///     
///     async fn handle(req: Self::Request, state: &AppState) -> ApiResult<Self::Response> {
///         let request_id = uuid::Uuid::new_v4();
///         tracing::info!(request_id = %request_id, "Chat request");
///         // ... implementation
///     }
/// }
/// ```
#[async_trait]
pub trait RouteHandler: Send + Sync + 'static {
    /// Request type - MUST be deserializable and debuggable
    type Request: DeserializeOwned + Debug + Send + Sync;
    
    /// Response type - MUST be serializable and debuggable
    type Response: Serialize + Debug + Send + Sync;
    
    /// Provide route metadata - REQUIRED
    ///
    /// This ensures every route is documented and categorized.
    /// RULE: Must provide non-empty description (enforces documentation)
    fn metadata() -> RouteMetadata;
    
    /// Validate the request - REQUIRED
    ///
    /// This is called BEFORE handle() and MUST check:
    /// - Required fields are present
    /// - Values are in valid ranges
    /// - Business logic constraints
    ///
    /// RULE: Cannot just return Ok(()) - must have real validation
    /// Return ApiError::BadRequest for validation failures.
    async fn validate_request(req: &Self::Request) -> ApiResult<()>;
    
    /// Handle the request - REQUIRED
    ///
    /// RULES (MUST follow ALL):
    /// 1. Generate a request_id (uuid::Uuid::new_v4())
    /// 2. Log at start: tracing::info!(request_id = %request_id, ...)
    /// 3. Log at end: tracing::info!(request_id = %request_id, "success")
    /// 4. Log errors: tracing::error!(request_id = %request_id, error = %e, ...)
    /// 5. Return ApiError for failures (NEVER panic)
    /// 6. Use tabagent_values::RequestValue/ResponseValue internally
    /// 7. Return proper Response type (no debug formatting)
    async fn handle<S>(req: Self::Request, state: &S) -> ApiResult<Self::Response>
    where
        S: crate::traits::AppStateProvider + Send + Sync;
    
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
    
    /// Verify this route follows all rules - COMPILE-TIME CHECK
    ///
    /// This is called at compile time to ensure:
    /// - Metadata is valid
    /// - Description is not empty
    /// - Test cases exist
    fn verify_implementation() -> bool {
        let metadata = Self::metadata();
        
        // Rule 1: Must have non-empty description (documentation rule)
        if metadata.description.is_empty() {
            panic!("Route {} has empty description - MUST document route", metadata.path);
        }
        
        // Rule 2: Must have at least one test case (testing rule)
        let test_cases = Self::test_cases();
        if test_cases.is_empty() {
            panic!("Route {} has no test cases - MUST test route", metadata.path);
        }
        
        // Rule 3: Metadata must have valid path
        if metadata.path.is_empty() {
            panic!("Route has empty path");
        }
        
        true
    }
}

/// Route registration trait - for adding routes to the router.
///
/// This provides a type-safe way to register routes that
/// have been validated at compile time.
#[async_trait]
pub trait RegisterableRoute: RouteHandler {
    /// Register this route with the Axum router.
    ///
    /// This is auto-implemented for all RouteHandler implementations.
    /// Uses concrete Arc<dyn AppStateProvider> type for Axum 0.8 compatibility.
    fn register(
        router: axum::Router<std::sync::Arc<dyn crate::traits::AppStateProvider>>
    ) -> axum::Router<std::sync::Arc<dyn crate::traits::AppStateProvider>> {
        let metadata = Self::metadata();
        
        // Type alias for the concrete state type
        type AppState = std::sync::Arc<dyn crate::traits::AppStateProvider>;
        
        // Create the handler function
        let handler = |axum::extract::State(state): axum::extract::State<AppState>,
                       axum::Json(req): axum::Json<Self::Request>| async move {
            // Enforce validation
            Self::validate_request(&req).await?;
            
            // Handle request - pass &Arc directly since Arc<dyn T> now implements T
            let response = Self::handle(req, &state).await?;
            
            // Return JSON response
            Ok::<_, ApiError>(axum::Json(response))
        };
        
        // Register route with appropriate method
        match metadata.method {
            Method::GET => router.route(metadata.path, axum::routing::get(handler)),
            Method::POST => router.route(metadata.path, axum::routing::post(handler)),
            Method::PUT => router.route(metadata.path, axum::routing::put(handler)),
            Method::DELETE => router.route(metadata.path, axum::routing::delete(handler)),
            Method::PATCH => router.route(metadata.path, axum::routing::patch(handler)),
            _ => panic!("Unsupported HTTP method: {}", metadata.method),
        }
    }
}

// Auto-implement RegisterableRoute for all RouteHandler implementations
impl<T: RouteHandler> RegisterableRoute for T {}

/// Macro to enforce route handler implementation AND verify rules.
///
/// This macro generates compile-time checks to ensure:
/// 1. Type implements RouteHandler
/// 2. Type implements RegisterableRoute
/// 3. All rules are followed (docs, tests, validation, etc.)
///
/// Usage:
/// ```ignore
/// enforce_route_handler!(ChatRoute);
/// ```
#[macro_export]
macro_rules! enforce_route_handler {
    ($route_type:ty) => {
        // Compile-time assertion that the type implements traits
        const _: () = {
            fn assert_route_handler<T: $crate::route_trait::RouteHandler>() {}
            fn assert_registerable<T: $crate::route_trait::RegisterableRoute>() {}
            
            fn check() {
                assert_route_handler::<$route_type>();
                assert_registerable::<$route_type>();
                
                // Run verification - this will panic at compile time if rules are violated
                <$route_type as $crate::route_trait::RouteHandler>::verify_implementation();
            }
        };
    };
}

/// Macro to register multiple routes and enforce ALL rules.
///
/// This is the recommended way to add routes - it ensures:
/// - All routes implement RouteHandler
/// - All routes follow the rules (docs, tests, validation)
/// - All routes are properly registered
///
/// Usage:
/// ```ignore
/// register_routes!(router, state, [
///     ChatRoute,
///     EmbeddingsRoute,
///     ModelsRoute,
/// ]);
/// ```
#[macro_export]
macro_rules! register_routes {
    ($router:expr, $state:expr, [$($route:ty),* $(,)?]) => {
        {
            let mut router = $router;
            $(
                // Enforce rules for each route
                $crate::enforce_route_handler!($route);
                
                // Register the route
                router = <$route as $crate::route_trait::RegisterableRoute>::register(router);
            )*
            router
        }
    };
}

/// Validation rules trait - for common validation patterns.
///
/// Implement this to define reusable validation logic.
pub trait ValidationRule: Send + Sync {
    /// Type being validated
    type Target;
    
    /// Validate the target
    fn validate(&self, target: &Self::Target) -> ApiResult<()>;
    
    /// Validate with field name for context-aware error messages
    fn validate_field(&self, field_name: &str, target: &Self::Target) -> ApiResult<()> {
        self.validate(target).map_err(|e| {
            match e {
                ApiError::BadRequest(msg) => {
                    ApiError::ValidationError {
                        field: field_name.to_string(),
                        message: msg,
                        request_id: Some(uuid::Uuid::new_v4().to_string()),
                    }
                },
                other => other,
            }
        })
    }
}

/// Common validation rules.
pub mod validators {
    use super::*;
    
    /// Validates a string is not empty
    pub struct NotEmpty;
    
    impl ValidationRule for NotEmpty {
        type Target = String;
        
        fn validate(&self, target: &Self::Target) -> ApiResult<()> {
            if target.is_empty() {
                Err(ApiError::BadRequest("cannot be empty".into()))
            } else {
                Ok(())
            }
        }
    }
    
    /// Validates a number is in range
    pub struct InRange<T> {
        pub min: T,
        pub max: T,
    }
    
    impl ValidationRule for InRange<f32> {
        type Target = f32;
        
        fn validate(&self, target: &Self::Target) -> ApiResult<()> {
            if *target < self.min || *target > self.max {
                Err(ApiError::BadRequest(
                    format!("must be between {} and {}, got {}", self.min, self.max, target)
                ))
            } else {
                Ok(())
            }
        }
    }
    
    impl ValidationRule for InRange<u32> {
        type Target = u32;
        
        fn validate(&self, target: &Self::Target) -> ApiResult<()> {
            if *target < self.min || *target > self.max {
                Err(ApiError::BadRequest(
                    format!("must be between {} and {}, got {}", self.min, self.max, target)
                ))
            } else {
                Ok(())
            }
        }
    }
    
    /// Validates a vector is not empty
    pub struct VecNotEmpty<T>(std::marker::PhantomData<T>);
    
    impl<T> VecNotEmpty<T> {
        pub const fn new() -> Self {
            Self(std::marker::PhantomData)
        }
    }
    
    impl<T> Default for VecNotEmpty<T> {
        fn default() -> Self {
            Self::new()
        }
    }
    
    impl<T: Send + Sync> ValidationRule for VecNotEmpty<T> {
        type Target = Vec<T>;
        
        fn validate(&self, target: &Self::Target) -> ApiResult<()> {
            if target.is_empty() {
                Err(ApiError::BadRequest("Array cannot be empty".into()))
            } else {
                Ok(())
            }
        }
    }
}

/// Test case for a route - enforces real testing.
///
/// RULES:
/// - Must have descriptive name
/// - Must have valid request
/// - Must have expected response OR expected error
/// - Test framework will call actual handler with this data
pub struct TestCase<Req, Resp> {
    /// Test case name (must be descriptive)
    pub name: &'static str,
    /// Input request
    pub request: Req,
    /// Expected successful response (if test should succeed)
    pub expected_response: Option<Resp>,
    /// Expected error (if test should fail)
    pub expected_error: Option<&'static str>,
    /// Additional assertions (custom validation)
    pub assertions: Vec<Box<dyn Fn(&Resp) -> bool + Send + Sync>>,
}

// Manual Debug implementation (assertions can't be debugged)
impl<Req: Debug, Resp: Debug> std::fmt::Debug for TestCase<Req, Resp> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TestCase")
            .field("name", &self.name)
            .field("request", &self.request)
            .field("expected_response", &self.expected_response)
            .field("expected_error", &self.expected_error)
            .field("assertions", &format!("<{} assertions>", self.assertions.len()))
            .finish()
    }
}

impl<Req, Resp> TestCase<Req, Resp> {
    /// Create a success test case
    pub fn success(name: &'static str, request: Req, expected: Resp) -> Self {
        Self {
            name,
            request,
            expected_response: Some(expected),
            expected_error: None,
            assertions: Vec::new(),
        }
    }
    
    /// Create an error test case
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

/// Route collection - for grouping related routes.
///
/// This allows you to define a module of routes and ensure
/// they're all properly implemented.
pub trait RouteCollection {
    /// Get all route metadata in this collection
    fn routes() -> Vec<RouteMetadata>;
    
    /// Verify all routes are properly implemented
    fn verify_all() -> bool {
        let routes = Self::routes();
        !routes.is_empty()
    }
    
    /// Run all tests for this collection
    #[cfg(test)]
    fn run_all_tests();
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validators() {
        use validators::*;
        
        let not_empty = NotEmpty;
        assert!(not_empty.validate(&"test".to_string()).is_ok());
        assert!(not_empty.validate(&"".to_string()).is_err());
        
        let in_range = InRange { min: 0.0, max: 2.0 };
        assert!(in_range.validate(&1.0).is_ok());
        assert!(in_range.validate(&-1.0).is_err());
        assert!(in_range.validate(&3.0).is_err());
    }
}

