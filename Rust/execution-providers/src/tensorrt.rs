//! TensorRT Execution Provider
//!
//! NVIDIA TensorRT for optimized inference on NVIDIA GPUs.
//! TensorRT provides layer fusion, precision calibration, and kernel auto-tuning.

use crate::{BackendType, impl_provider_base, ExecutionProvider, ProviderConfig, ProviderError, Result};
use crate::constants::*;
use tabagent_hardware::{detect_system, GpuVendor};

#[derive(Debug, Clone)]
pub struct TensorRTExecutionProvider {
    config: ProviderConfig,
}

impl_provider_base!(
    TensorRTExecutionProvider,
    "TensorrtExecutionProvider",
    BackendType::TensorRT
);

impl TensorRTExecutionProvider {
    /// Set the CUDA device ID (default: 0)
    pub fn with_device_id(mut self, device_id: i32) -> Self {
        self.config.set(DEVICE_ID, device_id);
        self
    }
    
    /// Set maximum workspace size in bytes for TensorRT
    pub fn with_max_workspace_size(mut self, size: usize) -> Self {
        self.config.set(TRT_MAX_WORKSPACE_SIZE, size);
        self
    }
    
    /// Set maximum batch size for TensorRT engine
    pub fn with_max_batch_size(mut self, size: i32) -> Self {
        self.config.set(TRT_MAX_BATCH_SIZE, size);
        self
    }
    
    /// Set minimum subgraph size to offload to TensorRT
    pub fn with_min_subgraph_size(mut self, size: i32) -> Self {
        self.config.set(TRT_MIN_SUBGRAPH_SIZE, size);
        self
    }
    
    /// Enable FP16 precision mode
    pub fn with_fp16_enable(mut self, enable: bool) -> Self {
        self.config.set(TRT_FP16_ENABLE, enable);
        self
    }
    
    /// Enable INT8 precision mode
    pub fn with_int8_enable(mut self, enable: bool) -> Self {
        self.config.set(TRT_INT8_ENABLE, enable);
        self
    }
    
    /// Set path to INT8 calibration table
    pub fn with_int8_calibration_table_name(mut self, path: &str) -> Self {
        self.config.set(TRT_INT8_CALIBRATION_TABLE_NAME, path);
        self
    }
    
    /// Use DLA (Deep Learning Accelerator) core
    pub fn with_dla_enable(mut self, enable: bool) -> Self {
        self.config.set(TRT_DLA_ENABLE, enable);
        self
    }
    
    /// Set DLA core ID (0 or 1 on Jetson)
    pub fn with_dla_core(mut self, core_id: i32) -> Self {
        self.config.set(TRT_DLA_CORE, core_id);
        self
    }
    
    /// Set directory to save/load TensorRT engine cache
    pub fn with_engine_cache_enable(mut self, enable: bool) -> Self {
        self.config.set(TRT_ENGINE_CACHE_ENABLE, enable);
        self
    }
    
    /// Set path for engine cache
    pub fn with_engine_cache_path(mut self, path: &str) -> Self {
        self.config.set(TRT_ENGINE_CACHE_PATH, path);
        self
    }
    
    /// Enable verbose TensorRT logging
    pub fn with_dump_subgraphs(mut self, enable: bool) -> Self {
        self.config.set(TRT_DUMP_SUBGRAPHS, enable);
        self
    }
    
    /// Force sequential engine execution
    pub fn with_force_sequential_engine_build(mut self, enable: bool) -> Self {
        self.config.set(TRT_FORCE_SEQUENTIAL_ENGINE_BUILD, enable);
        self
    }
    
    /// Enable context memory sharing
    pub fn with_context_memory_sharing_enable(mut self, enable: bool) -> Self {
        self.config.set(TRT_CONTEXT_MEMORY_SHARING_ENABLE, enable);
        self
    }
    
    /// Set layer normalization precision
    pub fn with_layer_norm_fp32_fallback(mut self, enable: bool) -> Self {
        self.config.set(TRT_LAYER_NORM_FP32_FALLBACK, enable);
        self
    }
    
    /// Set timing cache path
    pub fn with_timing_cache_enable(mut self, enable: bool) -> Self {
        self.config.set(TRT_TIMING_CACHE_ENABLE, enable);
        self
    }
    
    /// Set detailed build log
    pub fn with_detailed_build_log(mut self, enable: bool) -> Self {
        self.config.set(TRT_DETAILED_BUILD_LOG, enable);
        self
    }
    
    /// Enable builder optimization level (0-5, default: 3)
    pub fn with_builder_optimization_level(mut self, level: i32) -> Self {
        self.config.set(TRT_BUILDER_OPTIMIZATION_LEVEL, level);
        self
    }
    
    /// Enable auxiliary streams
    pub fn with_auxiliary_streams(mut self, num_streams: i32) -> Self {
        self.config.set(TRT_AUXILIARY_STREAMS, num_streams);
        self
    }
    
    /// Set tactic sources (comma-separated: CUBLAS,CUDNN,EDGE_MASK_CONVOLUTIONS)
    pub fn with_tactic_sources(mut self, sources: &str) -> Self {
        self.config.set(TRT_TACTIC_SOURCES, sources);
        self
    }
    
    /// Enable CUDA graph optimization
    pub fn with_cuda_graph_enable(mut self, enable: bool) -> Self {
        self.config.set(TRT_CUDA_GRAPH_ENABLE, enable);
        self
    }
    
    /// Set DLA SRAM size
    pub fn with_dla_sram_size(mut self, size: usize) -> Self {
        self.config.set(TRT_DLA_SRAM_SIZE, size);
        self
    }
    
    /// Set DLA local DRAM size
    pub fn with_dla_local_dram_size(mut self, size: usize) -> Self {
        self.config.set(TRT_DLA_LOCAL_DRAM_SIZE, size);
        self
    }
    
    /// Set DLA global DRAM size
    pub fn with_dla_global_dram_size(mut self, size: usize) -> Self {
        self.config.set(TRT_DLA_GLOBAL_DRAM_SIZE, size);
        self
    }
}

impl ExecutionProvider for TensorRTExecutionProvider {
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
        cfg!(any(target_os = "windows", target_os = "linux"))
    }
    
    fn is_available(&self) -> Result<bool> {
        let system = detect_system()
            .map_err(|e| ProviderError::Hardware(e.to_string()))?;
        
        // TensorRT requires NVIDIA GPU
        Ok(system.gpus.iter().any(|gpu| matches!(gpu.vendor, GpuVendor::Nvidia)))
    }
}

