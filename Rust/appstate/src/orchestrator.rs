///! Model Orchestrator
///!
///! Coordinates model lifecycle: download → cache → load → inference → unload
///! Bridges Rust (model-cache, native-handler/state) and Python (gRPC ML inference)

use std::sync::Arc;
use anyhow::{Context, Result, anyhow};
use tracing::{info, warn, error, debug};
use tabagent_model_cache::ModelCache;
use common::{MlClient, grpc::ml::{LoadModelRequest, UnloadModelRequest}};
use std::collections::HashMap;

/// Model Orchestrator
///
/// Manages the full lifecycle of ML models:
/// 1. **Download**: Uses `ModelCache` to download models from HuggingFace
/// 2. **Track**: Keeps track of loaded models, VRAM/RAM usage (via native-handler/state in future)
/// 3. **Load**: Delegates to Python via gRPC `LoadModel` RPC
/// 4. **Inference**: Routes inference requests to Python via gRPC
/// 5. **Unload**: Frees model from memory via Python gRPC `UnloadModel` RPC
pub struct ModelOrchestrator {
    /// Model cache for downloading and storing models
    cache: Arc<ModelCache>,
    
    /// ML client for Python gRPC communication
    ml_client: Arc<MlClient>,
    
    /// Currently loaded models (model_id -> LoadedModelInfo)
    /// TODO: This should be moved to native-handler/state for system-wide tracking
    loaded_models: Arc<tokio::sync::RwLock<HashMap<String, LoadedModelInfo>>>,
}

/// Information about a loaded model
#[derive(Debug, Clone)]
pub struct LoadedModelInfo {
    pub model_id: String,
    pub pipeline_type: String,
    pub vram_mb: u64,
    pub ram_mb: u64,
    pub loaded_at: std::time::SystemTime,
}

impl ModelOrchestrator {
    /// Create a new ModelOrchestrator
    pub fn new(cache: Arc<ModelCache>, ml_client: Arc<MlClient>) -> Self {
        info!("Initializing ModelOrchestrator");
        Self {
            cache,
            ml_client,
            loaded_models: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }
    
    /// Ensure model manifest exists and is scanned
    ///
    /// This delegates to ModelCache to ensure the model's manifest is ready
    pub async fn ensure_model_manifest(&self, repo_id: &str) -> Result<()> {
        info!("Ensuring model manifest exists: {}", repo_id);
        
        self.cache.ensure_manifest(repo_id)
            .await
            .context(format!("Failed to ensure manifest for: {}", repo_id))?;
        
        debug!("Model {} manifest ready", repo_id);
        Ok(())
    }
    
    /// Load a model into memory for inference
    ///
    /// This sends a gRPC LoadModel request to Python, which will:
    /// 1. Create the appropriate pipeline (via PipelineFactory)
    /// 2. Load the model using transformers
    /// 3. Cache the pipeline in memory
    ///
    /// Returns the loaded model info
    pub async fn load_model(
        &self,
        model_id: &str,
        pipeline_type: &str,
        architecture: Option<&str>,
        options: HashMap<String, String>,
    ) -> Result<LoadedModelInfo> {
        info!("Loading model: {} (pipeline: {})", model_id, pipeline_type);
        
        // Check if already loaded
        {
            let loaded = self.loaded_models.read().await;
            if let Some(info) = loaded.get(model_id) {
                info!("Model {} already loaded", model_id);
                return Ok(info.clone());
            }
        }
        
        // Ensure model manifest exists first
        self.ensure_model_manifest(model_id).await?;
        
        // Send LoadModel request to Python
        let request = LoadModelRequest {
            model_id: model_id.to_string(),
            pipeline_type: pipeline_type.to_string(),
            architecture: architecture.unwrap_or("").to_string(),
            options,
        };
        
        let response = self.ml_client.load_model(request)
            .await
            .context(format!("Failed to load model via gRPC: {}", model_id))?;
        
        if !response.success {
            error!("Python failed to load model: {}", response.message);
            return Err(anyhow!("Failed to load model: {}", response.message));
        }
        
        // Create loaded model info
        let model_info = LoadedModelInfo {
            model_id: model_id.to_string(),
            pipeline_type: pipeline_type.to_string(),
            vram_mb: response.vram_allocated_mb as u64,
            ram_mb: response.ram_allocated_mb as u64,
            loaded_at: std::time::SystemTime::now(),
        };
        
        // Store in loaded models
        {
            let mut loaded = self.loaded_models.write().await;
            loaded.insert(model_id.to_string(), model_info.clone());
        }
        
        info!("✅ Model {} loaded successfully (VRAM: {}MB, RAM: {}MB)", 
            model_id, response.vram_allocated_mb, response.ram_allocated_mb);
        
        Ok(model_info)
    }
    
    /// Unload a model from memory
    pub async fn unload_model(&self, model_id: &str) -> Result<()> {
        info!("Unloading model: {}", model_id);
        
        // Check if model is loaded
        {
            let loaded = self.loaded_models.read().await;
            if !loaded.contains_key(model_id) {
                warn!("Model {} not loaded, skipping unload", model_id);
                return Ok(());
            }
        }
        
        // Send UnloadModel request to Python
        let request = UnloadModelRequest {
            model_id: model_id.to_string(),
        };
        
        let response = self.ml_client.unload_model(request)
            .await
            .context(format!("Failed to unload model via gRPC: {}", model_id))?;
        
        if !response.success {
            error!("Python failed to unload model: {}", response.message);
            return Err(anyhow!("Failed to unload model: {}", response.message));
        }
        
        // Remove from loaded models
        {
            let mut loaded = self.loaded_models.write().await;
            loaded.remove(model_id);
        }
        
        info!("✅ Model {} unloaded successfully", model_id);
        
        Ok(())
    }
    
    /// Get list of loaded models
    pub async fn get_loaded_models(&self) -> Result<Vec<LoadedModelInfo>> {
        let loaded = self.loaded_models.read().await;
        Ok(loaded.values().cloned().collect())
    }
    
    /// Get info about a specific loaded model
    pub async fn get_model_info(&self, model_id: &str) -> Option<LoadedModelInfo> {
        let loaded = self.loaded_models.read().await;
        loaded.get(model_id).cloned()
    }
    
    /// Check if a model is loaded
    pub async fn is_model_loaded(&self, model_id: &str) -> bool {
        let loaded = self.loaded_models.read().await;
        loaded.contains_key(model_id)
    }
}

#[cfg(test)]
mod tests {
    // Tests would go here
    // TODO: Add comprehensive tests
}

