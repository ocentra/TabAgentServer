//! OneDNN Execution Provider
//! 
//! Intel OneDNN (formerly DNNL) for Intel CPUs and iGPUs.

use crate::{BackendType, impl_provider_base, ExecutionProvider, ProviderConfig, Result};
use crate::constants::*;

#[derive(Debug, Clone)]
pub struct OneDNNExecutionProvider {
    config: ProviderConfig,
}

impl_provider_base!(
    OneDNNExecutionProvider,
    "DnnlExecutionProvider",
    BackendType::OneDNN
);

impl OneDNNExecutionProvider {
    /// Enable/disable arena allocator
    pub fn with_arena_allocator(mut self, enable: bool) -> Self {
        self.config.set(USE_ARENA, enable);
        self
    }
}

impl ExecutionProvider for OneDNNExecutionProvider {
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
        cfg!(all(target_arch = "x86_64", any(target_os = "windows", target_os = "linux")))
    }
    
    fn is_available(&self) -> Result<bool> {
        // OneDNN works on x86_64 CPUs
        Ok(self.supported_by_platform())
    }
}

