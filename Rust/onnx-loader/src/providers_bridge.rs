//! Bridge between tabagent-execution-providers and ort execution providers
//!
//! This module converts our universal, format-agnostic execution providers
//! into ort-specific execution provider dispatch types.

use crate::error::{OnnxError, Result};
use tabagent_execution_providers::{ExecutionProvider, BackendType};
use std::sync::Arc;

/// Convert tabagent execution providers to ort execution providers
///
/// # Arguments
/// * `providers` - Slice of universal execution providers from tabagent-execution-providers
///
/// # Returns
/// Vector of ort ExecutionProviderDispatch ready to be used with SessionBuilder
///
/// # Note
/// Not all providers may be supported. Unsupported providers are logged and skipped.
pub fn bridge_to_ort(
    providers: &[Arc<dyn ExecutionProvider>]
) -> Result<Vec<ort::execution_providers::ExecutionProviderDispatch>> {
    let mut ort_providers = Vec::new();
    
    for provider in providers {
        match convert_provider(provider) {
            Some(ort_provider) => {
                log::info!("Bridged {} provider to ort", provider.name());
                ort_providers.push(ort_provider);
            }
            None => {
                log::warn!(
                    "Provider {} ({:?}) not yet supported for ONNX, skipping",
                    provider.name(),
                    provider.backend_type()
                );
            }
        }
    }
    
    if ort_providers.is_empty() {
        log::warn!("No providers successfully bridged, falling back to CPU");
        ort_providers.push(ort::execution_providers::CPUExecutionProvider::default().build());
    }
    
    Ok(ort_providers)
}

/// Convert a single execution provider to ort format
fn convert_provider(provider: &Arc<dyn ExecutionProvider>) -> Option<ort::execution_providers::ExecutionProviderDispatch> {
    use tabagent_execution_providers::constants::*;
    
    let config = provider.config();
    
    match provider.backend_type() {
        BackendType::Cuda => {
            #[cfg(feature = "cuda")]
            {
                let mut cuda = ort::execution_providers::CUDAExecutionProvider::default();
                
                // Device selection
                if let Some(device_id) = config.get(DEVICE_ID) {
                    if let Ok(id) = device_id.parse::<i32>() {
                        cuda = cuda.with_device_id(id);
                    }
                }
                
                // Memory limit
                if let Some(mem_limit) = config.get(GPU_MEM_LIMIT) {
                    if let Ok(limit) = mem_limit.parse::<usize>() {
                        cuda = cuda.with_memory_limit(limit);
                    }
                }
                
                log::debug!("Configured CUDA provider with {} options", config.iter().count());
                Some(cuda.build())
            }
            #[cfg(not(feature = "cuda"))]
            {
                log::warn!("CUDA provider requested but cuda feature not enabled");
                None
            }
        }
        
        BackendType::TensorRT => {
            #[cfg(feature = "tensorrt")]
            {
                let mut trt = ort::execution_providers::TensorRTExecutionProvider::default();
                
                // Device selection
                if let Some(device_id) = config.get(DEVICE_ID) {
                    if let Ok(id) = device_id.parse::<i32>() {
                        trt = trt.with_device_id(id);
                    }
                }
                
                // FP16 mode
                if config.get(TRT_FP16_ENABLE) == Some(&"true".to_string()) ||
                   config.get(TRT_FP16_ENABLE) == Some(&"1".to_string()) {
                    trt = trt.with_fp16(true);
                }
                
                // INT8 mode
                if config.get(TRT_INT8_ENABLE) == Some(&"true".to_string()) {
                    trt = trt.with_int8(true);
                }
                
                // Max workspace size
                if let Some(max_ws) = config.get(TRT_MAX_WORKSPACE_SIZE) {
                    if let Ok(size) = max_ws.parse::<usize>() {
                        trt = trt.with_max_workspace_size(size);
                    }
                }
                
                // Engine cache
                if config.get(TRT_ENGINE_CACHE_ENABLE) == Some(&"true".to_string()) {
                    if let Some(cache_path) = config.get(TRT_ENGINE_CACHE_PATH) {
                        trt = trt.with_engine_cache(true).with_engine_cache_path(cache_path);
                    }
                }
                
                // Timing cache
                if config.get(TRT_TIMING_CACHE_ENABLE) == Some(&"true".to_string()) {
                    trt = trt.with_timing_cache(true);
                }
                
                log::debug!("Configured TensorRT provider with {} options", config.iter().count());
                Some(trt.build())
            }
            #[cfg(not(feature = "tensorrt"))]
            {
                log::warn!("TensorRT provider requested but tensorrt feature not enabled");
                None
            }
        }
        
        BackendType::DirectML => {
            #[cfg(feature = "directml")]
            {
                let mut dml = ort::execution_providers::DirectMLExecutionProvider::default();
                
                // Device selection
                if let Some(device_id) = config.get(DEVICE_ID) {
                    if let Ok(id) = device_id.parse::<i32>() {
                        dml = dml.with_device_id(id);
                    }
                }
                
                log::debug!("Configured DirectML provider");
                Some(dml.build())
            }
            #[cfg(not(feature = "directml"))]
            {
                log::warn!("DirectML provider requested but directml feature not enabled");
                None
            }
        }
        
        BackendType::CoreML => {
            #[cfg(all(target_os = "macos", feature = "coreml"))]
            {
                let coreml = ort::execution_providers::CoreMLExecutionProvider::default();
                log::debug!("Configured CoreML provider");
                Some(coreml.build())
            }
            #[cfg(not(all(target_os = "macos", feature = "coreml")))]
            {
                log::warn!("CoreML only available on macOS with coreml feature");
                None
            }
        }
        
        BackendType::CPU => {
            let cpu = ort::execution_providers::CPUExecutionProvider::default();
            log::debug!("Configured CPU provider");
            Some(cpu.build())
        }
        
        // Add more providers as needed
        BackendType::ROCm => {
            #[cfg(feature = "rocm")]
            {
                let rocm = ort::execution_providers::ROCmExecutionProvider::default();
                log::debug!("Configured ROCm provider");
                Some(rocm.build())
            }
            #[cfg(not(feature = "rocm"))]
            {
                log::warn!("ROCm provider not enabled");
                None
            }
        }
        
        BackendType::OpenVINO => {
            #[cfg(feature = "openvino")]
            {
                let openvino = ort::execution_providers::OpenVINOExecutionProvider::default();
                log::debug!("Configured OpenVINO provider");
                Some(openvino.build())
            }
            #[cfg(not(feature = "openvino"))]
            {
                log::warn!("OpenVINO provider not enabled");
                None
            }
        }
        
        _ => {
            log::warn!("Provider {:?} not supported for ONNX", provider.backend_type());
            None
        }
    }
}

/// Auto-select execution providers based on hardware
///
/// This is a convenience function that uses the hardware detection
/// from tabagent-execution-providers to automatically select the best
/// providers for the current system.
pub fn auto_select_providers() -> Result<Vec<ort::execution_providers::ExecutionProviderDispatch>> {
    use tabagent_execution_providers::{
        CUDAExecutionProvider, TensorRTExecutionProvider,
        DirectMLExecutionProvider, CPUExecutionProvider
    };
    use tabagent_hardware::{detect_system, GpuVendor};
    
    let hw = detect_system()
        .map_err(|e| OnnxError::SessionCreationFailed(format!("Hardware detection failed: {}", e)))?;
    
    let mut providers: Vec<Arc<dyn ExecutionProvider>> = Vec::new();
    
    // Try GPU providers first
    if let Some(gpu) = hw.gpus.first() {
        match gpu.vendor {
            GpuVendor::Nvidia => {
                log::info!("NVIDIA GPU detected, adding TensorRT and CUDA");
                
                // TensorRT (with optimization)
                providers.push(
                    TensorRTExecutionProvider::new()
                        .with_fp16_enable(true)
                        .with_engine_cache_enable(true)
                        .build()
                );
                
                // CUDA (fallback)
                providers.push(
                    CUDAExecutionProvider::new()
                        .with_device_id(0)
                        .build()
                );
            }
            GpuVendor::Amd => {
                #[cfg(target_os = "windows")]
                {
                    log::info!("AMD GPU detected (Windows), adding DirectML");
                    providers.push(DirectMLExecutionProvider::new().build());
                }
            }
            GpuVendor::Intel => {
                #[cfg(target_os = "windows")]
                {
                    log::info!("Intel GPU detected (Windows), adding DirectML");
                    providers.push(DirectMLExecutionProvider::new().build());
                }
            }
            _ => {}
        }
    }
    
    // Always add CPU fallback
    providers.push(CPUExecutionProvider::new().build());
    
    // Convert to ort providers
    bridge_to_ort(&providers)
}
