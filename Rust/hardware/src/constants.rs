/*!
Hardware Detection Constants

Centralized constants for hardware detection to avoid string literals across crates.
*/

// ========== CPU Vendors ==========
pub const CPU_VENDOR_INTEL: &str = "Intel";
pub const CPU_VENDOR_AMD: &str = "AMD";
pub const CPU_VENDOR_APPLE: &str = "Apple";
pub const CPU_VENDOR_UNKNOWN: &str = "Unknown";

// ========== GPU Vendors ==========
pub const GPU_VENDOR_NVIDIA: &str = "NVIDIA";
pub const GPU_VENDOR_AMD: &str = "AMD";
pub const GPU_VENDOR_INTEL: &str = "Intel";
pub const GPU_VENDOR_APPLE: &str = "Apple";
pub const GPU_VENDOR_UNKNOWN: &str = "Unknown";

// ========== CPU Architecture Strings (for BitNet DLL selection) ==========
pub const ARCH_AMD_ZEN1: &str = "amd-zen1";
pub const ARCH_AMD_ZEN2: &str = "amd-zen2";
pub const ARCH_AMD_ZEN3: &str = "amd-zen3";
pub const ARCH_AMD_ZEN4: &str = "amd-zen4";
pub const ARCH_AMD_ZEN5: &str = "amd-zen5";

pub const ARCH_INTEL_SKYLAKE: &str = "intel-skylake";
pub const ARCH_INTEL_KABYLAKE: &str = "intel-kabylake";
pub const ARCH_INTEL_COFFEELAKE: &str = "intel-coffeelake";
pub const ARCH_INTEL_COMETLAKE: &str = "intel-cometlake";
pub const ARCH_INTEL_ICELAKE: &str = "intel-icelake";
pub const ARCH_INTEL_TIGERLAKE: &str = "intel-tigerlake";
pub const ARCH_INTEL_ALDERLAKE: &str = "intel-alderlake";
pub const ARCH_INTEL_RAPTORLAKE: &str = "intel-raptorlake";
pub const ARCH_INTEL_METEORLAKE: &str = "intel-meteorlake";

pub const ARCH_APPLE_M1: &str = "apple-m1";
pub const ARCH_APPLE_M2: &str = "apple-m2";
pub const ARCH_APPLE_M3: &str = "apple-m3";
pub const ARCH_APPLE_M4: &str = "apple-m4";

pub const ARCH_GENERIC_X86_64: &str = "generic-x86_64";
pub const ARCH_GENERIC_ARM64: &str = "generic-arm64";
pub const ARCH_UNKNOWN: &str = "unknown";

// ========== Execution Provider Strings ==========
pub const PROVIDER_CUDA: &str = "CUDA";
pub const PROVIDER_DIRECTML: &str = "DirectML";
pub const PROVIDER_COREML: &str = "CoreML";
pub const PROVIDER_ROCM: &str = "ROCm";
pub const PROVIDER_OPENVINO: &str = "OpenVINO";
pub const PROVIDER_CPU: &str = "CPU";

// ========== Detection Command Names ==========
pub const CMD_NVIDIA_SMI: &str = "nvidia-smi";
pub const CMD_ROCM_SMI: &str = "rocm-smi";
pub const CMD_WMIC: &str = "wmic";
pub const CMD_LSPCI: &str = "lspci";
pub const CMD_SYSTEM_PROFILER: &str = "system_profiler";
pub const CMD_SYSCTL: &str = "sysctl";
pub const CMD_POWERSHELL: &str = "powershell";

// ========== Memory Thresholds (MB) ==========
pub const LOW_VRAM_THRESHOLD_MB: u64 = 4096;      // < 4GB = low VRAM
pub const MEDIUM_VRAM_THRESHOLD_MB: u64 = 8192;   // 4-8GB = medium VRAM
pub const HIGH_VRAM_THRESHOLD_MB: u64 = 16384;    // 8-16GB = high VRAM
                                                    // > 16GB = very high VRAM

pub const LOW_RAM_THRESHOLD_MB: u64 = 8192;       // < 8GB = low RAM
pub const MEDIUM_RAM_THRESHOLD_MB: u64 = 16384;   // 8-16GB = medium RAM
pub const HIGH_RAM_THRESHOLD_MB: u64 = 32768;     // 16-32GB = high RAM
                                                    // > 32GB = very high RAM

// ========== BitNet DLL Naming ==========
pub const BITNET_DLL_PREFIX: &str = "bitnet";
pub const BITNET_DLL_SUFFIX: &str = ".dll";

// ========== OS Names ==========
pub const OS_WINDOWS: &str = "windows";
pub const OS_LINUX: &str = "linux";
pub const OS_MACOS: &str = "macos";
pub const OS_DARWIN: &str = "darwin";

// ========== Loading Strategies ==========
pub const LOAD_STRATEGY_GPU: &str = "gpu";
pub const LOAD_STRATEGY_CPU: &str = "cpu";
pub const LOAD_STRATEGY_SPLIT: &str = "split";

// ========== Memory Tiers ==========
pub const TIER_LOW: &str = "low";
pub const TIER_MEDIUM: &str = "medium";
pub const TIER_HIGH: &str = "high";
pub const TIER_VERY_HIGH: &str = "very_high";

// ========== GPU Keywords for Classification ==========
// NVIDIA
pub const GPU_KEYWORD_NVIDIA: &str = "nvidia";
pub const GPU_KEYWORD_GEFORCE: &str = "geforce";
pub const GPU_KEYWORD_RTX: &str = "rtx";
pub const GPU_KEYWORD_GTX: &str = "gtx";
pub const GPU_KEYWORD_QUADRO: &str = "quadro";
pub const GPU_KEYWORD_TESLA: &str = "tesla";

// AMD
pub const GPU_KEYWORD_AMD: &str = "amd";
pub const GPU_KEYWORD_ATI: &str = "ati";
pub const GPU_KEYWORD_RADEON: &str = "radeon";
pub const GPU_KEYWORD_RX: &str = "rx";
pub const GPU_KEYWORD_VEGA: &str = "vega";
pub const GPU_KEYWORD_RYZEN: &str = "ryzen";

// Intel
pub const GPU_KEYWORD_INTEL: &str = "intel";
pub const GPU_KEYWORD_IRIS: &str = "iris";
pub const GPU_KEYWORD_UHD: &str = "uhd";
pub const GPU_KEYWORD_ARC: &str = "arc";
pub const GPU_KEYWORD_HD_GRAPHICS: &str = "hd graphics";

// Apple
pub const GPU_KEYWORD_APPLE: &str = "apple";
pub const GPU_KEYWORD_M1: &str = "m1";
pub const GPU_KEYWORD_M2: &str = "m2";
pub const GPU_KEYWORD_M3: &str = "m3";
pub const GPU_KEYWORD_M4: &str = "m4";

// ========== CPU Keywords ==========
pub const CPU_KEYWORD_INTEL: &str = "intel";
pub const CPU_KEYWORD_AMD: &str = "amd";
pub const CPU_KEYWORD_AUTHENTICAMD: &str = "authenticamd";
pub const CPU_KEYWORD_GENUINEINTEL: &str = "genuineintel";

// ========== Detection Keywords ==========
pub const KEYWORD_BASIC_DISPLAY: &str = "basic display";
pub const KEYWORD_MICROSOFT_BASIC: &str = "microsoft basic";
pub const KEYWORD_VGA: &str = "vga";
pub const KEYWORD_3D: &str = "3d";

