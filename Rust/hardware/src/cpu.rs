/*!
CPU Architecture Detection

Detects CPU vendor and microarchitecture for optimal binary variant selection.
*/

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CpuVendor {
    Intel,
    Amd,
    Apple,
    Arm,
    Unknown,
}

impl fmt::Display for CpuVendor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Intel => write!(f, "Intel"),
            Self::Amd => write!(f, "AMD"),
            Self::Apple => write!(f, "Apple"),
            Self::Arm => write!(f, "ARM"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CpuArchitecture {
    // AMD Zen architectures
    AmdZen1,
    AmdZen2,
    AmdZen3,
    AmdZen4,
    AmdZen5,
    
    // Intel architectures
    IntelHaswell,
    IntelBroadwell,
    IntelSkylake,
    IntelIcelake,
    IntelRocketlake,
    IntelAlderlake,
    
    // Apple Silicon
    AppleM1,
    AppleM2,
    AppleM3,
   
    // ARM
    ArmV8,
    ArmV9,
    
    // Fallback
    Portable,
    Unknown,
}

impl CpuArchitecture {
    /// Get the BitNet binary variant name for this architecture
    pub fn variant_name(&self) -> &'static str {
        match self {
            // AMD
            Self::AmdZen1 => "bitnet-amd-zen1",
            Self::AmdZen2 => "bitnet-amd-zen2",
            Self::AmdZen3 => "bitnet-amd-zen3",
            Self::AmdZen4 => "bitnet-amd-zen4",
            Self::AmdZen5 => "bitnet-amd-zen5",
            
            // Intel
            Self::IntelHaswell => "bitnet-intel-haswell",
            Self::IntelBroadwell => "bitnet-intel-broadwell",
            Self::IntelSkylake => "bitnet-intel-skylake",
            Self::IntelIcelake => "bitnet-intel-icelake",
            Self::IntelRocketlake => "bitnet-intel-rocketlake",
            Self::IntelAlderlake => "bitnet-intel-alderlake",
            
            // Apple/ARM - use portable for now
            Self::AppleM1 | Self::AppleM2 | Self::AppleM3 => "bitnet-portable",
            Self::ArmV8 | Self::ArmV9 => "bitnet-portable",
            
            // Fallback
            Self::Portable | Self::Unknown => "bitnet-portable",
        }
    }
    
    /// Get standard (non-BitNet) variant name
    pub fn standard_variant(&self) -> &'static str {
        "standard"
    }
}

impl fmt::Display for CpuArchitecture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.variant_name())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    pub vendor: CpuVendor,
    pub architecture: CpuArchitecture,
    pub model_name: String,
    pub cores: u32,
    pub threads: u32,
    pub family: Option<u32>,
    pub model: Option<u32>,
    pub stepping: Option<u32>,
}

impl CpuInfo {
    pub fn variant_name(&self) -> &'static str {
        self.architecture.variant_name()
    }
}

/// Detect CPU information
pub fn detect_cpu() -> Result<CpuInfo> {
    #[cfg(target_os = "windows")]
    return crate::platform_windows::detect_cpu();
    
    #[cfg(target_os = "linux")]
    return crate::platform_linux::detect_cpu();
    
    #[cfg(target_os = "macos")]
    return crate::platform_macos::detect_cpu();
    
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    Err(HardwareError::UnsupportedPlatform(
        std::env::consts::OS.to_string()
    ))
}

/// Detect architecture from model name (coarse detection, refined by CPUID)
pub(crate) fn detect_from_name(model_name: &str, vendor: CpuVendor) -> CpuArchitecture {
    let name_lower = model_name.to_lowercase();
    
    match vendor {
        CpuVendor::Amd => {
            // AMD Ryzen detection
            if name_lower.contains("ryzen") {
                // Ryzen 9000 series (Zen 5)
                if name_lower.contains("9950") || name_lower.contains("9900") 
                    || name_lower.contains("9700") || name_lower.contains("9600") {
                    return CpuArchitecture::AmdZen5;
                }
                // Ryzen 7000 series (Zen 4)
                if name_lower.contains("7950") || name_lower.contains("7900") 
                    || name_lower.contains("7700") || name_lower.contains("7600") {
                    return CpuArchitecture::AmdZen4;
                }
                // Ryzen 5000 series (Zen 3)
                if name_lower.contains("5950") || name_lower.contains("5900") 
                    || name_lower.contains("5800") || name_lower.contains("5700")
                    || name_lower.contains("5600") {
                    return CpuArchitecture::AmdZen3;
                }
                // Ryzen 3000 series (Zen 2)
                if name_lower.contains("3950") || name_lower.contains("3900") 
                    || name_lower.contains("3700") || name_lower.contains("3600")
                    || name_lower.contains("3300") {
                    return CpuArchitecture::AmdZen2;
                }
                // Ryzen 2000 series (Zen+, treat as Zen2)
                if name_lower.contains("2700") || name_lower.contains("2600")
                    || name_lower.contains("2400") || name_lower.contains("2200") {
                    return CpuArchitecture::AmdZen2;
                }
                // Ryzen 1000 series (Zen 1)
                if name_lower.contains("1800") || name_lower.contains("1700")
                    || name_lower.contains("1600") || name_lower.contains("1500")
                    || name_lower.contains("1400") {
                    return CpuArchitecture::AmdZen1;
                }
            }
            
            // EPYC detection
            if name_lower.contains("epyc") {
                if name_lower.contains('9') {
                    return CpuArchitecture::AmdZen4;
                }
                if name_lower.contains('7') {
                    return CpuArchitecture::AmdZen3;
                }
                return CpuArchitecture::AmdZen2;
            }
        }
        
        CpuVendor::Intel => {
            // 12th gen+ (Alder Lake, Raptor Lake, Meteor Lake)
            if name_lower.contains("12th") || name_lower.contains("13th") || name_lower.contains("14th")
                || name_lower.contains("i9-12") || name_lower.contains("i7-12") 
                || name_lower.contains("i9-13") || name_lower.contains("i7-13")
                || name_lower.contains("i9-14") || name_lower.contains("i7-14") {
                return CpuArchitecture::IntelAlderlake;
            }
            
            // 11th gen
            if name_lower.contains("rocket lake") {
                return CpuArchitecture::IntelRocketlake;
            }
            if name_lower.contains("i9-11") || name_lower.contains("i7-11") {
                if name_lower.contains('k') || name_lower.contains("desktop") {
                    return CpuArchitecture::IntelRocketlake;
                }
                return CpuArchitecture::IntelIcelake;
            }
            
            // 10th gen
            if name_lower.contains("ice lake") {
                return CpuArchitecture::IntelIcelake;
            }
            if name_lower.contains("i9-10") || name_lower.contains("i7-10") {
                if name_lower.contains("-g") || name_lower.contains("ice") {
                    return CpuArchitecture::IntelIcelake;
                }
                return CpuArchitecture::IntelSkylake;
            }
            
            // 6th-9th gen (Skylake derivatives)
            if name_lower.contains("6th") || name_lower.contains("7th") 
                || name_lower.contains("8th") || name_lower.contains("9th")
                || name_lower.contains("i9-9") || name_lower.contains("i7-9")
                || name_lower.contains("i7-8") || name_lower.contains("i7-7")
                || name_lower.contains("i7-6") {
                return CpuArchitecture::IntelSkylake;
            }
            
            // 5th gen
            if name_lower.contains("broadwell") || name_lower.contains("5th")
                || name_lower.contains("i7-5") {
                return CpuArchitecture::IntelBroadwell;
            }
            
            // 4th gen
            if name_lower.contains("haswell") || name_lower.contains("4th")
                || name_lower.contains("i7-4") {
                return CpuArchitecture::IntelHaswell;
            }
            
            // Xeon
            if name_lower.contains("xeon") {
                if name_lower.contains("platinum") || name_lower.contains("gold") {
                    return CpuArchitecture::IntelSkylake;
                }
                return CpuArchitecture::IntelHaswell;
            }
        }
        
        CpuVendor::Apple => {
            if name_lower.contains("m3") {
                return CpuArchitecture::AppleM3;
            }
            if name_lower.contains("m2") {
                return CpuArchitecture::AppleM2;
            }
            if name_lower.contains("m1") || name_lower.contains("apple") {
                return CpuArchitecture::AppleM1;
            }
        }
        
        _ => {}
    }
    
    CpuArchitecture::Portable
}

/// Refine architecture detection using CPUID family/model
pub(crate) fn refine_from_cpuid(
    initial: CpuArchitecture,
    vendor: CpuVendor,
    family: u32,
    model: u32,
) -> CpuArchitecture {
    match vendor {
        CpuVendor::Amd => {
            // AMD Family 25 (0x19) = Zen 3/4
            if family == 25 {
                if model >= 0x60 {
                    return CpuArchitecture::AmdZen4;
                }
                return CpuArchitecture::AmdZen3;
            }
            
            // AMD Family 23 (0x17) = Zen 1/2/+
            if family == 23 {
                if model >= 0x30 {
                    return CpuArchitecture::AmdZen2;
                }
                if model >= 0x10 {
                    return CpuArchitecture::AmdZen2; // Zen+
                }
                return CpuArchitecture::AmdZen1;
            }
            
            // AMD Family 26 (0x1A) = Zen 5
            if family == 26 {
                return CpuArchitecture::AmdZen5;
            }
        }
        
        CpuVendor::Intel => {
            // Intel Family 6 (modern Intel CPUs)
            if family == 6 {
                // Alder Lake and newer (12th gen+)
                if matches!(model, 0x97 | 0x9A | 0xB7 | 0xBA | 0xBF) {
                    return CpuArchitecture::IntelAlderlake;
                }
                
                // Rocket Lake (11th gen desktop)
                if model == 0xA7 {
                    return CpuArchitecture::IntelRocketlake;
                }
                
                // Ice Lake (10th/11th gen mobile)
                if matches!(model, 0x7D | 0x7E | 0x6A | 0x6C) {
                    return CpuArchitecture::IntelIcelake;
                }
                
                // Skylake derivatives (6th-9th gen)
                if matches!(model, 0x4E | 0x5E | 0x8E | 0x9E | 0xA5 | 0xA6) {
                    return CpuArchitecture::IntelSkylake;
                }
                
                // Broadwell (5th gen)
                if matches!(model, 0x3D | 0x47 | 0x4F | 0x56) {
                    return CpuArchitecture::IntelBroadwell;
                }
                
                // Haswell (4th gen)
                if matches!(model, 0x3C | 0x3F | 0x45 | 0x46) {
                    return CpuArchitecture::IntelHaswell;
                }
            }
        }
        
        _ => {}
    }
    
    initial
}

