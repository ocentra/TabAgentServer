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

/// Detect GPUs (simplified for now)
pub fn detect_gpus() -> Result<Vec<GpuInfo>> {
    // TODO: Implement GPU detection
    // For now, return empty list
    // Future: Use nvidia-smi, wmic, vulkan info, etc.
    Ok(Vec::new())
}

