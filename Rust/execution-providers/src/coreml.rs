//! CoreML Execution Provider
//! 
//! Apple CoreML for optimized inference on macOS and iOS devices.

use crate::{BackendType, impl_provider_base, ExecutionProvider, ProviderConfig, Result};
use crate::constants::*;

#[derive(Debug, Clone)]
pub struct CoreMLExecutionProvider {
    config: ProviderConfig,
}

impl_provider_base!(
    CoreMLExecutionProvider,
    "CoreMLExecutionProvider",
    BackendType::CoreML
);

impl CoreMLExecutionProvider {
    /// Set CoreML model format: MLProgram or NeuralNetwork
    /// - "MLProgram": Modern format with better performance (iOS 15+, macOS 12+)
    /// - "NeuralNetwork": Legacy format for older devices
    pub fn with_model_format(mut self, format: &str) -> Self {
        self.config.set(ML_MODEL_FORMAT, format);
        self
    }
    
    /// Set compute units preference
    /// - "ALL": Use all available compute units (CPU, GPU, Neural Engine)
    /// - "CPU_ONLY": Use only CPU
    /// - "CPU_AND_GPU": Use CPU and GPU, avoid Neural Engine
    /// - "CPU_AND_NE": Use CPU and Neural Engine, avoid GPU
    pub fn with_compute_units(mut self, units: &str) -> Self {
        self.config.set(ML_COMPUTE_UNITS, units);
        self
    }
    
    /// Enable on subgraphs (default: false)
    /// If true, CoreML will be used for subgraphs that can be converted
    pub fn with_enable_on_subgraph(mut self, enable: bool) -> Self {
        self.config.set(ENABLE_ON_SUBGRAPH, enable);
        self
    }
    
    /// Only enable CoreML on specific node names (comma-separated)
    pub fn with_only_enable_device_with_ane(mut self, enable: bool) -> Self {
        self.config.set(ONLY_ENABLE_DEVICE_WITH_ANE, enable);
        self
    }
    
    /// Set minimum CoreML deployment target (e.g., "13", "14", "15")
    pub fn with_minimum_deployment_target(mut self, target: &str) -> Self {
        self.config.set(MINIMUM_DEPLOYMENT_TARGET, target);
        self
    }
    
    /// Create MLProgram in memory instead of on disk (default: false)
    pub fn with_create_mlprogram_in_memory(mut self, enable: bool) -> Self {
        self.config.set(CREATE_ML_PROGRAM_IN_MEMORY, enable);
        self
    }
    
    /// Set maximum wait time in seconds for loading model (default: 0 = no limit)
    pub fn with_max_wait_time_seconds(mut self, seconds: i32) -> Self {
        self.config.set(MAX_WAIT_TIME_SECONDS, seconds);
        self
    }
    
    /// Enable model input/output name capture for debugging
    pub fn with_enable_model_io_name_capture(mut self, enable: bool) -> Self {
        self.config.set(ENABLE_MODEL_IO_NAME_CAPTURE, enable);
        self
    }
    
    /// Set strategy for handling coreml:GetShape operations
    pub fn with_get_shape_strategy(mut self, strategy: &str) -> Self {
        self.config.set(GET_SHAPE_STRATEGY, strategy);
        self
    }
}

impl ExecutionProvider for CoreMLExecutionProvider {
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
        cfg!(target_os = "macos")
    }
    
    fn is_available(&self) -> Result<bool> {
        #[cfg(target_os = "macos")]
        {
            // CoreML is available on all macOS devices
            // For better performance, check for Apple Silicon (M1/M2/M3)
            use tabagent_hardware::{detect_system, CpuArchitecture};
            
            let system = detect_system()
                .map_err(|e| crate::ProviderError::Hardware(e.to_string()))?;
            
            // CoreML works on Intel and Apple Silicon, but excels on Apple Silicon
            let is_apple_silicon = matches!(
                system.cpu.architecture,
                CpuArchitecture::AppleM1 | CpuArchitecture::AppleM2 | CpuArchitecture::AppleM3
            );
            
            Ok(is_apple_silicon || true) // Available on all macOS, but log if not Apple Silicon
        }
        
        #[cfg(not(target_os = "macos"))]
        Ok(false)
    }
}

