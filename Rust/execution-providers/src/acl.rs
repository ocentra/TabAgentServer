//! ACL (Arm Compute Library) Execution Provider
//! 
//! ARM platform acceleration using Arm Compute Library.

use crate::{BackendType, impl_provider_base, ExecutionProvider, ProviderConfig, Result};
use crate::constants::*;

#[derive(Debug, Clone)]
pub struct ACLExecutionProvider {
    config: ProviderConfig,
}

impl_provider_base!(
    ACLExecutionProvider,
    "ACLExecutionProvider",
    BackendType::ACL
);

impl ACLExecutionProvider {
    /// Enable/disable ACL's fast math mode
    pub fn with_fast_math(mut self, enable: bool) -> Self {
        self.config.set(FAST_MATH, enable);
        self
    }
}

impl ExecutionProvider for ACLExecutionProvider {
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
        cfg!(target_arch = "aarch64")
    }
    
    fn is_available(&self) -> Result<bool> {
        #[cfg(target_arch = "aarch64")]
        {
            Ok(true) // Available on ARM64
        }
        
        #[cfg(not(target_arch = "aarch64"))]
        Ok(false)
    }
}

