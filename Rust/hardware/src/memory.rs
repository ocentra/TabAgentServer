/*!
Memory Detection (RAM and VRAM)
*/

use serde::{Deserialize, Serialize};
use crate::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    /// Total system RAM in MB
    pub total_ram_mb: u64,
    
    /// Available (free) system RAM in MB
    pub available_ram_mb: u64,
    
    /// Used system RAM in MB
    pub used_ram_mb: u64,
}

/// Detect system memory (RAM) using sysinfo crate
pub fn detect_memory() -> Result<MemoryInfo> {
    use sysinfo::System;
    
    let mut sys = System::new_all();
    sys.refresh_memory();
    
    let total_ram_mb = sys.total_memory() / 1024 / 1024;
    let available_ram_mb = sys.available_memory() / 1024 / 1024;
    let used_ram_mb = sys.used_memory() / 1024 / 1024;
    
    Ok(MemoryInfo {
        total_ram_mb,
        available_ram_mb,
        used_ram_mb,
    })
}

/// Calculate total VRAM across all GPUs
pub fn calculate_total_vram(gpus: &[crate::gpu::GpuInfo]) -> u64 {
    gpus.iter()
        .filter_map(|gpu| gpu.vram_mb)
        .sum()
}

/// Get memory tier (low/medium/high/very high)
pub fn get_ram_tier(total_ram_mb: u64) -> &'static str {
    use crate::constants::*;
    
    if total_ram_mb < LOW_RAM_THRESHOLD_MB {
        TIER_LOW
    } else if total_ram_mb < MEDIUM_RAM_THRESHOLD_MB {
        TIER_MEDIUM
    } else if total_ram_mb < HIGH_RAM_THRESHOLD_MB {
        TIER_HIGH
    } else {
        TIER_VERY_HIGH
    }
}

/// Get VRAM tier (low/medium/high/very high)
pub fn get_vram_tier(total_vram_mb: u64) -> &'static str {
    use crate::constants::*;
    
    if total_vram_mb < LOW_VRAM_THRESHOLD_MB {
        TIER_LOW
    } else if total_vram_mb < MEDIUM_VRAM_THRESHOLD_MB {
        TIER_MEDIUM
    } else if total_vram_mb < HIGH_VRAM_THRESHOLD_MB {
        TIER_HIGH
    } else {
        TIER_VERY_HIGH
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_memory() {
        let mem = detect_memory().unwrap();
        println!("Memory: {:#?}", mem);
        
        assert!(mem.total_ram_mb > 0);
        assert!(mem.available_ram_mb <= mem.total_ram_mb);
    }
    
    #[test]
    fn test_ram_tiers() {
        use crate::constants::*;
        assert_eq!(get_ram_tier(4096), TIER_LOW);
        assert_eq!(get_ram_tier(12288), TIER_MEDIUM);
        assert_eq!(get_ram_tier(24576), TIER_HIGH);
        assert_eq!(get_ram_tier(65536), TIER_VERY_HIGH);
    }
}

