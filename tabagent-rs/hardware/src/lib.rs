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
use std::fmt;
use thiserror::Error;

mod cpu;
mod gpu;

#[cfg(target_os = "windows")]
mod platform_windows;

#[cfg(target_os = "linux")]
mod platform_linux;

#[cfg(target_os = "macos")]
mod platform_macos;

pub use cpu::{CpuArchitecture, CpuInfo, CpuVendor};
pub use gpu::{GpuInfo, GpuVendor};

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
    pub os: OsInfo,
}

/// Operating system information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsInfo {
    pub name: String,
    pub version: String,
    pub arch: String,
}

impl OsInfo {
    pub fn detect() -> Self {
        Self {
            name: std::env::consts::OS.to_string(),
            version: "unknown".to_string(), // TODO: Get OS version
            arch: std::env::consts::ARCH.to_string(),
        }
    }
}

/// Detect complete system hardware
pub fn detect_system() -> Result<SystemInfo> {
    let cpu = cpu::detect_cpu()?;
    let gpus = gpu::detect_gpus()?;
    let os = OsInfo::detect();
    
    Ok(SystemInfo { cpu, gpus, os })
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
