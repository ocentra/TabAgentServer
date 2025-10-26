//! QNN Execution Provider
//! 
//! Qualcomm QNN (Qualcomm Neural Network SDK) for Qualcomm Snapdragon SoCs.

use crate::{BackendType, impl_provider_base, ExecutionProvider, ProviderConfig, Result};
use crate::constants::*;

#[derive(Debug, Clone)]
pub struct QNNExecutionProvider {
    config: ProviderConfig,
}

impl_provider_base!(
    QNNExecutionProvider,
    "QNNExecutionProvider",
    BackendType::QNN
);

impl QNNExecutionProvider {
    /// Set QNN backend library path (e.g., libQnnCpu.so or libQnnHtp.so)
    pub fn with_backend_path(mut self, path: &str) -> Self {
        self.config.set(BACKEND_PATH, path);
        self
    }
    
    /// Set profiling level (off, basic, detailed)
    pub fn with_profiling_level(mut self, level: &str) -> Self {
        self.config.set(PROFILING_LEVEL, level);
        self
    }
    
    /// Set RPC control latency in microseconds
    pub fn with_rpc_control_latency(mut self, latency: u32) -> Self {
        self.config.set(RPC_CONTROL_LATENCY, latency);
        self
    }
    
    /// Set VTCM size in MB
    pub fn with_vtcm_mb(mut self, mb: usize) -> Self {
        self.config.set(VTCM_MB, mb);
        self
    }
    
    /// Set HTP performance mode
    pub fn with_performance_mode(mut self, mode: &str) -> Self {
        self.config.set(HTP_PERFORMANCE_MODE, mode);
        self
    }
    
    /// Set device ID
    pub fn with_device_id(mut self, device: i32) -> Self {
        self.config.set(DEVICE_ID, device);
        self
    }
    
    /// Enable HTP FP16 precision
    pub fn with_htp_fp16_precision(mut self, enable: bool) -> Self {
        self.config.set(ENABLE_HTP_FP16_PRECISION, enable);
        self
    }
}

impl ExecutionProvider for QNNExecutionProvider {
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
        cfg!(all(
            target_arch = "aarch64",
            any(target_os = "windows", target_os = "linux", target_os = "android")
        ))
    }
    
    fn is_available(&self) -> Result<bool> {
        // QNN requires Qualcomm hardware
        Ok(self.supported_by_platform())
    }
}

