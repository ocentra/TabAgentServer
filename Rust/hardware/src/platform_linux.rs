/*!
Linux-specific hardware detection using /proc/cpuinfo
*/

use crate::cpu::{CpuArchitecture, CpuInfo, CpuVendor};
use crate::{HardwareError, Result};
use std::fs;

pub fn detect_cpu() -> Result<CpuInfo> {
    let cpuinfo = fs::read_to_string("/proc/cpuinfo")
        .map_err(|e| HardwareError::CpuDetection(format!("Failed to read /proc/cpuinfo: {}", e)))?;
    
    let mut model_name = String::from("Unknown CPU");
    let mut vendor_id = String::new();
    let mut family = None;
    let mut model = None;
    let mut stepping = None;
    let mut cores = 0u32;
    let mut threads = 0u32;
    
    for line in cpuinfo.lines() {
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim();
            
            match key {
                "model name" if model_name == "Unknown CPU" => {
                    model_name = value.to_string();
                }
                "vendor_id" if vendor_id.is_empty() => {
                    vendor_id = value.to_lowercase();
                }
                "cpu family" if family.is_none() => {
                    family = value.parse().ok();
                }
                "model" if model.is_none() => {
                    model = value.parse().ok();
                }
                "stepping" if stepping.is_none() => {
                    stepping = value.parse().ok();
                }
                "cpu cores" if cores == 0 => {
                    cores = value.parse().unwrap_or(0);
                }
                "siblings" if threads == 0 => {
                    threads = value.parse().unwrap_or(0);
                }
                _ => {}
            }
        }
    }
    
    // Fallback for cores/threads
    if cores == 0 {
        cores = threads;
    }
    if threads == 0 {
        threads = cores;
    }
    
    // Detect vendor
    use crate::constants::*;
    let vendor = if vendor_id.contains(CPU_KEYWORD_INTEL) || vendor_id.contains(CPU_KEYWORD_GENUINEINTEL) {
        CpuVendor::Intel
    } else if vendor_id.contains(CPU_KEYWORD_AMD) || vendor_id.contains(CPU_KEYWORD_AUTHENTICAMD) {
        CpuVendor::Amd
    } else {
        CpuVendor::Unknown
    };
    
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
        stepping,
    })
}

/// Detect GPUs on Linux using lspci and nvidia-smi
pub fn detect_gpus() -> Result<Vec<crate::gpu::GpuInfo>> {
    use crate::gpu::{GpuInfo, GpuVendor};
    use std::process::Command;
    
    let mut gpus = Vec::new();
    
    // Try nvidia-smi first
    if let Ok(output) = Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,memory.total,driver_version",
            "--format=csv,noheader,nounits"
        ])
        .output() 
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                if parts.len() >= 3 {
                    gpus.push(GpuInfo {
                        vendor: GpuVendor::Nvidia,
                        name: parts[0].to_string(),
                        vram_mb: parts[1].parse().ok(),
                        driver_version: Some(parts[2].to_string()),
                    });
                }
            }
        }
    }
    
    // Fall back to lspci
    if gpus.is_empty() {
        if let Ok(output) = Command::new("lspci").output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    use crate::constants::*;
                    let line_lower = line.to_lowercase();
                    if line_lower.contains(KEYWORD_VGA) || line_lower.contains(KEYWORD_3D) {
                        let vendor = if line_lower.contains(GPU_KEYWORD_NVIDIA) {
                            GpuVendor::Nvidia
                        } else if line_lower.contains(GPU_KEYWORD_AMD) || line_lower.contains(GPU_KEYWORD_ATI) {
                            GpuVendor::Amd
                        } else if line_lower.contains(GPU_KEYWORD_INTEL) {
                            GpuVendor::Intel
                        } else {
                            GpuVendor::Unknown
                        };
                        
                        // Extract GPU name (after colon)
                        let name = line.split(':')
                            .last()
                            .unwrap_or("Unknown GPU")
                            .trim()
                            .to_string();
                        
                        gpus.push(GpuInfo {
                            vendor,
                            name,
                            vram_mb: None, // lspci doesn't provide VRAM
                            driver_version: None,
                        });
                    }
                }
            }
        }
    }
    
    Ok(gpus)
}

