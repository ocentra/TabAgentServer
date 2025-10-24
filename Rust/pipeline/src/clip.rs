/// CLIP Pipeline Handler
///
/// Architecture-specific logic for CLIP dual-encoder models.
/// Handles embeddings and zero-shot classification.

use crate::error::{Result, PipelineError};
use crate::types::Architecture;
use tabagent_model_cache::detection::ModelInfo as DetectionModelInfo;

/// CLIP pipeline handler
///
/// Handles CLIP-specific:
/// - Dual-encoder architecture (text + vision)
/// - Embedding generation
/// - Zero-shot classification
pub struct ClipHandler;

impl ClipHandler {
    /// Create new CLIP handler
    pub fn new() -> Self {
        Self
    }
    
    /// Validate CLIP model info
    pub fn validate_model(model_info: &DetectionModelInfo) -> Result<()> {
        if let Some(arch_str) = &model_info.architecture {
            let arch = Architecture::from_str(arch_str)
                .unwrap_or(Architecture::Generic);
            
            if arch != Architecture::Clip && arch != Architecture::Clap {
                return Err(PipelineError::InvalidArchitecture {
                    expected: Architecture::Clip,
                    actual: arch,
                });
            }
        } else {
            return Err(PipelineError::InvalidArchitecture {
                expected: Architecture::Clip,
                actual: Architecture::Generic,
            });
        }
        
        Ok(())
    }
    
    /// Get model-specific configuration hints
    pub fn get_model_config_hints() -> Vec<(&'static str, &'static str)> {
        vec![
            ("task", "feature-extraction"),
            ("supports_vision", "true"),
            ("supports_text", "true"),
            ("output_type", "embeddings"),
        ]
    }
}

