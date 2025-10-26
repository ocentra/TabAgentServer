//! MIGraphX Execution Provider
//! 
//! AMD MIGraphX for GPU acceleration on AMD GPUs.

use crate::{BackendType, impl_provider_base, ExecutionProvider, ProviderConfig, ProviderError, Result};
use crate::constants::*;

#[cfg(any(target_os = "linux", target_os = "windows"))]
use tabagent_hardware::{detect_system, GpuVendor};

#[derive(Debug, Clone)]
pub struct MIGraphXExecutionProvider {
    config: ProviderConfig,
}

impl_provider_base!(
    MIGraphXExecutionProvider,
    "MIGraphXExecutionProvider",
    BackendType::MIGraphX
);

impl MIGraphXExecutionProvider {
    /// Set device ID
    pub fn with_device_id(mut self, device_id: i32) -> Self {
        self.config.set(DEVICE_ID, device_id);
        self
    }
    
    /// Enable FP16 quantization
    pub fn with_fp16(mut self, enable: bool) -> Self {
        self.config.set(MIGRAPHX_FP16_ENABLE, enable);
        self
    }
    
    /// Enable INT8 quantization
    pub fn with_int8(mut self, enable: bool) -> Self {
        self.config.set(MIGRAPHX_INT8_ENABLE, enable);
        self
    }
    
    /// Set INT8 calibration table path
    pub fn with_int8_calibration_table(mut self, path: &str, native: bool) -> Self {
        self.config.set(MIGRAPHX_INT8_CALIBRATION_TABLE_NAME, path);
        self.config.set(MIGRAPHX_USE_NATIVE_CALIBRATION_TABLE, native);
        self
    }
    
    /// Save compiled model to path
    pub fn with_save_model(mut self, path: &str) -> Self {
        self.config.set(MIGRAPHX_SAVE_MODEL_PATH, path);
        self.config.set(MIGRAPHX_SAVE_COMPILED_MODEL, true);
        self
    }
    
    /// Load compiled model from path
    pub fn with_load_model(mut self, path: &str) -> Self {
        self.config.set(MIGRAPHX_LOAD_MODEL_PATH, path);
        self.config.set(MIGRAPHX_LOAD_COMPILED_MODEL, true);
        self
    }
    
    /// Enable exhaustive tuning (slower load, faster inference)
    pub fn with_exhaustive_tune(mut self, enable: bool) -> Self {
        self.config.set(MIGRAPHX_EXHAUSTIVE_TUNE, enable);
        self
    }
}

impl ExecutionProvider for MIGraphXExecutionProvider {
    fn name(&self) -> &'static str {
        self.get_name()
    }
    
    fn backend_type(&self) -> BackendType {
        self.get_backend_type()
    }
    
    fn config(&self) -> &ProviderConfig {
        &self.config
    }
    
    fn supported_by_platform(&self) -> bool {
        cfg!(any(
            all(target_os = "linux", target_arch = "x86_64"),
            all(target_os = "windows", target_arch = "x86_64")
        ))
    }
    
    fn is_available(&self) -> Result<bool> {
        #[cfg(any(
            all(target_os = "linux", target_arch = "x86_64"),
            all(target_os = "windows", target_arch = "x86_64")
        ))]
        {
            let system = detect_system()
                .map_err(|e| ProviderError::Hardware(e.to_string()))?;
            
            Ok(system.gpus.iter().any(|gpu| matches!(gpu.vendor, GpuVendor::Amd)))
        }
        
        #[cfg(not(any(
            all(target_os = "linux", target_arch = "x86_64"),
            all(target_os = "windows", target_arch = "x86_64")
        )))]
        Ok(false)
    }
}

