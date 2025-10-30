//! Central application state.
//!
//! This module defines the `AppState` struct that holds all shared resources
//! and implements the business logic for the TabAgent backend.

use std::sync::Arc;
use std::path::PathBuf;
use dashmap::DashMap;
use tokio::sync::Mutex;
use anyhow::{Context, Result};
use async_trait::async_trait;

// Infrastructure crate imports
use storage::DatabaseCoordinator;
use tabagent_model_cache::ModelCache;
use tabagent_onnx_loader::OnnxSession;
use gguf_loader::Context as GgufContext;
use tabagent_hardware::{detect_system, SystemInfo};

// Type imports
use tabagent_values::{RequestValue, ResponseValue};
use common::backend::AppStateProvider;

use crate::hf_auth::HfAuthManager;

/// Configuration for AppState initialization.
pub struct AppStateConfig {
    pub db_path: PathBuf,
    pub model_cache_path: PathBuf,
}

impl AppStateConfig {
    /// Create a default configuration using platform-specific AppData paths.
    ///
    /// - **Windows**: `%APPDATA%\TabAgent\db\`
    /// - **macOS**: `~/Library/Application Support/TabAgent/db/`
    /// - **Linux**: `~/.local/share/TabAgent/db/`
    pub fn default() -> Self {
        let base_path = common::platform::get_default_db_path();
        Self {
            db_path: base_path.join("tabagent_db"),
            model_cache_path: base_path.join("models"),
        }
    }
}

/// Central application state.
///
/// Holds all shared resources and provides business logic operations.
#[derive(Clone)]
pub struct AppState {
    /// Database client (gRPC-based, works with in-process or remote storage)
    /// For backward compatibility during migration, this wraps DatabaseCoordinator
    /// but provides location-transparent access
    pub db_client: Arc<storage::DatabaseClient>,
    
    /// ML client (gRPC-based, connects to Python ML services)
    pub ml_client: Arc<common::MlClient>,
    
    /// Model orchestrator (coordinates download → load → inference → unload)
    pub orchestrator: Arc<crate::orchestrator::ModelOrchestrator>,
    
    /// Model cache (downloads, storage, metadata)
    pub cache: Arc<ModelCache>,
    
    /// Hardware system information
    pub hardware: Arc<SystemInfo>,
    
    /// Loaded ONNX models
    pub onnx_models: Arc<DashMap<String, Arc<OnnxSession>>>,
    
    /// GGUF model contexts (for inference)
    /// Wrapped in Mutex because GGUF contexts contain raw pointers (*mut LlamaContext)
    /// which are not Send/Sync, so we use async Mutex for thread-safe access
    pub gguf_contexts: Arc<DashMap<String, Arc<Mutex<GgufContext>>>>,
    
    /// HuggingFace auth manager (secure token storage)
    pub hf_auth: Arc<HfAuthManager>,
    
    /// Active generation cancellation tokens
    pub generation_tokens: Arc<DashMap<String, tokio_util::sync::CancellationToken>>,
}



impl AppState {
    /// Initialize application state.
    pub async fn new(config: AppStateConfig) -> Result<Self> {
        tracing::info!("Initializing AppState...");

        // Initialize database client (in-process mode by default)
        let db_path = config.db_path.clone();
        let db_coordinator = DatabaseCoordinator::with_base_path(Some(db_path.clone()))
            .context("Failed to initialize database")?;
        
        let db_client = storage::DatabaseClient::InProcess(Arc::new(db_coordinator));
        tracing::info!("Database client initialized (in-process) at: {:?}", db_path);

        // Initialize ML client (attempts connection to Python ML service)
        let ml_endpoint = std::env::var("ML_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:50051".to_string());
        
        let ml_client = match common::MlClient::new(&ml_endpoint).await {
            Ok(client) => {
                tracing::info!("ML client connected to: {}", ml_endpoint);
                client
            }
            Err(e) => {
                tracing::warn!("ML client connection failed ({}), using disabled mode", e);
                common::MlClient::disabled()
            }
        };

        // Initialize model cache
        let cache_path = config.model_cache_path.to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid cache path"))?;
        
        let cache = ModelCache::new(cache_path)
            .context("Failed to initialize model cache")?;
        
        tracing::info!("Model cache initialized at: {}", cache_path);

        // Detect hardware
        let hardware = detect_system()
            .context("Failed to detect system hardware")?;
        
        tracing::info!(
            "Hardware detected: {} cores, {:.2} GB RAM, {} GPUs",
            hardware.cpu.cores,
            hardware.memory.total_ram_mb as f64 / 1024.0,
            hardware.gpus.len()
        );

        // Initialize HF auth manager
        let hf_auth = HfAuthManager::new()
            .context("Failed to initialize HuggingFace auth manager")?;
        
        tracing::info!("HuggingFace auth manager initialized");

        // Wrap resources in Arc for sharing
        let cache_arc = Arc::new(cache);
        let ml_client_arc = Arc::new(ml_client);
        
        // Initialize model orchestrator
        let orchestrator = Arc::new(crate::orchestrator::ModelOrchestrator::new(
            cache_arc.clone(),
            ml_client_arc.clone()
        ));
        tracing::info!("Model orchestrator initialized");

        Ok(Self {
            db_client: Arc::new(db_client),
            ml_client: ml_client_arc,
            orchestrator,
            cache: cache_arc,
            hardware: Arc::new(hardware),
            onnx_models: Arc::new(DashMap::new()),
            gguf_contexts: Arc::new(DashMap::new()),
            hf_auth: Arc::new(hf_auth),
            generation_tokens: Arc::new(DashMap::new()),
        })
    }

    // ========== Model Registry Operations ==========

    /// Check if a model is loaded (any type)
    pub fn is_model_loaded(&self, model_id: &str) -> bool {
        self.onnx_models.contains_key(model_id) ||
        self.gguf_contexts.contains_key(model_id)
    }

    /// List all loaded models
    pub fn list_loaded_models(&self) -> Vec<String> {
        let mut models = Vec::new();
        
        for entry in self.onnx_models.iter() {
            models.push(entry.key().clone());
        }
        
        for entry in self.gguf_contexts.iter() {
            models.push(entry.key().clone());
        }
        
        models
    }

    // ========== ONNX Model Operations ==========

    /// Register an ONNX model
    pub fn register_onnx_model(&self, model_id: String, session: OnnxSession) {
        self.onnx_models.insert(model_id.clone(), Arc::new(session));
        tracing::info!("ONNX model registered: {}", model_id);
    }

    /// Get an ONNX model
    pub fn get_onnx_model(&self, model_id: &str) -> Option<Arc<OnnxSession>> {
        self.onnx_models.get(model_id).map(|entry| entry.value().clone())
    }

    /// Unregister an ONNX model
    pub fn unregister_onnx_model(&self, model_id: &str) {
        if self.onnx_models.remove(model_id).is_some() {
            tracing::info!("ONNX model unregistered: {}", model_id);
        }
    }

    // ========== GGUF Context Operations ==========
    
    /// Register a GGUF context (for inference)
    pub fn register_gguf_context(&self, model_id: String, context: GgufContext) {
        self.gguf_contexts.insert(model_id.clone(), Arc::new(Mutex::new(context)));
        tracing::info!("Registered GGUF context: {}", model_id);
    }

    /// Get a GGUF context (wrapped in Mutex for thread safety)
    pub fn get_gguf_context(&self, model_id: &str) -> Option<Arc<Mutex<GgufContext>>> {
        self.gguf_contexts.get(model_id).map(|entry| entry.value().clone())
    }

    /// Unregister a GGUF context
    pub fn unregister_gguf_context(&self, model_id: &str) {
        if self.gguf_contexts.remove(model_id).is_some() {
            tracing::info!("GGUF context unregistered: {}", model_id);
        }
    }

    // ========== Generation Control ==========

    /// Cancel an active generation
    pub async fn cancel_generation(&self, request_id: &str) {
        if let Some((_, token)) = self.generation_tokens.remove(request_id) {
            token.cancel();
            tracing::info!("Generation cancelled: {}", request_id);
        }
    }
}

// ========== Implement AppStateProvider ==========

#[async_trait]
impl AppStateProvider for AppState {
    async fn handle_request(&self, request: RequestValue) -> anyhow::Result<ResponseValue> {
        use tabagent_values::RequestType;
        
        tracing::debug!("AppState handling request: {:?}", request.value_type());

        // Dispatch to appropriate route handler
        match request.request_type() {
            // Core AI routes
            RequestType::Chat { model, messages, temperature, .. } => {
                crate::routes::handle_chat(self, model, messages, *temperature).await
            }
            RequestType::Generate { model, prompt, temperature, .. } => {
                crate::routes::handle_generate(self, model, prompt, *temperature).await
            }
            RequestType::Embeddings { model, input } => {
                crate::routes::handle_embeddings(self, model, input).await
            }
            
            // Model management routes
            RequestType::LoadModel { model_id, variant, .. } => {
                crate::routes::handle_load_model(self, model_id, variant.as_deref()).await
            }
            RequestType::UnloadModel { model_id } => {
                crate::routes::handle_unload_model(self, model_id).await
            }
            RequestType::ListModels { .. } => {
                crate::routes::handle_list_models(self).await
            }
            RequestType::ModelInfo { model_id } => {
                crate::routes::handle_model_info(self, model_id).await
            }
            
            // System routes
            RequestType::Health => {
                crate::routes::handle_health(self).await
            }
            RequestType::SystemInfo => {
                crate::routes::handle_system_info(self).await
            }
            
            // HuggingFace authentication routes
            RequestType::SetHfToken { token } => {
                crate::routes::handle_set_hf_token(self, token).await
            }
            RequestType::GetHfTokenStatus => {
                crate::routes::handle_get_hf_token_status(self).await
            }
            RequestType::ClearHfToken => {
                crate::routes::handle_clear_hf_token(self).await
            }
            
            // Hardware detection routes
            RequestType::GetHardwareInfo => {
                crate::routes::handle_get_hardware_info(self).await
            }
            RequestType::CheckModelFeasibility { model_size_mb } => {
                crate::routes::handle_check_model_feasibility(self, *model_size_mb).await
            }
            RequestType::GetRecommendedModels => {
                crate::routes::handle_get_recommended_models(self).await
            }
            
            // Session routes
            RequestType::ChatHistory { session_id, .. } => {
                crate::routes::handle_chat_history(self, session_id.as_deref()).await
            }
            RequestType::SaveMessage { session_id, message } => {
                crate::routes::handle_save_message(self, session_id, message).await
            }
            
            // RAG routes
            RequestType::RagQuery { query, top_k, .. } => {
                crate::routes::handle_rag_query(self, query, *top_k).await
            }
            RequestType::Rerank { model, query, documents, top_n } => {
                crate::routes::handle_rerank(self, model, query, documents, *top_n).await
            }
            RequestType::StopGeneration { request_id } => {
                crate::routes::handle_stop_generation(self, request_id).await
            }
            
            // Extended routes
            RequestType::GetParams => {
                crate::routes::handle_get_params(self).await
            }
            RequestType::SetParams { params } => {
                crate::routes::handle_set_params(self, params).await
            }
            RequestType::GetStats => {
                crate::routes::handle_get_stats(self).await
            }
            RequestType::GetResources => {
                crate::routes::handle_get_resources(self).await
            }
            RequestType::EstimateMemory { model, quantization } => {
                crate::routes::handle_estimate_memory(self, model, quantization.as_deref()).await
            }
            RequestType::SemanticSearchQuery { .. } => {
                anyhow::bail!("Semantic search requires vector DB integration (not yet implemented)")
            }
            RequestType::CalculateSimilarity { .. } => {
                anyhow::bail!("Similarity calculation requires embedding infrastructure (not yet implemented)")
            }
            RequestType::EvaluateEmbeddings { .. } => {
                anyhow::bail!("Embedding evaluation requires ML infrastructure (not yet implemented)")
            }
            RequestType::ClusterDocuments { .. } => {
                anyhow::bail!("Document clustering requires ML infrastructure (not yet implemented)")
            }
            RequestType::RecommendContent { .. } => {
                anyhow::bail!("Content recommendation requires ML infrastructure (not yet implemented)")
            }
            RequestType::PullModel { model, quantization } => {
                crate::routes::handle_pull_model(self, model, quantization.as_deref()).await
            }
            RequestType::DeleteModel { model_id } => {
                crate::routes::handle_delete_model(self, model_id).await
            }
            RequestType::GetRecipes => {
                crate::routes::handle_get_recipes(self).await
            }
            RequestType::GetEmbeddingModels => {
                crate::routes::handle_get_embedding_models(self).await
            }
            RequestType::GetLoadedModels => {
                crate::routes::handle_get_loaded_models(self).await
            }
            RequestType::SelectModel { model_id } => {
                crate::routes::handle_select_model(self, model_id).await
            }
            
            // WebRTC signaling is handled by API layer, not AppState
            // These routes should never reach here - they're handled directly by WebRtcManager
            RequestType::CreateWebRtcOffer { .. } => {
                anyhow::bail!("WebRTC signaling should be handled by API layer with WebRtcManager, not AppState")
            }
            RequestType::SubmitWebRtcAnswer { .. } => {
                anyhow::bail!("WebRTC signaling should be handled by API layer with WebRtcManager, not AppState")
            }
            RequestType::AddIceCandidate { .. } => {
                anyhow::bail!("WebRTC signaling should be handled by API layer with WebRtcManager, not AppState")
            }
            RequestType::GetWebRtcSession { .. } => {
                anyhow::bail!("WebRTC signaling should be handled by API layer with WebRtcManager, not AppState")
            }
            
            // Media streaming routes
            RequestType::AudioStream { codec, sample_rate, bitrate, channels } => {
                crate::routes::handle_audio_stream(self, codec, *sample_rate, *bitrate, *channels).await
            }
            RequestType::VideoStream { codec, resolution, bitrate, framerate, hardware_acceleration } => {
                crate::routes::handle_video_stream(self, codec, *resolution, *bitrate, *framerate, *hardware_acceleration).await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_state_initialization() {
        let config = AppStateConfig {
            db_path: PathBuf::from("./test_db"),
            model_cache_path: PathBuf::from("./test_models"),
        };

        let state = AppState::new(config).await;
        assert!(state.is_ok(), "State initialization should succeed");

        let state = state.unwrap();
        assert_eq!(state.list_loaded_models().len(), 0, "Should start with no loaded models");
    }
}

