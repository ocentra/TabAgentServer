//! System resource tracking and management
//! 
//! Monitors VRAM, RAM, and provides smart GPU/CPU split recommendations

use serde::{Deserialize, Serialize};
use tabagent_hardware::{detect_system, SystemInfo};

/// System resources snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResources {
    /// GPU information
    pub gpu: Option<GpuResources>,
    
    /// CPU information
    pub cpu: CpuResources,
    
    /// Total system RAM
    pub total_ram: u64,
    
    /// Available RAM
    pub available_ram: u64,
    
    /// Used RAM (by loaded models)
    pub used_ram: u64,
}

/// GPU resource information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuResources {
    /// GPU available
    pub available: bool,
    
    /// GPU name/model
    pub name: String,
    
    /// Total VRAM in bytes
    pub total_vram: u64,
    
    /// Available VRAM in bytes
    pub available_vram: u64,
    
    /// Used VRAM (by loaded models)
    pub used_vram: u64,
    
    /// GPU vendor
    pub vendor: String,
}

/// CPU resource information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuResources {
    /// CPU architecture (e.g., "AmdZen2", "IntelAlderlake")
    pub architecture: String,
    
    /// CPU model name
    pub model_name: String,
    
    /// Number of cores
    pub cores: u32,
    
    /// Number of threads
    pub threads: u32,
}

/// Smart split recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitRecommendation {
    /// Recommended GPU layers
    pub gpu_layers: u32,
    
    /// Recommended CPU layers
    pub cpu_layers: u32,
    
    /// Estimated VRAM usage
    pub estimated_vram: u64,
    
    /// Estimated RAM usage
    pub estimated_ram: u64,
    
    /// Expected performance tier
    pub performance_tier: PerformanceTier,
    
    /// Reasoning for recommendation
    pub reason: String,
}

/// Performance expectation tier
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PerformanceTier {
    /// Fast inference (mostly GPU)
    Fast,
    
    /// Medium speed (balanced split)
    Medium,
    
    /// Slower inference (mostly/all CPU)
    Slow,
}

/// Get current system resources
pub fn get_system_resources() -> Result<SystemResources, String> {
    // Detect hardware
    let system_info = detect_system()
        .map_err(|e| format!("Failed to detect system: {}", e))?;
    
    // Get RAM information
    let (total_ram, available_ram) = get_ram_info()?;
    
    // Calculate used RAM from loaded models
    let used_ram = calculate_used_ram();
    
    // Get GPU resources if available
    let gpu = get_gpu_resources(&system_info);
    
    // Build CPU resources
    let cpu = CpuResources {
        architecture: format!("{:?}", system_info.cpu.architecture),
        model_name: system_info.cpu.model_name.clone(),
        cores: system_info.cpu.cores,
        threads: system_info.cpu.threads,
    };
    
    Ok(SystemResources {
        gpu,
        cpu,
        total_ram,
        available_ram,
        used_ram,
    })
}

/// Get RAM information from the system
fn get_ram_info() -> Result<(u64, u64), String> {
    #[cfg(target_os = "windows")]
    {
        // Use Windows API via psutil-style approach
        // For now, return estimated values
        // TODO: Implement proper Windows RAM detection
        let total = 16 * 1024 * 1024 * 1024; // 16GB default
        let available = 12 * 1024 * 1024 * 1024; // 12GB available
        Ok((total, available))
    }
    
    #[cfg(target_os = "linux")]
    {
        // Read from /proc/meminfo
        let meminfo = fs::read_to_string("/proc/meminfo")
            .map_err(|e| format!("Failed to read /proc/meminfo: {}", e))?;
        
        let mut total = 0u64;
        let mut available = 0u64;
        
        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                total = parse_meminfo_line(line);
            } else if line.starts_with("MemAvailable:") {
                available = parse_meminfo_line(line);
            }
        }
        
        Ok((total, available))
    }
    
    #[cfg(target_os = "macos")]
    {
        // Use sysctl for macOS
        // For now, return estimated values
        // TODO: Implement proper macOS RAM detection
        let total = 16 * 1024 * 1024 * 1024; // 16GB default
        let available = 12 * 1024 * 1024 * 1024; // 12GB available
        Ok((total, available))
    }
}

#[cfg(target_os = "linux")]
fn parse_meminfo_line(line: &str) -> u64 {
    // Parse line like "MemTotal:       16384000 kB"
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 2 {
        if let Ok(kb) = parts[1].parse::<u64>() {
            return kb * 1024; // Convert KB to bytes
        }
    }
    0
}

/// Get GPU resources from system info
fn get_gpu_resources(system_info: &SystemInfo) -> Option<GpuResources> {
    if system_info.gpus.is_empty() {
        return None;
    }
    
    let gpu = &system_info.gpus[0]; // Use first GPU
    
    // Calculate used VRAM from loaded models
    let used_vram = calculate_used_vram();
    
    // Estimate total and available VRAM based on GPU vendor
    let (total_vram, available_vram) = estimate_vram(gpu, used_vram);
    
    Some(GpuResources {
        available: true,
        name: gpu.name.clone(),
        total_vram,
        available_vram,
        used_vram,
        vendor: format!("{:?}", gpu.vendor),
    })
}

/// Estimate VRAM based on GPU info
fn estimate_vram(gpu: &tabagent_hardware::GpuInfo, used: u64) -> (u64, u64) {
    // TODO: Implement proper VRAM detection via GPU APIs
    // For now, estimate based on GPU name patterns
    
    let total: u64 = if gpu.name.contains("4090") || gpu.name.contains("4080") {
        24 * 1024 * 1024 * 1024 // 24GB
    } else if gpu.name.contains("3090") || gpu.name.contains("3080") {
        12 * 1024 * 1024 * 1024 // 12GB
    } else if gpu.name.contains("7900") {
        16 * 1024 * 1024 * 1024 // 16GB
    } else {
        8 * 1024 * 1024 * 1024 // 8GB default
    };
    
    let available = total.saturating_sub(used);
    (total, available)
}

/// Calculate RAM used by loaded models
fn calculate_used_ram() -> u64 {
    let models = crate::state::get_loaded_models();
    models.iter().map(|m| m.ram_used).sum()
}

/// Calculate VRAM used by loaded models
fn calculate_used_vram() -> u64 {
    let models = crate::state::get_loaded_models();
    models.iter().map(|m| m.vram_used).sum()
}

/// Recommend GPU/CPU split for a model
pub fn recommend_split(
    model_size_bytes: u64,
    total_layers: u32,
) -> Result<SplitRecommendation, String> {
    let resources = get_system_resources()?;
    
    // If no GPU, use CPU only
    if resources.gpu.is_none() {
        return Ok(SplitRecommendation {
            gpu_layers: 0,
            cpu_layers: total_layers,
            estimated_vram: 0,
            estimated_ram: model_size_bytes,
            performance_tier: PerformanceTier::Slow,
            reason: "No GPU available - using CPU only".to_string(),
        });
    }
    
    let gpu = resources.gpu.as_ref()
        .expect("GPU should be Some here - checked above in if statement");
    
    // Estimate memory per layer
    let mem_per_layer = model_size_bytes / total_layers as u64;
    
    // Calculate how many layers fit in available VRAM (leave 1GB buffer)
    let vram_buffer = 1024 * 1024 * 1024; // 1GB
    let available_for_model = gpu.available_vram.saturating_sub(vram_buffer);
    let layers_fit_in_vram = (available_for_model / mem_per_layer) as u32;
    
    // Clamp to total layers
    let gpu_layers = layers_fit_in_vram.min(total_layers);
    let cpu_layers = total_layers - gpu_layers;
    
    // Determine performance tier
    let gpu_percentage = (gpu_layers as f64 / total_layers as f64) * 100.0;
    let (performance_tier, reason) = if gpu_percentage >= 90.0 {
        (
            PerformanceTier::Fast,
            format!("{}% of layers on GPU - excellent performance expected", gpu_percentage as u32),
        )
    } else if gpu_percentage >= 50.0 {
        (
            PerformanceTier::Medium,
            format!("{}% of layers on GPU - good performance expected", gpu_percentage as u32),
        )
    } else if gpu_percentage > 0.0 {
        (
            PerformanceTier::Slow,
            format!("{}% of layers on GPU - limited performance", gpu_percentage as u32),
        )
    } else {
        (
            PerformanceTier::Slow,
            "All layers on CPU - slow performance expected".to_string(),
        )
    };
    
    // Calculate estimated memory usage
    let estimated_vram = gpu_layers as u64 * mem_per_layer;
    let estimated_ram = cpu_layers as u64 * mem_per_layer;
    
    Ok(SplitRecommendation {
        gpu_layers,
        cpu_layers,
        estimated_vram,
        estimated_ram,
        performance_tier,
        reason,
    })
}

