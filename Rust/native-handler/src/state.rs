//! Global state management for model handling
//! 
//! Manages loaded models, downloads, and system resources

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

/// Global registry of currently loaded models
pub static LOADED_MODELS: Lazy<Arc<Mutex<HashMap<String, LoadedModelInfo>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

/// Track the currently active model (the one being used for generation)
pub static CURRENT_ACTIVE_MODEL: Lazy<Arc<Mutex<Option<String>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(None)));

/// Global model cache instance
pub static MODEL_CACHE: Lazy<Arc<Mutex<Option<tabagent_model_cache::ModelCache>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(None)));

/// Active download tracking
pub static ACTIVE_DOWNLOADS: Lazy<Arc<Mutex<HashMap<String, DownloadProgress>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

/// System resource snapshot (updated periodically)
pub static SYSTEM_RESOURCES_CACHE: Lazy<Arc<Mutex<Option<SystemResourcesSnapshot>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(None)));

/// Information about a loaded model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadedModelInfo {
    /// Model identifier (repo_id/file_path)
    pub model_id: String,
    
    /// Where the model is loaded (GPU, CPU, or split)
    pub loaded_to: LoadTarget,
    
    /// Number of layers on GPU (if split)
    pub gpu_layers: u32,
    
    /// Number of layers on CPU (if split)
    pub cpu_layers: u32,
    
    /// VRAM used in bytes
    pub vram_used: u64,
    
    /// RAM used in bytes
    pub ram_used: u64,
    
    /// Timestamp when loaded
    pub loaded_at: i64,
    
    /// Model configuration
    pub config: ModelConfigInfo,
}

/// Where a model is loaded
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LoadTarget {
    /// Fully loaded to GPU
    GPU,
    
    /// Fully loaded to CPU
    CPU,
    
    /// Split between GPU and CPU
    Split { gpu_layers: u32 },
}

/// Model configuration details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfigInfo {
    /// Vocabulary size
    pub vocab_size: Option<u32>,
    
    /// Context window size
    pub context_size: Option<u32>,
    
    /// Embedding dimensions
    pub embedding_dim: Option<u32>,
    
    /// Model file size in bytes
    pub file_size: u64,
    
    /// Model type (e.g., "gguf", "onnx")
    pub model_type: String,
}

/// Download progress tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    /// Repository ID
    pub repo_id: String,
    
    /// File path
    pub file_path: String,
    
    /// Bytes downloaded
    pub downloaded: u64,
    
    /// Total bytes
    pub total: u64,
    
    /// Progress percentage (0-100)
    pub progress: u8,
    
    /// Download status
    pub status: DownloadStatus,
    
    /// Started at timestamp
    pub started_at: i64,
}

/// Download status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DownloadStatus {
    /// Download in progress
    Downloading,
    
    /// Download completed
    Completed,
    
    /// Download failed
    Failed,
    
    /// Download cancelled
    Cancelled,
}

/// System resources snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResourcesSnapshot {
    /// Total system RAM in bytes
    pub total_ram: u64,
    
    /// Used RAM in bytes
    pub used_ram: u64,
    
    /// Available RAM in bytes
    pub available_ram: u64,
    
    /// Total VRAM in bytes (if GPU available)
    pub total_vram: Option<u64>,
    
    /// Used VRAM in bytes (if GPU available)
    pub used_vram: Option<u64>,
    
    /// Available VRAM in bytes (if GPU available)
    pub available_vram: Option<u64>,
    
    /// Number of loaded models
    pub loaded_models_count: usize,
    
    /// Total memory used by loaded models (approximation)
    pub models_memory_usage: u64,
    
    /// Timestamp when snapshot was taken
    pub timestamp: i64,
}

/// Initialize the model cache
pub fn init_cache(cache_dir: &str) -> Result<(), String> {
    let cache = tabagent_model_cache::ModelCache::new(cache_dir)
        .map_err(|e| format!("Failed to initialize cache: {}", e))?;
    
    let mut cache_lock = MODEL_CACHE.lock().expect("MODEL_CACHE mutex poisoned");
    *cache_lock = Some(cache);
    
    Ok(())
}

/// Get a reference to the model cache
pub fn get_cache() -> Result<Arc<Mutex<Option<tabagent_model_cache::ModelCache>>>, String> {
    Ok(MODEL_CACHE.clone())
}

/// Add a loaded model to the registry
pub fn register_loaded_model(model_id: String, info: LoadedModelInfo) {
    let mut models = LOADED_MODELS.lock().expect("LOADED_MODELS mutex poisoned");
    models.insert(model_id, info);
}

/// Remove a model from the registry
pub fn unregister_loaded_model(model_id: &str) -> Option<LoadedModelInfo> {
    let mut models = LOADED_MODELS.lock().expect("LOADED_MODELS mutex poisoned");
    models.remove(model_id)
}

/// Get all loaded models
pub fn get_loaded_models() -> Vec<LoadedModelInfo> {
    let models = LOADED_MODELS.lock().expect("LOADED_MODELS mutex poisoned");
    models.values().cloned().collect()
}

/// Get a specific loaded model
pub fn get_loaded_model(model_id: &str) -> Option<LoadedModelInfo> {
    let models = LOADED_MODELS.lock().expect("LOADED_MODELS mutex poisoned");
    models.get(model_id).cloned()
}

/// Check if a model is currently loaded
pub fn is_model_loaded(model_id: &str) -> bool {
    let models = LOADED_MODELS.lock().expect("LOADED_MODELS mutex poisoned");
    models.contains_key(model_id)
}

/// Track download progress
pub fn update_download_progress(progress: DownloadProgress) {
    let mut downloads = ACTIVE_DOWNLOADS.lock().expect("ACTIVE_DOWNLOADS mutex poisoned");
    let key = format!("{}/{}", progress.repo_id, progress.file_path);
    downloads.insert(key, progress);
}

/// Get download progress for a specific file
pub fn get_download_progress(repo_id: &str, file_path: &str) -> Option<DownloadProgress> {
    let downloads = ACTIVE_DOWNLOADS.lock().expect("ACTIVE_DOWNLOADS mutex poisoned");
    let key = format!("{}/{}", repo_id, file_path);
    downloads.get(&key).cloned()
}

/// Remove download tracking (when complete or failed)
pub fn clear_download_progress(repo_id: &str, file_path: &str) {
    let mut downloads = ACTIVE_DOWNLOADS.lock().expect("ACTIVE_DOWNLOADS mutex poisoned");
    let key = format!("{}/{}", repo_id, file_path);
    downloads.remove(&key);
}

/// Get all active downloads
pub fn get_active_downloads() -> Vec<DownloadProgress> {
    let downloads = ACTIVE_DOWNLOADS.lock().expect("ACTIVE_DOWNLOADS mutex poisoned");
    downloads.values().cloned().collect()
}

/// Set the currently active model
pub fn set_current_model(model_id: String) {
    let mut current = CURRENT_ACTIVE_MODEL.lock().expect("CURRENT_ACTIVE_MODEL mutex poisoned");
    *current = Some(model_id);
}

/// Get the currently active model
pub fn get_current_model() -> Option<String> {
    let current = CURRENT_ACTIVE_MODEL.lock().expect("CURRENT_ACTIVE_MODEL mutex poisoned");
    current.clone()
}

/// Clear the currently active model
pub fn clear_current_model() {
    let mut current = CURRENT_ACTIVE_MODEL.lock().expect("CURRENT_ACTIVE_MODEL mutex poisoned");
    *current = None;
}

/// Update system resources cache
pub fn update_system_resources(snapshot: SystemResourcesSnapshot) {
    let mut cache = SYSTEM_RESOURCES_CACHE.lock().expect("SYSTEM_RESOURCES_CACHE mutex poisoned");
    *cache = Some(snapshot);
}

/// Get cached system resources
pub fn get_system_resources_snapshot() -> Option<SystemResourcesSnapshot> {
    let cache = SYSTEM_RESOURCES_CACHE.lock().expect("SYSTEM_RESOURCES_CACHE mutex poisoned");
    cache.clone()
}

