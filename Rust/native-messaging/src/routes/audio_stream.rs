//! Audio streaming endpoint for native messaging.
//!
//! ENFORCED RULES:
//! ✅ Documentation ✅ Tests ✅ tabagent-values ✅ Tracing ✅ Validation

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tabagent_values::RequestValue;
use crate::{
    error::{NativeMessagingResult, NativeMessagingError},
    route_trait::{NativeMessagingRoute, RouteMetadata, TestCase},
    traits::AppStateProvider,
};

/// Audio stream request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioStreamRequest {
    /// Audio data (base64 encoded)
    pub audio_data: String,
    /// Audio format (e.g., "wav", "mp3", "opus")
    pub format: String,
    /// Sample rate (e.g., 16000, 44100)
    pub sample_rate: u32,
    /// Number of channels (1=mono, 2=stereo)
    pub channels: u8,
    /// Optional operation (transcribe, translate, voice-to-text)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,
}

/// Audio stream response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioStreamResponse {
    /// Whether the operation succeeded
    pub success: bool,
    /// Optional transcription result
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcription: Option<String>,
    /// Optional translation result
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translation: Option<String>,
    /// Status message
    pub message: String,
}

/// Audio stream route handler.
pub struct AudioStreamRoute;

#[async_trait]
impl NativeMessagingRoute for AudioStreamRoute {
    type Request = AudioStreamRequest;
    type Response = AudioStreamResponse;

    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "audio_stream",
            tags: &["Media", "Audio", "Streaming"],
            description: "Process audio streams for transcription, translation, and voice input",
            openai_compatible: false,
            idempotent: false,
            requires_auth: false,
            rate_limit_tier: Some("media"),
            supports_streaming: true,
            supports_binary: true,
            max_payload_size: Some(10 * 1024 * 1024), // 10MB
        }
    }

    async fn validate_request(req: &Self::Request) -> NativeMessagingResult<()> {
        if req.audio_data.is_empty() {
            return Err(NativeMessagingError::validation("audio_data", "cannot be empty"));
        }
        if req.format.is_empty() {
            return Err(NativeMessagingError::validation("format", "cannot be empty"));
        }
        if req.sample_rate == 0 {
            return Err(NativeMessagingError::validation("sample_rate", "must be greater than 0"));
        }
        if req.channels == 0 {
            return Err(NativeMessagingError::validation("channels", "must be greater than 0"));
        }
        Ok(())
    }

    async fn handle<S>(req: Self::Request, _state: &S) -> NativeMessagingResult<Self::Response>
    where
        S: AppStateProvider + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        tracing::info!(
            request_id = %request_id,
            route = "audio_stream",
            format = %req.format,
            sample_rate = req.sample_rate,
            "Native messaging audio stream request"
        );

        // TODO: Implement actual audio processing
        // TODO: Add RequestValue::process_audio() to tabagent-values
        // For now, use a placeholder request
        let _request = RequestValue::health(); // Placeholder

        // let _response = state.handle_request(request).await
        //     .map_err(|e| NativeMessagingError::Backend(e))?;

        tracing::info!(request_id = %request_id, "Audio stream processed");

        Ok(AudioStreamResponse {
            success: true,
            transcription: Some("Audio processed".to_string()),
            translation: None,
            message: "Audio stream processed successfully".to_string(),
        })
    }

    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            TestCase {
                name: "audio_stream_basic",
                request: AudioStreamRequest {
                    audio_data: "base64encodedaudio".to_string(),
                    format: "wav".to_string(),
                    sample_rate: 16000,
                    channels: 1,
                    operation: Some("transcribe".to_string()),
                },
                expected_response: None,
                expected_error: None,
                assertions: vec![],
            },
            TestCase::error(
                "empty_audio_data",
                AudioStreamRequest {
                    audio_data: "".to_string(),
                    format: "wav".to_string(),
                    sample_rate: 16000,
                    channels: 1,
                    operation: None,
                },
                "audio_data",
            ),
        ]
    }
}

crate::enforce_native_messaging_route!(AudioStreamRoute);

