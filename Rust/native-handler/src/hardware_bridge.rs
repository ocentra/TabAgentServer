//! Hardware Bridge - PyO3 bindings for tabagent-hardware
//!
//! Exposes ResourceManager and hardware detection to Python/UI

use pyo3::prelude::*;
use serde_json::json;
use tabagent_hardware::{
    detect_system,
    CpuArchitecture, CpuVendor, GpuVendor
};

/// Get complete system hardware information
#[pyfunction]
pub fn get_hardware_info() -> PyResult<String> {
    let system = detect_system()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            format!("Failed to detect hardware: {}", e)
        ))?;
    
    // Get execution provider recommendation
    let provider_rec = system.recommended_execution_provider();
    
    // Build complete response with all hardware info
    let response = json!({
        "cpu": {
            "vendor": format!("{:?}", system.cpu.vendor),
            "architecture": format!("{:?}", system.cpu.architecture),
            "model_name": system.cpu.model_name,
            "cores": system.cpu.cores,
            "threads": system.cpu.threads,
            "family": system.cpu.family,
            "model": system.cpu.model,
            "stepping": system.cpu.stepping,
            "bitnet_dll_variant": system.bitnet_dll_variant(),
            "bitnet_dll_filename": system.bitnet_dll_filename(),
        },
        "memory": {
            "total_ram_mb": system.memory.total_ram_mb,
            "available_ram_mb": system.memory.available_ram_mb,
            "used_ram_mb": system.memory.used_ram_mb,
            "ram_tier": system.ram_tier,
        },
        "gpus": system.gpus.iter().enumerate().map(|(idx, gpu)| {
            json!({
                "index": idx,
                "name": gpu.name,
                "vendor": format!("{:?}", gpu.vendor),
                "vram_mb": gpu.vram_mb,
                "driver_version": gpu.driver_version,
            })
        }).collect::<Vec<_>>(),
        "vram": {
            "total_vram_mb": system.total_vram_mb,
            "vram_tier": system.vram_tier,
        },
        "execution_provider": {
            "primary": provider_rec.primary,
            "fallbacks": provider_rec.fallbacks,
            "reason": provider_rec.reason,
        },
        "os": {
            "name": system.os.name,
            "version": system.os.version,
            "arch": system.os.arch,
        }
    });
    
    Ok(response.to_string())
}

/// Check if we can load a model of given size
#[pyfunction]
pub fn can_load_model(model_size_mb: u64, _prefer_gpu: bool) -> PyResult<String> {
    let system = detect_system()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            format!("Failed to detect hardware: {}", e)
        ))?;
    
    let strategy = system.recommended_loading_strategy(model_size_mb);
    
    let response = json!({
        "can_load": true,
        "target": strategy.target,
        "gpu_index": strategy.gpu_index,
        "gpu_percent": strategy.gpu_percent,
        "cpu_percent": strategy.cpu_percent,
        "reason": strategy.reason,
    });
    
    Ok(response.to_string())
}

/// Calculate maximum concurrent agents
#[pyfunction]
pub fn max_concurrent_agents(_agent_size_mb: u64, _prefer_gpu: bool) -> PyResult<String> {
    // TODO: Re-implement with hardware resource management
    let response = json!({
        "max_on_gpu": 0,
        "max_on_cpu": 1,
        "max_total": 1,
        "per_agent_mb": 0,
        "note": "Resource management not yet fully implemented"
    });
    
    /*
    let rm = ResourceManager::new()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            format!("Failed to detect hardware: {}", e)
        ))?;
    
    let estimate = rm.max_concurrent_agents(agent_size_mb, prefer_gpu);
    
    let response = json!({
        "max_on_gpu": estimate.max_on_gpu,
        "max_on_cpu": estimate.max_on_cpu,
        "max_total": estimate.max_total,
        "per_agent_mb": estimate.per_agent_mb,
    });
    */
    
    Ok(response.to_string())
}

/// Get quick resource summary for UI
#[pyfunction]
pub fn get_resource_summary() -> PyResult<String> {
    let system = detect_system()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            format!("Failed to detect hardware: {}", e)
        ))?;
    
    let ram_gb = system.memory.total_ram_mb as f32 / 1024.0;
    let vram_gb = system.total_vram_mb as f32 / 1024.0;
    
    let response = json!({
        "cpu": format!("{} ({} cores)", system.cpu.model_name, system.cpu.cores),
        "ram": format!("{:.1} GB ({}, {:.1} GB available)", 
            ram_gb, 
            system.ram_tier,
            system.memory.available_ram_mb as f32 / 1024.0
        ),
        "vram": format!("{:.1} GB ({})", vram_gb, system.vram_tier),
        "gpus": system.gpus.iter().map(|gpu| {
            let vram_str = if let Some(vram_mb) = gpu.vram_mb {
                format!(" - {:.1} GB VRAM", vram_mb as f32 / 1024.0)
            } else {
                String::new()
            };
            format!("{}{}", gpu.name, vram_str)
        }).collect::<Vec<_>>(),
        "bitnet_dll": system.bitnet_dll_filename(),
    });
    
    Ok(response.to_string())
}

/// Check if a model can be loaded and get detailed recommendations
#[pyfunction]
pub fn check_model_feasibility(model_size_mb: u64) -> PyResult<String> {
    let system = detect_system()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            format!("Failed to detect hardware: {}", e)
        ))?;
    
    let strategy = system.recommended_loading_strategy(model_size_mb);
    
    // Determine if model can be loaded
    let can_load = strategy.target != "insufficient";
    
    // Calculate percentages for UI
    let model_gb = model_size_mb as f32 / 1024.0;
    let total_ram_gb = system.memory.total_ram_mb as f32 / 1024.0;
    let available_ram_gb = system.memory.available_ram_mb as f32 / 1024.0;
    let total_vram_gb = system.total_vram_mb as f32 / 1024.0;
    
    let response = json!({
        "can_load": can_load,
        "model_size_mb": model_size_mb,
        "model_size_gb": format!("{:.1}", model_gb),
        "strategy": {
            "target": strategy.target,
            "gpu_index": strategy.gpu_index,
            "gpu_percent": strategy.gpu_percent,
            "cpu_percent": strategy.cpu_percent,
            "reason": strategy.reason,
        },
        "hardware": {
            "total_ram_mb": system.memory.total_ram_mb,
            "total_ram_gb": format!("{:.1}", total_ram_gb),
            "available_ram_mb": system.memory.available_ram_mb,
            "available_ram_gb": format!("{:.1}", available_ram_gb),
            "ram_tier": system.ram_tier,
            "total_vram_mb": system.total_vram_mb,
            "total_vram_gb": format!("{:.1}", total_vram_gb),
            "vram_tier": system.vram_tier,
            "gpu_count": system.gpus.len(),
        },
        "recommendation": if can_load {
            format!(
                "✅ This {:.1}GB model can be loaded using: {}",
                model_gb,
                strategy.reason
            )
        } else {
            format!(
                "❌ This {:.1}GB model is too large. Available: {:.1}GB RAM, {:.1}GB VRAM. Consider using a smaller model variant.",
                model_gb,
                available_ram_gb,
                total_vram_gb
            )
        },
    });
    
    Ok(response.to_string())
}

/// Get recommended model size for current hardware
#[pyfunction]
pub fn get_recommended_model_size() -> PyResult<String> {
    let system = detect_system()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            format!("Failed to detect hardware: {}", e)
        ))?;
    
    // Conservative recommendations based on available memory
    let available_ram_gb = system.memory.available_ram_mb as f32 / 1024.0;
    let total_vram_gb = system.total_vram_mb as f32 / 1024.0;
    
    // For GPU loading: 80% of VRAM
    let max_gpu_model_gb = total_vram_gb * 0.8;
    
    // For CPU loading: 50% of available RAM (conservative)
    let max_cpu_model_gb = available_ram_gb * 0.5;
    
    // Overall max
    let max_model_gb = if total_vram_gb > 4.0 {
        max_gpu_model_gb.max(max_cpu_model_gb)
    } else {
        max_cpu_model_gb
    };
    
    let response = json!({
        "max_model_size_gb": format!("{:.1}", max_model_gb),
        "max_model_size_mb": (max_model_gb * 1024.0) as u64,
        "recommendations": {
            "max_gpu_model_gb": format!("{:.1}", max_gpu_model_gb),
            "max_cpu_model_gb": format!("{:.1}", max_cpu_model_gb),
        },
        "suggested_models": {
            "tiny": if max_model_gb >= 0.5 { "✅ 0.5GB models (e.g., TinyLlama)" } else { "❌ Not recommended" },
            "small": if max_model_gb >= 2.0 { "✅ 2GB models (e.g., Phi-2)" } else { "❌ Not recommended" },
            "medium": if max_model_gb >= 4.0 { "✅ 4GB models (e.g., Llama-7B-Q4)" } else { "❌ Not recommended" },
            "large": if max_model_gb >= 8.0 { "✅ 8GB models (e.g., Llama-13B-Q4)" } else { "❌ Not recommended" },
            "xlarge": if max_model_gb >= 16.0 { "✅ 16GB+ models (e.g., Llama-70B-Q4)" } else { "❌ Not recommended" },
        },
        "hardware_summary": {
            "ram": format!("{:.1}GB total, {:.1}GB available", 
                system.memory.total_ram_mb as f32 / 1024.0,
                available_ram_gb
            ),
            "vram": format!("{:.1}GB total", total_vram_gb),
            "tier": format!("RAM: {}, VRAM: {}", system.ram_tier, system.vram_tier),
        }
    });
    
    Ok(response.to_string())
}

/// Register hardware bridge functions with Python
pub fn register_hardware_functions(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_hardware_info, m)?)?;
    m.add_function(wrap_pyfunction!(can_load_model, m)?)?;
    m.add_function(wrap_pyfunction!(check_model_feasibility, m)?)?;
    m.add_function(wrap_pyfunction!(get_recommended_model_size, m)?)?;
    m.add_function(wrap_pyfunction!(max_concurrent_agents, m)?)?;
    m.add_function(wrap_pyfunction!(get_resource_summary, m)?)?;
    Ok(())
}

