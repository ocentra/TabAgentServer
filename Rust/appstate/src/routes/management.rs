//! Extended model management handlers.

use anyhow::{Result, Context};
use tabagent_values::{ResponseValue, TokenUsage};

use crate::AppState;

/// Handle get recipes request.
pub async fn handle_get_recipes(state: &AppState) -> Result<ResponseValue> {
    // Use hardware crate to get actual execution provider recommendation
    let exec_provider = state.hardware.recommended_execution_provider();
    let has_gpu = !state.hardware.gpus.is_empty();
    let ram_gb = state.hardware.memory.total_ram_mb / 1024;
    
    let recipes = serde_json::json!({
        "current_system": {
            "ram_gb": ram_gb,
            "gpu_available": has_gpu,
            "recommended_provider": format!("{:?}", exec_provider.primary),
            "reason": &exec_provider.reason,
            "tier": &state.hardware.ram_tier,
        },
        "recipes": [
            {"name": "low_memory", "ram_gb": 8, "gpu_required": false, "suitable": ram_gb >= 8},
            {"name": "balanced", "ram_gb": 16, "gpu_required": false, "suitable": ram_gb >= 16},
            {"name": "high_performance", "ram_gb": 32, "gpu_required": true, "suitable": ram_gb >= 32 && has_gpu},
        ]
    });
    
    Ok(ResponseValue::chat(
        "recipes",
        "system",
        recipes.to_string(),
        TokenUsage::zero(),
    ))
}

/// Handle get embedding models request.
pub async fn handle_get_embedding_models(_state: &AppState) -> Result<ResponseValue> {
    let models = serde_json::json!([
        {"name": "all-MiniLM-L6-v2", "dimensions": 384, "type": "sentence-transformers"},
        {"name": "bge-small-en-v1.5", "dimensions": 384, "type": "bge"},
        {"name": "e5-small-v2", "dimensions": 384, "type": "e5"},
    ]);
    
    Ok(ResponseValue::chat(
        "embedding_models",
        "system",
        models.to_string(),
        TokenUsage::zero(),
    ))
}

/// Handle get loaded models request.
pub async fn handle_get_loaded_models(state: &AppState) -> Result<ResponseValue> {
    let loaded = state.list_loaded_models();
    
    Ok(ResponseValue::chat(
        "loaded_models",
        "system",
        serde_json::json!(loaded).to_string(),
        TokenUsage::zero(),
    ))
}

/// Handle select model request.
pub async fn handle_select_model(_state: &AppState, model_id: &str) -> Result<ResponseValue> {
    tracing::info!("Selected model: {}", model_id);
    Ok(ResponseValue::chat(
        "model_selected",
        "system",
        format!("Model {} selected as active", model_id),
        TokenUsage::zero(),
    ))
}

/// Handle pull model request.
pub async fn handle_pull_model(
    state: &AppState,
    model: &str,
    quantization: Option<&str>,
) -> Result<ResponseValue> {
    tracing::info!("Pull model request: {} (quant: {:?})", model, quantization);
    
    // 1. Scan repo to discover variants (if not already scanned)
    let manifest = state.cache.scan_repo(model).await
        .context("Failed to scan repository")?;
    
    // 2. Download specific quant (or default if None)
    let quant_to_download = quantization.unwrap_or_else(|| {
        // Use first available quant if no specific quant requested
        manifest.quants.keys().next()
            .map(|k| k.as_str())
            .unwrap_or("default")
    });
    
    if let Some(quant_info) = manifest.quants.get(quant_to_download) {
        // Download all files for this quant variant
        for file in &quant_info.files {
            tracing::debug!("Downloading file: {}", file);
            state.cache.download_file(model, file, None).await
                .with_context(|| format!("Failed to download file: {}", file))?;
        }
        
        Ok(ResponseValue::chat(
            "downloaded",
            "system",
            format!("Model {} (variant: {}) downloaded successfully", model, quant_to_download),
            TokenUsage::zero(),
        ))
    } else {
        anyhow::bail!("Quantization variant '{}' not found for model '{}'. Available: {:?}", 
                     quant_to_download, model, manifest.quants.keys().collect::<Vec<_>>())
    }
}

/// Handle delete model request.
pub async fn handle_delete_model(_state: &AppState, model_id: &str) -> Result<ResponseValue> {
    tracing::info!("Delete model request: {}", model_id);
    anyhow::bail!("Model deletion not yet implemented")
}

