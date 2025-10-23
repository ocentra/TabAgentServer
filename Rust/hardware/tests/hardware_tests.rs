/// Hardware detection tests
/// 
/// REAL TESTS - NO MOCKS:
/// - Queries actual system hardware
/// - Tests real CPU detection
/// - Tests real GPU detection (if available)
/// - Tests real memory readings

use tabagent_hardware::{detect_system, detect_cpu_architecture, CpuArchitecture};

#[test]
fn test_real_cpu_detection() {
    println!("\n🧪 Testing REAL CPU detection...");
    
    let arch = detect_cpu_architecture();
    println!("✅ Detected CPU: {:?}", arch);
    
    // Must detect SOMETHING real
    assert_ne!(arch, CpuArchitecture::Unknown, "Should detect actual CPU architecture");
    
    // On common platforms, should be specific
    #[cfg(target_arch = "x86_64")]
    {
        assert!(
            matches!(arch, 
                CpuArchitecture::IntelCorelake | 
                CpuArchitecture::IntelAlderake | 
                CpuArchitecture::IntelRocketlake |
                CpuArchitecture::IntelRaptorake |
                CpuArchitecture::IntelSapphireRapids |
                CpuArchitecture::AmdZen3 |
                CpuArchitecture::AmdZen4 |
                CpuArchitecture::Generic
            ),
            "Should detect known x86_64 architecture, got: {:?}", arch
        );
    }
}

#[test]
fn test_real_system_info() {
    println!("\n🧪 Testing REAL system information...");
    
    let system_info = detect_system();
    
    // CPU info must be present
    println!("📊 CPU Info:");
    println!("   Vendor: {}", system_info.cpu.vendor);
    println!("   Brand: {}", system_info.cpu.brand);
    println!("   Cores: {} physical", system_info.cpu.physical_cores);
    if let Some(threads) = system_info.cpu.threads {
        println!("   Threads: {}", threads);
    }
    println!("   Frequency: {} MHz", system_info.cpu.frequency_mhz);
    
    assert!(!system_info.cpu.vendor.is_empty(), "CPU vendor should not be empty");
    assert!(!system_info.cpu.brand.is_empty(), "CPU brand should not be empty");
    assert!(system_info.cpu.physical_cores > 0, "Should have at least 1 core");
    assert!(system_info.cpu.frequency_mhz > 0, "Should have non-zero frequency");
    
    println!("✅ CPU info looks valid");
    
    // Memory info
    println!("📊 Memory Info:");
    println!("   Total RAM: {} GB", system_info.memory.total_ram / (1024 * 1024 * 1024));
    println!("   Available RAM: {} GB", system_info.memory.available_ram / (1024 * 1024 * 1024));
    
    assert!(system_info.memory.total_ram > 0, "Should have non-zero RAM");
    assert!(system_info.memory.available_ram > 0, "Should have available RAM");
    assert!(
        system_info.memory.available_ram <= system_info.memory.total_ram,
        "Available RAM should not exceed total"
    );
    
    println!("✅ Memory info looks valid");
    
    // GPU info (may or may not be present)
    if system_info.gpu.available {
        println!("📊 GPU Info:");
        println!("   Name: {}", system_info.gpu.name);
        println!("   VRAM: {} GB", system_info.gpu.vram_total / (1024 * 1024 * 1024));
        
        assert!(!system_info.gpu.name.is_empty(), "GPU name should not be empty if available");
        assert!(system_info.gpu.vram_total > 0, "GPU should have VRAM if available");
        
        println!("✅ GPU detected and info looks valid");
    } else {
        println!("⚠️  No GPU detected (this is fine for CPU-only systems)");
    }
}

#[test]
fn test_real_memory_consistency() {
    println!("\n🧪 Testing REAL memory consistency...");
    
    let info1 = detect_system();
    std::thread::sleep(std::time::Duration::from_millis(100));
    let info2 = detect_system();
    
    // Total RAM should be consistent
    assert_eq!(
        info1.memory.total_ram, 
        info2.memory.total_ram,
        "Total RAM should not change between calls"
    );
    
    // Available RAM might change slightly, but should be in reasonable range
    let diff = if info1.memory.available_ram > info2.memory.available_ram {
        info1.memory.available_ram - info2.memory.available_ram
    } else {
        info2.memory.available_ram - info1.memory.available_ram
    };
    
    let max_diff = info1.memory.total_ram / 10; // Allow 10% variance
    assert!(
        diff < max_diff,
        "Available RAM changed too much: {} bytes ({}%)",
        diff,
        (diff * 100) / info1.memory.total_ram
    );
    
    println!("✅ Memory readings are consistent");
    println!("   Reading 1: {} MB available", info1.memory.available_ram / (1024 * 1024));
    println!("   Reading 2: {} MB available", info2.memory.available_ram / (1024 * 1024));
    println!("   Difference: {} MB", diff / (1024 * 1024));
}

#[test]
fn test_cpu_features() {
    println!("\n🧪 Testing CPU feature flags...");
    
    let system_info = detect_system();
    
    println!("📊 CPU Features:");
    println!("   AVX: {}", system_info.cpu.features.avx);
    println!("   AVX2: {}", system_info.cpu.features.avx2);
    println!("   AVX512: {}", system_info.cpu.features.avx512);
    println!("   FMA: {}", system_info.cpu.features.fma);
    println!("   NEON: {}", system_info.cpu.features.neon);
    
    // On x86_64, modern CPUs should have at least AVX
    #[cfg(target_arch = "x86_64")]
    {
        // Most modern x86_64 CPUs have AVX
        if system_info.cpu.features.avx {
            println!("✅ AVX support detected");
        } else {
            println!("⚠️  No AVX - very old CPU or emulated environment");
        }
    }
    
    // On ARM64, should have NEON
    #[cfg(target_arch = "aarch64")]
    {
        assert!(system_info.cpu.features.neon, "ARM64 should always have NEON");
        println!("✅ NEON support confirmed");
    }
}

#[test]
#[cfg(windows)]
fn test_windows_specific_info() {
    println!("\n🧪 Testing Windows-specific information...");
    
    let system_info = detect_system();
    
    // On Windows, should be able to get detailed CPU info
    assert!(!system_info.cpu.vendor.is_empty());
    assert!(!system_info.cpu.brand.is_empty());
    
    println!("✅ Windows CPU detection works");
    println!("   {}: {}", system_info.cpu.vendor, system_info.cpu.brand);
}

#[test]
#[cfg(target_os = "linux")]
fn test_linux_specific_info() {
    println!("\n🧪 Testing Linux-specific information...");
    
    let system_info = detect_system();
    
    // On Linux, should read from /proc/cpuinfo and /proc/meminfo
    assert!(system_info.cpu.physical_cores > 0);
    assert!(system_info.memory.total_ram > 0);
    
    println!("✅ Linux system detection works");
}

#[test]
#[cfg(target_os = "macos")]
fn test_macos_specific_info() {
    println!("\n🧪 Testing macOS-specific information...");
    
    let system_info = detect_system();
    
    // On macOS, should use sysctl
    assert!(system_info.cpu.physical_cores > 0);
    assert!(system_info.memory.total_ram > 0);
    
    // Check for Apple Silicon
    if system_info.cpu.vendor.contains("Apple") {
        println!("✅ Apple Silicon detected: {}", system_info.cpu.brand);
    } else {
        println!("✅ Intel Mac detected: {}", system_info.cpu.brand);
    }
}

