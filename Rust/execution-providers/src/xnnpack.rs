//! XNNPACK Execution Provider
//! 
//! Google XNNPACK for ARM, x86, and WASM platforms.

use crate::{BackendType, impl_provider_base, ExecutionProvider, ProviderConfig, Result};
use crate::constants::*;

#[derive(Debug, Clone)]
pub struct XNNPACKExecutionProvider {
    config: ProviderConfig,
}

impl_provider_base!(
    XNNPACKExecutionProvider,
    "XnnpackExecutionProvider",
    BackendType::XNNPACK
);

impl XNNPACKExecutionProvider {
    /// Set number of threads for XNNPACK's internal threadpool
    pub fn with_intra_op_num_threads(mut self, num_threads: usize) -> Self {
        self.config.set(INTRA_OP_NUM_THREADS, num_threads);
        self
    }
}

impl ExecutionProvider for XNNPACKExecutionProvider {
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
            target_arch = "aarch64",
            all(target_arch = "arm", any(target_os = "linux", target_os = "android")),
            target_arch = "x86_64",
            target_arch = "wasm32"
        ))
    }
    
    fn is_available(&self) -> Result<bool> {
        Ok(self.supported_by_platform())
    }
}

