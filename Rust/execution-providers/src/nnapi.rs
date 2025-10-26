//! NNAPI Execution Provider
//! 
//! Android Neural Networks API for hardware acceleration on Android devices.

use crate::{BackendType, impl_provider_base, ExecutionProvider, ProviderConfig, Result};
use crate::constants::*;

#[derive(Debug, Clone)]
pub struct NNAPIExecutionProvider {
    config: ProviderConfig,
}

impl_provider_base!(
    NNAPIExecutionProvider,
    "NnapiExecutionProvider",
    BackendType::NNAPI
);

impl NNAPIExecutionProvider {
    /// Use FP16 relaxation (may improve performance, reduce accuracy)
    pub fn with_fp16(mut self, enable: bool) -> Self {
        self.config.set(USE_FP16, enable);
        self
    }
    
    /// Use NCHW layout (Android API 29+)
    pub fn with_nchw(mut self, enable: bool) -> Self {
        self.config.set(USE_NCHW, enable);
        self
    }
    
    /// Disable CPU fallback (Android API 29+)
    pub fn with_disable_cpu(mut self, enable: bool) -> Self {
        self.config.set(DISABLE_CPU, enable);
        self
    }
    
    /// Use CPU only (Android API 29+)
    pub fn with_cpu_only(mut self, enable: bool) -> Self {
        self.config.set(CPU_ONLY, enable);
        self
    }
}

impl ExecutionProvider for NNAPIExecutionProvider {
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
        cfg!(target_os = "android")
    }
    
    fn is_available(&self) -> Result<bool> {
        #[cfg(target_os = "android")]
        {
            Ok(true) // NNAPI is built into Android
        }
        
        #[cfg(not(target_os = "android"))]
        Ok(false)
    }
}

