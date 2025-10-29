//! Audio streaming route for WebRTC.
//!
//! Reference implementation showing media route enforcement with audio codecs.

use crate::error::{WebRtcError, WebRtcResult};
use crate::route_trait::{DataChannelRoute, RouteMetadata, TestCase, MediaType};
use crate::route_trait::validators::{ValidAudioCodec, ValidBitrate};
use crate::traits::RequestHandler;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tabagent_values::RequestValue;

/// Audio stream configuration request
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AudioStreamRequest {
    /// Audio codec (opus, g722, pcmu, pcma, aac)
    pub codec: String,
    
    /// Sample rate in Hz (8000, 16000, 48000)
    pub sample_rate: u32,
    
    /// Bitrate in bits per second
    pub bitrate: u32,
    
    /// Number of channels (1 = mono, 2 = stereo)
    pub channels: u8,
}

/// Audio stream response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioStreamResponse {
    /// Stream ID
    pub stream_id: String,
    
    /// Actual codec being used
    pub codec: String,
    
    /// Actual sample rate
    pub sample_rate: u32,
    
    /// Actual bitrate
    pub bitrate: u32,
    
    /// Actual channels
    pub channels: u8,
    
    /// Status message
    pub status: String,
}

/// Audio streaming route handler
pub struct AudioStreamRoute;

#[async_trait]
impl DataChannelRoute for AudioStreamRoute {
    type Request = AudioStreamRequest;
    type Response = AudioStreamResponse;
    
    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "audio_stream",
            tags: &["Media", "Audio", "WebRTC"],
            description: "Configure and start audio streaming over WebRTC with codec validation and quality control",
            supports_streaming: true,
            supports_binary: true,
            requires_auth: true,
            rate_limit_tier: Some("media"),
            max_payload_size: Some(1024 * 1024), // 1MB for audio chunks
            media_type: Some(MediaType::Audio),
        }
    }
    
    async fn validate_request(req: &Self::Request) -> WebRtcResult<()> {
        let request_id = uuid::Uuid::new_v4();
        tracing::debug!(
            request_id = %request_id,
            codec = %req.codec,
            sample_rate = req.sample_rate,
            bitrate = req.bitrate,
            channels = req.channels,
            "Validating audio stream request"
        );
        
        // Validate codec
        use crate::route_trait::ValidationRule;
        ValidAudioCodec.validate_field("codec", &req.codec)?;
        
        // Validate sample rate
        const VALID_SAMPLE_RATES: &[u32] = &[8000, 16000, 24000, 48000];
        if !VALID_SAMPLE_RATES.contains(&req.sample_rate) {
            return Err(WebRtcError::ValidationError {
                field: "sample_rate".to_string(),
                message: format!(
                    "Invalid sample rate: {} Hz. Valid rates: {:?}",
                    req.sample_rate, VALID_SAMPLE_RATES
                ),
            });
        }
        
        // Validate bitrate
        ValidBitrate.validate_field("bitrate", &req.bitrate)?;
        
        // Validate channels
        if req.channels < 1 || req.channels > 8 {
            return Err(WebRtcError::ValidationError {
                field: "channels".to_string(),
                message: format!(
                    "Invalid channel count: {} (must be 1-8)",
                    req.channels
                ),
            });
        }
        
        // Business logic: Opus at 48kHz should have higher bitrate
        if req.codec.to_lowercase() == "opus" && req.sample_rate == 48000 && req.bitrate < 96_000 {
            tracing::warn!(
                request_id = %request_id,
                "Opus at 48kHz with bitrate {} is lower than recommended 96kbps",
                req.bitrate
            );
        }
        
        tracing::debug!(
            request_id = %request_id,
            "Audio stream request validation passed"
        );
        
        Ok(())
    }
    
    async fn handle<H>(req: Self::Request, handler: &H) -> WebRtcResult<Self::Response>
    where
        H: RequestHandler + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        
        tracing::info!(
            request_id = %request_id,
            codec = %req.codec,
            sample_rate = req.sample_rate,
            bitrate = req.bitrate,
            channels = req.channels,
            "Starting audio stream configuration"
        );
        
        // Forward to appstate for real media pipeline handling
        let request_value = RequestValue::from_json(&serde_json::to_string(&serde_json::json!({
            "action": "audio_stream",
            "codec": req.codec,
            "sample_rate": req.sample_rate,
            "bitrate": req.bitrate,
            "channels": req.channels
        })).map_err(|e| WebRtcError::InternalError(e.to_string()))?)
            .map_err(|e| WebRtcError::InternalError(format!("Failed to create request: {}", e)))?;

        let response = handler.handle_request(request_value).await
            .map_err(|e| {
                tracing::error!(request_id = %request_id, error = %e, "Audio stream request failed");
                WebRtcError::from(e)
            })?;

        let json_str = response.to_json()
            .map_err(|e| WebRtcError::InternalError(format!("Failed to serialize response: {}", e)))?;
        let data: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| WebRtcError::InternalError(e.to_string()))?;
        
        let stream_id = data["stream_id"].as_str().unwrap_or("unknown").to_string();
        
        tracing::info!(
            request_id = %request_id,
            stream_id = %stream_id,
            "Audio stream configuration successful"
        );
        
        Ok(AudioStreamResponse {
            stream_id,
            codec: req.codec,
            sample_rate: req.sample_rate,
            bitrate: req.bitrate,
            channels: req.channels,
            status: data["status"].as_str().unwrap_or("configured").to_string(),
        })
    }
    
    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            // Test case 1: Valid Opus stereo stream
            TestCase::success(
                "valid_opus_stereo",
                AudioStreamRequest {
                    codec: "opus".to_string(),
                    sample_rate: 48000,
                    bitrate: 128_000,
                    channels: 2,
                },
                AudioStreamResponse {
                    stream_id: "test-audio-1".to_string(),
                    codec: "opus".to_string(),
                    sample_rate: 48000,
                    bitrate: 128_000,
                    channels: 2,
                    status: "Audio stream configured".to_string(),
                },
            ),
            
            // Test case 2: Valid G.722 mono stream
            TestCase::success(
                "valid_g722_mono",
                AudioStreamRequest {
                    codec: "g722".to_string(),
                    sample_rate: 16000,
                    bitrate: 64_000,
                    channels: 1,
                },
                AudioStreamResponse {
                    stream_id: "test-audio-2".to_string(),
                    codec: "g722".to_string(),
                    sample_rate: 16000,
                    bitrate: 64_000,
                    channels: 1,
                    status: "Audio stream configured".to_string(),
                },
            ),
            
            // Test case 3: Invalid codec
            TestCase::error(
                "invalid_codec",
                AudioStreamRequest {
                    codec: "mp3".to_string(), // Not supported for WebRTC
                    sample_rate: 48000,
                    bitrate: 128_000,
                    channels: 2,
                },
                "Unsupported audio codec",
            ),
            
            // Test case 4: Invalid sample rate
            TestCase::error(
                "invalid_sample_rate",
                AudioStreamRequest {
                    codec: "opus".to_string(),
                    sample_rate: 44100, // Not standard for WebRTC
                    bitrate: 128_000,
                    channels: 2,
                },
                "Invalid sample rate",
            ),
            
            // Test case 5: Invalid channel count
            TestCase::error(
                "invalid_channels",
                AudioStreamRequest {
                    codec: "opus".to_string(),
                    sample_rate: 48000,
                    bitrate: 128_000,
                    channels: 0, // Invalid
                },
                "Invalid channel count",
            ),
        ]
    }
}

// COMPILE-TIME ENFORCEMENT
crate::enforce_data_channel_route!(AudioStreamRoute);

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_validate_valid_opus() {
        let req = AudioStreamRequest {
            codec: "opus".to_string(),
            sample_rate: 48000,
            bitrate: 128_000,
            channels: 2,
        };
        
        let result = AudioStreamRoute::validate_request(&req).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_validate_invalid_codec() {
        let req = AudioStreamRequest {
            codec: "mp3".to_string(),
            sample_rate: 48000,
            bitrate: 128_000,
            channels: 2,
        };
        
        let result = AudioStreamRoute::validate_request(&req).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_validate_invalid_sample_rate() {
        let req = AudioStreamRequest {
            codec: "opus".to_string(),
            sample_rate: 44100,
            bitrate: 128_000,
            channels: 2,
        };
        
        let result = AudioStreamRoute::validate_request(&req).await;
        assert!(result.is_err());
    }
    
    #[test]
    fn test_metadata() {
        let metadata = AudioStreamRoute::metadata();
        assert_eq!(metadata.route_id, "audio_stream");
        assert_eq!(metadata.media_type, Some(MediaType::Audio));
        assert!(!metadata.description.is_empty());
    }
}

