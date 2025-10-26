//! ROCm Execution Provider
//! 
//! AMD ROCm for GPU acceleration on AMD GPUs (Linux only).

use crate::{BackendType, impl_provider_base, ExecutionProvider, ProviderConfig, Result};
use crate::constants::*;

#[cfg(target_os = "linux")]
use crate::ProviderError;

#[cfg(target_os = "linux")]
use tabagent_hardware::{detect_system, GpuVendor};

#[derive(Debug, Clone)]
pub struct ROCmExecutionProvider {
    config: ProviderConfig,
}

impl_provider_base!(
    ROCmExecutionProvider,
    "ROCmExecutionProvider",
    BackendType::ROCm
);

impl ROCmExecutionProvider {
    /// Set the ROCm device ID (default: 0)
    pub fn with_device_id(mut self, device_id: i32) -> Self {
        self.config.set(DEVICE_ID, device_id);
        self
    }
    
    /// Set GPU memory limit in bytes
    pub fn with_memory_limit(mut self, limit: usize) -> Self {
        self.config.set(GPU_MEM_LIMIT, limit);
        self
    }
    
    /// Set arena extend strategy
    pub fn with_arena_extend_strategy(mut self, strategy: i32) -> Self {
        self.config.set(ARENA_EXTEND_STRATEGY, strategy);
        self
    }
    
    /// Set MIOpen convolution algorithm search type
    /// - "EXHAUSTIVE": Exhaustive search
    /// - "HEURISTIC": Heuristic search (default)
    /// - "DEFAULT": Use default algorithm
    pub fn with_miopen_conv_algo_search(mut self, search_type: &str) -> Self {
        self.config.set(MIOPEN_CONV_ALGO_SEARCH, search_type);
        self
    }
    
    /// Enable copying in default stream
    pub fn with_do_copy_in_default_stream(mut self, enable: bool) -> Self {
        self.config.set(DO_COPY_IN_DEFAULT_STREAM, enable);
        self
    }
    
    /// Enable using max workspace size for MIOpen convolutions
    pub fn with_miopen_conv_use_max_workspace(mut self, enable: bool) -> Self {
        self.config.set(MIOPEN_CONV_USE_MAX_WORKSPACE, enable);
        self
    }
    
    /// Set user compute stream for ROCm operations
    pub fn with_user_compute_stream(mut self, stream_ptr: usize) -> Self {
        self.config.set(USER_COMPUTE_STREAM, stream_ptr);
        self
    }
    
    /// Enable tunable operations
    pub fn with_tunable_op_enable(mut self, enable: bool) -> Self {
        self.config.set(TUNABLE_OP_ENABLE, enable);
        self
    }
    
    /// Enable tunable operation tuning
    pub fn with_tunable_op_tuning_enable(mut self, enable: bool) -> Self {
        self.config.set(TUNABLE_OP_TUNING_ENABLE, enable);
        self
    }
    
    /// Set tunable operation max tuning duration in milliseconds
    pub fn with_tunable_op_max_tuning_duration_ms(mut self, duration_ms: i32) -> Self {
        self.config.set(TUNABLE_OP_MAX_TUNING_DURATION_MS, duration_ms);
        self
    }
}

impl ExecutionProvider for ROCmExecutionProvider {
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
        // ROCm is primarily Linux, with experimental Windows support
        cfg!(target_os = "linux")
    }
    
    fn is_available(&self) -> Result<bool> {
        #[cfg(target_os = "linux")]
        {
            let system = detect_system()
                .map_err(|e| ProviderError::Hardware(e.to_string()))?;
            
            Ok(system.gpus.iter().any(|gpu| matches!(gpu.vendor, GpuVendor::Amd)))
        }
        
        #[cfg(not(target_os = "linux"))]
        Ok(false)
    }
}

