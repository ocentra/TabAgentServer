//! Model management handlers (load/unload/list/info).

use anyhow::{Context, Result};
use tabagent_values::{ResponseValue, TokenUsage};
use tabagent_model_cache::{detect_from_file_path, detect_from_repo_name, Backend};
use tabagent_onnx_loader::OnnxSession;
use gguf_loader::{Model as GgufModel, Context as GgufContext, GenerationParams as GgufGenParams};

use crate::AppState;

/// Handle model loading request.
pub async fn handle_load(
    state: &AppState,
    model_id: &str,
    variant: Option<&str>,
) -> Result<ResponseValue> {
    tracing::info!("Load model: {} (variant: {:?})", model_id, variant);

    // Check if already loaded
    if state.is_model_loaded(model_id) {
        return Ok(ResponseValue::chat(
            "already-loaded",
            "system",
            format!("Model {} is already loaded", model_id),
            TokenUsage::zero(),
        ));
    }

    // Detect model type
    let model_info = detect_from_file_path(model_id)
        .or_else(|| detect_from_repo_name(model_id))
        .with_context(|| format!("Model '{}' not found", model_id))?;

    // Route to appropriate loader based on backend
    match &model_info.backend {
        Backend::Rust { engine } if engine.contains("onnx") => {
            // Get model file path from cache
            let file_path = state.cache.get_file_path(model_id, "model.onnx").await
                .context("Failed to get model from cache")?
                .with_context(|| format!("{} not in cache", model_id))?;

            // Load ONNX session
            let session = OnnxSession::load(&file_path)
                .context("Failed to load ONNX session")?;

            // Store in state
            state.register_onnx_model(model_id.to_string(), session);

            Ok(ResponseValue::chat(
                "loaded",
                "system",
                format!("ONNX model {} loaded successfully", model_id),
                TokenUsage::zero(),
            ))
        }
        
        Backend::Rust { engine } if engine.contains("llama") || engine.contains("bitnet") => {
            // Get model file path from cache
            let file_path = state.cache.get_file_path(model_id, variant.unwrap_or("model.gguf")).await
                .context("Failed to get model from cache")?
                .with_context(|| format!("{} not in cache", model_id))?;

            // Auto-select library variant based on hardware
            let prefer_gpu = !state.hardware.gpus.is_empty();
            let library_path = gguf_loader::auto_select_library(&std::path::PathBuf::from("External/BitNet"), prefer_gpu)
                .context("Failed to auto-select GGUF library")?;

            // Create model config and load
            let config = gguf_loader::ModelConfig::new(&file_path);
            let model = GgufModel::load(&library_path, config)
                .context("Failed to load GGUF model")?;

            // Create context with generation params
            let params = GgufGenParams::default();
            let context = GgufContext::new(std::sync::Arc::new(model), params)
                .context("Failed to create GGUF context")?;

            // Store in state (wrapped in Mutex for thread safety)
            state.register_gguf_context(model_id.to_string(), context);

            Ok(ResponseValue::chat(
                "loaded",
                "system",
                format!("GGUF model {} loaded successfully", model_id),
                TokenUsage::zero(),
            ))
        }
        
        Backend::Python { .. } => {
            // Python models are loaded on-demand by the Python bridge
            Ok(ResponseValue::chat(
                "registered",
                "system",
                format!("Python model {} registered (will load on first inference)", model_id),
                TokenUsage::zero(),
            ))
        }
        
        _ => anyhow::bail!("Unsupported backend: {:?}", model_info.backend),
    }
}

/// Handle model unloading request.
pub async fn handle_unload(
    state: &AppState,
    model_id: &str,
) -> Result<ResponseValue> {
    tracing::info!("Unload model: {}", model_id);

    if !state.is_model_loaded(model_id) {
        anyhow::bail!("Model '{}' is not loaded", model_id);
    }

    // Unregister from all model stores
    state.unregister_onnx_model(model_id);
    state.unregister_gguf_context(model_id);
    
    Ok(ResponseValue::chat(
        "unloaded",
        "system",
        format!("Model {} unloaded successfully", model_id),
        TokenUsage::zero(),
    ))
}

/// Handle list models request.
pub async fn handle_list(
    state: &AppState,
) -> Result<ResponseValue> {
    let loaded_models = state.list_loaded_models();
    
    // Convert to ModelInfo format
    use tabagent_values::ModelInfo;
    let models: Vec<ModelInfo> = loaded_models.iter().map(|id| {
        ModelInfo {
            id: id.clone(),
            name: id.clone(),
            backend: "rust".to_string(),
            loaded: true,
            size_bytes: None,
            parameters: None,
        }
    }).collect();
    
    Ok(ResponseValue::model_list(models))
}

/// Handle model info request.
pub async fn handle_info(
    state: &AppState,
    model_id: &str,
) -> Result<ResponseValue> {
    if !state.is_model_loaded(model_id) {
        anyhow::bail!("Model '{}' is not loaded", model_id);
    }

    // Detect model type to get metadata
    let model_info = detect_from_file_path(model_id)
        .or_else(|| detect_from_repo_name(model_id))
        .with_context(|| format!("Model '{}' not found", model_id))?;

    let info_text = format!(
        "Model: {}\nType: {:?}\nBackend: {:?}\nTask: {:?}",
        model_id,
        model_info.model_type,
        model_info.backend,
        model_info.task
    );
    
    Ok(ResponseValue::chat(
        "info",
        "system",
        info_text,
        TokenUsage::zero(),
    ))
}

/// Handle get available quants request (for UI dropdown).
pub async fn handle_get_quants(
    state: &AppState,
    repo_id: &str,
) -> Result<ResponseValue> {
    tracing::debug!("Get quants for: {}", repo_id);
    
    let quants = state.cache.get_available_quants(repo_id).await
        .context("Failed to get available quants")?;
    
    Ok(ResponseValue::chat(
        "quants",
        "system",
        serde_json::to_string(&quants)?,
        TokenUsage::zero(),
    ))
}

/// Handle get inference settings request.
pub async fn handle_get_inference_settings(
    state: &AppState,
    repo_id: &str,
    variant: &str,
) -> Result<ResponseValue> {
    tracing::debug!("Get settings for: {}:{}", repo_id, variant);
    
    let settings = state.cache.get_inference_settings(repo_id, variant).await
        .context("Failed to get inference settings")?;
    
    Ok(ResponseValue::chat(
        "settings",
        "system",
        serde_json::to_string(&settings)?,
        TokenUsage::zero(),
    ))
}

/// Handle save inference settings request.
pub async fn handle_save_inference_settings(
    state: &AppState,
    repo_id: &str,
    variant: &str,
    settings: &tabagent_values::InferenceSettings,
) -> Result<ResponseValue> {
    tracing::info!("Save settings for: {}:{}", repo_id, variant);
    
    state.cache.save_inference_settings(repo_id, variant, settings).await
        .context("Failed to save inference settings")?;
    
    Ok(ResponseValue::chat(
        "settings_saved",
        "system",
        format!("Settings saved for {}:{}", repo_id, variant),
        TokenUsage::zero(),
    ))
}

