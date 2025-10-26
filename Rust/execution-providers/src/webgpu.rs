//! WebGPU Execution Provider
//! 
//! WebGPU for GPU acceleration in browsers and native applications.

use crate::{BackendType, impl_provider_base, ExecutionProvider, ProviderConfig, Result};
use crate::constants::*;

#[derive(Debug, Clone)]
pub struct WebGPUExecutionProvider {
    config: ProviderConfig,
}

impl_provider_base!(
    WebGPUExecutionProvider,
    "WebGpuExecutionProvider",
    BackendType::WebGPU
);

impl WebGPUExecutionProvider {
    /// Set preferred layout (NCHW or NHWC)
    pub fn with_preferred_layout(mut self, layout: &str) -> Self {
        self.config.set(WEBGPU_PREFERRED_LAYOUT, layout);
        self
    }
    
    /// Enable graph capture
    pub fn with_enable_graph_capture(mut self, enable: bool) -> Self {
        self.config.set(WEBGPU_ENABLE_GRAPH_CAPTURE, enable);
        self
    }
    
    /// Set device ID
    pub fn with_device_id(mut self, id: i32) -> Self {
        self.config.set(WEBGPU_DEVICE_ID, id);
        self
    }
    
    /// Set storage buffer cache mode
    pub fn with_storage_buffer_cache_mode(mut self, mode: &str) -> Self {
        self.config.set(WEBGPU_STORAGE_BUFFER_CACHE_MODE, mode);
        self
    }
    
    /// Set validation mode (disabled, wgpuOnly, basic, full)
    pub fn with_validation_mode(mut self, mode: &str) -> Self {
        self.config.set(WEBGPU_VALIDATION_MODE, mode);
        self
    }
}

impl ExecutionProvider for WebGPUExecutionProvider {
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
        cfg!(any(target_os = "windows", target_os = "linux", target_arch = "wasm32"))
    }
    
    fn is_available(&self) -> Result<bool> {
        // WebGPU availability depends on browser/driver support
        Ok(self.supported_by_platform())
    }
}

