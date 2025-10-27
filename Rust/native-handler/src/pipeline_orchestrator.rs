use crate::error::Result;
use crate::state;
use serde_json::{json, Value};
use tabagent_model_cache::{detect_from_file_path, detect_from_repo_name, ModelInfo as DetectionModelInfo};
use tabagent_hardware::detect_system;
use tabagent_execution_providers::{CPUExecutionProvider, CUDAExecutionProvider, TensorRTExecutionProvider, DirectMLExecutionProvider};
use tabagent_pipeline::{OnnxPipeline, Pipeline, PipelineFactory, PipelineType};
use std::sync::{Arc, Mutex};

pub struct PipelineOrchestrator;

impl PipelineOrchestrator {
    /// Load model: detect → hardware → providers → pipeline → store
    pub async fn load_model(model_id: &str, source: &str, options: Option<Value>) -> Result<Value> {
        // 1. Detect model
        let model_info = detect_from_file_path(source)
            .or_else(|| detect_from_repo_name(source))
            .ok_or_else(|| format!("Could not detect model type for: {}", source))?;
        
        log::info!("Detected model: type={:?}, task={:?}, arch={:?}", 
            model_info.model_type, model_info.task, model_info.architecture);
        
        // 2. Detect hardware
        let hw = detect_system()
            .map_err(|e| format!("Hardware detection failed: {}", e))?;
        
        log::info!("Hardware: CPU={}, GPUs={}", hw.cpu.model, hw.gpus.len());
        
        // 3. Select execution providers based on hardware
        let providers = Self::select_providers(&hw)?;
        log::info!("Selected {} execution providers", providers.len());
        
        // 4. Get pipeline type
        let pipeline_type = PipelineFactory::get_pipeline_type(&model_info);
        log::info!("Pipeline type: {:?}", pipeline_type);
        
        // 5. Create appropriate pipeline
        let pipeline = Self::create_pipeline(&model_info, pipeline_type)?;
        
        // 6. Load model into pipeline
        let load_options = json!({
            "model_path": source,
            "tokenizer_path": options.as_ref()
                .and_then(|o| o.get("tokenizer_path"))
                .and_then(|v| v.as_str()),
            "execution_providers": providers.iter().map(|p| p.name()).collect::<Vec<_>>(),
        });
        
        pipeline.lock().unwrap().load(source, Some(load_options))
            .map_err(|e| format!("Pipeline load failed: {}", e))?;
        
        // 7. Store in state
        state::store_pipeline(model_id, pipeline.clone());
        
        Ok(json!({
            "status": "success",
            "model_id": model_id,
            "model_type": format!("{:?}", model_info.model_type),
            "pipeline_type": pipeline_type.to_hf_tag(),
            "task": model_info.task,
            "architecture": model_info.architecture,
            "backend": format!("{:?}", model_info.backend),
        }))
    }
    
    /// Generate: get pipeline → route by type → generate
    pub async fn generate(model_id: &str, input: Value) -> Result<Value> {
        // Get pipeline from state
        let pipeline = state::get_pipeline(model_id)
            .ok_or_else(|| format!("Model not loaded: {}", model_id))?;
        
        // Call pipeline generate
        let output = pipeline.lock().unwrap().generate(input)
            .map_err(|e| format!("Generation failed: {}", e))?;
        
        Ok(output)
    }
    
    /// Unload model
    pub async fn unload_model(model_id: &str) -> Result<Value> {
        if let Some(pipeline) = state::get_pipeline(model_id) {
            pipeline.lock().unwrap().unload()
                .map_err(|e| format!("Unload failed: {}", e))?;
            state::remove_pipeline(model_id);
        }
        
        Ok(json!({
            "status": "success",
            "message": format!("Model {} unloaded", model_id)
        }))
    }
    
    fn select_providers(hw: &tabagent_hardware::SystemInfo) -> Result<Vec<Arc<dyn tabagent_execution_providers::ExecutionProvider>>> {
        use tabagent_hardware::GpuVendor;
        
        let mut providers: Vec<Arc<dyn tabagent_execution_providers::ExecutionProvider>> = Vec::new();
        
        // GPU providers
        if let Some(gpu) = hw.gpus.first() {
            match gpu.vendor {
                GpuVendor::Nvidia => {
                    providers.push(
                        TensorRTExecutionProvider::new()
                            .with_fp16_enable(true)
                            .with_engine_cache_enable(true)
                            .build()
                    );
                    providers.push(
                        CUDAExecutionProvider::new()
                            .with_device_id(0)
                            .build()
                    );
                }
                GpuVendor::Amd => {
                    #[cfg(target_os = "windows")]
                    providers.push(DirectMLExecutionProvider::new().build());
                }
                GpuVendor::Intel => {
                    #[cfg(target_os = "windows")]
                    providers.push(DirectMLExecutionProvider::new().build());
                }
                _ => {}
            }
        }
        
        // CPU fallback
        providers.push(CPUExecutionProvider::new().build());
        
        Ok(providers)
    }
    
    fn create_pipeline(
        model_info: &DetectionModelInfo,
        pipeline_type: PipelineType,
    ) -> Result<Arc<Mutex<Box<dyn Pipeline>>>> {
        use tabagent_model_cache::ModelType;
        
        match model_info.model_type {
            ModelType::ONNX => {
                let pipeline = OnnxPipeline::new(pipeline_type);
                Ok(Arc::new(Mutex::new(Box::new(pipeline) as Box<dyn Pipeline>)))
            }
            ModelType::GGUF | ModelType::BitNet => {
                Err("GGUF/BitNet pipeline not yet implemented".to_string())
            }
            ModelType::SafeTensors => {
                Err("SafeTensors pipeline routes to Python backend".to_string())
            }
            ModelType::LiteRT => {
                Err("LiteRT pipeline routes to Python MediaPipe".to_string())
            }
            ModelType::Unknown => {
                Err("Unknown model type cannot be loaded".to_string())
            }
        }
    }
}

