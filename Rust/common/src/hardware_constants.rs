//! Hardware-related constants
//!
//! Platform names, vendor strings, and hardware detection constants
//! used across the codebase for consistency.

/// Operating system names
pub mod os {
    pub const WINDOWS: &str = "windows";
    pub const LINUX: &str = "linux";
    pub const MACOS: &str = "macos";
    pub const FREEBSD: &str = "freebsd";
    pub const ANDROID: &str = "android";
    pub const IOS: &str = "ios";
}

/// CPU architecture names
pub mod arch {
    pub const X86_64: &str = "x86_64";
    pub const AARCH64: &str = "aarch64";
    pub const ARM64: &str = "arm64";
    pub const ARM: &str = "arm";
    pub const X86: &str = "x86";
}

/// GPU vendor names (for string matching)
pub mod gpu_vendor {
    pub const NVIDIA: &str = "nvidia";
    pub const GEFORCE: &str = "geforce";
    pub const RTX: &str = "rtx";
    pub const GTX: &str = "gtx";
    pub const QUADRO: &str = "quadro";
    pub const TESLA: &str = "tesla";
    
    pub const AMD: &str = "amd";
    pub const RADEON: &str = "radeon";
    pub const RX: &str = "rx ";
    pub const VEGA: &str = "vega";
    
    pub const INTEL: &str = "intel";
    pub const ARC: &str = "arc";
    pub const IRIS: &str = "iris";
    pub const UHD: &str = "uhd";
    pub const UHD_GRAPHICS: &str = "uhd graphics";
    
    pub const APPLE: &str = "apple";
    pub const M1: &str = "m1";
    pub const M2: &str = "m2";
    pub const M3: &str = "m3";
}

/// CPU vendor names
pub mod cpu_vendor {
    pub const INTEL: &str = "intel";
    pub const GENUINE_INTEL: &str = "genuineintel";
    
    pub const AMD: &str = "amd";
    pub const AUTHENTIC_AMD: &str = "authenticamd";
    
    pub const APPLE: &str = "apple";
    pub const ARM: &str = "arm";
}

/// CPU microarchitecture names
pub mod cpu_arch {
    // AMD Ryzen generations
    pub const RYZEN: &str = "ryzen";
    pub const EPYC: &str = "epyc";
    
    // Intel generations
    pub const XEON: &str = "xeon";
    pub const CORE: &str = "core";
    pub const PENTIUM: &str = "pentium";
    pub const CELERON: &str = "celeron";
    
    // Apple Silicon
    pub const M1: &str = "m1";
    pub const M2: &str = "m2";
    pub const M3: &str = "m3";
}

/// PCI Vendor IDs (hex strings)
pub mod pci_vendor_id {
    pub const NVIDIA: &str = "10de";
    pub const AMD: &str = "1002";
    pub const INTEL: &str = "8086";
}

/// System command names
pub mod commands {
    // Windows
    pub const POWERSHELL: &str = "powershell";
    pub const WMIC: &str = "wmic";
    
    // Linux
    pub const LSPCI: &str = "lspci";
    pub const NVIDIA_SMI: &str = "nvidia-smi";
    pub const ROCM_SMI: &str = "rocm-smi";
    
    // macOS
    pub const SYSCTL: &str = "sysctl";
    pub const SYSTEM_PROFILER: &str = "system_profiler";
}

/// Unit conversion constants
pub mod units {
    pub const BYTES_PER_KB: u64 = 1024;
    pub const BYTES_PER_MB: u64 = 1024 * 1024;
    pub const BYTES_PER_GB: u64 = 1024 * 1024 * 1024;
    
    pub const MB_PER_GB: u64 = 1024;
    pub const KB_PER_MB: u64 = 1024;
}

/// Memory thresholds for decision-making
pub mod memory_thresholds {
    /// Minimum free RAM to consider loading a model (2 GB)
    pub const MIN_FREE_RAM_MB: u64 = 2048;
    
    /// Minimum free VRAM to consider loading a model on GPU (1 GB)
    pub const MIN_FREE_VRAM_MB: u64 = 1024;
    
    /// Safety margin for memory calculations (500 MB)
    pub const MEMORY_SAFETY_MARGIN_MB: u64 = 500;
    
    /// Percentage of total memory to reserve for system (20%)
    pub const SYSTEM_MEMORY_RESERVE_PERCENT: f64 = 0.20;
}

/// Model size estimates (in MB)
pub mod model_sizes {
    /// Typical embedding model (all-MiniLM-L6-v2)
    pub const SMALL_EMBEDDING: u64 = 90;
    
    /// Small language model (Phi-2)
    pub const SMALL_LLM: u64 = 2700;
    
    /// Medium language model (Mistral-7B)
    pub const MEDIUM_LLM: u64 = 7000;
    
    /// Large language model (Llama-13B)
    pub const LARGE_LLM: u64 = 13000;
    
    /// Very large language model (Llama-70B)
    pub const XLARGE_LLM: u64 = 70000;
}

