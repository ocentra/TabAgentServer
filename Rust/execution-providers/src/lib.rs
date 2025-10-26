//! Tabagent Execution Providers
//! 
//! Universal, format-agnostic execution provider system for hardware acceleration.
//! Works with ONNX Runtime, llama.cpp, and future inference engines.
//! 
//! # Examples
//! 
//! ```rust
//! use tabagent_execution_providers::*;
//! 
//! // Create a CUDA provider with configuration
//! let cuda = CUDAExecutionProvider::new()
//!     .with_device_id(0)
//!     .with_memory_limit(2_000_000_000)
//!     .with_use_tf32(true)
//!     .build();
//! 
//! // Check if available
//! if cuda.is_available().unwrap_or(false) {
//!     println!("CUDA is available!");
//! }
//! 
//! // Create a dispatch with multiple providers
//! let providers = vec![
//!     cuda,
//!     CPUExecutionProvider::new().build(), // Fallback
//! ];
//! 
//! let dispatch = ExecutionProviderDispatch::new(providers);
//! ```

use std::fmt::Debug;
use std::sync::Arc;
use std::collections::HashMap;

pub mod error;
pub mod constants;

// All execution providers (mirroring ort's structure)
pub mod cpu;
pub mod cuda;
pub mod tensorrt;
pub mod directml;
pub mod openvino;
pub mod rocm;
pub mod coreml;
pub mod acl;
pub mod armnn;
pub mod azure;
pub mod cann;
pub mod migraphx;
pub mod nnapi;
pub mod nv;
pub mod onednn;
pub mod qnn;
pub mod rknpu;
pub mod tvm;
pub mod vitis;
pub mod wasm;
pub mod webgpu;
pub mod webnn;
pub mod xnnpack;

pub use error::{ProviderError, Result};
pub use cpu::CPUExecutionProvider;
pub use cuda::CUDAExecutionProvider;
pub use tensorrt::TensorRTExecutionProvider;
pub use directml::DirectMLExecutionProvider;
pub use openvino::OpenVINOExecutionProvider;
pub use rocm::ROCmExecutionProvider;
pub use coreml::CoreMLExecutionProvider;
pub use acl::ACLExecutionProvider;
pub use armnn::ArmNNExecutionProvider;
pub use azure::AzureExecutionProvider;
pub use cann::CANNExecutionProvider;
pub use migraphx::MIGraphXExecutionProvider;
pub use nnapi::NNAPIExecutionProvider;
pub use nv::NVExecutionProvider;
pub use onednn::OneDNNExecutionProvider;
pub use qnn::QNNExecutionProvider;
pub use rknpu::RKNPUExecutionProvider;
pub use tvm::TVMExecutionProvider;
pub use vitis::VitisAIExecutionProvider;
pub use wasm::WASMExecutionProvider;
pub use webgpu::WebGPUExecutionProvider;
pub use webnn::WebNNExecutionProvider;
pub use xnnpack::XNNPACKExecutionProvider;

/// Backend type enum - categorizes different hardware acceleration backends
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BackendType {
    // NVIDIA
    Cuda,
    TensorRT,
    NVExecutionProvider,
    
    // AMD
    ROCm,
    MIGraphX,
    
    // Intel
    OpenVINO,
    OneDNN,
    
    // Apple
    CoreML,
    
    // Microsoft
    DirectML,
    Azure,
    
    // Qualcomm
    QNN,
    SNPE,
    
    // Huawei
    CANN,
    
    // ARM
    ArmNN,
    ACL,
    
    // Mobile/Embedded
    NNAPI,
    XNNPACK,
    RKNPU,
    
    // Web
    WebGPU,
    WebNN,
    WASM,
    
    // Other
    TVM,
    Vitis,
    
    // Fallback
    CPU,
}

/// Generic configuration store for provider options
#[derive(Debug, Clone, Default)]
pub struct ProviderConfig {
    options: HashMap<String, String>,
}

impl ProviderConfig {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn set(&mut self, key: impl Into<String>, value: impl ToString) {
        self.options.insert(key.into(), value.to_string());
    }
    
    pub fn get(&self, key: &str) -> Option<&str> {
        self.options.get(key).map(String::as_str)
    }
    
    pub fn get_as<T: std::str::FromStr>(&self, key: &str) -> Option<T> {
        self.get(key)?.parse().ok()
    }
    
    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.options.iter()
    }
}

/// Universal execution provider trait (format-agnostic)
/// 
/// This trait defines the interface that all execution providers must implement.
/// It is intentionally generic and does NOT depend on any specific inference
/// engine (ONNX Runtime, llama.cpp, etc.).
pub trait ExecutionProvider: Send + Sync + Debug {
    /// Human-readable name (e.g., "CUDAExecutionProvider")
    fn name(&self) -> &'static str;
    
    /// Backend type for this provider
    fn backend_type(&self) -> BackendType;
    
    /// Check if this provider is supported on the current platform (compile-time)
    fn supported_by_platform(&self) -> bool;
    
    /// Configuration options for this provider
    fn config(&self) -> &ProviderConfig;
    
    /// Check if this provider is available on the current system (runtime)
    /// This checks for actual hardware/drivers/libraries
    fn is_available(&self) -> Result<bool>;
}

/// Type-erased execution provider for dynamic dispatch
pub type DynExecutionProvider = Arc<dyn ExecutionProvider>;

/// Execution provider dispatch - collection of providers with priority order
#[derive(Debug, Clone)]
pub struct ExecutionProviderDispatch {
    providers: Vec<DynExecutionProvider>,
}

impl ExecutionProviderDispatch {
    pub fn new(providers: Vec<DynExecutionProvider>) -> Self {
        Self { providers }
    }
    
    pub fn providers(&self) -> &[DynExecutionProvider] {
        &self.providers
    }
    
    pub fn into_providers(self) -> Vec<DynExecutionProvider> {
        self.providers
    }
    
    /// Filter to only available providers
    pub fn filter_available(&self) -> Vec<DynExecutionProvider> {
        self.providers
            .iter()
            .filter(|p| p.is_available().unwrap_or(false))
            .cloned()
            .collect()
    }
}

/// Macro to reduce boilerplate when implementing ExecutionProvider base methods
#[macro_export]
macro_rules! impl_provider_base {
    ($struct_name:ident, $name:expr, $backend:expr) => {
        impl $struct_name {
            pub fn new() -> Self {
                Self {
                    config: $crate::ProviderConfig::new(),
                }
            }
            
            pub fn build(self) -> std::sync::Arc<dyn $crate::ExecutionProvider> {
                std::sync::Arc::new(self)
            }
            
            fn get_name(&self) -> &'static str {
                $name
            }
            
            fn get_backend_type(&self) -> $crate::BackendType {
                $backend
            }
        }
    };
}
