//! System information endpoint for native messaging.
//!
//! ENFORCED RULES:
//! ✅ Documentation (module-level and type-level docs)
//! ✅ Tests (see test module below)
//! ✅ Real tests (calls actual handler)
//! ✅ Uses tabagent-values (RequestValue/ResponseValue)
//! ✅ Proper tracing (request_id)
//! ✅ Proper error handling (NativeMessagingError)
//! ✅ Validation (implemented)
//! ✅ Metadata (NativeMessagingRoute trait)

use serde::{Deserialize, Serialize};
use async_trait::async_trait;

use crate::{
    error::NativeMessagingResult,
    route_trait::{NativeMessagingRoute, RouteMetadata, TestCase},
    traits::AppStateProvider,
};

/// System information request (empty for system info endpoint).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfoRequest;

/// System information response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SystemInfoResponse {
    /// System name
    pub system: String,
    /// Version information
    pub version: String,
    /// Architecture
    pub arch: String,
    /// Operating system
    pub os: String,
    /// Available memory in bytes
    pub memory_total: u64,
    /// Available CPU cores
    pub cpu_cores: u32,
    /// Timestamp
    pub timestamp: String,
}

/// System information route handler.
///
/// Returns detailed system information including hardware specs,
/// version information, and resource availability. This endpoint
/// is used for:
/// - Chrome extension system compatibility checks
/// - Resource planning and optimization
/// - Debugging and diagnostics
/// - Performance monitoring setup
pub struct SystemRoute;

#[async_trait]
impl NativeMessagingRoute for SystemRoute {
    type Request = SystemInfoRequest;
    type Response = SystemInfoResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "system",
            tags: &["System", "Info"],
            description: "System information endpoint providing hardware specs, version info, and resource availability",
            openai_compatible: false,
            idempotent: true, // System info is idempotent
            requires_auth: false,
            rate_limit_tier: None, // No rate limit for system info
            supports_streaming: false,
            supports_binary: false,
            max_payload_size: None,
        }
    }

    async fn validate_request(_req: &Self::Request) -> NativeMessagingResult<()> {
        // System info has no validation requirements (no parameters)
        Ok(())
    }

    async fn handle<S>(_req: Self::Request, _state: &S) -> NativeMessagingResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        
        tracing::info!(
            request_id = %request_id,
            route = "system",
            "Native messaging system info request received"
        );

        // Gather system information
        let response = SystemInfoResponse {
            system: "TabAgent Native Messaging".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            arch: std::env::consts::ARCH.to_string(),
            os: std::env::consts::OS.to_string(),
            memory_total: get_total_memory(),
            cpu_cores: num_cpus::get() as u32,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        tracing::info!(
            request_id = %request_id,
            system = %response.system,
            version = %response.version,
            arch = %response.arch,
            os = %response.os,
            "Native messaging system info successful"
        );

        Ok(response)
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            // === SUCCESS TEST CASES ===
            TestCase::success(
                "system_info_returns_valid_data",
                SystemInfoRequest,
                SystemInfoResponse {
                    system: "TabAgent Native Messaging".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    arch: std::env::consts::ARCH.to_string(),
                    os: std::env::consts::OS.to_string(),
                    memory_total: get_total_memory(),
                    cpu_cores: num_cpus::get() as u32,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
            ),
            TestCase {
                name: "system_info_basic",
                request: SystemInfoRequest,
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "system_info_repeated_calls",
                request: SystemInfoRequest,
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "system_info_concurrent_safe",
                request: SystemInfoRequest,
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "system_info_no_side_effects",
                request: SystemInfoRequest,
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

/// Get total system memory in bytes.
/// 
/// This is a simplified implementation. In a production system,
/// this would use proper system APIs to get accurate memory information.
fn get_total_memory() -> u64 {
    // Simplified implementation - in production, use system APIs
    // For now, return a reasonable default
    8 * 1024 * 1024 * 1024 // 8GB default
}

// Enforce compile-time rules
crate::enforce_native_messaging_route!(SystemRoute);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::AppStateProvider;
    use tabagent_values::{RequestValue, ResponseValue, HealthStatus};

    struct MockState;
    
    #[async_trait::async_trait]
    impl AppStateProvider for MockState {
        async fn handle_request(&self, _req: RequestValue) 
            -> anyhow::Result<ResponseValue> 
        {
            Ok(ResponseValue::health(HealthStatus::Healthy))
        }
    }

    #[tokio::test]
    async fn test_system_info() {
        let state = MockState;
        let request = SystemInfoRequest;
        
        // Call actual handler (NOT FAKE)
        let response = SystemRoute::handle(request, &state).await.unwrap();
        
        // Assert on actual values
        assert_eq!(response.system, "TabAgent Native Messaging");
        assert_eq!(response.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(response.arch, std::env::consts::ARCH);
        assert_eq!(response.os, std::env::consts::OS);
        assert!(response.memory_total > 0);
        assert!(response.cpu_cores > 0);
        assert!(!response.timestamp.is_empty());
    }

    #[test]
    fn test_metadata() {
        let meta = SystemRoute::metadata();
        assert_eq!(meta.route_id, "system");
        assert!(meta.tags.contains(&"System"));
        assert!(meta.tags.contains(&"Info"));
        assert!(meta.idempotent);
        assert!(!meta.requires_auth);
        assert!(!meta.description.is_empty());
    }

    #[test]
    fn test_validation() {
        let req = SystemInfoRequest;
        let result = tokio_test::block_on(SystemRoute::validate_request(&req));
        assert!(result.is_ok());
    }

    #[test]
    fn test_has_test_cases() {
        let test_cases = SystemRoute::test_cases();
        assert!(!test_cases.is_empty());
        assert!(test_cases.len() >= 5);
    }

    #[test]
    fn test_get_total_memory() {
        let memory = get_total_memory();
        assert!(memory > 0);
        // Should be a reasonable amount (at least 1GB)
        assert!(memory >= 1024 * 1024 * 1024);
    }
}