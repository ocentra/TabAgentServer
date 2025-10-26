//! CANN Execution Provider
//! 
//! Huawei CANN (Compute Architecture for Neural Networks) for Ascend AI processors.

use crate::{BackendType, impl_provider_base, ExecutionProvider, ProviderConfig, Result};
use crate::constants::*;

#[derive(Debug, Clone)]
pub struct CANNExecutionProvider {
    config: ProviderConfig,
}

impl_provider_base!(
    CANNExecutionProvider,
    "CANNExecutionProvider",
    BackendType::CANN
);

impl CANNExecutionProvider {
    /// Set device ID
    pub fn with_device_id(mut self, device_id: i32) -> Self {
        self.config.set(DEVICE_ID, device_id);
        self
    }
    
    /// Set NPU memory limit in bytes
    pub fn with_memory_limit(mut self, limit: usize) -> Self {
        self.config.set(NPU_MEM_LIMIT, limit);
        self
    }
    
    /// Set arena extend strategy
    pub fn with_arena_extend_strategy(mut self, strategy: &str) -> Self {
        self.config.set(ARENA_EXTEND_STRATEGY, strategy);
        self
    }
    
    /// Enable CANN graph inference engine
    pub fn with_cann_graph(mut self, enable: bool) -> Self {
        self.config.set(ENABLE_CANN_GRAPH, enable);
        self
    }
    
    /// Enable subgraph dumping for analysis
    pub fn with_dump_graphs(mut self, enable: bool) -> Self {
        self.config.set(DUMP_GRAPHS, enable);
        self
    }
    
    /// Set precision mode (force_fp32, force_fp16, allow_fp32_to_fp16, etc.)
    pub fn with_precision_mode(mut self, mode: &str) -> Self {
        self.config.set(PRECISION_MODE, mode);
        self
    }
    
    /// Set operator implementation mode (high_precision, high_performance)
    pub fn with_implementation_mode(mut self, mode: &str) -> Self {
        self.config.set(OP_SELECT_IMPL_MODE, mode);
        self
    }
}

impl ExecutionProvider for CANNExecutionProvider {
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
        cfg!(all(target_os = "linux", any(target_arch = "aarch64", target_arch = "x86_64")))
    }
    
    fn is_available(&self) -> Result<bool> {
        // CANN requires Huawei Ascend hardware
        // Runtime check would need to probe for CANN libraries
        Ok(self.supported_by_platform())
    }
}

