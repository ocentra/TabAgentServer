/*!
macOS-specific hardware detection using sysctl
*/

use crate::cpu::{CpuArchitecture, CpuInfo, CpuVendor};
use crate::{HardwareError, Result};
use std::process::Command;

pub fn detect_cpu() -> Result<CpuInfo> {
    // Get CPU brand string
    let output = Command::new("sysctl")
        .args(&["-n", "machdep.cpu.brand_string"])
        .output()
        .map_err(|e| HardwareError::CpuDetection(format!("sysctl failed: {}", e)))?;
    
    let model_name = String::from_utf8_lossy(&output.stdout).trim().to_string();
    
    // Check if Apple Silicon
    use crate::constants::*;
    let arch = std::env::consts::ARCH;
    let vendor = if arch == "aarch64" || arch == "arm64" {
        CpuVendor::Apple
    } else if model_name.to_lowercase().contains(CPU_KEYWORD_INTEL) {
        CpuVendor::Intel
    } else {
        CpuVendor::Unknown
    };
    
    // Get cores
    let cores_output = Command::new("sysctl")
        .args(&["-n", "hw.physicalcpu"])
        .output()
        .ok();
    let cores = cores_output
        .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse().ok())
        .unwrap_or(0);
    
    // Get threads
    let threads_output = Command::new("sysctl")
        .args(&["-n", "hw.logicalcpu"])
        .output()
        .ok();
    let threads = threads_output
        .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse().ok())
        .unwrap_or(cores);
    
    // Get CPUID family/model
    let family_output = Command::new("sysctl")
        .args(&["-n", "machdep.cpu.family"])
        .output()
        .ok();
    let family = family_output
        .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse().ok());
    
    let model_output = Command::new("sysctl")
        .args(&["-n", "machdep.cpu.model"])
        .output()
        .ok();
    let model = model_output
        .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse().ok());
    
    // Detect architecture
    let mut architecture = crate::cpu::detect_from_name(&model_name, vendor);
    
    if let (Some(fam), Some(model_num)) = (family, model) {
        architecture = crate::cpu::refine_from_cpuid(architecture, vendor, fam, model_num);
    }
    
    Ok(CpuInfo {
        vendor,
        architecture,
        model_name,
        cores,
        threads,
        family,
        model,
        stepping: None,
    })
}

/// Detect GPUs on macOS using system_profiler
pub fn detect_gpus() -> Result<Vec<crate::gpu::GpuInfo>> {
    use crate::gpu::{GpuInfo, GpuVendor};
    
    let output = Command::new("system_profiler")
        .args(&["SPDisplaysDataType", "-json"])
        .output()
        .map_err(|e| HardwareError::GpuDetection(format!("system_profiler failed: {}", e)))?;
    
    if !output.status.success() {
        return Ok(Vec::new());
    }
    
    let json_str = String::from_utf8_lossy(&output.stdout);
    let data: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| HardwareError::GpuDetection(format!("JSON parse failed: {}", e)))?;
    
    let mut gpus = Vec::new();
    
    if let Some(displays) = data["SPDisplaysDataType"].as_array() {
        for display in displays {
            if let Some(name) = display["sppci_model"].as_str() {
                use crate::constants::*;
                let name_lower = name.to_lowercase();
                
                let vendor = if name_lower.contains(GPU_KEYWORD_APPLE) || name_lower.contains(GPU_KEYWORD_M1) 
                    || name_lower.contains(GPU_KEYWORD_M2) || name_lower.contains(GPU_KEYWORD_M3) {
                    GpuVendor::Apple
                } else if name_lower.contains(GPU_KEYWORD_NVIDIA) {
                    GpuVendor::Nvidia
                } else if name_lower.contains(GPU_KEYWORD_AMD) || name_lower.contains(GPU_KEYWORD_RADEON) {
                    GpuVendor::Amd
                } else if name_lower.contains(GPU_KEYWORD_INTEL) {
                    GpuVendor::Intel
                } else {
                    GpuVendor::Unknown
                };
                
                // Extract VRAM if available
                let vram_mb = display["sppci_vram"]
                    .as_str()
                    .and_then(|s| {
                        // Parse strings like "8 GB" or "8192 MB"
                        let parts: Vec<&str> = s.split_whitespace().collect();
                        if parts.len() >= 2 {
                            let value: u64 = parts[0].parse().ok()?;
                            let unit = parts[1].to_uppercase();
                            if unit.starts_with("GB") {
                                Some(value * 1024)
                            } else {
                                Some(value)
                            }
                        } else {
                            None
                        }
                    });
                
                gpus.push(GpuInfo {
                    vendor,
                    name: name.to_string(),
                    vram_mb,
                    driver_version: None, // macOS doesn't expose driver version easily
                });
            }
        }
    }
    
    Ok(gpus)
}

