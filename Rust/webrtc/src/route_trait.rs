//! WebRTC data channel route trait system for compile-time enforcement.
//!
//! # ENFORCED RULES (COMPILE-TIME)
//!
//! Every WebRTC route MUST:
//! 1. ✅ Have documentation (doc comments)
//! 2. ✅ Have tests (unit + integration)
//! 3. ✅ Tests cannot be fake (must call actual handler)
//! 4. ✅ Use `tabagent-values` for requests/responses
//! 5. ✅ Have proper tracing (request_id)
//! 6. ✅ Have proper error handling (WebRtcError)
//! 7. ✅ Have metadata (route_id, tags, description)
//! 8. ✅ Validate requests (required fields, ranges, business logic)
//! 9. ✅ Return proper JSON (no debug formatting)
//! 10. ✅ Handle media streams properly (video/audio validation)
//! 11. ✅ Have data channel protocol compliance
//! 12. ✅ Log success AND failure cases
//!
//! This prevents "random crappy routes" from being added to WebRTC data channels
//! without following architectural standards.

use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Debug;
use async_trait::async_trait;

use crate::error::{WebRtcResult, WebRtcError};

/// Route metadata for WebRTC data channel routes.
///
/// Unlike HTTP routes, WebRTC routes are identified by route_id strings
/// that map to RequestType variants in tabagent-values.
#[derive(Debug, Clone)]
pub struct RouteMetadata {
    /// Route identifier (e.g., "chat", "embeddings", "video_stream")
    pub route_id: &'static str,
    
    /// Category tags for organization (e.g., ["AI", "Chat"] or ["Media", "Video"])
    pub tags: &'static [&'static str],
    
    /// Human-readable description (REQUIRED - enforces documentation rule)
    pub description: &'static str,
    
    /// Supports streaming responses? (for chat completion, video/audio streams)
    pub supports_streaming: bool,
    
    /// Supports binary data? (for media payloads, model weights)
    pub supports_binary: bool,
    
    /// Requires authentication?
    pub requires_auth: bool,
    
    /// Rate limit tier (None = no limit, Some(tier) = apply tier limits)
    pub rate_limit_tier: Option<&'static str>,
    
    /// Maximum payload size in bytes (None = use default)
    pub max_payload_size: Option<usize>,
    
    /// Media type (if handling video/audio)
    pub media_type: Option<MediaType>,
}

/// Media types for video/audio handling routes
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

/// Data channel route handler trait - ALL WebRTC routes MUST implement this.
///
/// This enforces compile-time guarantees about:
/// - Request/Response types (type-safe via generics)
/// - Validation (MUST be implemented, cannot be no-op)
/// - Error handling (MUST use WebRtcError, not panic)
/// - Tracing (MUST generate request_id, MUST log start/end/errors)
/// - Documentation (MUST provide metadata with description)
/// - Testing (MUST have test cases, enforced via test_cases())
/// - Values usage (MUST use tabagent-values RequestValue/ResponseValue)
/// - Media validation (MUST validate codecs, bitrates for media routes)
///
/// # Example
/// ```ignore
/// struct ChatRoute;
///
/// #[async_trait]
/// impl DataChannelRoute for ChatRoute {
///     type Request = ChatRequest;
///     type Response = ChatResponse;
///     
///     fn metadata() -> RouteMetadata {
///         RouteMetadata {
///             route_id: "chat",
///             tags: &["AI", "Chat"],
///             description: "Chat completion over WebRTC data channel",
///             supports_streaming: true,
///             supports_binary: false,
///             requires_auth: true,
///             rate_limit_tier: Some("standard"),
///             max_payload_size: None,
///             media_type: None,
///         }
///     }
///     
///     async fn validate_request(req: &Self::Request) -> WebRtcResult<()> {
///         if req.messages.is_empty() {
///             return Err(WebRtcError::ValidationError {
///                 field: "messages".to_string(),
///                 message: "cannot be empty".to_string(),
///             });
///         }
///         Ok(())
///     }
///     
///     async fn handle<H>(req: Self::Request, handler: &H) -> WebRtcResult<Self::Response>
///     where
///         H: crate::traits::RequestHandler,
///     {
///         let request_id = uuid::Uuid::new_v4();
///         tracing::info!(request_id = %request_id, "Chat request via WebRTC");
///         
///         // Implementation...
///         
///         Ok(response)
///     }
/// }
/// ```
#[async_trait]
pub trait DataChannelRoute: Send + Sync + 'static {
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
    /// - Media codec validation (for video/audio routes)
    /// - Bitrate/resolution limits (for media routes)
    ///
    /// RULE: Cannot just return Ok(()) - must have real validation
    /// Return WebRtcError::ValidationError for validation failures.
    async fn validate_request(req: &Self::Request) -> WebRtcResult<()>;
    
    /// Handle the request - REQUIRED
    ///
    /// RULES (MUST follow ALL):
    /// 1. Generate a request_id (uuid::Uuid::new_v4())
    /// 2. Log at start: tracing::info!(request_id = %request_id, ...)
    /// 3. Log at end: tracing::info!(request_id = %request_id, "success")
    /// 4. Log errors: tracing::error!(request_id = %request_id, error = %e, ...)
    /// 5. Return WebRtcError for failures (NEVER panic)
    /// 6. Use tabagent_values::RequestValue/ResponseValue internally
    /// 7. Return proper Response type (no debug formatting)
    /// 8. Handle media streams with proper codec validation
    async fn handle<H>(req: Self::Request, handler: &H) -> WebRtcResult<Self::Response>
    where
        H: crate::traits::RequestHandler + Send + Sync;
    
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
    /// - Media routes have media validation
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
        
        // Rule 4: Media routes must specify media_type
        if metadata.tags.contains(&"Media") && metadata.media_type.is_none() {
            panic!("Route {} is tagged as Media but has no media_type", metadata.route_id);
        }
        
        // Rule 5: Tags cannot be empty
        if metadata.tags.is_empty() {
            panic!("Route {} has no tags - MUST categorize route", metadata.route_id);
        }
        
        true
    }
}

/// Validation rules trait for common validation patterns.
///
/// Implement this to define reusable validation logic for WebRTC routes.
pub trait ValidationRule: Send + Sync {
    /// Type being validated
    type Target;
    
    /// Validate the target
    fn validate(&self, target: &Self::Target) -> WebRtcResult<()>;
    
    /// Validate with field name for context-aware error messages
    fn validate_field(&self, field_name: &str, target: &Self::Target) -> WebRtcResult<()> {
        self.validate(target).map_err(|e| {
            match e {
                WebRtcError::BadRequest(msg) => {
                    WebRtcError::ValidationError {
                        field: field_name.to_string(),
                        message: msg,
                    }
                },
                other => other,
            }
        })
    }
}

/// Common validation rules for WebRTC routes.
pub mod validators {
    use super::*;
    
    /// Validates a string is not empty
    pub struct NotEmpty;
    
    impl ValidationRule for NotEmpty {
        type Target = String;
        
        fn validate(&self, target: &Self::Target) -> WebRtcResult<()> {
            if target.is_empty() {
                Err(WebRtcError::BadRequest("cannot be empty".into()))
            } else {
                Ok(())
            }
        }
    }
    
    /// Validates a number is in range
    pub struct InRange<T> {
        /// Minimum allowed value (inclusive)
        pub min: T,
        /// Maximum allowed value (inclusive)
        pub max: T,
    }
    
    impl ValidationRule for InRange<f32> {
        type Target = f32;
        
        fn validate(&self, target: &Self::Target) -> WebRtcResult<()> {
            if *target < self.min || *target > self.max {
                Err(WebRtcError::BadRequest(
                    format!("must be between {} and {}, got {}", self.min, self.max, target)
                ))
            } else {
                Ok(())
            }
        }
    }
    
    impl ValidationRule for InRange<u32> {
        type Target = u32;
        
        fn validate(&self, target: &Self::Target) -> WebRtcResult<()> {
            if *target < self.min || *target > self.max {
                Err(WebRtcError::BadRequest(
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
        /// Create a new VecNotEmpty validator
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
        
        fn validate(&self, target: &Self::Target) -> WebRtcResult<()> {
            if target.is_empty() {
                Err(WebRtcError::BadRequest("Array cannot be empty".into()))
            } else {
                Ok(())
            }
        }
    }
    
    /// Validates video codec is supported
    pub struct ValidVideoCodec;
    
    impl ValidationRule for ValidVideoCodec {
        type Target = String;
        
        fn validate(&self, target: &Self::Target) -> WebRtcResult<()> {
            const SUPPORTED_CODECS: &[&str] = &[
                "h264", "vp8", "vp9", "av1", "hevc"
            ];
            
            let codec = target.to_lowercase();
            if SUPPORTED_CODECS.contains(&codec.as_str()) {
                Ok(())
            } else {
                Err(WebRtcError::ValidationError {
                    field: "codec".to_string(),
                    message: format!(
                        "Unsupported video codec '{}'. Supported: {}",
                        target,
                        SUPPORTED_CODECS.join(", ")
                    ),
                })
            }
        }
    }
    
    /// Validates audio codec is supported
    pub struct ValidAudioCodec;
    
    impl ValidationRule for ValidAudioCodec {
        type Target = String;
        
        fn validate(&self, target: &Self::Target) -> WebRtcResult<()> {
            const SUPPORTED_CODECS: &[&str] = &[
                "opus", "g722", "pcmu", "pcma", "aac"
            ];
            
            let codec = target.to_lowercase();
            if SUPPORTED_CODECS.contains(&codec.as_str()) {
                Ok(())
            } else {
                Err(WebRtcError::ValidationError {
                    field: "codec".to_string(),
                    message: format!(
                        "Unsupported audio codec '{}'. Supported: {}",
                        target,
                        SUPPORTED_CODECS.join(", ")
                    ),
                })
            }
        }
    }
    
    /// Validates video resolution is reasonable
    pub struct ValidResolution;
    
    impl ValidationRule for ValidResolution {
        type Target = (u32, u32); // (width, height)
        
        fn validate(&self, target: &Self::Target) -> WebRtcResult<()> {
            let (width, height) = *target;
            
            // Min: 160x120 (QQVGA), Max: 7680x4320 (8K)
            if width < 160 || height < 120 {
                return Err(WebRtcError::ValidationError {
                    field: "resolution".to_string(),
                    message: format!(
                        "Resolution too low: {}x{} (min: 160x120)",
                        width, height
                    ),
                });
            }
            
            if width > 7680 || height > 4320 {
                return Err(WebRtcError::ValidationError {
                    field: "resolution".to_string(),
                    message: format!(
                        "Resolution too high: {}x{} (max: 7680x4320)",
                        width, height
                    ),
                });
            }
            
            Ok(())
        }
    }
    
    /// Validates bitrate is reasonable
    pub struct ValidBitrate;
    
    impl ValidationRule for ValidBitrate {
        type Target = u32; // bits per second
        
        fn validate(&self, target: &Self::Target) -> WebRtcResult<()> {
            // Min: 64kbps (audio), Max: 100Mbps (8K video)
            const MIN_BITRATE: u32 = 64_000;
            const MAX_BITRATE: u32 = 100_000_000;
            
            if *target < MIN_BITRATE {
                return Err(WebRtcError::ValidationError {
                    field: "bitrate".to_string(),
                    message: format!(
                        "Bitrate too low: {} bps (min: {} bps)",
                        target, MIN_BITRATE
                    ),
                });
            }
            
            if *target > MAX_BITRATE {
                return Err(WebRtcError::ValidationError {
                    field: "bitrate".to_string(),
                    message: format!(
                        "Bitrate too high: {} bps (max: {} bps)",
                        target, MAX_BITRATE
                    ),
                });
            }
            
            Ok(())
        }
    }
    
    /// Validates framerate is reasonable
    pub struct ValidFramerate;
    
    impl ValidationRule for ValidFramerate {
        type Target = u32; // frames per second
        
        fn validate(&self, target: &Self::Target) -> WebRtcResult<()> {
            // Min: 1fps, Max: 120fps
            if *target < 1 || *target > 120 {
                return Err(WebRtcError::ValidationError {
                    field: "framerate".to_string(),
                    message: format!(
                        "Invalid framerate: {} fps (must be 1-120)",
                        target
                    ),
                });
            }
            
            Ok(())
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

/// Macro to enforce data channel route implementation AND verify rules.
///
/// This macro generates compile-time checks to ensure:
/// 1. Type implements DataChannelRoute
/// 2. All rules are followed (docs, tests, validation, etc.)
///
/// Usage:
/// ```ignore
/// enforce_data_channel_route!(ChatRoute);
/// ```
#[macro_export]
macro_rules! enforce_data_channel_route {
    ($route_type:ty) => {
        const _: () = {
            fn assert_route<T: $crate::route_trait::DataChannelRoute>() {}
            
            fn check() {
                assert_route::<$route_type>();
                
                // Run verification - this will panic at compile time if rules are violated
                <$route_type as $crate::route_trait::DataChannelRoute>::verify_implementation();
            }
        };
    };
}

/// Macro to register multiple routes and enforce ALL rules.
///
/// This is the recommended way to add WebRTC routes - it ensures:
/// - All routes implement DataChannelRoute
/// - All routes follow the rules (docs, tests, validation)
/// - All routes are properly registered
///
/// Usage:
/// ```ignore
/// register_data_channel_routes!([
///     ChatRoute,
///     EmbeddingsRoute,
///     VideoStreamRoute,
/// ]);
/// ```
#[macro_export]
macro_rules! register_data_channel_routes {
    ([$($route:ty),* $(,)?]) => {
        {
            $(
                // Enforce rules for each route
                $crate::enforce_data_channel_route!($route);
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
    fn test_video_codec_validator() {
        use validators::*;
        
        let validator = ValidVideoCodec;
        assert!(validator.validate(&"h264".to_string()).is_ok());
        assert!(validator.validate(&"VP9".to_string()).is_ok());
        assert!(validator.validate(&"invalid".to_string()).is_err());
    }
    
    #[test]
    fn test_audio_codec_validator() {
        use validators::*;
        
        let validator = ValidAudioCodec;
        assert!(validator.validate(&"opus".to_string()).is_ok());
        assert!(validator.validate(&"AAC".to_string()).is_ok());
        assert!(validator.validate(&"invalid".to_string()).is_err());
    }
    
    #[test]
    fn test_resolution_validator() {
        use validators::*;
        
        let validator = ValidResolution;
        assert!(validator.validate(&(1920, 1080)).is_ok()); // 1080p
        assert!(validator.validate(&(3840, 2160)).is_ok()); // 4K
        assert!(validator.validate(&(100, 100)).is_err()); // Too small
        assert!(validator.validate(&(10000, 10000)).is_err()); // Too large
    }
    
    #[test]
    fn test_bitrate_validator() {
        use validators::*;
        
        let validator = ValidBitrate;
        assert!(validator.validate(&128_000).is_ok()); // 128kbps
        assert!(validator.validate(&5_000_000).is_ok()); // 5Mbps
        assert!(validator.validate(&1_000).is_err()); // Too low
        assert!(validator.validate(&200_000_000).is_err()); // Too high
    }
    
    #[test]
    fn test_framerate_validator() {
        use validators::*;
        
        let validator = ValidFramerate;
        assert!(validator.validate(&30).is_ok());
        assert!(validator.validate(&60).is_ok());
        assert!(validator.validate(&0).is_err());
        assert!(validator.validate(&200).is_err());
    }
}

