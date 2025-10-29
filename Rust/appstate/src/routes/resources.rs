//! Resource management handlers.

use anyhow::Result;
use tabagent_values::{ResponseValue, TokenUsage};

use crate::AppState;

/// Handle get resources request.
pub async fn handle_get_resources(state: &AppState) -> Result<ResponseValue> {
    let hw_info = &state.hardware;
    let resources = serde_json::json!({
        "cpu_cores": hw_info.cpu.cores,
        "total_memory_mb": hw_info.memory.total_ram_mb,
        "available_memory_mb": hw_info.memory.available_ram_mb,
        "used_memory_mb": hw_info.memory.used_ram_mb,
        "gpu_count": hw_info.gpus.len(),
        "total_vram_mb": hw_info.total_vram_mb,
        "ram_tier": &hw_info.ram_tier,
        "vram_tier": &hw_info.vram_tier,
        "os": format!("{} {}", hw_info.os.name, hw_info.os.version),
    });
    
    Ok(ResponseValue::chat(
        "resources",
        "system",
        resources.to_string(),
        TokenUsage::zero(),
    ))
}

/// Handle estimate memory request.
pub async fn handle_estimate_memory(
    state: &AppState,
    model: &str,
    quantization: Option<&str>,
) -> Result<ResponseValue> {
    // Rough estimation based on model size heuristics
    let base_size_gb = if model.contains("7b") { 7.0 }
        else if model.contains("13b") { 13.0 }
        else if model.contains("70b") { 70.0 }
        else { 3.0 };
    
    let multiplier = match quantization {
        Some("q4") | Some("Q4") => 0.25,
        Some("q5") | Some("Q5") => 0.3125,
        Some("q8") | Some("Q8") => 0.5,
        Some("fp16") | Some("FP16") => 0.5,
        _ => 1.0, // fp32
    };
    
    let estimated_gb = base_size_gb * multiplier;
    let estimated_mb = (estimated_gb * 1024.0) as u64;
    
    // Get loading strategy recommendation from hardware crate
    let loading_strategy = state.hardware.recommended_loading_strategy(estimated_mb);
    let can_load = state.hardware.memory.available_ram_mb >= estimated_mb;
    
    Ok(ResponseValue::chat(
        "memory_estimate",
        "system",
        serde_json::json!({
            "model": model,
            "quantization": quantization,
            "estimated_memory_gb": estimated_gb,
            "estimated_memory_mb": estimated_mb,
            "available_memory_mb": state.hardware.memory.available_ram_mb,
            "can_load": can_load,
            "loading_strategy": format!("{:?}", loading_strategy),
            "recommended_total_gb": estimated_gb * 1.5,
        }).to_string(),
        TokenUsage::zero(),
    ))
}

