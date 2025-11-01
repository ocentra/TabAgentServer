/*!
# TabAgent Hardware Detection

Detects CPU and GPU hardware for optimal binary variant selection.

## Features

- CPU microarchitecture detection (Zen2, Zen3, Alderlake, etc.)
- GPU vendor and capability detection
- Cross-platform support (Windows, Linux, macOS)
- Serializable for JSON output

## Example

```rust,no_run
use tabagent_hardware::{detect_system, CpuArchitecture};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let system = detect_system()?;
    println!("CPU: {}", system.cpu.model_name);
    println!("Architecture: {:?}", system.cpu.architecture);

    // Select optimal binary variant
    let variant = system.cpu.architecture.variant_name();
    println!("Use binary variant: {}", variant);
    
    Ok(())
}
```
*/

use serde::{Deserialize, Serialize};
use thiserror::Error;
use sysinfo::System;

mod cpu;
mod gpu;
mod memory;
pub mod constants;
pub mod recommendations;

#[cfg(target_os = "windows")]
mod platform_windows;

#[cfg(target_os = "linux")]
mod platform_linux;

#[cfg(target_os = "macos")]
mod platform_macos;

pub use cpu::{CpuArchitecture, CpuInfo, CpuVendor};
pub use gpu::{GpuInfo, GpuVendor};
pub use memory::{MemoryInfo, detect_memory, calculate_total_vram, get_ram_tier, get_vram_tier};
pub use recommendations::{
    get_bitnet_dll_variant, 
    get_bitnet_dll_filename,
    recommend_execution_provider,
    recommend_loading_strategy,
    ExecutionProviderRecommendation,
    ModelLoadingStrategy,
};

#[derive(Debug, Error)]
pub enum HardwareError {
    #[error("Failed to detect CPU: {0}")]
    CpuDetection(String),
    
    #[error("Failed to detect GPU: {0}")]
    GpuDetection(String),
    
    #[error("Platform not supported: {0}")]
    UnsupportedPlatform(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, HardwareError>;

/// Complete system hardware information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub cpu: CpuInfo,
    pub gpus: Vec<GpuInfo>,
    pub memory: MemoryInfo,
    pub os: OsInfo,
    
    // Computed fields
    pub total_vram_mb: u64,
    pub ram_tier: String,
    pub vram_tier: String,
}

impl SystemInfo {
    /// Get the recommended BitNet DLL variant for this CPU
    pub fn bitnet_dll_variant(&self) -> &'static str {
        get_bitnet_dll_variant(self.cpu.architecture)
    }
    
    /// Get the full BitNet DLL filename (Windows)
    pub fn bitnet_dll_filename(&self) -> String {
        get_bitnet_dll_filename(self.cpu.architecture)
    }
    
    /// Get recommended execution provider
    pub fn recommended_execution_provider(&self) -> ExecutionProviderRecommendation {
        recommend_execution_provider(&self.gpus, &self.os.name)
    }
    
    /// Get recommended loading strategy for a model
    pub fn recommended_loading_strategy(&self, model_size_mb: u64) -> ModelLoadingStrategy {
        recommend_loading_strategy(
            model_size_mb,
            self.memory.total_ram_mb,
            self.memory.available_ram_mb,
            &self.gpus,
        )
    }
}

/// Operating system information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsInfo {
    pub name: String,
    pub version: String,
    pub arch: String,
}

impl OsInfo {
    /// Detect operating system information including version.
    ///
    /// Uses `sysinfo` crate for cross-platform OS version detection.
    ///
    /// # Returns
    ///
    /// Returns `OsInfo` with OS name, version, and architecture.
    pub fn detect() -> Self {
        let os_name = std::env::consts::OS.to_string();
        let os_version = System::long_os_version()
            .or_else(|| System::os_version())
            .unwrap_or_else(|| "unknown".to_string());
        
        Self {
            name: os_name,
            version: os_version,
            arch: std::env::consts::ARCH.to_string(),
        }
    }
}

/// Detect complete system hardware
pub fn detect_system() -> Result<SystemInfo> {
    let cpu = cpu::detect_cpu()?;
    let gpus = gpu::detect_gpus()?;
    let memory = detect_memory()?;
    let os = OsInfo::detect();
    
    // Calculate totals and tiers
    let total_vram_mb = calculate_total_vram(&gpus);
    let ram_tier = get_ram_tier(memory.total_ram_mb).to_string();
    let vram_tier = get_vram_tier(total_vram_mb).to_string();
    
    Ok(SystemInfo { 
        cpu, 
        gpus, 
        memory, 
        os,
        total_vram_mb,
        ram_tier,
        vram_tier,
    })
}

/// Quick CPU architecture detection
pub fn detect_cpu_architecture() -> Result<CpuArchitecture> {
    let cpu = cpu::detect_cpu()?;
    Ok(cpu.architecture)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_system() {
        let system = detect_system().unwrap();
        println!("System Info: {:#?}", system);
        
        assert!(!system.cpu.model_name.is_empty());
        assert_ne!(system.cpu.architecture, CpuArchitecture::Unknown);
    }
    
    #[test]
    fn test_cpu_variant_name() {
        let arch = CpuArchitecture::AmdZen2;
        assert_eq!(arch.variant_name(), "bitnet-amd-zen2");
        
        let arch = CpuArchitecture::IntelAlderlake;
        assert_eq!(arch.variant_name(), "bitnet-intel-alderlake");
    }
}
