//! RKNPU Execution Provider
//! 
//! Rockchip NPU for hardware acceleration on Rockchip SoCs.

use crate::{BackendType, impl_provider_base, ExecutionProvider, ProviderConfig, Result};

#[derive(Debug, Clone)]
pub struct RKNPUExecutionProvider {
    config: ProviderConfig,
}

impl_provider_base!(
    RKNPUExecutionProvider,
    "RknpuExecutionProvider",
    BackendType::RKNPU
);

impl RKNPUExecutionProvider {}

impl ExecutionProvider for RKNPUExecutionProvider {
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
        cfg!(all(target_arch = "aarch64", target_os = "linux"))
    }
    
    fn is_available(&self) -> Result<bool> {
        // RKNPU requires Rockchip hardware
        Ok(self.supported_by_platform())
    }
}

