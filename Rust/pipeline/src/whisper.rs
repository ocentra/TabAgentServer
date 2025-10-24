/// Whisper Pipeline Handler
///
/// Architecture-specific logic for Whisper speech recognition models.
/// Handles audio preprocessing and ASR-specific configuration.

use crate::error::{Result, PipelineError};
use crate::types::Architecture;
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
        if let Some(arch_str) = &model_info.architecture {
            let arch = Architecture::from_str(arch_str)
                .unwrap_or(Architecture::Generic);
            
            if arch != Architecture::Whisper && arch != Architecture::Moonshine {
                return Err(PipelineError::InvalidArchitecture {
                    expected: Architecture::Whisper,
                    actual: arch,
                });
            }
        } else {
            return Err(PipelineError::InvalidArchitecture {
                expected: Architecture::Whisper,
                actual: Architecture::Generic,
            });
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

