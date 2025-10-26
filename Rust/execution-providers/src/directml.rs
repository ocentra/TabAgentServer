//! DirectML Execution Provider
//! 
//! Microsoft DirectML for GPU acceleration on Windows (supports NVIDIA, AMD, Intel).

use crate::{BackendType, impl_provider_base, ExecutionProvider, ProviderConfig, Result};
use crate::constants::*;

#[derive(Debug, Clone)]
pub struct DirectMLExecutionProvider {
    config: ProviderConfig,
}

impl_provider_base!(
    DirectMLExecutionProvider,
    "DmlExecutionProvider",
    BackendType::DirectML
);

impl DirectMLExecutionProvider {
    /// Set the DirectX device ID (default: 0)
    pub fn with_device_id(mut self, device_id: i32) -> Self {
        self.config.set(DEVICE_ID, device_id);
        self
    }
    
    /// Enable metacommands (optimized DirectML operations)
    pub fn with_enable_metacommands(mut self, enable: bool) -> Self {
        self.config.set(ENABLE_METACOMMANDS, enable);
        self
    }
    
    /// Disable specific metacommands (comma-separated list)
    pub fn with_disable_metacommands(mut self, commands: &str) -> Self {
        self.config.set(DISABLE_METACOMMANDS, commands);
        self
    }
    
    /// Enable dynamic graph fusion
    pub fn with_enable_dynamic_graph_fusion(mut self, enable: bool) -> Self {
        self.config.set(ENABLE_DYNAMIC_GRAPH_FUSION, enable);
        self
    }
    
    /// Set graph fusion filter level (0-9, higher = more aggressive)
    pub fn with_graph_fusion_filter_level(mut self, level: i32) -> Self {
        self.config.set(GRAPH_FUSION_FILTER_LEVEL, level);
        self
    }
    
    /// Enable GPU upload heap for faster data transfer
    pub fn with_enable_gpu_upload_heap(mut self, enable: bool) -> Self {
        self.config.set(ENABLE_GPU_UPLOAD_HEAP, enable);
        self
    }
}

impl ExecutionProvider for DirectMLExecutionProvider {
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
        cfg!(target_os = "windows")
    }
    
    fn is_available(&self) -> Result<bool> {
        // DirectML is Windows-only
        #[cfg(target_os = "windows")]
        {
            use tabagent_hardware::detect_system;
            
            let system = detect_system()
                .map_err(|e| crate::ProviderError::Hardware(e.to_string()))?;
            
            // DirectML works with any GPU (NVIDIA, AMD, Intel)
            Ok(!system.gpus.is_empty())
        }
        
        #[cfg(not(target_os = "windows"))]
        Ok(false)
    }
}

