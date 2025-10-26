//! Vitis AI Execution Provider
//! 
//! Xilinx Vitis AI for FPGA acceleration.

use crate::{BackendType, impl_provider_base, ExecutionProvider, ProviderConfig, Result};
use crate::constants::*;

#[derive(Debug, Clone)]
pub struct VitisAIExecutionProvider {
    config: ProviderConfig,
}

impl_provider_base!(
    VitisAIExecutionProvider,
    "VitisAIExecutionProvider",
    BackendType::Vitis
);

impl VitisAIExecutionProvider {
    /// Set config file path
    pub fn with_config_file(mut self, path: &str) -> Self {
        self.config.set(CONFIG_FILE, path);
        self
    }
    
    /// Set cache directory
    pub fn with_cache_dir(mut self, path: &str) -> Self {
        self.config.set(CACHE_DIR, path);
        self
    }
    
    /// Set cache key
    pub fn with_cache_key(mut self, key: &str) -> Self {
        self.config.set(CACHE_KEY, key);
        self
    }
}

impl ExecutionProvider for VitisAIExecutionProvider {
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
        // Vitis AI requires Xilinx FPGA hardware
        Ok(self.supported_by_platform())
    }
}

