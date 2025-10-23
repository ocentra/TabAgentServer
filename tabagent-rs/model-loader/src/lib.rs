//! Model Loader - FFI bindings to llama.cpp/BitNet inference engines
//!
//! This crate provides safe Rust wrappers around the llama.cpp C API,
//! enabling efficient GGUF model loading and inference.

pub mod ffi;
pub mod model;
pub mod context;
pub mod error;

use tabagent_hardware::CpuArchitecture;
use std::path::{Path, PathBuf};

pub use error::{ModelError, Result};
pub use model::{Model, ModelConfig};
pub use context::{Context, GenerationParams};

/// Initialize the model loader library
pub fn init() -> Result<()> {
    log::info!("Initializing model-loader");
    // Any global initialization goes here
    Ok(())
}

/// Get the optimal DLL/shared library for the given CPU architecture
/// 
/// Maps CPU architecture to the best-performing llama.cpp binary
pub fn get_optimal_dll_for_platform(arch: CpuArchitecture) -> String {
    #[cfg(target_os = "windows")]
    let ext = ".dll";
    #[cfg(target_os = "linux")]
    let ext = ".so";
    #[cfg(target_os = "macos")]
    let ext = ".dylib";
    
    match arch {
        // Intel architectures
        CpuArchitecture::IntelAlderlake => format!("llama_intel_alderlake{}", ext),
        CpuArchitecture::IntelRocketlake => format!("llama_intel_rocketlake{}", ext),
        CpuArchitecture::IntelIcelake => format!("llama_intel_icelake{}", ext),
        
        // AMD architectures
        CpuArchitecture::AmdZen3 => format!("llama_amd_zen3{}", ext),
        CpuArchitecture::AmdZen4 => format!("llama_amd_zen4{}", ext),
        
        // Apple Silicon
        CpuArchitecture::AppleM1 => format!("llama_apple_m1{}", ext),
        CpuArchitecture::AppleM2 => format!("llama_apple_m2{}", ext),
        CpuArchitecture::AppleM3 => format!("llama_apple_m3{}", ext),
        
        // Generic fallback - covers all other architectures
        _ => format!("llama_generic{}", ext),
    }
}

/// Get full DLL path for the given architecture
/// 
/// Combines base path with architecture-specific DLL name
pub fn get_dll_path_for_architecture(base_path: &Path, arch: CpuArchitecture) -> PathBuf {
    let dll_name = get_optimal_dll_for_platform(arch);
    base_path.join(dll_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        assert!(init().is_ok());
    }
}
