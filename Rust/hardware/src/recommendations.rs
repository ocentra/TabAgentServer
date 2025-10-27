/*!
Hardware-based Recommendations

Provides recommendations for:
- BitNet DLL selection based on CPU microarchitecture
- Execution provider selection based on GPU availability
- Model loading strategies based on available memory
*/

use crate::cpu::CpuArchitecture;
use crate::gpu::GpuVendor;
use crate::constants::*;
use serde::{Deserialize, Serialize};

/// Get the BitNet DLL variant name for the detected CPU architecture
pub fn get_bitnet_dll_variant(arch: CpuArchitecture) -> &'static str {
    match arch {
        // AMD Zen architectures
        CpuArchitecture::AmdZen1 => ARCH_AMD_ZEN1,
        CpuArchitecture::AmdZen2 => ARCH_AMD_ZEN2,
        CpuArchitecture::AmdZen3 => ARCH_AMD_ZEN3,
        CpuArchitecture::AmdZen4 => ARCH_AMD_ZEN4,
        CpuArchitecture::AmdZen5 => ARCH_AMD_ZEN5,
        
        // Intel architectures
        CpuArchitecture::IntelHaswell => "intel-haswell",
        CpuArchitecture::IntelBroadwell => "intel-broadwell",
        CpuArchitecture::IntelSkylake => ARCH_INTEL_SKYLAKE,
        CpuArchitecture::IntelIcelake => ARCH_INTEL_ICELAKE,
        CpuArchitecture::IntelRocketlake => "intel-rocketlake",
        CpuArchitecture::IntelAlderlake => ARCH_INTEL_ALDERLAKE,
        
        // Apple Silicon
        CpuArchitecture::AppleM1 => ARCH_APPLE_M1,
        CpuArchitecture::AppleM2 => ARCH_APPLE_M2,
        CpuArchitecture::AppleM3 => ARCH_APPLE_M3,
        
        // ARM
        CpuArchitecture::ArmV8 => "arm-v8",
        CpuArchitecture::ArmV9 => "arm-v9",
        
        // Fallbacks
        CpuArchitecture::Portable => "portable",
        CpuArchitecture::Unknown => ARCH_UNKNOWN,
    }
}

/// Get the full BitNet DLL filename for Windows
pub fn get_bitnet_dll_filename(arch: CpuArchitecture) -> String {
    let variant = get_bitnet_dll_variant(arch);
    format!("{}-{}{}", BITNET_DLL_PREFIX, variant, BITNET_DLL_SUFFIX)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionProviderRecommendation {
    /// Primary recommended provider
    pub primary: String,
    
    /// Fallback providers in priority order
    pub fallbacks: Vec<String>,
    
    /// Reason for recommendation
    pub reason: String,
}

/// Recommend execution provider based on GPU availability
pub fn recommend_execution_provider(
    gpus: &[crate::gpu::GpuInfo],
    os_name: &str,
) -> ExecutionProviderRecommendation {
    // Check for NVIDIA GPU
    if gpus.iter().any(|gpu| gpu.vendor == GpuVendor::Nvidia) {
        return ExecutionProviderRecommendation {
            primary: PROVIDER_CUDA.to_string(),
            fallbacks: vec![
                PROVIDER_DIRECTML.to_string(),
                PROVIDER_CPU.to_string(),
            ],
            reason: "NVIDIA GPU detected, CUDA is optimal".to_string(),
        };
    }
    
    // Check for AMD GPU
    if gpus.iter().any(|gpu| gpu.vendor == GpuVendor::Amd) {
        let primary = if os_name.to_lowercase().contains(OS_LINUX) {
            PROVIDER_ROCM
        } else {
            PROVIDER_DIRECTML
        };
        
        return ExecutionProviderRecommendation {
            primary: primary.to_string(),
            fallbacks: vec![PROVIDER_CPU.to_string()],
            reason: format!("AMD GPU detected, {} is optimal", primary),
        };
    }
    
    // Check for Intel GPU
    if gpus.iter().any(|gpu| gpu.vendor == GpuVendor::Intel) {
        return ExecutionProviderRecommendation {
            primary: PROVIDER_OPENVINO.to_string(),
            fallbacks: vec![
                PROVIDER_DIRECTML.to_string(),
                PROVIDER_CPU.to_string(),
            ],
            reason: "Intel GPU detected, OpenVINO is optimal".to_string(),
        };
    }
    
    // Check for Apple Silicon
    if gpus.iter().any(|gpu| gpu.vendor == GpuVendor::Apple) {
        return ExecutionProviderRecommendation {
            primary: PROVIDER_COREML.to_string(),
            fallbacks: vec![PROVIDER_CPU.to_string()],
            reason: "Apple Silicon detected, CoreML is optimal".to_string(),
        };
    }
    
    // Fallback to CPU
    ExecutionProviderRecommendation {
        primary: PROVIDER_CPU.to_string(),
        fallbacks: vec![],
        reason: format!("No dedicated GPU detected, using {}", PROVIDER_CPU),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelLoadingStrategy {
    /// Where to load the model ("gpu", "cpu", "split")
    pub target: String,
    
    /// GPU index to use (if applicable)
    pub gpu_index: Option<usize>,
    
    /// Percentage of model on GPU (for split loading)
    pub gpu_percent: Option<f32>,
    
    /// Percentage of model on CPU (for split loading)
    pub cpu_percent: Option<f32>,
    
    /// Reason for strategy
    pub reason: String,
}

/// Recommend model loading strategy based on available memory
pub fn recommend_loading_strategy(
    model_size_mb: u64,
    _total_ram_mb: u64,
    available_ram_mb: u64,
    gpus: &[crate::gpu::GpuInfo],
) -> ModelLoadingStrategy {
    // Find GPU with most VRAM
    let best_gpu = gpus.iter()
        .enumerate()
        .filter_map(|(idx, gpu)| gpu.vram_mb.map(|vram| (idx, vram)))
        .max_by_key(|(_, vram)| *vram);
    
    if let Some((gpu_idx, vram_mb)) = best_gpu {
        // Check if model fits entirely on GPU (with 20% safety margin)
        let required_vram = model_size_mb + (model_size_mb / 5);
        
        if vram_mb >= required_vram {
            return ModelLoadingStrategy {
                target: LOAD_STRATEGY_GPU.to_string(),
                gpu_index: Some(gpu_idx),
                gpu_percent: Some(100.0),
                cpu_percent: Some(0.0),
                reason: format!("Model fits in GPU VRAM ({} MB available)", vram_mb),
            };
        }
        
        // Check if we can split between GPU and RAM
        let required_ram = model_size_mb + (model_size_mb / 10);
        if available_ram_mb >= required_ram / 2 {
            // Calculate split percentages
            let gpu_percent = (vram_mb as f32 / model_size_mb as f32) * 100.0;
            let gpu_percent = gpu_percent.min(100.0).max(0.0);
            let cpu_percent = 100.0 - gpu_percent;
            
            return ModelLoadingStrategy {
                target: LOAD_STRATEGY_SPLIT.to_string(),
                gpu_index: Some(gpu_idx),
                gpu_percent: Some(gpu_percent),
                cpu_percent: Some(cpu_percent),
                reason: format!(
                    "Model split between GPU ({:.0}%) and CPU ({:.0}%)",
                    gpu_percent, cpu_percent
                ),
            };
        }
    }
    
    // Fallback to CPU
    ModelLoadingStrategy {
        target: LOAD_STRATEGY_CPU.to_string(),
        gpu_index: None,
        gpu_percent: Some(0.0),
        cpu_percent: Some(100.0),
        reason: format!("Loading model on {}", PROVIDER_CPU),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitnet_dll_naming() {
        let filename = get_bitnet_dll_filename(CpuArchitecture::AmdZen3);
        assert_eq!(filename, "bitnet-amd-zen3.dll");
        
        let filename = get_bitnet_dll_filename(CpuArchitecture::IntelAlderlake);
        assert_eq!(filename, "bitnet-intel-alderlake.dll");
    }
    
    #[test]
    fn test_execution_provider_recommendation() {
        use crate::gpu::{GpuInfo, GpuVendor};
        
        let gpus = vec![GpuInfo {
            vendor: GpuVendor::Nvidia,
            name: "RTX 4090".to_string(),
            vram_mb: Some(24576),
            driver_version: Some("550.0".to_string()),
        }];
        
        let rec = recommend_execution_provider(&gpus, "Windows");
        assert_eq!(rec.primary, PROVIDER_CUDA);
    }
}

