//! Legacy execution provider configuration (deprecated)
//!
//! This module is kept for backward compatibility but is deprecated.
//! New code should use `tabagent-execution-providers` crate directly
//! and the `providers_bridge` module for conversion to ort.

#[deprecated(
    since = "0.1.0",
    note = "Use tabagent-execution-providers crate instead"
)]
pub use tabagent_execution_providers::{
    ExecutionProvider,
    BackendType,
    CUDAExecutionProvider,
    TensorRTExecutionProvider,
    DirectMLExecutionProvider,
    CPUExecutionProvider,
};

/// Auto-select execution providers (deprecated)
///
/// Use `providers_bridge::auto_select_providers()` instead.
#[deprecated(
    since = "0.1.0",
    note = "Use providers_bridge::auto_select_providers() instead"
)]
pub fn auto_select() -> Vec<std::sync::Arc<dyn ExecutionProvider>> {
    use tabagent_hardware::{detect_system, GpuVendor};
    
    let hw = match detect_system() {
        Ok(hw) => hw,
        Err(e) => {
            log::warn!("Failed to detect hardware: {}, using CPU only", e);
            return vec![CPUExecutionProvider::new().build()];
        }
    };
    
    let mut providers: Vec<std::sync::Arc<dyn ExecutionProvider>> = Vec::new();
    
    // Try GPU providers first
    if let Some(gpu) = hw.gpus.first() {
        match gpu.vendor {
            GpuVendor::Nvidia => {
                log::info!("NVIDIA GPU detected, adding TensorRT and CUDA providers");
                providers.push(
                    TensorRTExecutionProvider::new()
                        .with_fp16_enable(true)
                        .build()
                );
                providers.push(
                    CUDAExecutionProvider::new()
                        .with_device_id(0)
                        .build()
                );
            }
            GpuVendor::Amd => {
                #[cfg(target_os = "windows")]
                {
                    log::info!("AMD GPU detected (Windows), adding DirectML provider");
                    providers.push(DirectMLExecutionProvider::new().build());
                }
            }
            GpuVendor::Intel => {
                #[cfg(target_os = "windows")]
                {
                    log::info!("Intel GPU detected (Windows), adding DirectML provider");
                    providers.push(DirectMLExecutionProvider::new().build());
                }
            }
            _ => {}
        }
    }
    
    // Always add CPU as fallback
    providers.push(CPUExecutionProvider::new().build());
    
    log::info!("Auto-selected {} execution providers", providers.len());
    providers
}
