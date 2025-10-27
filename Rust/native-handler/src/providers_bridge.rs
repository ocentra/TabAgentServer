//! Execution Providers Bridge - PyO3 bindings for tabagent-execution-providers
//!
//! Exposes execution provider info to Python/UI

use pyo3::prelude::*;
use serde_json::json;
use tabagent_execution_providers::{
    BackendType, CPUExecutionProvider, CUDAExecutionProvider, TensorRTExecutionProvider,
    DirectMLExecutionProvider, ROCmExecutionProvider, ExecutionProvider
};
use tabagent_hardware::{detect_system, GpuVendor};
use std::sync::Arc;

/// Get list of available execution providers for this system
#[pyfunction]
pub fn get_available_providers() -> PyResult<String> {
    let hw = detect_system()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            format!("Hardware detection failed: {}", e)
        ))?;
    
    let mut providers = Vec::new();
    
    // Always have CPU
    providers.push(json!({
        "name": "CPU",
        "backend_type": "CPU",
        "available": true,
        "platform_supported": true,
        "description": "CPU execution (always available)"
    }));
    
    // Check GPU-specific providers
    if let Some(gpu) = hw.gpus.first() {
        match gpu.vendor {
            GpuVendor::Nvidia => {
                // CUDA
                let cuda = CUDAExecutionProvider::new().build();
                providers.push(json!({
                    "name": "CUDA",
                    "backend_type": "CUDA",
                    "available": cuda.is_available(),
                    "platform_supported": cuda.supported_by_platform(),
                    "description": "NVIDIA CUDA GPU acceleration"
                }));
                
                // TensorRT
                let trt = TensorRTExecutionProvider::new().build();
                providers.push(json!({
                    "name": "TensorRT",
                    "backend_type": "TensorRT",
                    "available": trt.is_available(),
                    "platform_supported": trt.supported_by_platform(),
                    "description": "NVIDIA TensorRT optimized inference"
                }));
            }
            GpuVendor::Amd => {
                #[cfg(target_os = "windows")]
                {
                    let dml = DirectMLExecutionProvider::new().build();
                    providers.push(json!({
                        "name": "DirectML",
                        "backend_type": "DirectML",
                        "available": dml.is_available(),
                        "platform_supported": dml.supported_by_platform(),
                        "description": "DirectML GPU acceleration (AMD/Intel/NVIDIA on Windows)"
                    }));
                }
                
                #[cfg(target_os = "linux")]
                {
                    let rocm = ROCmExecutionProvider::new().build();
                    providers.push(json!({
                        "name": "ROCm",
                        "backend_type": "ROCm",
                        "available": rocm.is_available(),
                        "platform_supported": rocm.supported_by_platform(),
                        "description": "AMD ROCm GPU acceleration"
                    }));
                }
            }
            GpuVendor::Intel => {
                #[cfg(target_os = "windows")]
                {
                    let dml = DirectMLExecutionProvider::new().build();
                    providers.push(json!({
                        "name": "DirectML",
                        "backend_type": "DirectML",
                        "available": dml.is_available(),
                        "platform_supported": dml.supported_by_platform(),
                        "description": "DirectML GPU acceleration (Intel/AMD/NVIDIA on Windows)"
                    }));
                }
            }
            _ => {}
        }
    }
    
    let response = json!({
        "providers": providers,
        "gpu_detected": !hw.gpus.is_empty(),
        "gpu_vendor": hw.gpus.first().map(|g| format!("{}", g.vendor)),
    });
    
    Ok(response.to_string())
}

/// Get recommended execution providers for a given task
#[pyfunction]
pub fn get_recommended_providers(task: String) -> PyResult<String> {
    let hw = detect_system()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            format!("Hardware detection failed: {}", e)
        ))?;
    
    let mut recommended = Vec::new();
    
    if let Some(gpu) = hw.gpus.first() {
        match gpu.vendor {
            GpuVendor::Nvidia => {
                recommended.push("TensorRT");
                recommended.push("CUDA");
            }
            GpuVendor::Amd => {
                #[cfg(target_os = "windows")]
                recommended.push("DirectML");
                #[cfg(target_os = "linux")]
                recommended.push("ROCm");
            }
            GpuVendor::Intel => {
                #[cfg(target_os = "windows")]
                recommended.push("DirectML");
            }
            _ => {}
        }
    }
    
    recommended.push("CPU"); // Always fallback
    
    let response = json!({
        "recommended": recommended,
        "reasoning": format!(
            "Based on {} GPU detected",
            hw.gpus.first().map(|g| format!("{} {}", g.vendor, g.name)).unwrap_or_else(|| "No".to_string())
        ),
    });
    
    Ok(response.to_string())
}

/// Register provider bridge functions with Python
pub fn register_provider_functions(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_available_providers, m)?)?;
    m.add_function(wrap_pyfunction!(get_recommended_providers, m)?)?;
    Ok(())
}

