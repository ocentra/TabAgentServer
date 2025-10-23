//! Model list management using database-backed catalog
//! 
//! This module wraps the ModelCatalog from model-cache crate
//! Models are stored in the database and can be edited via JSON config

use tabagent_model_cache::{ModelCatalog, ModelCatalogEntry};
use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::path::PathBuf;

/// Global model catalog instance
pub static MODEL_CATALOG: Lazy<Mutex<Option<ModelCatalog>>> = 
    Lazy::new(|| Mutex::new(None));

/// Initialize the model catalog
pub fn init_catalog(cache_dir: &str) -> Result<(), String> {
    let cache_path = PathBuf::from(cache_dir);
    let catalog = ModelCatalog::open(&cache_path)
        .map_err(|e| format!("Failed to open model catalog: {}", e))?;
    
    // Check if catalog is empty, if so init from default JSON
    let model_count = catalog.get_all_models()
        .map_err(|e| format!("Failed to get models: {}", e))?
        .len();
    
    if model_count == 0 {
        // Load default models from JSON
        let default_json_path = cache_path.join("default_models.json");
        
        // If default_models.json doesn't exist in cache, look for it in the binary's directory
        let json_path = if default_json_path.exists() {
            default_json_path
        } else {
            // Try to find it relative to the current executable or working directory
            let exe_dir = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                .unwrap_or_else(|| PathBuf::from("."));
            
            let candidates = vec![
                exe_dir.join("default_models.json"),
                PathBuf::from("Server/tabagent-rs/model-cache/default_models.json"),
                PathBuf::from("../model-cache/default_models.json"),
                PathBuf::from("./default_models.json"),
            ];
            
            candidates.into_iter()
                .find(|p| p.exists())
                .ok_or_else(|| "default_models.json not found".to_string())?
        };
        
        catalog.init_from_json(&json_path)
            .map_err(|e| format!("Failed to init catalog from JSON: {}", e))?;
    }
    
    let mut global = MODEL_CATALOG.lock().expect("MODEL_CATALOG mutex poisoned");
    *global = Some(catalog);
    
    Ok(())
}

/// Get the global model catalog
pub fn get_catalog() -> Result<std::sync::MutexGuard<'static, Option<ModelCatalog>>, String> {
    Ok(MODEL_CATALOG.lock().expect("MODEL_CATALOG mutex poisoned"))
}

// Convenience functions that wrap ModelCatalog

/// Get all available models
pub fn get_all_models() -> Result<Vec<ModelCatalogEntry>, String> {
    let catalog_lock = get_catalog()?;
    let catalog = catalog_lock.as_ref()
        .ok_or_else(|| "Model catalog not initialized".to_string())?;
    
    catalog.get_all_models()
        .map_err(|e| format!("Failed to get all models: {}", e))
}

/// Get a specific model by ID
pub fn get_model(model_id: &str) -> Result<Option<ModelCatalogEntry>, String> {
    let catalog_lock = get_catalog()?;
    let catalog = catalog_lock.as_ref()
        .ok_or_else(|| "Model catalog not initialized".to_string())?;
    
    catalog.get_model(model_id)
        .map_err(|e| format!("Failed to get model: {}", e))
}

/// Add a user model
pub fn add_user_model(model: ModelCatalogEntry) -> Result<(), String> {
    let catalog_lock = get_catalog()?;
    let catalog = catalog_lock.as_ref()
        .ok_or_else(|| "Model catalog not initialized".to_string())?;
    
    catalog.insert_model(model)
        .map_err(|e| format!("Failed to add user model: {}", e))
}

/// Remove a user model
pub fn remove_user_model(model_id: &str) -> Result<(), String> {
    let catalog_lock = get_catalog()?;
    let catalog = catalog_lock.as_ref()
        .ok_or_else(|| "Model catalog not initialized".to_string())?;
    
    catalog.delete_model(model_id)
        .map(|_| ())
        .map_err(|e| format!("Failed to remove user model: {}", e))
}

/// Mark model as downloaded
pub fn mark_model_downloaded(model_id: &str, downloaded: bool) -> Result<(), String> {
    let catalog_lock = get_catalog()?;
    let catalog = catalog_lock.as_ref()
        .ok_or_else(|| "Model catalog not initialized".to_string())?;
    
    catalog.mark_downloaded(model_id, downloaded)
        .map_err(|e| format!("Failed to mark model downloaded: {}", e))
}

/// Get only suggested models
pub fn get_suggested_models() -> Result<Vec<ModelCatalogEntry>, String> {
    let catalog_lock = get_catalog()?;
    let catalog = catalog_lock.as_ref()
        .ok_or_else(|| "Model catalog not initialized".to_string())?;
    
    catalog.get_suggested_models()
        .map_err(|e| format!("Failed to get suggested models: {}", e))
}

/// Get only downloaded models
pub fn get_downloaded_models() -> Result<Vec<ModelCatalogEntry>, String> {
    let catalog_lock = get_catalog()?;
    let catalog = catalog_lock.as_ref()
        .ok_or_else(|| "Model catalog not initialized".to_string())?;
    
    catalog.get_downloaded_models()
        .map_err(|e| format!("Failed to get downloaded models: {}", e))
}

/// Get models by type (gguf, bitnet, onnx, etc.)
pub fn get_models_by_type(model_type: &str) -> Result<Vec<ModelCatalogEntry>, String> {
    let catalog_lock = get_catalog()?;
    let catalog = catalog_lock.as_ref()
        .ok_or_else(|| "Model catalog not initialized".to_string())?;
    
    catalog.get_models_by_type(model_type)
        .map_err(|e| format!("Failed to get models by type: {}", e))
}

/// Filter models by tag/label
pub fn get_models_by_label(label: &str) -> Result<Vec<ModelCatalogEntry>, String> {
    let catalog_lock = get_catalog()?;
    let catalog = catalog_lock.as_ref()
        .ok_or_else(|| "Model catalog not initialized".to_string())?;
    
    catalog.get_models_by_tag(label)
        .map_err(|e| format!("Failed to get models by label: {}", e))
}

/// Search models by name or checkpoint
pub fn search_models(query: &str) -> Result<Vec<ModelCatalogEntry>, String> {
    let catalog_lock = get_catalog()?;
    let catalog = catalog_lock.as_ref()
        .ok_or_else(|| "Model catalog not initialized".to_string())?;
    
    catalog.search_models(query)
        .map_err(|e| format!("Failed to search models: {}", e))
}

/// Get default/recommended model for a given type
/// 
/// Prioritizes models tagged with "default", then falls back to smallest suggested model
/// Current defaults:
/// - GGUF: Qwen3-30B-A3B-Q4_K_M (unsloth/Qwen3-30B-A3B-GGUF)
/// - BitNet: Falcon3-1B-Instruct-1.58bit
/// - ONNX: Phi-3.5-mini-instruct-q4
pub fn get_default_model_for_type(model_type: &str) -> Result<Option<ModelCatalogEntry>, String> {
    let catalog_lock = get_catalog()?;
    let catalog = catalog_lock.as_ref()
        .ok_or_else(|| "Model catalog not initialized".to_string())?;
    
    catalog.get_default_for_type(model_type)
        .map_err(|e| format!("Failed to get default model for type: {}", e))
}

/// Detect model type from file extension or model name
pub fn detect_model_type(repo_id: &str, file_path: Option<&str>) -> String {
    // Check file extension first
    if let Some(path) = file_path {
        if path.ends_with(".gguf") {
            return "gguf".to_string();
        } else if path.ends_with(".onnx") {
            return "onnx".to_string();
        } else if path.ends_with(".safetensors") {
            return "safetensors".to_string();
        }
    }
    
    // Check repo ID for BitNet models
    if repo_id.contains("1bitLLM") || 
       repo_id.contains("1.58") || 
       repo_id.contains("BitNet") ||
       repo_id.contains("Falcon-E") {
        return "bitnet".to_string();
    }
    
    // Default to gguf if unknown
    "gguf".to_string()
}

/// Export catalog to JSON file
pub fn export_catalog_to_json(json_path: &str) -> Result<(), String> {
    let catalog_lock = get_catalog()?;
    let catalog = catalog_lock.as_ref()
        .ok_or_else(|| "Model catalog not initialized".to_string())?;
    
    catalog.export_to_json(&PathBuf::from(json_path))
        .map_err(|e| format!("Failed to export catalog: {}", e))
}

/// Re-import catalog from JSON file
pub fn import_catalog_from_json(json_path: &str) -> Result<(), String> {
    let catalog_lock = get_catalog()?;
    let catalog = catalog_lock.as_ref()
        .ok_or_else(|| "Model catalog not initialized".to_string())?;
    
    catalog.init_from_json(&PathBuf::from(json_path))
        .map_err(|e| format!("Failed to import catalog: {}", e))
}
