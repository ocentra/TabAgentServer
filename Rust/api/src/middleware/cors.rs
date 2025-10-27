//! CORS middleware configuration.

use tower_http::cors::{Any, CorsLayer};
use crate::config::ApiConfig;

/// Create CORS layer from configuration.
///
/// This enables cross-origin requests for web clients.
pub fn cors_layer(config: &ApiConfig) -> CorsLayer {
    if !config.enable_cors {
        return CorsLayer::new();
    }

    let mut cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any);

    // Configure origins
    if config.cors_origins.contains(&"*".to_string()) {
        cors = cors.allow_origin(Any);
    } else {
        // Parse specific origins
        let origins: Vec<_> = config
            .cors_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        cors = cors.allow_origin(origins);
    }

    cors
}

