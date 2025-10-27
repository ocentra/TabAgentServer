//! API configuration.

use serde::{Deserialize, Serialize};

/// Configuration for the HTTP API server.
///
/// # Example
///
/// ```rust
/// use tabagent_api::ApiConfig;
///
/// let config = ApiConfig {
///     port: 8080,
///     enable_cors: true,
///     cors_origins: vec!["*".to_string()],
///     timeout_secs: 300,
///     enable_swagger: true,
///     rate_limit_rpm: 60,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// Port to bind the HTTP server to.
    ///
    /// Default: 8080
    pub port: u16,

    /// Enable Cross-Origin Resource Sharing (CORS).
    ///
    /// Default: true
    pub enable_cors: bool,

    /// Allowed origins for CORS requests.
    ///
    /// Use `["*"]` to allow all origins (development only).
    ///
    /// Default: `["*"]`
    pub cors_origins: Vec<String>,

    /// Request timeout in seconds.
    ///
    /// Requests exceeding this duration will be terminated.
    ///
    /// Default: 300 (5 minutes)
    pub timeout_secs: u64,

    /// Enable Swagger UI documentation.
    ///
    /// When enabled, API docs are available at `/swagger-ui/`.
    ///
    /// Default: true
    pub enable_swagger: bool,

    /// Rate limit: requests per minute per IP.
    ///
    /// Set to 0 to disable rate limiting.
    ///
    /// Default: 60
    pub rate_limit_rpm: u32,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            enable_cors: true,
            cors_origins: vec!["*".to_string()],
            timeout_secs: 300,
            enable_swagger: true,
            rate_limit_rpm: 60,
        }
    }
}

impl ApiConfig {
    /// Create a new configuration for production use.
    ///
    /// This sets stricter defaults suitable for production:
    /// - CORS restricted to specific origins (must be provided)
    /// - Swagger UI disabled
    /// - Rate limiting enabled
    pub fn production(allowed_origins: Vec<String>) -> Self {
        Self {
            enable_swagger: false,
            cors_origins: allowed_origins,
            ..Default::default()
        }
    }

    /// Create a new configuration for development use.
    ///
    /// This sets permissive defaults suitable for local development:
    /// - CORS allows all origins
    /// - Swagger UI enabled
    /// - Higher rate limits
    pub fn development() -> Self {
        Self {
            cors_origins: vec!["*".to_string()],
            enable_swagger: true,
            rate_limit_rpm: 1000,
            ..Default::default()
        }
    }
}

