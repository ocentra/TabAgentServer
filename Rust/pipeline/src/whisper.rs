/// Whisper Pipeline Handler
///
/// Architecture-specific logic for Whisper speech recognition models.
/// Handles audio preprocessing and ASR-specific configuration.

use crate::error::{Result, PipelineError};
use tabagent_model_cache::detection::ModelInfo as DetectionModelInfo;

/// Whisper pipeline handler
///
/// Handles Whisper-specific:
/// - Audio sampling rate normalization (16kHz)
/// - Language detection
/// - Timestamp generation
pub struct WhisperHandler;

impl WhisperHandler {
    /// Create new Whisper handler
    pub fn new() -> Self {
        Self
    }
    
    /// Validate Whisper model info
    pub fn validate_model(model_info: &DetectionModelInfo) -> Result<()> {
        if let Some(arch) = &model_info.architecture {
            let arch_lower = arch.to_lowercase();
            if arch_lower != "whisper" && arch_lower != "moonshine" {
                return Err(PipelineError::InvalidArchitecture(
                    format!("Expected Whisper, got: {}", arch)
                ));
            }
        } else {
            return Err(PipelineError::InvalidArchitecture(
                "No architecture specified for Whisper".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Get recommended sampling rate
    pub fn get_sampling_rate() -> u32 {
        16000  // Whisper expects 16kHz audio
    }
    
    /// Get model-specific configuration hints
    pub fn get_model_config_hints() -> Vec<(&'static str, &'static str)> {
        vec![
            ("sampling_rate", "16000"),
            ("task", "transcribe"),
            ("language", "auto"),
            ("return_timestamps", "false"),
        ]
    }
}

