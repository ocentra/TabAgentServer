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
    let arch = std::env::consts::ARCH;
    let vendor = if arch == "aarch64" || arch == "arm64" {
        CpuVendor::Apple
    } else if model_name.to_lowercase().contains("intel") {
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
    
    if let (Some(fam), Some(mod)) = (family, model) {
        architecture = crate::cpu::refine_from_cpuid(architecture, vendor, fam, mod);
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

