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
    let vendor = if vendor_id.contains("intel") || vendor_id.contains("genuineintel") {
        CpuVendor::Intel
    } else if vendor_id.contains("amd") || vendor_id.contains("authenticamd") {
        CpuVendor::Amd
    } else {
        CpuVendor::Unknown
    };
    
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
        stepping,
    })
}

