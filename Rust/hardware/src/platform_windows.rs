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
    let vendor = if manufacturer.contains("intel") || model_name.to_lowercase().contains("intel") {
        CpuVendor::Intel
    } else if manufacturer.contains("amd") || manufacturer.contains("authenticamd") 
        || model_name.to_lowercase().contains("amd") {
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
}

