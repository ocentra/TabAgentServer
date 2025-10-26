//! CUDA Execution Provider
//! 
//! NVIDIA GPU acceleration via CUDA. Mirrors ort's CUDAExecutionProvider
//! with all 25+ configuration options.

use crate::{BackendType, impl_provider_base, ExecutionProvider, ProviderConfig, ProviderError, Result};
use crate::constants::*;
use tabagent_hardware::{detect_system, GpuVendor};

#[derive(Debug, Clone)]
pub struct CUDAExecutionProvider {
    config: ProviderConfig,
}

impl_provider_base!(
    CUDAExecutionProvider,
    "CUDAExecutionProvider",
    BackendType::Cuda
);

impl CUDAExecutionProvider {
    /// Set the CUDA device ID (default: 0)
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
    /// - 0: kNextPowerOfTwo (default)
    /// - 1: kSameAsRequested
    pub fn with_arena_extend_strategy(mut self, strategy: i32) -> Self {
        self.config.set(ARENA_EXTEND_STRATEGY, strategy);
        self
    }
    
    /// Set cuDNN convolution algorithm search type
    /// - "EXHAUSTIVE": Exhaustive search (slowest, best performance)
    /// - "HEURISTIC": Heuristic search (default)
    /// - "DEFAULT": Use default algorithm
    pub fn with_cudnn_conv_algo_search(mut self, search_type: &str) -> Self {
        self.config.set(CUDNN_CONV_ALGO_SEARCH, search_type);
        self
    }
    
    /// Enable copying in default stream
    pub fn with_do_copy_in_default_stream(mut self, enable: bool) -> Self {
        self.config.set(DO_COPY_IN_DEFAULT_STREAM, enable);
        self
    }
    
    /// Enable using max workspace size for cuDNN convolutions
    pub fn with_cudnn_conv_use_max_workspace(mut self, enable: bool) -> Self {
        self.config.set(CUDNN_CONV_USE_MAX_WORKSPACE, enable);
        self
    }
    
    /// Enable padding Conv1D to NC1D format for cuDNN
    pub fn with_cudnn_conv1d_pad_to_nc1d(mut self, enable: bool) -> Self {
        self.config.set(CUDNN_CONV1D_PAD_TO_NC1D, enable);
        self
    }
    
    /// Enable CUDA graph optimization
    pub fn with_enable_cuda_graph(mut self, enable: bool) -> Self {
        self.config.set(ENABLE_CUDA_GRAPH, enable);
        self
    }
    
    /// Set specific cuDNN convolution algorithm (use with caution)
    pub fn with_cudnn_conv_algorithm(mut self, algo: i32) -> Self {
        self.config.set(CUDNN_CONV_ALGORITHM, algo);
        self
    }
    
    /// Enable EP-level unified stream
    pub fn with_use_ep_level_unified_stream(mut self, enable: bool) -> Self {
        self.config.set(USE_EP_LEVEL_UNIFIED_STREAM, enable);
        self
    }
    
    /// Enable TensorFloat-32 (TF32) mode for matrix multiplications
    /// TF32 is faster but slightly less precise than FP32
    pub fn with_use_tf32(mut self, enable: bool) -> Self {
        self.config.set(USE_TF32, enable);
        self
    }
    
    /// Set preferred cuDNN algorithm for RNN operations
    pub fn with_cudnn_rnn_mode(mut self, mode: i32) -> Self {
        self.config.set(CUDNN_RNN_MODE, mode);
        self
    }
    
    /// Enable cuDNN batch normalization spatial persistent mode
    pub fn with_cudnn_bn_spatial_persistent(mut self, enable: bool) -> Self {
        self.config.set(CUDNN_BN_SPATIAL_PERSISTENT, enable);
        self
    }
    
    /// Enable memory pattern optimization
    pub fn with_enable_mem_pattern(mut self, enable: bool) -> Self {
        self.config.set(ENABLE_MEM_PATTERN, enable);
        self
    }
    
    /// Set tunable operation enable/disable
    pub fn with_tunable_op_enable(mut self, enable: bool) -> Self {
        self.config.set(TUNABLE_OP_ENABLE, enable);
        self
    }
    
    /// Set tunable operation tuning enable/disable
    pub fn with_tunable_op_tuning_enable(mut self, enable: bool) -> Self {
        self.config.set(TUNABLE_OP_TUNING_ENABLE, enable);
        self
    }
    
    /// Set tunable operation max tuning duration in milliseconds
    pub fn with_tunable_op_max_tuning_duration_ms(mut self, duration_ms: i32) -> Self {
        self.config.set(TUNABLE_OP_MAX_TUNING_DURATION_MS, duration_ms);
        self
    }
    
    /// Enable skip-layer normalization in attention
    pub fn with_enable_skip_layer_norm_strict_mode(mut self, enable: bool) -> Self {
        self.config.set(ENABLE_SKIP_LAYER_NORM_STRICT_MODE, enable);
        self
    }
    
    /// Prefer NHWC (channels-last) format for convolutions
    pub fn with_prefer_nhwc(mut self, enable: bool) -> Self {
        self.config.set(PREFER_NHWC, enable);
        self
    }
    
    /// Enable cuDNN frontend APIs
    pub fn with_use_cudnn_frontend(mut self, enable: bool) -> Self {
        self.config.set(USE_CUDNN_FRONTEND, enable);
        self
    }
    
    /// Enable fusing batch normalization and activation
    pub fn with_fuse_bn_relu(mut self, enable: bool) -> Self {
        self.config.set(FUSE_BN_RELU, enable);
        self
    }
    
    /// Enable fusing batch normalization, addition, and activation
    pub fn with_fuse_bn_add_relu(mut self, enable: bool) -> Self {
        self.config.set(FUSE_BN_ADD_RELU, enable);
        self
    }
    
    /// Set user compute stream for CUDA operations
    pub fn with_user_compute_stream(mut self, stream_ptr: usize) -> Self {
        self.config.set(USER_COMPUTE_STREAM, stream_ptr);
        self
    }
    
    /// Set default memory arena configuration
    pub fn with_default_memory_arena_cfg(mut self, cfg: &str) -> Self {
        self.config.set(DEFAULT_MEMORY_ARENA_CFG, cfg);
        self
    }
    
    /// Enable CUDA memory pool
    pub fn with_enable_cuda_mem_pool(mut self, enable: bool) -> Self {
        self.config.set(ENABLE_CUDA_MEM_POOL, enable);
        self
    }
}

impl ExecutionProvider for CUDAExecutionProvider {
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
        
        Ok(system.gpus.iter().any(|gpu| matches!(gpu.vendor, GpuVendor::Nvidia)))
    }
}

