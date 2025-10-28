//! Video streaming route for WebRTC.
//!
//! This is a reference implementation showing how to properly implement
//! a DataChannelRoute with full enforcement compliance.

use crate::error::{WebRtcError, WebRtcResult};
use crate::route_trait::{DataChannelRoute, RouteMetadata, TestCase, MediaType};
use crate::route_trait::validators::{ValidVideoCodec, ValidResolution, ValidBitrate, ValidFramerate};
use crate::traits::RequestHandler;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Video stream configuration request
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VideoStreamRequest {
    /// Video codec (h264, vp8, vp9, av1, hevc)
    pub codec: String,
    
    /// Resolution (width, height)
    pub resolution: (u32, u32),
    
    /// Target bitrate in bits per second
    pub bitrate: u32,
    
    /// Target framerate in frames per second
    pub framerate: u32,
    
    /// Enable hardware acceleration
    #[serde(default)]
    pub hardware_acceleration: bool,
}

/// Video stream response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoStreamResponse {
    /// Stream ID
    pub stream_id: String,
    
    /// Actual codec being used
    pub codec: String,
    
    /// Actual resolution
    pub resolution: (u32, u32),
    
    /// Actual bitrate
    pub bitrate: u32,
    
    /// Actual framerate
    pub framerate: u32,
    
    /// Status message
    pub status: String,
}

/// Video streaming route handler
pub struct VideoStreamRoute;

#[async_trait]
impl DataChannelRoute for VideoStreamRoute {
    type Request = VideoStreamRequest;
    type Response = VideoStreamResponse;
    
    fn metadata() -> RouteMetadata {
        RouteMetadata {
            route_id: "video_stream",
            tags: &["Media", "Video", "WebRTC"],
            description: "Configure and start video streaming over WebRTC data channel with codec validation",
            supports_streaming: true,
            supports_binary: true,
            requires_auth: true,
            rate_limit_tier: Some("media"),
            max_payload_size: Some(10 * 1024 * 1024), // 10MB for video frames
            media_type: Some(MediaType::Video),
        }
    }
    
    async fn validate_request(req: &Self::Request) -> WebRtcResult<()> {
        let request_id = uuid::Uuid::new_v4();
        tracing::debug!(
            request_id = %request_id,
            codec = %req.codec,
            resolution = ?req.resolution,
            bitrate = req.bitrate,
            framerate = req.framerate,
            "Validating video stream request"
        );
        
        // Validate codec
        use crate::route_trait::ValidationRule;
        ValidVideoCodec.validate_field("codec", &req.codec)?;
        
        // Validate resolution
        ValidResolution.validate_field("resolution", &req.resolution)?;
        
        // Validate bitrate
        ValidBitrate.validate_field("bitrate", &req.bitrate)?;
        
        // Validate framerate
        ValidFramerate.validate_field("framerate", &req.framerate)?;
        
        // Business logic validation: check if bitrate is reasonable for resolution
        let (width, height) = req.resolution;
        let pixels = width * height;
        let min_bitrate = pixels / 10; // Rough minimum: 0.1 bpp
        
        if req.bitrate < min_bitrate {
            return Err(WebRtcError::ValidationError {
                field: "bitrate".to_string(),
                message: format!(
                    "Bitrate {} bps is too low for {}x{} resolution (recommended minimum: {} bps)",
                    req.bitrate, width, height, min_bitrate
                ),
            });
        }
        
        tracing::debug!(
            request_id = %request_id,
            "Video stream request validation passed"
        );
        
        Ok(())
    }
    
    async fn handle<H>(req: Self::Request, _handler: &H) -> WebRtcResult<Self::Response>
    where
        H: RequestHandler + Send + Sync,
    {
        let request_id = uuid::Uuid::new_v4();
        
        tracing::info!(
            request_id = %request_id,
            codec = %req.codec,
            resolution = ?req.resolution,
            bitrate = req.bitrate,
            framerate = req.framerate,
            hw_accel = req.hardware_acceleration,
            "Starting video stream configuration"
        );
        
        // For now, this is a mock implementation
        // In production, this would:
        // 1. Initialize video encoder with specified codec
        // 2. Configure encoder with resolution, bitrate, framerate
        // 3. Set up hardware acceleration if requested
        // 4. Create stream ID and register with media pipeline
        // 5. Return stream configuration
        
        let stream_id = uuid::Uuid::new_v4().to_string();
        
        let response = VideoStreamResponse {
            stream_id: stream_id.clone(),
            codec: req.codec.clone(),
            resolution: req.resolution,
            bitrate: req.bitrate,
            framerate: req.framerate,
            status: format!(
                "Video stream {} configured: {} @ {}x{}, {} bps, {} fps",
                stream_id,
                req.codec,
                req.resolution.0,
                req.resolution.1,
                req.bitrate,
                req.framerate
            ),
        };
        
        tracing::info!(
            request_id = %request_id,
            stream_id = %stream_id,
            "Video stream configuration successful"
        );
        
        Ok(response)
    }
    
    fn test_cases() -> Vec<TestCase<Self::Request, Self::Response>> {
        vec![
            // Test case 1: Valid 1080p H.264 stream
            TestCase::success(
                "valid_1080p_h264",
                VideoStreamRequest {
                    codec: "h264".to_string(),
                    resolution: (1920, 1080),
                    bitrate: 5_000_000, // 5 Mbps
                    framerate: 30,
                    hardware_acceleration: true,
                },
                VideoStreamResponse {
                    stream_id: "test-stream-1".to_string(),
                    codec: "h264".to_string(),
                    resolution: (1920, 1080),
                    bitrate: 5_000_000,
                    framerate: 30,
                    status: "Video stream configured".to_string(),
                },
            ),
            
            // Test case 2: Valid 4K VP9 stream
            TestCase::success(
                "valid_4k_vp9",
                VideoStreamRequest {
                    codec: "vp9".to_string(),
                    resolution: (3840, 2160),
                    bitrate: 20_000_000, // 20 Mbps
                    framerate: 60,
                    hardware_acceleration: true,
                },
                VideoStreamResponse {
                    stream_id: "test-stream-2".to_string(),
                    codec: "vp9".to_string(),
                    resolution: (3840, 2160),
                    bitrate: 20_000_000,
                    framerate: 60,
                    status: "Video stream configured".to_string(),
                },
            ),
            
            // Test case 3: Invalid codec should fail validation
            TestCase::error(
                "invalid_codec",
                VideoStreamRequest {
                    codec: "invalid_codec".to_string(),
                    resolution: (1920, 1080),
                    bitrate: 5_000_000,
                    framerate: 30,
                    hardware_acceleration: false,
                },
                "Unsupported video codec",
            ),
            
            // Test case 4: Resolution too small should fail
            TestCase::error(
                "resolution_too_small",
                VideoStreamRequest {
                    codec: "h264".to_string(),
                    resolution: (100, 100), // Too small
                    bitrate: 1_000_000,
                    framerate: 30,
                    hardware_acceleration: false,
                },
                "Resolution too low",
            ),
            
            // Test case 5: Bitrate too high should fail
            TestCase::error(
                "bitrate_too_high",
                VideoStreamRequest {
                    codec: "h264".to_string(),
                    resolution: (1920, 1080),
                    bitrate: 200_000_000, // 200 Mbps - too high
                    framerate: 30,
                    hardware_acceleration: false,
                },
                "Bitrate too high",
            ),
            
            // Test case 6: Invalid framerate should fail
            TestCase::error(
                "invalid_framerate",
                VideoStreamRequest {
                    codec: "h264".to_string(),
                    resolution: (1920, 1080),
                    bitrate: 5_000_000,
                    framerate: 0, // Invalid
                    hardware_acceleration: false,
                },
                "Invalid framerate",
            ),
        ]
    }
}

// COMPILE-TIME ENFORCEMENT: This will panic at compile time if any rules are violated!
crate::enforce_data_channel_route!(VideoStreamRoute);

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_validate_valid_request() {
        let req = VideoStreamRequest {
            codec: "h264".to_string(),
            resolution: (1920, 1080),
            bitrate: 5_000_000,
            framerate: 30,
            hardware_acceleration: true,
        };
        
        let result = VideoStreamRoute::validate_request(&req).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_validate_invalid_codec() {
        let req = VideoStreamRequest {
            codec: "invalid".to_string(),
            resolution: (1920, 1080),
            bitrate: 5_000_000,
            framerate: 30,
            hardware_acceleration: true,
        };
        
        let result = VideoStreamRoute::validate_request(&req).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), WebRtcError::ValidationError { .. }));
    }
    
    #[tokio::test]
    async fn test_validate_invalid_resolution() {
        let req = VideoStreamRequest {
            codec: "h264".to_string(),
            resolution: (100, 100), // Too small
            bitrate: 5_000_000,
            framerate: 30,
            hardware_acceleration: true,
        };
        
        let result = VideoStreamRoute::validate_request(&req).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_validate_bitrate_too_low_for_resolution() {
        let req = VideoStreamRequest {
            codec: "h264".to_string(),
            resolution: (3840, 2160), // 4K
            bitrate: 100_000, // Way too low for 4K
            framerate: 30,
            hardware_acceleration: true,
        };
        
        let result = VideoStreamRoute::validate_request(&req).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), WebRtcError::ValidationError { .. }));
    }
    
    #[test]
    fn test_metadata() {
        let metadata = VideoStreamRoute::metadata();
        assert_eq!(metadata.route_id, "video_stream");
        assert!(metadata.supports_streaming);
        assert!(metadata.supports_binary);
        assert_eq!(metadata.media_type, Some(MediaType::Video));
        assert!(!metadata.description.is_empty());
    }
    
    #[test]
    fn test_has_test_cases() {
        let test_cases = VideoStreamRoute::test_cases();
        assert!(!test_cases.is_empty(), "Route must have at least one test case");
        assert!(test_cases.len() >= 3, "Should have multiple test cases for different scenarios");
    }
}

