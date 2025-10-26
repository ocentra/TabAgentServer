//! OpenVINO Execution Provider
//! 
//! Intel OpenVINO for optimized inference on Intel CPUs, GPUs, and VPUs.

use crate::{BackendType, impl_provider_base, ExecutionProvider, ProviderConfig, ProviderError, Result};
use crate::constants::*;
use tabagent_hardware::{detect_system, GpuVendor};

#[derive(Debug, Clone)]
pub struct OpenVINOExecutionProvider {
    config: ProviderConfig,
}

impl_provider_base!(
    OpenVINOExecutionProvider,
    "OpenVINOExecutionProvider",
    BackendType::OpenVINO
);

impl OpenVINOExecutionProvider {
    /// Set device type: CPU, GPU, VPU, MYRIAD, HDDL, etc.
    pub fn with_device_type(mut self, device_type: &str) -> Self {
        self.config.set(DEVICE_TYPE, device_type);
        self
    }
    
    /// Set device ID for GPU/VPU (e.g., "GPU.0", "GPU.1")
    pub fn with_device_id(mut self, device_id: &str) -> Self {
        self.config.set(DEVICE_ID, device_id);
        self
    }
    
    /// Enable NNCF (Neural Network Compression Framework) for INT8
    pub fn with_enable_nncf(mut self, enable: bool) -> Self {
        self.config.set(ENABLE_NNCF, enable);
        self
    }
    
    /// Set number of threads for CPU inference
    pub fn with_num_of_threads(mut self, threads: i32) -> Self {
        self.config.set(NUM_OF_THREADS, threads);
        self
    }
    
    /// Set cache directory for compiled models
    pub fn with_cache_dir(mut self, path: &str) -> Self {
        self.config.set(CACHE_DIR, path);
        self
    }
    
    /// Set inference precision: FP32, FP16, INT8
    pub fn with_precision(mut self, precision: &str) -> Self {
        self.config.set(PRECISION, precision);
        self
    }
    
    /// Enable OpenVINO dynamic shapes
    pub fn with_enable_dynamic_shapes(mut self, enable: bool) -> Self {
        self.config.set(ENABLE_DYNAMIC_SHAPES, enable);
        self
    }
    
    /// Set execution mode: SYNC, ASYNC
    pub fn with_execution_mode(mut self, mode: &str) -> Self {
        self.config.set(EXECUTION_MODE, mode);
        self
    }
    
    /// Enable model caching
    pub fn with_enable_model_caching(mut self, enable: bool) -> Self {
        self.config.set(ENABLE_MODEL_CACHING, enable);
        self
    }
    
    /// Set number of streams for throughput mode
    pub fn with_num_streams(mut self, streams: i32) -> Self {
        self.config.set(NUM_STREAMS, streams);
        self
    }
}

impl ExecutionProvider for OpenVINOExecutionProvider {
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
        // OpenVINO supports Windows, Linux, macOS
        cfg!(any(target_os = "windows", target_os = "linux", target_os = "macos"))
    }
    
    fn is_available(&self) -> Result<bool> {
        let system = detect_system()
            .map_err(|e| ProviderError::Hardware(e.to_string()))?;
        
        // OpenVINO works best with Intel hardware but can run on any CPU
        // Check for Intel GPU or any CPU
        let has_intel_gpu = system.gpus.iter().any(|gpu| matches!(gpu.vendor, GpuVendor::Intel));
        let has_cpu = true; // Always have CPU
        
        Ok(has_intel_gpu || has_cpu)
    }
}

