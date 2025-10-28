//! Integration tests for the unified routing trait system.
//!
//! These tests verify that the `RouteHandler<M>` trait works correctly
//! with different transport metadata types and that the trait bounds
//! are enforced properly.

use common::{
    AppStateProvider,
    RouteHandler, TestCase,
    HttpMetadata, NativeMessagingMetadata, WebRtcMetadata,
    TransportMetadata, MediaType,
};
use tabagent_values::{RequestValue, ResponseValue, HealthStatus};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

// === Mock Backend ===

struct MockBackend;

#[async_trait]
impl AppStateProvider for MockBackend {
    async fn handle_request(&self, _request: RequestValue) -> anyhow::Result<ResponseValue> {
        Ok(ResponseValue::health(HealthStatus::Healthy))
    }
}

// === Test Request/Response Types ===

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestRequest {
    value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestResponse {
    result: String,
}

// === HTTP Route Handler ===

struct HttpTestRoute;

#[async_trait]
impl RouteHandler<HttpMetadata> for HttpTestRoute {
    type Request = TestRequest;
    type Response = TestResponse;

    fn metadata() -> HttpMetadata {
        HttpMetadata {
            path: "/test",
            method: "POST",
            tags: &["Test"],
            description: "HTTP test route",
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
            result: "http-success".to_string(),
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::error("empty_value", TestRequest { value: "".to_string() }, "value cannot be empty"),
            TestCase::success("valid", TestRequest { value: "test".to_string() }, TestResponse { result: "http-success".to_string() }),
        ]
    }
}

// === Native Messaging Route Handler ===

struct NativeMessagingTestRoute;

#[async_trait]
impl RouteHandler<NativeMessagingMetadata> for NativeMessagingTestRoute {
    type Request = TestRequest;
    type Response = TestResponse;

    fn metadata() -> NativeMessagingMetadata {
        NativeMessagingMetadata {
            route_id: "test",
            tags: &["Test"],
            description: "Native messaging test route",
            openai_compatible: false,
            idempotent: true,
            requires_auth: false,
            rate_limit_tier: None,
            supports_streaming: false,
            supports_binary: false,
            max_payload_size: Some(1024),
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
            result: "native-success".to_string(),
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::error("empty_value", TestRequest { value: "".to_string() }, "value cannot be empty"),
            TestCase::success("valid", TestRequest { value: "test".to_string() }, TestResponse { result: "native-success".to_string() }),
        ]
    }
}

// === WebRTC Route Handler ===

struct WebRtcTestRoute;

#[async_trait]
impl RouteHandler<WebRtcMetadata> for WebRtcTestRoute {
    type Request = TestRequest;
    type Response = TestResponse;

    fn metadata() -> WebRtcMetadata {
        WebRtcMetadata {
            route_id: "test",
            tags: &["Test"],
            description: "WebRTC test route",
            supports_streaming: true,
            supports_binary: false,
            requires_auth: true,
            rate_limit_tier: Some("standard"),
            max_payload_size: None,
            media_type: Some(MediaType::Video),
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
            result: "webrtc-success".to_string(),
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::error("empty_value", TestRequest { value: "".to_string() }, "value cannot be empty"),
            TestCase::success("valid", TestRequest { value: "test".to_string() }, TestResponse { result: "webrtc-success".to_string() }),
        ]
    }
}

// === Test HTTP Metadata ===

#[test]
fn test_http_metadata() {
    let metadata = HttpTestRoute::metadata();
    
    assert_eq!(metadata.route_identifier(), "/test");
    assert_eq!(metadata.description(), "HTTP test route");
    assert_eq!(metadata.tags(), &["Test"]);
    assert!(metadata.idempotent());
    assert!(!metadata.requires_auth());
    assert_eq!(metadata.rate_limit_tier(), None);
}

// === Test Native Messaging Metadata ===

#[test]
fn test_native_messaging_metadata() {
    let metadata = NativeMessagingTestRoute::metadata();
    
    assert_eq!(metadata.route_identifier(), "test");
    assert_eq!(metadata.description(), "Native messaging test route");
    assert_eq!(metadata.tags(), &["Test"]);
    assert!(metadata.idempotent());
    assert!(!metadata.requires_auth());
    assert_eq!(metadata.max_payload_size, Some(1024));
}

// === Test WebRTC Metadata ===

#[test]
fn test_webrtc_metadata() {
    let metadata = WebRtcTestRoute::metadata();
    
    assert_eq!(metadata.route_identifier(), "test");
    assert_eq!(metadata.description(), "WebRTC test route");
    assert_eq!(metadata.tags(), &["Test"]);
    assert!(!metadata.idempotent()); // WebRTC routes are not idempotent by default
    assert!(metadata.requires_auth());
    assert_eq!(metadata.media_type, Some(MediaType::Video));
}

// === Test Validation ===

#[tokio::test]
async fn test_http_validation() {
    let valid = TestRequest { value: "test".to_string() };
    assert!(HttpTestRoute::validate_request(&valid).await.is_ok());
    
    let invalid = TestRequest { value: "".to_string() };
    assert!(HttpTestRoute::validate_request(&invalid).await.is_err());
}

#[tokio::test]
async fn test_native_messaging_validation() {
    let valid = TestRequest { value: "test".to_string() };
    assert!(NativeMessagingTestRoute::validate_request(&valid).await.is_ok());
    
    let invalid = TestRequest { value: "".to_string() };
    assert!(NativeMessagingTestRoute::validate_request(&invalid).await.is_err());
}

#[tokio::test]
async fn test_webrtc_validation() {
    let valid = TestRequest { value: "test".to_string() };
    assert!(WebRtcTestRoute::validate_request(&valid).await.is_ok());
    
    let invalid = TestRequest { value: "".to_string() };
    assert!(WebRtcTestRoute::validate_request(&invalid).await.is_err());
}

// === Test Handlers ===

#[tokio::test]
async fn test_http_handler() {
    let backend = MockBackend;
    let req = TestRequest { value: "test".to_string() };
    
    let response = HttpTestRoute::handle(req, &backend).await.unwrap();
    assert_eq!(response.result, "http-success");
}

#[tokio::test]
async fn test_native_messaging_handler() {
    let backend = MockBackend;
    let req = TestRequest { value: "test".to_string() };
    
    let response = NativeMessagingTestRoute::handle(req, &backend).await.unwrap();
    assert_eq!(response.result, "native-success");
}

#[tokio::test]
async fn test_webrtc_handler() {
    let backend = MockBackend;
    let req = TestRequest { value: "test".to_string() };
    
    let response = WebRtcTestRoute::handle(req, &backend).await.unwrap();
    assert_eq!(response.result, "webrtc-success");
}

// === Test TestCase Helpers ===

#[test]
fn test_test_case_success() {
    let test_case = TestCase::success(
        "test",
        TestRequest { value: "test".to_string() },
        TestResponse { result: "success".to_string() },
    );
    
    assert_eq!(test_case.name, "test");
    assert!(test_case.expected_response.is_some());
    assert!(test_case.expected_error.is_none());
}

#[test]
fn test_test_case_error() {
    let test_case = TestCase::<TestRequest, TestResponse>::error(
        "test",
        TestRequest { value: "".to_string() },
        "error message",
    );
    
    assert_eq!(test_case.name, "test");
    assert!(test_case.expected_response.is_none());
    assert_eq!(test_case.expected_error, Some("error message"));
}

// === Test Route Test Cases ===

#[test]
fn test_http_route_has_test_cases() {
    let cases = HttpTestRoute::test_cases();
    assert!(!cases.is_empty(), "Routes must have test cases");
    assert_eq!(cases.len(), 2);
}

#[test]
fn test_native_messaging_route_has_test_cases() {
    let cases = NativeMessagingTestRoute::test_cases();
    assert!(!cases.is_empty(), "Routes must have test cases");
    assert_eq!(cases.len(), 2);
}

#[test]
fn test_webrtc_route_has_test_cases() {
    let cases = WebRtcTestRoute::test_cases();
    assert!(!cases.is_empty(), "Routes must have test cases");
    assert_eq!(cases.len(), 2);
}

// === Test Media Type Enum ===

#[test]
fn test_media_type_variants() {
    assert_eq!(MediaType::Video, MediaType::Video);
    assert_ne!(MediaType::Video, MediaType::Audio);
    assert_ne!(MediaType::Audio, MediaType::VideoAudio);
    assert_ne!(MediaType::VideoAudio, MediaType::ScreenShare);
}

// === Test Send + Sync Bounds ===

#[test]
fn test_metadata_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    
    assert_send_sync::<HttpMetadata>();
    assert_send_sync::<NativeMessagingMetadata>();
    assert_send_sync::<WebRtcMetadata>();
}

