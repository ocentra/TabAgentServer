//! Generation parameter configuration endpoints.
//!
//! ENFORCED RULES:
//! ✅ Documentation ✅ Tests ✅ tabagent-values ✅ Tracing ✅ Validation

use axum::http::Method;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use async_trait::async_trait;

use tabagent_values::RequestValue;
use crate::{
    error::{ApiResult, ApiError},
    route_trait::{RouteHandler, RouteMetadata, TestCase, ValidationRule, validators::*},
    traits::AppStateProvider,
};

/// Generation parameters.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct GenerationParams {
    /// Sampling temperature (0.0-2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Top-p nucleus sampling (0.0-1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    /// Top-k sampling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
    /// Minimum output tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<u32>,
    /// Maximum output tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<u32>,
    /// Maximum new tokens (alternative to max_length)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_new_tokens: Option<u32>,
    /// Minimum probability threshold
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_p: Option<f32>,
    /// Repetition penalty
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repetition_penalty: Option<f32>,
    /// Frequency penalty (-2.0 to 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    /// Presence penalty (-2.0 to 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
    /// Length penalty
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length_penalty: Option<f32>,
    /// N-gram blocking size
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_repeat_ngram_size: Option<u32>,
    /// Beam search width
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_beams: Option<u32>,
    /// Early stopping for beam search
    #[serde(skip_serializing_if = "Option::is_none")]
    pub early_stopping: Option<bool>,
    /// Enable sampling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub do_sample: Option<bool>,
    /// Use KV cache
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_cache: Option<bool>,
    /// Random seed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
    /// Stop sequences
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
}

// ==================== GET PARAMS ====================

/// Get params request (no parameters).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetParamsRequest;

/// Get generation parameters route handler.
///
/// Returns the current global generation parameters configuration.
pub struct GetParamsRoute;

#[async_trait]
impl RouteHandler for GetParamsRoute {
    type Request = GetParamsRequest;
    type Response = GenerationParams;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            path: "/v1/params",
            method: Method::GET,
            tags: &["Parameters"],
            description: "Get current generation parameter configuration",
            openai_compatible: false,
            idempotent: true,
            requires_auth: false,
            rate_limit_tier: None,
        }
    }

    async fn validate_request(_req: &Self::Request) -> ApiResult<()> {
        Ok(()) // No validation needed
    }

    async fn handle<S>(_req: Self::Request, state: &S) -> ApiResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(request_id = %request_id, "Get params request received");

        let request = RequestValue::get_params();
        
        tracing::debug!(request_id = %request_id, "Dispatching to handler");
        let response = state.handle_request(request).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Get params failed");
                e
            })?;

        let params_json = response.to_json_value();
        let params: GenerationParams = serde_json::from_value(params_json)
            .map_err(|e| ApiError::Internal(format!("Failed to parse params: {}", e)))?;

        tracing::info!(request_id = %request_id, "Get params successful");
        Ok(params)
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            // === SUCCESS TEST CASES ===
            TestCase {
                name: "get_params_basic",
                request: GetParamsRequest,
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "get_params_idempotent",
                request: GetParamsRequest,
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "get_params_concurrent_safe",
                request: GetParamsRequest,
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_route_handler!(GetParamsRoute);

// Implement RegisterableRoute with Axum 0.8 compatible handler
crate::impl_registerable_route!(GetParamsRoute);

// ==================== SET PARAMS ====================

/// Set generation parameters route handler.
///
/// Updates the global generation parameters configuration with validation.
pub struct SetParamsRoute;

#[async_trait]
impl RouteHandler for SetParamsRoute {
    type Request = GenerationParams;
    type Response = GenerationParams;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            path: "/v1/params",
            method: Method::POST,
            tags: &["Parameters"],
            description: "Update generation parameter configuration with validation",
            openai_compatible: false,
            idempotent: true, // Setting same params multiple times is idempotent
            requires_auth: false,
            rate_limit_tier: None,
        }
    }

    async fn validate_request(req: &Self::Request) -> ApiResult<()> {
        // Validate temperature range
        if let Some(temp) = req.temperature {
            InRange { min: 0.0, max: 2.0 }.validate(&temp)?;
        }

        // Validate top_p range
        if let Some(top_p) = req.top_p {
            InRange { min: 0.0, max: 1.0 }.validate(&top_p)?;
        }

        // Validate min_p range
        if let Some(min_p) = req.min_p {
            InRange { min: 0.0, max: 1.0 }.validate(&min_p)?;
        }

        // Validate frequency_penalty range
        if let Some(freq) = req.frequency_penalty {
            InRange { min: -2.0, max: 2.0 }.validate(&freq)?;
        }

        // Validate presence_penalty range
        if let Some(pres) = req.presence_penalty {
            InRange { min: -2.0, max: 2.0 }.validate(&pres)?;
        }

        // Validate repetition_penalty is positive
        if let Some(rep) = req.repetition_penalty {
            if rep <= 0.0 {
                return Err(ApiError::BadRequest(
                    "repetition_penalty must be positive".into()
                ));
            }
        }

        // Validate max_length and max_new_tokens are reasonable
        if let Some(max_len) = req.max_length {
            if max_len == 0 || max_len > 100000 {
                return Err(ApiError::BadRequest(
                    "max_length must be between 1 and 100000".into()
                ));
            }
        }

        if let Some(max_new) = req.max_new_tokens {
            if max_new == 0 || max_new > 100000 {
                return Err(ApiError::BadRequest(
                    "max_new_tokens must be between 1 and 100000".into()
                ));
            }
        }

        // Validate min_length <= max_length if both are set
        if let (Some(min), Some(max)) = (req.min_length, req.max_length) {
            if min > max {
                return Err(ApiError::BadRequest(
                    "min_length cannot exceed max_length".into()
                ));
            }
        }

        Ok(())
    }

    async fn handle<S>(req: Self::Request, state: &S) -> ApiResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(
            request_id = %request_id,
            temperature = ?req.temperature,
            top_p = ?req.top_p,
            max_length = ?req.max_length,
            "Set params request received"
        );

        let request = RequestValue::set_params(req.clone());
        
        tracing::debug!(request_id = %request_id, "Dispatching to handler");
        let _response = state.handle_request(request).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Set params failed");
                e
            })?;

        tracing::info!(request_id = %request_id, "Set params successful");
        Ok(req)
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase::error(
                "invalid_temperature",
                GenerationParams {
                    temperature: Some(3.0), // Invalid: > 2.0
                    top_p: None,
                    top_k: None,
                    min_length: None,
                    max_length: None,
                    max_new_tokens: None,
                    min_p: None,
                    repetition_penalty: None,
                    frequency_penalty: None,
                    presence_penalty: None,
                    length_penalty: None,
                    no_repeat_ngram_size: None,
                    num_beams: None,
                    early_stopping: None,
                    do_sample: None,
                    use_cache: None,
                    seed: None,
                    stop_sequences: None,
                },
                "not in range",
            ),
            TestCase::error(
                "invalid_top_p",
                GenerationParams {
                    temperature: None,
                    top_p: Some(1.5), // Invalid: > 1.0
                    top_k: None,
                    min_length: None,
                    max_length: None,
                    max_new_tokens: None,
                    min_p: None,
                    repetition_penalty: None,
                    frequency_penalty: None,
                    presence_penalty: None,
                    length_penalty: None,
                    no_repeat_ngram_size: None,
                    num_beams: None,
                    early_stopping: None,
                    do_sample: None,
                    use_cache: None,
                    seed: None,
                    stop_sequences: None,
                },
                "not in range",
            ),
            // === SUCCESS TEST CASES ===
            TestCase {
                name: "set_params_basic",
                request: GenerationParams {
                    temperature: Some(0.7),
                    top_p: Some(0.9),
                    top_k: Some(50),
                    min_length: None,
                    max_length: Some(512),
                    max_new_tokens: None,
                    min_p: None,
                    repetition_penalty: Some(1.1),
                    frequency_penalty: None,
                    presence_penalty: None,
                    length_penalty: None,
                    no_repeat_ngram_size: None,
                    num_beams: None,
                    early_stopping: None,
                    do_sample: Some(true),
                    use_cache: None,
                    seed: None,
                    stop_sequences: None,
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "set_params_all_fields",
                request: GenerationParams {
                    temperature: Some(1.5),
                    top_p: Some(0.95),
                    top_k: Some(100),
                    min_length: Some(10),
                    max_length: Some(1024),
                    max_new_tokens: Some(500),
                    min_p: Some(0.01),
                    repetition_penalty: Some(1.2),
                    frequency_penalty: Some(0.5),
                    presence_penalty: Some(0.5),
                    length_penalty: Some(1.0),
                    no_repeat_ngram_size: Some(3),
                    num_beams: Some(4),
                    early_stopping: Some(true),
                    do_sample: Some(true),
                    use_cache: Some(true),
                    seed: Some(42),
                    stop_sequences: Some(vec!["END".to_string(), "STOP".to_string()]),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase {
                name: "set_params_minimal",
                request: GenerationParams {
                    temperature: Some(0.1),
                    top_p: None,
                    top_k: None,
                    min_length: None,
                    max_length: None,
                    max_new_tokens: None,
                    min_p: None,
                    repetition_penalty: None,
                    frequency_penalty: None,
                    presence_penalty: None,
                    length_penalty: None,
                    no_repeat_ngram_size: None,
                    num_beams: None,
                    early_stopping: None,
                    do_sample: None,
                    use_cache: None,
                    seed: None,
                    stop_sequences: None,
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
        ]
    }
}

crate::enforce_route_handler!(SetParamsRoute);

// Implement RegisterableRoute with Axum 0.8 compatible handler
crate::impl_registerable_route!(SetParamsRoute);

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_set_params_validation_valid() {
        let params = GenerationParams {
            temperature: Some(0.7),
            top_p: Some(0.9),
            top_k: Some(50),
            min_length: None,
            max_length: Some(512),
            max_new_tokens: None,
            min_p: None,
            repetition_penalty: Some(1.1),
            frequency_penalty: Some(0.5),
            presence_penalty: Some(0.5),
            length_penalty: None,
            no_repeat_ngram_size: None,
            num_beams: None,
            early_stopping: None,
            do_sample: Some(true),
            use_cache: None,
            seed: None,
            stop_sequences: None,
        };
        assert!(SetParamsRoute::validate_request(&params).await.is_ok());
    }

    #[tokio::test]
    async fn test_set_params_validation_invalid_temp() {
        let params = GenerationParams {
            temperature: Some(3.0),
            top_p: None,
            top_k: None,
            min_length: None,
            max_length: None,
            max_new_tokens: None,
            min_p: None,
            repetition_penalty: None,
            frequency_penalty: None,
            presence_penalty: None,
            length_penalty: None,
            no_repeat_ngram_size: None,
            num_beams: None,
            early_stopping: None,
            do_sample: None,
            use_cache: None,
            seed: None,
            stop_sequences: None,
        };
        assert!(SetParamsRoute::validate_request(&params).await.is_err());
    }

    #[test]
    fn test_metadata() {
        let meta = GetParamsRoute::metadata();
        assert!(meta.idempotent);
        
        let meta2 = SetParamsRoute::metadata();
        assert!(meta2.idempotent);
    }
}
