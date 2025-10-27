//! ONNX Pipeline implementation
//!
//! Implements Pipeline trait for ONNX models, composing:
//! - model-cache (download/detection)
//! - onnx-loader (inference)

use crate::base::Pipeline;
use crate::error::Result;
use crate::types::PipelineType;
use serde_json::Value;
use tabagent_onnx_loader::OnnxSession;

pub struct OnnxPipeline {
    session: Option<OnnxSession>,
    pipeline_type: PipelineType,
}

impl OnnxPipeline {
    pub fn new(pipeline_type: PipelineType) -> Self {
        Self {
            session: None,
            pipeline_type,
        }
    }
}

impl Pipeline for OnnxPipeline {
    fn pipeline_type(&self) -> PipelineType {
        self.pipeline_type
    }
    
    fn is_loaded(&self) -> bool {
        self.session.is_some()
    }
    
    fn load(&mut self, model_id: &str, options: Option<Value>) -> Result<()> {
        log::info!("Loading ONNX model: {}", model_id);
        
        // Extract options
        let opts = options.unwrap_or(Value::Null);
        let model_path = opts["model_path"]
            .as_str()
            .unwrap_or(model_id);
        let tokenizer_path = opts["tokenizer_path"].as_str();
        
        // Load ONNX session
        let mut session = OnnxSession::load(model_path)
            .map_err(|e| crate::error::PipelineError::BackendError(e.to_string()))?;
        
        // Load tokenizer if provided
        if let Some(tok_path) = tokenizer_path {
            session.load_tokenizer(tok_path)
                .map_err(|e| crate::error::PipelineError::BackendError(e.to_string()))?;
        }
        
        self.session = Some(session);
        log::info!("ONNX model loaded successfully");
        
        Ok(())
    }
    
    fn generate(&self, input: Value) -> Result<Value> {
        use tabagent_onnx_loader::text_generation::GenerationConfig;
        
        let session = self.session.as_ref()
            .ok_or(crate::error::PipelineError::ModelNotLoaded)?;
        
        // Extract input based on pipeline type
        match self.pipeline_type {
            PipelineType::TextGeneration | PipelineType::FeatureExtraction => {
                let prompt = input.get("prompt")
                    .or_else(|| input.get("text"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| crate::error::PipelineError::InvalidInput(
                        "prompt or text is required".to_string()
                    ))?;
                
                // Handle embeddings vs generation
                if matches!(self.pipeline_type, PipelineType::FeatureExtraction) {
                    let embedding = session.generate_embedding(prompt)
                        .map_err(|e| crate::error::PipelineError::BackendError(e.to_string()))?;
                    
                    return Ok(serde_json::json!({
                        "embedding": embedding
                    }));
                }
                
                // Text generation
                let config = GenerationConfig {
                    max_new_tokens: input.get("maxTokens")
                        .or_else(|| input.get("max_tokens"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(512) as usize,
                    temperature: input.get("temperature")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(1.0) as f32,
                    top_k: input.get("topK")
                        .or_else(|| input.get("top_k"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(50) as usize,
                    top_p: input.get("topP")
                        .or_else(|| input.get("top_p"))
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.9) as f32,
                    do_sample: input.get("doSample")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true),
                    repetition_penalty: input.get("repetitionPenalty")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(1.1) as f32,
                };
                
                let output = session.generate_text(prompt, &config)
                    .map_err(|e| crate::error::PipelineError::BackendError(e.to_string()))?;
                
                Ok(serde_json::json!({
                    "text": output
                }))
            }
            _ => {
                Err(crate::error::PipelineError::UnsupportedPipelineType(
                    format!("Pipeline type {:?} not yet implemented for ONNX", self.pipeline_type)
                ))
            }
        }
    }
    
    fn unload(&mut self) -> Result<()> {
        self.session = None;
        log::info!("ONNX model unloaded");
        Ok(())
    }
}

