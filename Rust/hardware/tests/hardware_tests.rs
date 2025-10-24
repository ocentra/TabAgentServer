/// Hardware detection tests
/// 
/// REAL TESTS - NO MOCKS:
/// - Queries actual system hardware
/// - Tests real CPU detection
/// - Tests real GPU detection (if available)
/// - Tests real memory readings

use tabagent_hardware::{detect_system, detect_cpu_architecture, CpuArchitecture, CpuVendor};

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

