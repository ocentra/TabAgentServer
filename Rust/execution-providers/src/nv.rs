//! NV Execution Provider
//! 
//! NVIDIA TensorRT RTX (NV) execution provider.

use crate::{BackendType, impl_provider_base, ExecutionProvider, ProviderConfig, ProviderError, Result};
use crate::constants::*;

#[cfg(target_os = "windows")]
use tabagent_hardware::{detect_system, GpuVendor};

#[derive(Debug, Clone)]
pub struct NVExecutionProvider {
    config: ProviderConfig,
}

impl_provider_base!(
    NVExecutionProvider,
    "NvTensorRTRTXExecutionProvider",
    BackendType::NVExecutionProvider
);

impl NVExecutionProvider {
    /// Set device ID
    pub fn with_device_id(mut self, device_id: u32) -> Self {
        self.config.set(EP_NV_DEVICE_ID, device_id);
        self
    }
    
    /// Enable CUDA graph
    pub fn with_cuda_graph(mut self, enable: bool) -> Self {
        self.config.set(EP_NV_CUDA_GRAPH_ENABLE, enable);
        self
    }
}

impl ExecutionProvider for NVExecutionProvider {
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
        cfg!(all(target_os = "windows", target_arch = "x86_64"))
    }
    
    fn is_available(&self) -> Result<bool> {
        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        {
            let system = detect_system()
                .map_err(|e| ProviderError::Hardware(e.to_string()))?;
            
            Ok(system.gpus.iter().any(|gpu| matches!(gpu.vendor, GpuVendor::Nvidia)))
        }
        
        #[cfg(not(all(target_os = "windows", target_arch = "x86_64")))]
        Ok(false)
    }
}

