//! CPU Execution Provider
//! 
//! Fallback provider that runs on CPU. Always available.

use crate::{BackendType, impl_provider_base, ExecutionProvider, ProviderConfig, Result};
use crate::constants::*;

#[derive(Debug, Clone)]
pub struct CPUExecutionProvider {
    config: ProviderConfig,
}

impl_provider_base!(
    CPUExecutionProvider,
    "CPUExecutionProvider",
    BackendType::CPU
);

impl CPUExecutionProvider {
    pub fn with_arena_extend_strategy(mut self, strategy: i32) -> Self {
        self.config.set(ARENA_EXTEND_STRATEGY, strategy);
        self
    }
    
    pub fn with_enable_cpu_mem_arena(mut self, enable: bool) -> Self {
        self.config.set(ENABLE_CPU_MEM_ARENA, enable);
        self
    }
}

impl ExecutionProvider for CPUExecutionProvider {
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
        true // Always supported
    }
    
    fn is_available(&self) -> Result<bool> {
        Ok(true) // Always available
    }
}

