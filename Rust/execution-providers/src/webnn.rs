//! WebNN Execution Provider
//! 
//! Web Neural Network API for browser-based ML acceleration.

use crate::{BackendType, impl_provider_base, ExecutionProvider, ProviderConfig, Result};
use crate::constants::*;

#[derive(Debug, Clone)]
pub struct WebNNExecutionProvider {
    config: ProviderConfig,
}

impl_provider_base!(
    WebNNExecutionProvider,
    "WebNNExecutionProvider",
    BackendType::WebNN
);

impl WebNNExecutionProvider {
    /// Set device type (cpu, gpu, npu)
    pub fn with_device_type(mut self, device_type: &str) -> Self {
        self.config.set(DEVICE_TYPE_WEBNN, device_type);
        self
    }
    
    /// Set power preference (default, high-performance, low-power)
    pub fn with_power_preference(mut self, pref: &str) -> Self {
        self.config.set(POWER_PREFERENCE, pref);
        self
    }
    
    /// Set number of threads
    pub fn with_threads(mut self, threads: u32) -> Self {
        self.config.set(NUM_THREADS, threads);
        self
    }
}

impl ExecutionProvider for WebNNExecutionProvider {
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
        cfg!(target_arch = "wasm32")
    }
    
    fn is_available(&self) -> Result<bool> {
        #[cfg(target_arch = "wasm32")]
        {
            Ok(true) // WebNN availability depends on browser support
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        Ok(false)
    }
}

