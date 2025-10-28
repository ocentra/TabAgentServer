//! Library variant selection matching BitnetRelease structure
//!
//! This module provides a trait-based system for selecting the correct
//! llama.cpp/BitNet library variant based on hardware and platform.
//!
//! Maps exactly to External/BitNet/BitnetRelease/ structure and will
//! produce the same structure in TabAgentDist/Release/ for server binaries.

use std::path::{Path, PathBuf};
use tabagent_hardware::{CpuArchitecture, GpuVendor};
use crate::{ModelError, Result};

/// Library variant trait - implemented by each variant type
pub trait LibraryVariant {
    /// Get the variant directory name (e.g., "bitnet-amd-zen2", "standard-cuda-vulkan")
    fn variant_name(&self) -> &'static str;
    
    /// Get the base directory type (cpu or gpu)
    fn base_type(&self) -> &'static str;
    
    /// Get platform-specific library name
    fn library_name(&self) -> &'static str;
    
    /// Build full path to library
    fn library_path(&self, base_path: &Path) -> PathBuf {
        base_path
            .join("BitnetRelease")
            .join(self.base_type())
            .join(Self::os_name())
            .join(self.variant_name())
            .join(self.library_name())
    }
    
    /// Get OS directory name
    fn os_name() -> &'static str {
        #[cfg(target_os = "windows")]
        { "windows" }
        
        #[cfg(target_os = "linux")]
        { "linux" }
        
        #[cfg(target_os = "macos")]
        { "macos" }
    }
}

/// BitNet CPU variants (TL1/TL2 optimized kernels)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitNetCpuVariant {
    AmdZen1,
    AmdZen2,
    AmdZen3,
    AmdZen4,
    AmdZen5,
    IntelHaswell,
    IntelBroadwell,
    IntelSkylake,
    IntelIcelake,
    IntelRocketlake,
    IntelAlderlake,
    Arm,        // For Apple Silicon and ARM
    Portable,   // Fallback
}

impl BitNetCpuVariant {
    /// Auto-detect from CPU architecture
    pub fn from_architecture(arch: &CpuArchitecture) -> Self {
        match arch {
            CpuArchitecture::AmdZen1 => Self::AmdZen1,
            CpuArchitecture::AmdZen2 => Self::AmdZen2,
            CpuArchitecture::AmdZen3 => Self::AmdZen3,
            CpuArchitecture::AmdZen4 => Self::AmdZen4,
            CpuArchitecture::AmdZen5 => Self::AmdZen5,
            CpuArchitecture::IntelHaswell => Self::IntelHaswell,
            CpuArchitecture::IntelBroadwell => Self::IntelBroadwell,
            CpuArchitecture::IntelSkylake => Self::IntelSkylake,
            CpuArchitecture::IntelIcelake => Self::IntelIcelake,
            CpuArchitecture::IntelRocketlake => Self::IntelRocketlake,
            CpuArchitecture::IntelAlderlake => Self::IntelAlderlake,
            CpuArchitecture::AppleM1 | CpuArchitecture::AppleM2 | CpuArchitecture::AppleM3 => Self::Arm,
            CpuArchitecture::ArmV8 | CpuArchitecture::ArmV9 => Self::Arm,
            _ => Self::Portable,
        }
    }
}

impl LibraryVariant for BitNetCpuVariant {
    fn variant_name(&self) -> &'static str {
        match self {
            Self::AmdZen1 => "bitnet-amd-zen1",
            Self::AmdZen2 => "bitnet-amd-zen2",
            Self::AmdZen3 => "bitnet-amd-zen3",
            Self::AmdZen4 => "bitnet-amd-zen4",
            Self::AmdZen5 => "bitnet-amd-zen5",
            Self::IntelHaswell => "bitnet-intel-haswell",
            Self::IntelBroadwell => "bitnet-intel-broadwell",
            Self::IntelSkylake => "bitnet-intel-skylake",
            Self::IntelIcelake => "bitnet-intel-icelake",
            Self::IntelRocketlake => "bitnet-intel-rocketlake",
            Self::IntelAlderlake => "bitnet-intel-alderlake",
            Self::Arm => "bitnet-arm",
            Self::Portable => "bitnet-portable",
        }
    }
    
    fn base_type(&self) -> &'static str {
        "cpu"
    }
    
    fn library_name(&self) -> &'static str {
        #[cfg(target_os = "windows")]
        { "llama.dll" }
        
        #[cfg(target_os = "linux")]
        { "libllama.so" }
        
        #[cfg(target_os = "macos")]
        { "libllama.dylib" }
    }
}

/// Standard CPU variant (supports all GGUF quantizations)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StandardCpuVariant;

impl LibraryVariant for StandardCpuVariant {
    fn variant_name(&self) -> &'static str {
        "standard"
    }
    
    fn base_type(&self) -> &'static str {
        "cpu"
    }
    
    fn library_name(&self) -> &'static str {
        #[cfg(target_os = "windows")]
        { "llama.dll" }
        
        #[cfg(target_os = "linux")]
        { "libllama.so" }
        
        #[cfg(target_os = "macos")]
        { "libllama.dylib" }
    }
}

/// BitNet GPU variants (TL1/TL2 optimized, CUDA-only, Windows/Linux only)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BitNetGpuVariant;

impl LibraryVariant for BitNetGpuVariant {
    fn variant_name(&self) -> &'static str {
        "bitnet-cuda"
    }
    
    fn base_type(&self) -> &'static str {
        "gpu"
    }
    
    fn library_name(&self) -> &'static str {
        #[cfg(target_os = "windows")]
        { "llama.dll" }
        
        #[cfg(target_os = "linux")]
        { "libllama.so" }
        
        #[cfg(target_os = "macos")]
        { "libllama.dylib" } // Not available on macOS, but needed for compilation
    }
}

/// Standard GPU variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StandardGpuVariant {
    CudaVulkan,  // NVIDIA + AMD (Vulkan fallback)
    Metal,       // macOS/Apple Silicon
    OpenCL,      // Intel GPUs
}

impl StandardGpuVariant {
    /// Auto-detect from GPU vendor
    pub fn from_gpu_vendor(vendor: GpuVendor) -> Self {
        #[cfg(target_os = "macos")]
        { return Self::Metal; }
        
        #[cfg(not(target_os = "macos"))]
        match vendor {
            GpuVendor::Intel => Self::OpenCL,
            _ => Self::CudaVulkan, // NVIDIA, AMD, or unknown
        }
    }
}

impl LibraryVariant for StandardGpuVariant {
    fn variant_name(&self) -> &'static str {
        match self {
            Self::CudaVulkan => "standard-cuda-vulkan",
            Self::Metal => "standard-metal",
            Self::OpenCL => "standard-opencl",
        }
    }
    
    fn base_type(&self) -> &'static str {
        "gpu"
    }
    
    fn library_name(&self) -> &'static str {
        #[cfg(target_os = "windows")]
        { "llama.dll" }
        
        #[cfg(target_os = "linux")]
        { "libllama.so" }
        
        #[cfg(target_os = "macos")]
        { "libllama.dylib" }
    }
}

/// Unified variant selector
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variant {
    BitNetCpu(BitNetCpuVariant),
    BitNetGpu(BitNetGpuVariant),
    StandardCpu(StandardCpuVariant),
    StandardGpu(StandardGpuVariant),
}

impl Variant {
    /// Get library path for this variant
    pub fn library_path(&self, base_path: &Path) -> Result<PathBuf> {
        let path = match self {
            Self::BitNetCpu(v) => v.library_path(base_path),
            Self::BitNetGpu(v) => v.library_path(base_path),
            Self::StandardCpu(v) => v.library_path(base_path),
            Self::StandardGpu(v) => v.library_path(base_path),
        };
        
        if !path.exists() {
            return Err(ModelError::LibraryLoadError(format!(
                "Library not found at: {} (variant: {:?})",
                path.display(),
                self
            )));
        }
        
        log::info!("Selected library: {} (variant: {:?})", path.display(), self);
        Ok(path)
    }
    
    /// Get variant name
    pub fn name(&self) -> &'static str {
        match self {
            Self::BitNetCpu(v) => v.variant_name(),
            Self::BitNetGpu(v) => v.variant_name(),
            Self::StandardCpu(v) => v.variant_name(),
            Self::StandardGpu(v) => v.variant_name(),
        }
    }
}

/// List all available variants for current platform
pub fn list_available_variants(base_path: &Path) -> Vec<(Variant, PathBuf)> {
    let mut variants = Vec::new();
    
    // All CPU variants
    let cpu_variants: Vec<Variant> = vec![
        Variant::BitNetCpu(BitNetCpuVariant::AmdZen1),
        Variant::BitNetCpu(BitNetCpuVariant::AmdZen2),
        Variant::BitNetCpu(BitNetCpuVariant::AmdZen3),
        Variant::BitNetCpu(BitNetCpuVariant::AmdZen4),
        Variant::BitNetCpu(BitNetCpuVariant::AmdZen5),
        Variant::BitNetCpu(BitNetCpuVariant::IntelHaswell),
        Variant::BitNetCpu(BitNetCpuVariant::IntelBroadwell),
        Variant::BitNetCpu(BitNetCpuVariant::IntelSkylake),
        Variant::BitNetCpu(BitNetCpuVariant::IntelIcelake),
        Variant::BitNetCpu(BitNetCpuVariant::IntelRocketlake),
        Variant::BitNetCpu(BitNetCpuVariant::IntelAlderlake),
        Variant::BitNetCpu(BitNetCpuVariant::Arm),
        Variant::BitNetCpu(BitNetCpuVariant::Portable),
        Variant::StandardCpu(StandardCpuVariant),
    ];
    
    // All GPU variants
    let mut gpu_variants: Vec<Variant> = vec![
        Variant::StandardGpu(StandardGpuVariant::CudaVulkan),
        Variant::StandardGpu(StandardGpuVariant::Metal),
        Variant::StandardGpu(StandardGpuVariant::OpenCL),
    ];
    
    // BitNet GPU (CUDA-only, Windows/Linux only)
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    gpu_variants.insert(0, Variant::BitNetGpu(BitNetGpuVariant));
    
    // Check which ones exist
    for variant in cpu_variants.into_iter().chain(gpu_variants.into_iter()) {
        if let Ok(path) = variant.library_path(base_path) {
            variants.push((variant, path));
        }
    }
    
    variants
}

