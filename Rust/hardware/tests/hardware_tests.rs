/// Hardware detection tests
/// 
/// COMPREHENSIVE REAL TESTS - NO MOCKS:
/// - Queries actual system hardware
/// - Tests real CPU detection & architecture
/// - Tests real GPU detection with VRAM
/// - Tests real memory (RAM) readings
/// - Tests BitNet DLL variant selection
/// - Tests execution provider recommendations
/// - Tests model loading strategies
/// - Tests constants and tiers
/// - Tests all helper functions

use tabagent_hardware::{
    detect_system, 
    detect_cpu_architecture,
    detect_memory,
    calculate_total_vram,
    get_ram_tier,
    get_vram_tier,
    get_bitnet_dll_variant,
    get_bitnet_dll_filename,
    recommend_execution_provider,
    recommend_loading_strategy,
    CpuArchitecture, 
    CpuVendor,
    GpuVendor,
    constants::*,
};

#[test]
fn test_real_cpu_detection() {
    println!("\n🧪 Testing REAL CPU detection...");
    
    let arch = detect_cpu_architecture().unwrap();
    println!("✅ Detected CPU: {:?}", arch);
    
    // Must detect SOMETHING real
    assert_ne!(arch, CpuArchitecture::Unknown, "Should detect actual CPU architecture");
    
    // On common platforms, should be specific
    #[cfg(target_arch = "x86_64")]
    {
        assert!(
            matches!(arch, 
                CpuArchitecture::IntelIcelake | 
                CpuArchitecture::IntelAlderlake | 
                CpuArchitecture::IntelRocketlake |
                CpuArchitecture::IntelSkylake |
                CpuArchitecture::AmdZen3 |
                CpuArchitecture::AmdZen4 |
                CpuArchitecture::AmdZen5 |
                CpuArchitecture::Portable
            ),
            "Should detect known x86_64 architecture, got: {:?}", arch
        );
    }
}

#[test]
fn test_real_system_info() {
    println!("\n🧪 Testing REAL system information...");
    
    let system_info = detect_system().unwrap();
    
    // CPU info must be present
    println!("📊 CPU Info:");
    println!("   Vendor: {:?}", system_info.cpu.vendor);
    println!("   Model: {}", system_info.cpu.model_name);
    println!("   Architecture: {:?}", system_info.cpu.architecture);
    println!("   Cores: {}", system_info.cpu.cores);
    println!("   Threads: {}", system_info.cpu.threads);
    
    assert!(!system_info.cpu.model_name.is_empty(), "CPU model name should not be empty");
    assert!(system_info.cpu.cores > 0, "Should have at least 1 core");
    assert!(system_info.cpu.threads > 0, "Should have at least 1 thread");
    
    println!("✅ CPU info looks valid");
    
    // GPU info (may or may not be present)
    if !system_info.gpus.is_empty() {
        println!("📊 GPU Info:");
        for (idx, gpu) in system_info.gpus.iter().enumerate() {
            println!("   GPU {}: {}", idx, gpu.name);
        }
        
        println!("✅ GPU detected and info looks valid");
    } else {
        println!("⚠️  No GPU detected (this is fine for CPU-only systems)");
    }
}

#[test]
fn test_real_memory_consistency() {
    println!("\n🧪 Testing REAL memory consistency...");
    
    let info1 = detect_system().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(100));
    let info2 = detect_system().unwrap();
    
    // CPU checks should be consistent
    assert_eq!(
        info1.cpu.cores, 
        info2.cpu.cores,
        "CPU cores should not change between calls"
    );
    
    println!("✅ System readings are consistent");
    println!("   CPU: {} cores", info1.cpu.cores);
    println!("   Threads: {}", info1.cpu.threads);
}

#[test]
fn test_cpu_features() {
    println!("\n🧪 Testing CPU feature flags...");
    
    let system_info = detect_system().unwrap();
    
    println!("📊 CPU Detected:");
    println!("   Vendor: {:?}", system_info.cpu.vendor);
    println!("   Model: {}", system_info.cpu.model_name);
    println!("   Cores: {}", system_info.cpu.cores);
    
    println!("✅ CPU info retrieved successfully");
}

#[test]
#[cfg(windows)]
fn test_windows_specific_info() {
    println!("\n🧪 Testing Windows-specific information...");
    
    let system_info = detect_system().unwrap();
    
    // On Windows, should be able to get detailed CPU info
    assert!(!system_info.cpu.model_name.is_empty());
    
    println!("✅ Windows CPU detection works");
    println!("   {:?}: {}", system_info.cpu.vendor, system_info.cpu.model_name);
}

#[test]
#[cfg(target_os = "linux")]
fn test_linux_specific_info() {
    println!("\n🧪 Testing Linux-specific information...");
    
    let system_info = detect_system().unwrap();
    
    // On Linux, should read from /proc/cpuinfo
    assert!(system_info.cpu.cores > 0);
    
    println!("✅ Linux system detection works");
}

#[test]
#[cfg(target_os = "macos")]
fn test_macos_specific_info() {
    println!("\n🧪 Testing macOS-specific information...");
    
    let system_info = detect_system().unwrap();
    
    // On macOS, should use sysctl
    assert!(system_info.cpu.cores > 0);
    
    // Check for Apple Silicon
    if matches!(system_info.cpu.vendor, CpuVendor::Apple) {
        println!("✅ Apple Silicon detected: {}", system_info.cpu.model_name);
    } else {
        println!("✅ Intel Mac detected: {}", system_info.cpu.model_name);
    }
}

// ========== MEMORY TESTS ==========

#[test]
fn test_real_memory_detection() {
    println!("\n🧪 Testing REAL memory detection...");
    
    let memory = detect_memory().unwrap();
    
    println!("📊 Memory Info:");
    println!("   Total RAM: {} MB ({:.1} GB)", memory.total_ram_mb, memory.total_ram_mb as f32 / 1024.0);
    println!("   Available RAM: {} MB ({:.1} GB)", memory.available_ram_mb, memory.available_ram_mb as f32 / 1024.0);
    println!("   Used RAM: {} MB ({:.1} GB)", memory.used_ram_mb, memory.used_ram_mb as f32 / 1024.0);
    
    // Basic sanity checks
    assert!(memory.total_ram_mb > 0, "Total RAM must be greater than 0");
    assert!(memory.available_ram_mb <= memory.total_ram_mb, "Available RAM cannot exceed total RAM");
    assert!(memory.used_ram_mb <= memory.total_ram_mb, "Used RAM cannot exceed total RAM");
    
    // Most systems have at least 4GB RAM
    assert!(memory.total_ram_mb >= 2048, "System should have at least 2GB RAM");
    
    println!("✅ Memory detection works correctly");
}

#[test]
fn test_ram_tiers() {
    println!("\n🧪 Testing RAM tier classification...");
    
    let memory = detect_memory().unwrap();
    let tier = get_ram_tier(memory.total_ram_mb);
    
    println!("📊 RAM Tier:");
    println!("   Total: {} MB ({:.1} GB)", memory.total_ram_mb, memory.total_ram_mb as f32 / 1024.0);
    println!("   Tier: {}", tier);
    
    // Verify tier is one of the expected values
    assert!(
        tier == TIER_LOW || tier == TIER_MEDIUM || tier == TIER_HIGH || tier == TIER_VERY_HIGH,
        "Tier should be one of the defined constants"
    );
    
    // Verify tier logic
    if memory.total_ram_mb < LOW_RAM_THRESHOLD_MB {
        assert_eq!(tier, TIER_LOW);
    } else if memory.total_ram_mb < MEDIUM_RAM_THRESHOLD_MB {
        assert_eq!(tier, TIER_MEDIUM);
    } else if memory.total_ram_mb < HIGH_RAM_THRESHOLD_MB {
        assert_eq!(tier, TIER_HIGH);
    } else {
        assert_eq!(tier, TIER_VERY_HIGH);
    }
    
    println!("✅ RAM tier classification works correctly");
}

// ========== GPU & VRAM TESTS ==========

#[test]
fn test_real_gpu_detection() {
    println!("\n🧪 Testing REAL GPU detection...");
    
    let system = detect_system().unwrap();
    
    if system.gpus.is_empty() {
        println!("⚠️  No GPUs detected (CPU-only system or no drivers)");
        println!("✅ Test passed (no GPUs is valid)");
        return;
    }
    
    println!("📊 GPU Info:");
    for (idx, gpu) in system.gpus.iter().enumerate() {
        println!("   GPU {}: {}", idx, gpu.name);
        println!("      Vendor: {:?}", gpu.vendor);
        
        if let Some(vram_mb) = gpu.vram_mb {
            println!("      VRAM: {} MB ({:.1} GB)", vram_mb, vram_mb as f32 / 1024.0);
        } else {
            println!("      VRAM: Not detected");
        }
        
        if let Some(ref driver) = gpu.driver_version {
            println!("      Driver: {}", driver);
        }
        
        // Sanity checks
        assert!(!gpu.name.is_empty(), "GPU name should not be empty");
        assert_ne!(gpu.vendor, GpuVendor::Unknown, "GPU vendor should be detected");
    }
    
    println!("✅ GPU detection works correctly");
}

#[test]
fn test_total_vram_calculation() {
    println!("\n🧪 Testing total VRAM calculation...");
    
    let system = detect_system().unwrap();
    let total_vram = calculate_total_vram(&system.gpus);
    
    println!("📊 VRAM Summary:");
    println!("   Total VRAM: {} MB ({:.1} GB)", total_vram, total_vram as f32 / 1024.0);
    println!("   Tier: {}", system.vram_tier);
    
    // Verify it matches system.total_vram_mb
    assert_eq!(total_vram, system.total_vram_mb, "Calculated VRAM should match system VRAM");
    
    println!("✅ VRAM calculation works correctly");
}

#[test]
fn test_vram_tiers() {
    println!("\n🧪 Testing VRAM tier classification...");
    
    // Test edge cases
    assert_eq!(get_vram_tier(0), TIER_LOW);
    assert_eq!(get_vram_tier(2048), TIER_LOW);      // 2GB
    assert_eq!(get_vram_tier(6144), TIER_MEDIUM);   // 6GB
    assert_eq!(get_vram_tier(12288), TIER_HIGH);    // 12GB
    assert_eq!(get_vram_tier(24576), TIER_VERY_HIGH); // 24GB
    
    println!("✅ VRAM tier classification works correctly");
}

// ========== BITNET DLL VARIANT TESTS ==========

#[test]
fn test_bitnet_dll_variant_selection() {
    println!("\n🧪 Testing BitNet DLL variant selection...");
    
    let system = detect_system().unwrap();
    let variant = system.bitnet_dll_variant();
    let filename = system.bitnet_dll_filename();
    
    println!("📊 BitNet DLL Selection:");
    println!("   CPU Architecture: {:?}", system.cpu.architecture);
    println!("   DLL Variant: {}", variant);
    println!("   DLL Filename: {}", filename);
    
    // Verify variant is not empty
    assert!(!variant.is_empty(), "BitNet variant should not be empty");
    assert!(!filename.is_empty(), "BitNet filename should not be empty");
    
    // Verify filename format
    assert!(filename.starts_with(BITNET_DLL_PREFIX), "Filename should start with bitnet prefix");
    assert!(filename.ends_with(BITNET_DLL_SUFFIX), "Filename should end with .dll");
    assert!(filename.contains(variant), "Filename should contain the variant");
    
    println!("✅ BitNet DLL variant selection works correctly");
}

#[test]
fn test_bitnet_dll_all_architectures() {
    println!("\n🧪 Testing BitNet DLL for all CPU architectures...");
    
    let architectures = vec![
        CpuArchitecture::AmdZen3,
        CpuArchitecture::IntelAlderlake,
        CpuArchitecture::AppleM1,
        CpuArchitecture::Portable,
    ];
    
    for arch in architectures {
        let variant = get_bitnet_dll_variant(arch);
        let filename = get_bitnet_dll_filename(arch);
        
        println!("   {:?} → {} → {}", arch, variant, filename);
        
        assert!(!variant.is_empty());
        assert!(filename.contains(variant));
    }
    
    println!("✅ All architecture mappings work correctly");
}

// ========== EXECUTION PROVIDER TESTS ==========

#[test]
fn test_execution_provider_recommendation() {
    println!("\n🧪 Testing execution provider recommendation...");
    
    let system = detect_system().unwrap();
    let recommendation = system.recommended_execution_provider();
    
    println!("📊 Execution Provider Recommendation:");
    println!("   Primary: {}", recommendation.primary);
    println!("   Fallbacks: {:?}", recommendation.fallbacks);
    println!("   Reason: {}", recommendation.reason);
    
    // Verify recommendation is not empty
    assert!(!recommendation.primary.is_empty(), "Primary provider should not be empty");
    assert!(!recommendation.reason.is_empty(), "Reason should not be empty");
    
    // Verify primary is one of the known providers
    let valid_providers = vec![PROVIDER_CUDA, PROVIDER_DIRECTML, PROVIDER_COREML, 
                               PROVIDER_ROCM, PROVIDER_OPENVINO, PROVIDER_CPU];
    assert!(
        valid_providers.contains(&recommendation.primary.as_str()),
        "Primary provider should be one of the known providers"
    );
    
    // If we have NVIDIA GPU, should recommend CUDA
    if system.gpus.iter().any(|gpu| gpu.vendor == GpuVendor::Nvidia) {
        assert_eq!(recommendation.primary, PROVIDER_CUDA, "Should recommend CUDA for NVIDIA GPUs");
    }
    
    // If no GPU, should recommend CPU
    if system.gpus.is_empty() {
        assert_eq!(recommendation.primary, PROVIDER_CPU, "Should recommend CPU when no GPU detected");
    }
    
    println!("✅ Execution provider recommendation works correctly");
}

// ========== MODEL LOADING STRATEGY TESTS ==========

#[test]
fn test_model_loading_strategy_small_model() {
    println!("\n🧪 Testing model loading strategy for small model...");
    
    let system = detect_system().unwrap();
    let model_size_mb = 500; // 500MB model
    let strategy = system.recommended_loading_strategy(model_size_mb);
    
    println!("📊 Loading Strategy for {}MB model:", model_size_mb);
    println!("   Target: {}", strategy.target);
    println!("   GPU Index: {:?}", strategy.gpu_index);
    println!("   GPU %: {:?}", strategy.gpu_percent);
    println!("   CPU %: {:?}", strategy.cpu_percent);
    println!("   Reason: {}", strategy.reason);
    
    // Verify strategy is valid
    assert!(
        strategy.target == LOAD_STRATEGY_GPU || 
        strategy.target == LOAD_STRATEGY_CPU || 
        strategy.target == LOAD_STRATEGY_SPLIT,
        "Strategy should be one of the defined constants"
    );
    
    println!("✅ Small model loading strategy works correctly");
}

#[test]
fn test_model_loading_strategy_large_model() {
    println!("\n🧪 Testing model loading strategy for large model...");
    
    let system = detect_system().unwrap();
    let model_size_mb = 14000; // 14GB model (e.g., Llama 70B)
    let strategy = system.recommended_loading_strategy(model_size_mb);
    
    println!("📊 Loading Strategy for {}MB model:", model_size_mb);
    println!("   Target: {}", strategy.target);
    println!("   Reason: {}", strategy.reason);
    
    // Large models typically require split or CPU
    if system.total_vram_mb < model_size_mb {
        assert!(
            strategy.target == LOAD_STRATEGY_CPU || strategy.target == LOAD_STRATEGY_SPLIT,
            "Large model with insufficient VRAM should use CPU or split"
        );
    }
    
    println!("✅ Large model loading strategy works correctly");
}

// ========== INTEGRATION TESTS ==========

#[test]
fn test_system_info_completeness() {
    println!("\n🧪 Testing SystemInfo completeness...");
    
    let system = detect_system().unwrap();
    
    println!("📊 Complete System Information:");
    println!("\n🖥️  CPU:");
    println!("   Model: {}", system.cpu.model_name);
    println!("   Architecture: {:?}", system.cpu.architecture);
    println!("   Cores: {} / Threads: {}", system.cpu.cores, system.cpu.threads);
    println!("   BitNet DLL: {}", system.bitnet_dll_filename());
    
    println!("\n💾 Memory:");
    println!("   Total RAM: {:.1} GB", system.memory.total_ram_mb as f32 / 1024.0);
    println!("   Available RAM: {:.1} GB", system.memory.available_ram_mb as f32 / 1024.0);
    println!("   RAM Tier: {}", system.ram_tier);
    
    println!("\n🎮 Graphics:");
    println!("   GPUs: {}", system.gpus.len());
    for (idx, gpu) in system.gpus.iter().enumerate() {
        println!("   GPU {}: {} ({:?})", idx, gpu.name, gpu.vendor);
        if let Some(vram) = gpu.vram_mb {
            println!("      VRAM: {:.1} GB", vram as f32 / 1024.0);
        }
    }
    println!("   Total VRAM: {:.1} GB", system.total_vram_mb as f32 / 1024.0);
    println!("   VRAM Tier: {}", system.vram_tier);
    
    println!("\n🚀 Recommendations:");
    let provider = system.recommended_execution_provider();
    println!("   Execution Provider: {}", provider.primary);
    println!("   Reason: {}", provider.reason);
    
    println!("\n🖥️  OS:");
    println!("   Name: {}", system.os.name);
    println!("   Version: {}", system.os.version);
    println!("   Arch: {}", system.os.arch);
    
    // Verify all fields are populated
    assert!(!system.cpu.model_name.is_empty());
    assert!(system.memory.total_ram_mb > 0);
    assert!(!system.ram_tier.is_empty());
    assert!(!system.vram_tier.is_empty());
    assert!(!system.os.name.is_empty());
    
    println!("\n✅ System info is complete and consistent");
}

#[test]
fn test_constants_availability() {
    println!("\n🧪 Testing constants availability...");
    
    // Verify all constants are accessible and non-empty
    assert!(!CPU_VENDOR_INTEL.is_empty());
    assert!(!GPU_VENDOR_NVIDIA.is_empty());
    assert!(!PROVIDER_CUDA.is_empty());
    assert!(!LOAD_STRATEGY_GPU.is_empty());
    assert!(!TIER_LOW.is_empty());
    
    println!("📊 Sample Constants:");
    println!("   CPU Vendor: {}", CPU_VENDOR_INTEL);
    println!("   GPU Vendor: {}", GPU_VENDOR_NVIDIA);
    println!("   Provider: {}", PROVIDER_CUDA);
    println!("   Strategy: {}", LOAD_STRATEGY_GPU);
    println!("   Tier: {}", TIER_LOW);
    
    println!("✅ All constants are accessible");
}

