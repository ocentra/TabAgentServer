/*!
Windows-specific hardware detection using WMI and Windows APIs
*/

use crate::cpu::{CpuInfo, CpuVendor};
use crate::{HardwareError, Result};
use std::process::Command;

/// Detect CPU on Windows using PowerShell and WMI
pub fn detect_cpu() -> Result<CpuInfo> {
    // Get CPU info via PowerShell (modern, cross-version compatible)
    let output = Command::new("powershell")
        .args([
            "-Command",
            "Get-CimInstance -ClassName Win32_Processor | Select-Object Name, Manufacturer, NumberOfCores, NumberOfLogicalProcessors, Level, Revision | ConvertTo-Json"
        ])
        .output()
        .map_err(|e| HardwareError::CpuDetection(format!("PowerShell failed: {}", e)))?;
    
    if !output.status.success() {
        return Err(HardwareError::CpuDetection(
            String::from_utf8_lossy(&output.stderr).to_string()
        ));
    }
    
    // Parse JSON output
    let json_str = String::from_utf8_lossy(&output.stdout);
    let data: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| HardwareError::CpuDetection(format!("JSON parse failed: {}", e)))?;
    
    // Handle single CPU (object) or multiple CPUs (array)
    let cpu_data = if data.is_array() {
        &data[0]
    } else {
        &data
    };
    
    // Extract fields
    let model_name = cpu_data["Name"]
        .as_str()
        .unwrap_or("Unknown CPU")
        .trim()
        .to_string();
    
    let manufacturer = cpu_data["Manufacturer"]
        .as_str()
        .unwrap_or("")
        .to_lowercase();
    
    let cores = cpu_data["NumberOfCores"]
        .as_u64()
        .unwrap_or(0) as u32;
    
    let threads = cpu_data["NumberOfLogicalProcessors"]
        .as_u64()
        .unwrap_or(0) as u32;
    
    let family = cpu_data["Level"]
        .as_u64()
        .map(|v| v as u32);
    
    let revision = cpu_data["Revision"]
        .as_u64()
        .map(|v| v as u32);
    
    // Decode revision into model and stepping
    let (model_num, stepping) = if let Some(rev) = revision {
        let model_n = (rev >> 8) & 0xFF;
        let step = rev & 0xFF;
        (Some(model_n), Some(step))
    } else {
        (None, None)
    };
    
    // Detect vendor
    use crate::constants::*;
    let vendor = if manufacturer.contains(CPU_KEYWORD_INTEL) || model_name.to_lowercase().contains(CPU_KEYWORD_INTEL) {
        CpuVendor::Intel
    } else if manufacturer.contains(CPU_KEYWORD_AMD) || manufacturer.contains(CPU_KEYWORD_AUTHENTICAMD) 
        || model_name.to_lowercase().contains(CPU_KEYWORD_AMD) {
        CpuVendor::Amd
    } else {
        CpuVendor::Unknown
    };
    
    // Detect architecture from model name
    let mut architecture = crate::cpu::detect_from_name(&model_name, vendor);
    
    // Refine with CPUID if available
    if let (Some(fam), Some(model_n)) = (family, model_num) {
        architecture = crate::cpu::refine_from_cpuid(architecture, vendor, fam, model_n);
    }
    
    Ok(CpuInfo {
        vendor,
        architecture,
        model_name,
        cores,
        threads,
        family,
        model: model_num,
        stepping,
    })
}

/// Detect GPUs on Windows using nvidia-smi and wmic
pub fn detect_gpus() -> Result<Vec<crate::gpu::GpuInfo>> {
    let mut gpus = Vec::new();
    
    // Try nvidia-smi first (most accurate for NVIDIA GPUs)
    if let Ok(nvidia_gpus) = detect_nvidia_gpus() {
        gpus.extend(nvidia_gpus);
    }
    
    // Fall back to wmic for all GPUs (includes AMD, Intel, integrated)
    if gpus.is_empty() {
        if let Ok(wmic_gpus) = detect_gpus_wmic() {
            gpus.extend(wmic_gpus);
        }
    }
    
    Ok(gpus)
}

/// Detect NVIDIA GPUs using nvidia-smi
fn detect_nvidia_gpus() -> Result<Vec<crate::gpu::GpuInfo>> {
    use crate::gpu::{GpuInfo, GpuVendor};
    
    let output = Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,memory.total,driver_version",
            "--format=csv,noheader,nounits"
        ])
        .output()
        .map_err(|_| HardwareError::GpuDetection("nvidia-smi not found".to_string()))?;
    
    if !output.status.success() {
        return Err(HardwareError::GpuDetection(
            "nvidia-smi failed".to_string()
        ));
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut gpus = Vec::new();
    
    for line in stdout.lines() {
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        if parts.len() >= 3 {
            let name = parts[0].to_string();
            let vram_mb = parts[1].parse::<u64>().ok();
            let driver_version = Some(parts[2].to_string());
            
            gpus.push(GpuInfo {
                vendor: GpuVendor::Nvidia,
                name,
                vram_mb,
                driver_version,
            });
        }
    }
    
    if gpus.is_empty() {
        Err(HardwareError::GpuDetection("No NVIDIA GPUs found".to_string()))
    } else {
        Ok(gpus)
    }
}

/// Detect GPUs using wmic (fallback for AMD, Intel, integrated)
fn detect_gpus_wmic() -> Result<Vec<crate::gpu::GpuInfo>> {
    use crate::gpu::{GpuInfo, GpuVendor};
    
    let output = Command::new("wmic")
        .args([
            "path",
            "win32_VideoController",
            "get",
            "Name,AdapterRAM,DriverVersion",
            "/format:csv"
        ])
        .output()
        .map_err(|e| HardwareError::GpuDetection(format!("wmic failed: {}", e)))?;
    
    if !output.status.success() {
        return Err(HardwareError::GpuDetection(
            "wmic failed".to_string()
        ));
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut gpus = Vec::new();
    
    for (idx, line) in stdout.lines().enumerate() {
        // Skip header
        if idx == 0 || line.trim().is_empty() {
            continue;
        }
        
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        if parts.len() >= 4 {
            let name = parts[2].to_string();
            let name_lower = name.to_lowercase();
            
            // Skip Microsoft Basic Display Adapter and other virtual adapters
            use crate::constants::*;
            if name_lower.contains(KEYWORD_BASIC_DISPLAY) 
                || name_lower.contains(KEYWORD_MICROSOFT_BASIC) {
                continue;
            }
            
            // Parse VRAM (in bytes, convert to MB)
            let vram_mb = parts[1].parse::<u64>()
                .ok()
                .map(|bytes| bytes / (1024 * 1024));
            
            let driver_version = if !parts[3].is_empty() {
                Some(parts[3].to_string())
            } else {
                None
            };
            
            // Detect vendor from name
            let vendor = if name_lower.contains(GPU_KEYWORD_NVIDIA) || name_lower.contains(GPU_KEYWORD_GEFORCE) 
                || name_lower.contains(GPU_KEYWORD_RTX) || name_lower.contains(GPU_KEYWORD_GTX) {
                GpuVendor::Nvidia
            } else if name_lower.contains(GPU_KEYWORD_AMD) || name_lower.contains(GPU_KEYWORD_RADEON) 
                || name_lower.contains(GPU_KEYWORD_RYZEN) {
                GpuVendor::Amd
            } else if name_lower.contains(GPU_KEYWORD_INTEL) || name_lower.contains(GPU_KEYWORD_IRIS) 
                || name_lower.contains(GPU_KEYWORD_UHD) || name_lower.contains(GPU_KEYWORD_HD_GRAPHICS) {
                GpuVendor::Intel
            } else {
                GpuVendor::Unknown
            };
            
            gpus.push(GpuInfo {
                vendor,
                name,
                vram_mb,
                driver_version,
            });
        }
    }
    
    if gpus.is_empty() {
        Err(HardwareError::GpuDetection("No GPUs found via wmic".to_string()))
    } else {
        Ok(gpus)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_cpu_windows() {
        let cpu = detect_cpu().unwrap();
        println!("CPU: {:#?}", cpu);
        
        assert!(!cpu.model_name.is_empty());
        assert_ne!(cpu.vendor, CpuVendor::Unknown);
        assert!(cpu.cores > 0);
    }
    
    #[test]
    fn test_detect_gpus_windows() {
        let gpus = detect_gpus().unwrap();
        println!("GPUs: {:#?}", gpus);
        
        // Should detect at least one GPU
        assert!(!gpus.is_empty());
    }
}

