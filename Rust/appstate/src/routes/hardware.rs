//! Hardware detection and feasibility check handlers.

use anyhow::Result;
use tabagent_values::ResponseValue;

use crate::AppState;

/// Handle get hardware info request.
pub async fn handle_get_info(state: &AppState) -> Result<ResponseValue> {
    tracing::debug!("Getting hardware information");
    
    let hw = &state.hardware;
    
    // Build comprehensive hardware info response
    let info = serde_json::json!({
        "cpu": {
            "vendor": format!("{:?}", hw.cpu.vendor),
            "architecture": format!("{:?}", hw.cpu.architecture),
            "model_name": hw.cpu.model_name,
            "cores": hw.cpu.cores,
            "threads": hw.cpu.threads,
            "family": hw.cpu.family,
            "model": hw.cpu.model,
            "stepping": hw.cpu.stepping,
        },
        "memory": {
            "total_ram_mb": hw.memory.total_ram_mb,
            "available_ram_mb": hw.memory.available_ram_mb,
            "used_ram_mb": hw.memory.used_ram_mb,
            "ram_tier": hw.ram_tier,
        },
        "gpus": hw.gpus.iter().enumerate().map(|(idx, gpu)| {
            serde_json::json!({
                "index": idx,
                "name": gpu.name,
                "vendor": format!("{:?}", gpu.vendor),
                "vram_mb": gpu.vram_mb,
                "driver_version": gpu.driver_version,
            })
        }).collect::<Vec<_>>(),
        "vram": {
            "total_vram_mb": hw.total_vram_mb,
            "vram_tier": hw.vram_tier,
        },
        "execution_provider": format!("{:?}", hw.recommended_execution_provider()),
        "bitnet_dll_variant": hw.bitnet_dll_variant(),
        "bitnet_dll_filename": hw.bitnet_dll_filename(),
    });
    
    Ok(ResponseValue::generic(info))
}

/// Handle check model feasibility request.
pub async fn handle_check_feasibility(state: &AppState, model_size_mb: u64) -> Result<ResponseValue> {
    tracing::debug!("Checking model feasibility for size: {} MB", model_size_mb);
    
    let hw = &state.hardware;
    let available_ram = hw.memory.available_ram_mb;
    let available_vram = hw.total_vram_mb;
    
    // Simple feasibility check
    let can_load_ram = model_size_mb < available_ram;
    let can_load_vram = model_size_mb < available_vram;
    let can_load = can_load_ram || can_load_vram;
    
    let recommendation = if can_load_vram {
        format!("Model can fit in VRAM ({} MB available)", available_vram)
    } else if can_load_ram {
        format!("Model can fit in RAM ({} MB available), will run on CPU", available_ram)
    } else {
        format!("Model too large! Need {} MB but only have {} MB RAM and {} MB VRAM", 
                model_size_mb, available_ram, available_vram)
    };
    
    Ok(ResponseValue::generic(serde_json::json!({
        "can_load": can_load,
        "model_size_mb": model_size_mb,
        "available_ram_mb": available_ram,
        "available_vram_mb": available_vram,
        "recommendation": recommendation,
    })))
}

/// Handle get recommended models request.
pub async fn handle_get_recommended(state: &AppState) -> Result<ResponseValue> {
    tracing::debug!("Getting recommended model sizes");
    
    let hw = &state.hardware;
    let available_ram = hw.memory.available_ram_mb;
    let available_vram = hw.total_vram_mb;
    
    // Recommend models based on available memory (conservative: use 70% of available)
    let safe_ram = (available_ram as f64 * 0.7) as u64;
    let safe_vram = (available_vram as f64 * 0.7) as u64;
    
    let recommendations = if safe_vram >= 24000 {
        vec!["70B", "34B", "13B", "7B", "3B"]
    } else if safe_vram >= 12000 {
        vec!["34B", "13B", "7B", "3B", "1B"]
    } else if safe_vram >= 6000 {
        vec!["13B", "7B", "3B", "1B"]
    } else if safe_ram >= 16000 {
        vec!["7B", "3B", "1B"]
    } else if safe_ram >= 8000 {
        vec!["3B", "1B"]
    } else {
        vec!["1B"]
    };
    
    Ok(ResponseValue::generic(serde_json::json!({
        "available_ram_mb": available_ram,
        "available_vram_mb": available_vram,
        "safe_ram_mb": safe_ram,
        "safe_vram_mb": safe_vram,
        "recommended_sizes": recommendations,
        "recommendation": format!(
            "For your system ({} MB RAM, {} MB VRAM), we recommend models up to {}",
            available_ram, available_vram, recommendations[0]
        ),
    })))
}

