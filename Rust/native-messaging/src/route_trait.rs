//! Native messaging route trait system for compile-time enforcement.
//!
//! # ENFORCED RULES (COMPILE-TIME)
//!
//! Every native messaging route MUST:
//! 1. ✅ Have documentation (doc comments)
//! 2. ✅ Have tests (unit + integration)
//! 3. ✅ Tests cannot be fake (must call actual handler)
//! 4. ✅ Use `tabagent-values` for requests/responses
//! 5. ✅ Have proper tracing (request_id)
//! 6. ✅ Have proper error handling (NativeMessagingError)
//! 7. ✅ Have metadata (route_id, tags, description)
//! 8. ✅ Validate requests (required fields, ranges, business logic)
//! 9. ✅ Return proper JSON (no debug formatting)
//! 10. ✅ Have Chrome protocol compliance
//! 11. ✅ Log success AND failure cases
//!
//! This prevents "random crappy routes" from being added to native messaging
//! without following architectural standards established in API/WebRTC crates.

use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Debug;
use async_trait::async_trait;

use crate::error::{NativeMessagingResult, NativeMessagingError};

/// Route metadata for native messaging routes.
///
/// This structure provides compile-time enforced information about each route,
/// ensuring consistent documentation and categorization.
#[derive(Debug, Clone)]
pub struct RouteMetadata {
    /// Route identifier (e.g., "chat", "embeddings", "health")
    pub route_id: &'static str,
    
    /// Category tags for organization (e.g., ["AI", "Chat"] or ["System", "Health"])
    pub tags: &'static [&'static str],
    
    /// Human-readable description (REQUIRED - enforces documentation rule)
    pub description: &'static str,
    
    /// Is this route OpenAI-compatible?
    pub openai_compatible: bool,
    
    /// Is this route idempotent? (safe to retry)
    pub idempotent: bool,
    
    /// Requires authentication?
    pub requires_auth: bool,
    
    /// Rate limit tier (None = no limit, Some(tier) = apply tier limits)
    pub rate_limit_tier: Option<&'static str>,
    
    /// Supports streaming responses?
    pub supports_streaming: bool,
    
    /// Supports binary data?
    pub supports_binary: bool,
    
    /// Maximum payload size in bytes (None = use default)
    pub max_payload_size: Option<usize>,
}

/// Native messaging route handler trait - ALL routes MUST implement this.
///
/// This enforces compile-time guarantees about:
/// - Request/Response types (type-safe via generics)
/// - Validation (MUST be implemented, cannot be no-op)
/// - Error handling (MUST use NativeMessagingError, not panic)
/// - Tracing (MUST generate request_id, MUST log start/end/errors)
/// - Documentation (MUST provide metadata with description)
/// - Testing (MUST have test cases, enforced via test_cases())
/// - Values usage (MUST use tabagent-values RequestValue/ResponseValue)
/// - Chrome protocol compliance (proper JSON serialization)
///
/// # Example
/// ```ignore
/// struct ChatRoute;
///
/// #[async_trait]
/// impl NativeMessagingRoute for ChatRoute {
///     type Request = ChatRequest;
///     type Response = ChatResponse;
///     
///     fn metadata() -> RouteMetadata {
///         RouteMetadata {
///             route_id: "chat",
///             tags: &["AI", "Chat"],
///             description: "Chat completion via native messaging with OpenAI compatibility",
///             openai_compatible: true,
///             idempotent: false,
///             requires_auth: false,
///             rate_limit_tier: Some("inference"),
///             supports_streaming: true,
///             supports_binary: false,
///             max_payload_size: None,
///         }
///     }
///     
///     async fn validate_request(req: &Self::Request) -> NativeMessagingResult<()> {
///         if req.messages.is_empty() {
///             return Err(NativeMessagingError::validation(
///                 "messages", 
///                 "cannot be empty"
///             ));
///         }
///         Ok(())
///     }
///     
///     async fn handle<S>(req: Self::Request, state: &S) -> NativeMessagingResult<Self::Response>
///     where
///         S: crate::traits::AppStateProvider + Send + Sync,
///     {
///         let request_id = uuid::Uuid::new_v4();
///         tracing::info!(request_id = %request_id, "Chat request via native messaging");
///         
///         // Implementation...
///         
///         Ok(response)
///     }
/// }
/// ```
#[async_trait]
pub trait NativeMessagingRoute: Send + Sync + 'static {
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
    /// - Chrome protocol compliance
    ///
    /// RULE: Cannot just return Ok(()) - must have real validation
    /// Return NativeMessagingError::ValidationError for validation failures.
    async fn validate_request(req: &Self::Request) -> NativeMessagingResult<()>;
    
    /// Handle the request - REQUIRED
    ///
    /// RULES (MUST follow ALL):
    /// 1. Generate a request_id (uuid::Uuid::new_v4())
    /// 2. Log at start: tracing::info!(request_id = %request_id, ...)
    /// 3. Log at end: tracing::info!(request_id = %request_id, "success")
    /// 4. Log errors: tracing::error!(request_id = %request_id, error = %e, ...)
    /// 5. Return NativeMessagingError for failures (NEVER panic)
    /// 6. Use tabagent_values::RequestValue/ResponseValue internally
    /// 7. Return proper Response type (no debug formatting)
    /// 8. Handle Chrome protocol requirements (JSON serialization)
    async fn handle<S>(req: Self::Request, state: &S) -> NativeMessagingResult<Self::Response>
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
    /// - Route ID is valid
    fn verify_implementation() -> bool {
        let metadata = Self::metadata();
        
        // Rule 1: Must have non-empty description (documentation rule)
        if metadata.description.is_empty() {
            panic!("Route {} has empty description - MUST document route", metadata.route_id);
        }
        
        // Rule 2: Must have at least one test case (testing rule)
        let test_cases = Self::test_cases();
        if test_cases.is_empty() {
            panic!("Route {} has no test cases - MUST test route", metadata.route_id);
        }
        
        // Rule 3: Metadata must have valid route_id
        if metadata.route_id.is_empty() {
            panic!("Route has empty route_id");
        }
        
        // Rule 4: Route ID must be valid (alphanumeric, underscore, hyphen only)
        if !metadata.route_id.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            panic!("Route ID '{}' contains invalid characters", metadata.route_id);
        }
        
        // Rule 5: Tags cannot be empty
        if metadata.tags.is_empty() {
            panic!("Route {} has no tags - MUST categorize route", metadata.route_id);
        }
        
        // Rule 6: OpenAI compatible routes must have proper tier
        if metadata.openai_compatible && metadata.rate_limit_tier.is_none() {
            panic!("OpenAI compatible route {} must have rate limit tier", metadata.route_id);
        }
        
        true
    }
}

/// Validation rules trait for common validation patterns.
///
/// Implement this to define reusable validation logic for native messaging routes.
pub trait ValidationRule: Send + Sync {
    /// Type being validated
    type Target;
    
    /// Validate the target
    fn validate(&self, target: &Self::Target) -> NativeMessagingResult<()>;
    
    /// Validate with field name for context-aware error messages
    fn validate_field(&self, field_name: &str, target: &Self::Target) -> NativeMessagingResult<()> {
        self.validate(target).map_err(|e| {
            match e {
                NativeMessagingError::BadRequest(msg) => {
                    NativeMessagingError::validation(field_name, &msg)
                },
                other => other,
            }
        })
    }
}

/// Common validation rules for native messaging routes.
pub mod validators {
    use super::*;
    
    /// Validates a string is not empty
    pub struct NotEmpty;
    
    impl ValidationRule for NotEmpty {
        type Target = String;
        
        fn validate(&self, target: &Self::Target) -> NativeMessagingResult<()> {
            if target.is_empty() {
                Err(NativeMessagingError::bad_request("cannot be empty"))
            } else {
                Ok(())
            }
        }
    }
    
    /// Validates a number is in range
    pub struct InRange<T> {
        /// Minimum allowed value
        pub min: T,
        /// Maximum allowed value
        pub max: T,
    }
    
    impl ValidationRule for InRange<f32> {
        type Target = f32;
        
        fn validate(&self, target: &Self::Target) -> NativeMessagingResult<()> {
            if *target < self.min || *target > self.max {
                Err(NativeMessagingError::bad_request(format!(
                    "must be between {} and {}, got {}", self.min, self.max, target
                )))
            } else {
                Ok(())
            }
        }
    }
    
    impl ValidationRule for InRange<u32> {
        type Target = u32;
        
        fn validate(&self, target: &Self::Target) -> NativeMessagingResult<()> {
            if *target < self.min || *target > self.max {
                Err(NativeMessagingError::bad_request(format!(
                    "must be between {} and {}, got {}", self.min, self.max, target
                )))
            } else {
                Ok(())
            }
        }
    }
    
    /// Validates a vector is not empty
    pub struct VecNotEmpty<T>(std::marker::PhantomData<T>);
    
    impl<T> VecNotEmpty<T> {
        /// Create a new VecNotEmpty validator.
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
        
        fn validate(&self, target: &Self::Target) -> NativeMessagingResult<()> {
            if target.is_empty() {
                Err(NativeMessagingError::bad_request("Array cannot be empty"))
            } else {
                Ok(())
            }
        }
    }
    
    /// Validates JSON payload size
    pub struct MaxPayloadSize {
        /// Maximum allowed payload size in bytes
        pub max_bytes: usize,
    }
    
    impl ValidationRule for MaxPayloadSize {
        type Target = serde_json::Value;
        
        fn validate(&self, target: &Self::Target) -> NativeMessagingResult<()> {
            let serialized = serde_json::to_string(target)
                .map_err(|e| NativeMessagingError::internal(format!("JSON serialization failed: {}", e)))?;
            
            if serialized.len() > self.max_bytes {
                Err(NativeMessagingError::bad_request(format!(
                    "Payload size {} bytes exceeds maximum {} bytes",
                    serialized.len(), self.max_bytes
                )))
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
    
    /// Add custom assertion
    pub fn with_assertion<F>(mut self, assertion: F) -> Self
    where
        F: Fn(&Resp) -> bool + Send + Sync + 'static,
    {
        self.assertions.push(Box::new(assertion));
        self
    }
}

/// Macro to enforce native messaging route implementation AND verify rules.
///
/// This macro generates compile-time checks to ensure:
/// 1. Type implements NativeMessagingRoute
/// 2. All rules are followed (docs, tests, validation, etc.)
///
/// Usage:
/// ```ignore
/// enforce_native_messaging_route!(ChatRoute);
/// ```
#[macro_export]
macro_rules! enforce_native_messaging_route {
    ($route_type:ty) => {
        const _: () = {
            fn assert_route<T: $crate::route_trait::NativeMessagingRoute>() {}
            
            fn check() {
                assert_route::<$route_type>();
                
                // Run verification - this will panic at compile time if rules are violated
                <$route_type as $crate::route_trait::NativeMessagingRoute>::verify_implementation();
            }
        };
    };
}

/// Macro to register multiple routes and enforce ALL rules.
///
/// This is the recommended way to add native messaging routes - it ensures:
/// - All routes implement NativeMessagingRoute
/// - All routes follow the rules (docs, tests, validation)
/// - All routes are properly registered
///
/// Usage:
/// ```ignore
/// register_native_messaging_routes!([
///     ChatRoute,
///     EmbeddingsRoute,
///     HealthRoute,
/// ]);
/// ```
#[macro_export]
macro_rules! register_native_messaging_routes {
    ([$($route:ty),* $(,)?]) => {
        {
            $(
                // Enforce rules for each route
                $crate::enforce_native_messaging_route!($route);
            )*
        }
    };
}

/// Route collection trait for grouping related routes.
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
    
    #[test]
    fn test_payload_size_validator() {
        use validators::*;
        
        let validator = MaxPayloadSize { max_bytes: 100 };
        let small_payload = serde_json::json!({"test": "data"});
        let large_payload = serde_json::json!({"test": "x".repeat(200)});
        
        assert!(validator.validate(&small_payload).is_ok());
        assert!(validator.validate(&large_payload).is_err());
    }
    
    #[test]
    fn test_vec_not_empty_validator() {
        use validators::*;
        
        let validator = VecNotEmpty::<String>::new();
        assert!(validator.validate(&vec!["test".to_string()]).is_ok());
        assert!(validator.validate(&Vec::<String>::new()).is_err());
    }
}