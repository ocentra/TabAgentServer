//! ArmNN Execution Provider
//! 
//! ARM platform acceleration using Arm NN.

use crate::{BackendType, impl_provider_base, ExecutionProvider, ProviderConfig, Result};
use crate::constants::*;

#[derive(Debug, Clone)]
pub struct ArmNNExecutionProvider {
    config: ProviderConfig,
}

impl_provider_base!(
    ArmNNExecutionProvider,
    "ArmNNExecutionProvider",
    BackendType::ArmNN
);

impl ArmNNExecutionProvider {
    /// Enable/disable arena allocator
    pub fn with_arena_allocator(mut self, enable: bool) -> Self {
        self.config.set(USE_ARENA, enable);
        self
    }
}

impl ExecutionProvider for ArmNNExecutionProvider {
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
        cfg!(all(target_arch = "aarch64", any(target_os = "linux", target_os = "android")))
    }
    
    fn is_available(&self) -> Result<bool> {
        #[cfg(all(target_arch = "aarch64", any(target_os = "linux", target_os = "android")))]
        {
            Ok(true)
        }
        
        #[cfg(not(all(target_arch = "aarch64", any(target_os = "linux", target_os = "android"))))]
        Ok(false)
    }
}

