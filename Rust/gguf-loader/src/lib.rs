//! GGUF Loader - FFI bindings to llama.cpp/BitNet inference engines
//!
//! This crate provides safe Rust wrappers around the llama.cpp C API,
//! enabling efficient GGUF model loading and inference.
//!
//! # Library Variant Structure
//!
//! Maps exactly to External/BitNet/BitnetRelease/ directory structure:
//!
//! ```text
//! BitnetRelease/
//!   ├── cpu/
//!   │   ├── windows/
//!   │   │   ├── bitnet-amd-zen1/llama.dll
//!   │   │   ├── bitnet-amd-zen2/llama.dll
//!   │   │   ├── ...
//!   │   │   ├── bitnet-intel-skylake/llama.dll
//!   │   │   ├── ...
//!   │   │   └── standard/llama.dll
//!   │   ├── linux/
//!   │   │   └── (same structure, libllama.so)
//!   │   └── macos/
//!   │       └── (same structure, libllama.dylib)
//!   └── gpu/
//!       ├── windows/
//!       │   ├── bitnet-cuda/llama.dll          (BitNet GPU, NVIDIA only)
//!       │   ├── standard-cuda-vulkan/llama.dll (NVIDIA + AMD)
//!       │   └── standard-opencl/llama.dll      (Intel)
//!       ├── linux/
//!       │   └── (same structure)
//!       └── macos/
//!           └── standard-metal/libllama.dylib  (Apple Silicon)
//! ```
//!
//! This structure will be replicated in TabAgentDist/Release/ for server binaries.
//! The installer script will detect hardware and install the correct variant.

pub mod ffi;
pub mod model;
pub mod context;
pub mod error;
pub mod variant;

use std::path::{Path, PathBuf};

pub use error::{ModelError, Result};
pub use model::{Model, ModelConfig};
pub use context::{Context, GenerationParams};
pub use variant::{
    Variant, 
    BitNetCpuVariant, 
    BitNetGpuVariant,
    StandardCpuVariant, 
    StandardGpuVariant,
    LibraryVariant,
    list_available_variants,
};

/// Initialize the GGUF loader library
pub fn init() -> Result<()> {
    log::info!("Initializing gguf-loader");
    Ok(())
}

/// Auto-select optimal library variant based on system hardware
///
/// Uses `tabagent_hardware::detect_system()` to choose the best variant.
///
/// # Selection Priority
/// 1. **BitNet GPU** (CUDA-only, Windows/Linux, NVIDIA GPUs)
/// 2. **Standard GPU** (CUDA/Vulkan/Metal/OpenCL based on vendor)
/// 3. **BitNet CPU** (Architecture-specific TL1/TL2 kernels)
/// 4. **Standard CPU** (Fallback for unsupported architectures)
///
/// # Returns
/// The best available `Variant` for the detected hardware
pub fn auto_select_variant(prefer_gpu: bool) -> Result<Variant> {
    use tabagent_hardware::{detect_system, GpuVendor};
    
    let system = detect_system()
        .map_err(|e| ModelError::LibraryLoadError(format!("Hardware detection failed: {}", e)))?;
    
    // GPU selection (if preferred and available)
    if prefer_gpu && !system.gpus.is_empty() {
        // SAFETY: We just checked !is_empty(), so first() is guaranteed to return Some
        let gpu = system.gpus.first()
            .ok_or_else(|| ModelError::LibraryLoadError("No GPU available despite non-empty check".to_string()))?;
        
        // BitNet GPU for NVIDIA on Windows/Linux
        #[cfg(any(target_os = "windows", target_os = "linux"))]
        if matches!(gpu.vendor, GpuVendor::Nvidia) {
            log::info!("Selected BitNet GPU (CUDA) for NVIDIA GPU");
            return Ok(Variant::BitNetGpu(BitNetGpuVariant));
        }
        
        // Standard GPU variants
        log::info!("Selected Standard GPU for {:?}", gpu.vendor);
        return Ok(Variant::StandardGpu(StandardGpuVariant::from_gpu_vendor(gpu.vendor)));
    }
    
    // CPU selection
    let cpu_variant = BitNetCpuVariant::from_architecture(&system.cpu.architecture);
    log::info!("Selected BitNet CPU variant: {:?}", cpu_variant);
    Ok(Variant::BitNetCpu(cpu_variant))
}

/// Get library path for auto-selected variant
///
/// # Arguments
/// * `base_path` - Base directory containing BitnetRelease/
/// * `prefer_gpu` - Whether to prefer GPU variants
///
/// # Returns
/// Absolute path to the selected library file
pub fn auto_select_library(base_path: &Path, prefer_gpu: bool) -> Result<PathBuf> {
    let variant = auto_select_variant(prefer_gpu)?;
    variant.library_path(base_path)
}

/// Get library path for specific variant
///
/// # Arguments
/// * `base_path` - Base directory containing BitnetRelease/
/// * `variant` - Specific variant to use
///
/// # Returns
/// Absolute path to the library file
pub fn select_library_path(base_path: &Path, variant: Variant) -> Result<PathBuf> {
    variant.library_path(base_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        assert!(init().is_ok());
    }
}
