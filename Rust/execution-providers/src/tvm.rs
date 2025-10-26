//! TVM Execution Provider
//! 
//! Apache TVM for cross-platform ML acceleration.

use crate::{BackendType, impl_provider_base, ExecutionProvider, ProviderConfig, Result};
use crate::constants::*;

#[derive(Debug, Clone)]
pub struct TVMExecutionProvider {
    config: ProviderConfig,
}

impl_provider_base!(
    TVMExecutionProvider,
    "TvmExecutionProvider",
    BackendType::TVM
);

impl TVMExecutionProvider {
    /// Set executor type (graph or vm)
    pub fn with_executor(mut self, executor: &str) -> Self {
        self.config.set(EXECUTOR, executor);
        self
    }
    
    /// Set path to compiled model folder (.so/.dll files)
    pub fn with_so_folder(mut self, path: &str) -> Self {
        self.config.set(SO_FOLDER, path);
        self
    }
    
    /// Enable hash checking for model
    pub fn with_check_hash(mut self, enable: bool) -> Self {
        self.config.set(CHECK_HASH, enable);
        self
    }
    
    /// Set target device (e.g., "llvm", "cuda", "opencl")
    pub fn with_target(mut self, target: &str) -> Self {
        self.config.set(TARGET, target);
        self
    }
    
    /// Set target host
    pub fn with_target_host(mut self, target_host: &str) -> Self {
        self.config.set(TARGET_HOST, target_host);
        self
    }
    
    /// Set optimization level (0-3)
    pub fn with_opt_level(mut self, level: usize) -> Self {
        self.config.set(OPT_LEVEL, level);
        self
    }
    
    /// Freeze weights (keep on compilation stage)
    pub fn with_freeze_weights(mut self, enable: bool) -> Self {
        self.config.set(FREEZE_WEIGHTS, enable);
        self
    }
    
    /// Set tuning file path (AutoTVM or Ansor)
    pub fn with_tuning_file_path(mut self, path: &str) -> Self {
        self.config.set(TUNING_FILE_PATH, path);
        self
    }
}

impl ExecutionProvider for TVMExecutionProvider {
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
        true // TVM supports many platforms
    }
    
    fn is_available(&self) -> Result<bool> {
        Ok(true) // TVM availability depends on runtime installation
    }
}

