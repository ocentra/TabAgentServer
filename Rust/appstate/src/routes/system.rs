//! System health and info handlers.

use anyhow::Result;
use tabagent_values::{ResponseValue, HealthStatus, TokenUsage};

use crate::AppState;

/// Handle health check request.
pub async fn handle_health(
    _state: &AppState,
) -> Result<ResponseValue> {
    Ok(ResponseValue::health(HealthStatus::Healthy))
}

/// Handle system info request.
pub async fn handle_system_info(
    state: &AppState,
) -> Result<ResponseValue> {
    let info = format!(
        "CPU: {} cores, Total RAM: {:.1}GB, GPUs: {}, VRAM: {} MB, OS: {:?}",
        state.hardware.cpu.cores,
        state.hardware.memory.total_ram_mb as f64 / 1024.0,
        state.hardware.gpus.len(),
        state.hardware.total_vram_mb,
        state.hardware.os.name
    );
    
    Ok(ResponseValue::chat(
        "success",
        "system",
        info,
        TokenUsage::zero(),
    ))
}

