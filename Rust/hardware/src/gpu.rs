/*!
GPU Detection

Detects GPU vendor and capabilities for acceleration selection.
*/

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Apple,
    Unknown,
}

impl fmt::Display for GpuVendor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Nvidia => write!(f, "NVIDIA"),
            Self::Amd => write!(f, "AMD"),
            Self::Intel => write!(f, "Intel"),
            Self::Apple => write!(f, "Apple"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub vendor: GpuVendor,
    pub name: String,
    pub vram_mb: Option<u64>,
    pub driver_version: Option<String>,
}

/// Detect GPUs using platform-specific methods
pub fn detect_gpus() -> Result<Vec<GpuInfo>> {
    #[cfg(target_os = "windows")]
    {
        crate::platform_windows::detect_gpus()
    }
    
    #[cfg(target_os = "linux")]
    {
        crate::platform_linux::detect_gpus()
    }
    
    #[cfg(target_os = "macos")]
    {
        crate::platform_macos::detect_gpus()
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        // Unsupported platform
        Ok(Vec::new())
    }
}

