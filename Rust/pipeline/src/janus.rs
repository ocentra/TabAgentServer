/// Janus Pipeline Handler
///
/// Architecture-specific logic for Janus multimodal models.

use crate::error::{Result, PipelineError};
use crate::types::Architecture;
use tabagent_model_cache::detection::ModelInfo as DetectionModelInfo;

/// Janus pipeline handler
pub struct JanusHandler;

impl JanusHandler {
    pub fn new() -> Self {
        Self
    }
    
    pub fn validate_model(model_info: &DetectionModelInfo) -> Result<()> {
        if let Some(arch_str) = &model_info.architecture {
            let arch = Architecture::from_str(arch_str)
                .unwrap_or(Architecture::Generic);
            
            if arch != Architecture::Janus {
                return Err(PipelineError::InvalidArchitecture {
                    expected: Architecture::Janus,
                    actual: arch,
                });
            }
        } else {
            return Err(PipelineError::InvalidArchitecture {
                expected: Architecture::Janus,
                actual: Architecture::Generic,
            });
        }
        Ok(())
    }
    
    pub fn get_model_config_hints() -> Vec<(&'static str, &'static str)> {
        vec![
            ("task", "image-to-text"),
            ("supports_vision", "true"),
            ("architecture", "janus"),
        ]
    }
}

