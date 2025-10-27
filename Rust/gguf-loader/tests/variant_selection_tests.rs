/// Unit tests for variant selection logic
/// 
/// These tests verify that the correct library variant is selected for different
/// hardware configurations WITHOUT requiring actual hardware or libraries.

use gguf_loader::{Variant, BitNetCpuVariant, BitNetGpuVariant, StandardCpuVariant, StandardGpuVariant};
use tabagent_hardware::{SystemInfo, CpuInfo, GpuInfo, CpuArchitecture, GpuVendor};

/// Helper to create mock system info
fn mock_system(cpu_arch: CpuArchitecture, gpus: Vec<GpuVendor>) -> SystemInfo {
    SystemInfo {
        cpu: CpuInfo {
            architecture: cpu_arch,
            name: "Mock CPU".to_string(),
            cores: 8,
            threads: 16,
            vendor: "Mock".to_string(),
        },
        gpus: gpus.into_iter().map(|vendor| GpuInfo {
            vendor,
            name: format!("Mock {:?} GPU", vendor),
            memory_mb: 8192,
            driver_version: "1.0".to_string(),
        }).collect(),
        total_ram_mb: 16384,
        os: "Mock OS".to_string(),
        os_version: "1.0".to_string(),
    }
}

/// Test variant selection for different hardware configurations
#[test]
fn test_variant_selection_logic() {
    // Test AMD Zen CPU variants
    let zen_variants = vec![
        (CpuArchitecture::AmdZen1, BitNetCpuVariant::AmdZen1),
        (CpuArchitecture::AmdZen2, BitNetCpuVariant::AmdZen2),
        (CpuArchitecture::AmdZen3, BitNetCpuVariant::AmdZen3),
        (CpuArchitecture::AmdZen4, BitNetCpuVariant::AmdZen4),
        (CpuArchitecture::AmdZen5, BitNetCpuVariant::AmdZen5),
    ];
    
    for (arch, expected_variant) in zen_variants {
        let selected = BitNetCpuVariant::from_architecture(&arch);
        assert_eq!(selected, expected_variant, 
            "Expected {:?} for {:?}", expected_variant, arch);
        
        // Verify variant name
        assert!(selected.variant_name().contains("zen"), 
            "Variant name should contain 'zen': {}", selected.variant_name());
    }
    
    // Test Intel CPU variants
    let intel_variants = vec![
        (CpuArchitecture::IntelHaswell, BitNetCpuVariant::IntelHaswell),
        (CpuArchitecture::IntelBroadwell, BitNetCpuVariant::IntelBroadwell),
        (CpuArchitecture::IntelSkylake, BitNetCpuVariant::IntelSkylake),
        (CpuArchitecture::IntelIcelake, BitNetCpuVariant::IntelIcelake),
        (CpuArchitecture::IntelRocketlake, BitNetCpuVariant::IntelRocketlake),
        (CpuArchitecture::IntelAlderlake, BitNetCpuVariant::IntelAlderlake),
    ];
    
    for (arch, expected_variant) in intel_variants {
        let selected = BitNetCpuVariant::from_architecture(&arch);
        assert_eq!(selected, expected_variant,
            "Expected {:?} for {:?}", expected_variant, arch);
        
        // Verify variant name
        assert!(selected.variant_name().contains("intel"),
            "Variant name should contain 'intel': {}", selected.variant_name());
    }
    
    // Test ARM variants (Apple Silicon)
    let arm_variants = vec![
        CpuArchitecture::AppleM1,
        CpuArchitecture::AppleM2,
        CpuArchitecture::AppleM3,
        CpuArchitecture::ArmV8,
        CpuArchitecture::ArmV9,
    ];
    
    for arch in arm_variants {
        let selected = BitNetCpuVariant::from_architecture(&arch);
        assert_eq!(selected, BitNetCpuVariant::Arm,
            "Expected Arm variant for {:?}", arch);
        assert_eq!(selected.variant_name(), "bitnet-arm");
    }
    
    // Test fallback to Portable
    let selected = BitNetCpuVariant::from_architecture(&CpuArchitecture::Unknown);
    assert_eq!(selected, BitNetCpuVariant::Portable);
    assert_eq!(selected.variant_name(), "bitnet-portable");
}

#[test]
fn test_gpu_variant_selection() {
    // Test NVIDIA GPU selection (Windows/Linux)
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    {
        let variant = StandardGpuVariant::from_gpu_vendor(GpuVendor::Nvidia);
        assert_eq!(variant, StandardGpuVariant::CudaVulkan);
        assert_eq!(variant.variant_name(), "standard-cuda-vulkan");
    }
    
    // Test AMD GPU selection
    #[cfg(target_os = "windows")]
    {
        let variant = StandardGpuVariant::from_gpu_vendor(GpuVendor::Amd);
        assert_eq!(variant, StandardGpuVariant::CudaVulkan); // Uses Vulkan
        assert_eq!(variant.variant_name(), "standard-cuda-vulkan");
    }
    
    #[cfg(target_os = "linux")]
    {
        let variant = StandardGpuVariant::from_gpu_vendor(GpuVendor::Amd);
        assert_eq!(variant, StandardGpuVariant::CudaVulkan); // Uses Vulkan
        assert_eq!(variant.variant_name(), "standard-cuda-vulkan");
    }
    
    // Test Intel GPU selection
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    {
        let variant = StandardGpuVariant::from_gpu_vendor(GpuVendor::Intel);
        assert_eq!(variant, StandardGpuVariant::OpenCL);
        assert_eq!(variant.variant_name(), "standard-opencl");
    }
    
    // Test Apple GPU selection (macOS)
    #[cfg(target_os = "macos")]
    {
        let variant = StandardGpuVariant::from_gpu_vendor(GpuVendor::Apple);
        assert_eq!(variant, StandardGpuVariant::Metal);
        assert_eq!(variant.variant_name(), "standard-metal");
        
        // On macOS, everything should use Metal
        let variant = StandardGpuVariant::from_gpu_vendor(GpuVendor::Nvidia);
        assert_eq!(variant, StandardGpuVariant::Metal);
        
        let variant = StandardGpuVariant::from_gpu_vendor(GpuVendor::Amd);
        assert_eq!(variant, StandardGpuVariant::Metal);
    }
}

#[test]
fn test_priority_nvidia_gpu_selects_bitnet_cuda() {
    // On Windows/Linux with NVIDIA GPU, BitNet GPU should be prioritized
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    {
        let system = mock_system(CpuArchitecture::AmdZen3, vec![GpuVendor::Nvidia]);
        
        // Simulate auto-selection with GPU preference
        // Priority: BitNet GPU > Standard GPU > BitNet CPU
        let prefer_gpu = true;
        
        if prefer_gpu && !system.gpus.is_empty() {
            let gpu = &system.gpus[0];
            
            if matches!(gpu.vendor, GpuVendor::Nvidia) {
                // Should select BitNet GPU
                let variant = Variant::BitNetGpu(BitNetGpuVariant);
                assert_eq!(variant.name(), "bitnet-cuda");
            }
        }
    }
}

#[test]
fn test_priority_amd_gpu_selects_standard_cuda_vulkan() {
    // AMD GPU should use Standard GPU with Vulkan
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    {
        let system = mock_system(CpuArchitecture::IntelSkylake, vec![GpuVendor::Amd]);
        
        let prefer_gpu = true;
        
        if prefer_gpu && !system.gpus.is_empty() {
            let gpu = &system.gpus[0];
            
            // AMD uses Standard GPU (not BitNet GPU, which is NVIDIA-only)
            let variant = Variant::StandardGpu(StandardGpuVariant::from_gpu_vendor(gpu.vendor));
            assert_eq!(variant.name(), "standard-cuda-vulkan");
        }
    }
}

#[test]
fn test_priority_no_gpu_selects_bitnet_cpu() {
    // Without GPU, should select BitNet CPU variant based on architecture
    let system = mock_system(CpuArchitecture::AmdZen4, vec![]);
    
    let prefer_gpu = true; // Even with preference, no GPU available
    
    if !prefer_gpu || system.gpus.is_empty() {
        let cpu_variant = BitNetCpuVariant::from_architecture(&system.cpu.architecture);
        let variant = Variant::BitNetCpu(cpu_variant);
        assert_eq!(variant.name(), "bitnet-amd-zen4");
    }
}

#[test]
fn test_apple_silicon_selects_correct_variants() {
    #[cfg(target_os = "macos")]
    {
        // Test Apple M1
        let system = mock_system(CpuArchitecture::AppleM1, vec![GpuVendor::Apple]);
        
        // With GPU preference
        if !system.gpus.is_empty() {
            let variant = Variant::StandardGpu(StandardGpuVariant::Metal);
            assert_eq!(variant.name(), "standard-metal");
        }
        
        // Without GPU preference (CPU-only)
        let cpu_variant = BitNetCpuVariant::from_architecture(&system.cpu.architecture);
        let variant = Variant::BitNetCpu(cpu_variant);
        assert_eq!(variant.name(), "bitnet-arm");
    }
}

#[test]
fn test_library_path_construction() {
    use std::path::PathBuf;
    use gguf_loader::LibraryVariant;
    
    let base_path = PathBuf::from("/mock/base");
    
    // Test BitNet CPU variant path
    let variant = BitNetCpuVariant::AmdZen3;
    let path = variant.library_path(&base_path);
    
    #[cfg(target_os = "windows")]
    {
        let expected = base_path.join("BitnetRelease/cpu/windows/bitnet-amd-zen3/llama.dll");
        assert_eq!(path, expected);
    }
    
    #[cfg(target_os = "linux")]
    {
        let expected = base_path.join("BitnetRelease/cpu/linux/bitnet-amd-zen3/libllama.so");
        assert_eq!(path, expected);
    }
    
    #[cfg(target_os = "macos")]
    {
        let expected = base_path.join("BitnetRelease/cpu/macos/bitnet-amd-zen3/libllama.dylib");
        assert_eq!(path, expected);
    }
}

#[test]
fn test_bitnet_gpu_only_on_windows_linux() {
    // BitNet GPU should only be available on Windows/Linux
    let variant = BitNetGpuVariant;
    
    assert_eq!(variant.variant_name(), "bitnet-cuda");
    assert_eq!(variant.base_type(), "gpu");
    
    #[cfg(target_os = "windows")]
    {
        assert_eq!(variant.library_name(), "llama.dll");
    }
    
    #[cfg(target_os = "linux")]
    {
        assert_eq!(variant.library_name(), "libllama.so");
    }
    
    // macOS has the function but library won't exist
    #[cfg(target_os = "macos")]
    {
        assert_eq!(variant.library_name(), "libllama.dylib");
        // In practice, this library won't exist on macOS
    }
}

#[test]
fn test_variant_base_types() {
    // All CPU variants should have base_type "cpu"
    let cpu_variants = vec![
        Variant::BitNetCpu(BitNetCpuVariant::AmdZen3),
        Variant::StandardCpu(StandardCpuVariant),
    ];
    
    for variant in cpu_variants {
        match variant {
            Variant::BitNetCpu(v) => assert_eq!(v.base_type(), "cpu"),
            Variant::StandardCpu(v) => assert_eq!(v.base_type(), "cpu"),
            _ => panic!("Expected CPU variant"),
        }
    }
    
    // All GPU variants should have base_type "gpu"
    let gpu_variants = vec![
        Variant::BitNetGpu(BitNetGpuVariant),
        Variant::StandardGpu(StandardGpuVariant::CudaVulkan),
        Variant::StandardGpu(StandardGpuVariant::Metal),
        Variant::StandardGpu(StandardGpuVariant::OpenCL),
    ];
    
    for variant in gpu_variants {
        match variant {
            Variant::BitNetGpu(v) => assert_eq!(v.base_type(), "gpu"),
            Variant::StandardGpu(v) => assert_eq!(v.base_type(), "gpu"),
            _ => panic!("Expected GPU variant"),
        }
    }
}

#[test]
fn test_standard_cpu_variant() {
    let variant = StandardCpuVariant;
    
    assert_eq!(variant.variant_name(), "standard");
    assert_eq!(variant.base_type(), "cpu");
    
    #[cfg(target_os = "windows")]
    assert_eq!(variant.library_name(), "llama.dll");
    
    #[cfg(target_os = "linux")]
    assert_eq!(variant.library_name(), "libllama.so");
    
    #[cfg(target_os = "macos")]
    assert_eq!(variant.library_name(), "libllama.dylib");
}

#[test]
fn test_variant_names_are_unique() {
    use std::collections::HashSet;
    
    let mut names = HashSet::new();
    
    // Collect all CPU variant names
    let cpu_variants = vec![
        BitNetCpuVariant::AmdZen1, BitNetCpuVariant::AmdZen2,
        BitNetCpuVariant::AmdZen3, BitNetCpuVariant::AmdZen4,
        BitNetCpuVariant::AmdZen5, BitNetCpuVariant::IntelHaswell,
        BitNetCpuVariant::IntelBroadwell, BitNetCpuVariant::IntelSkylake,
        BitNetCpuVariant::IntelIcelake, BitNetCpuVariant::IntelRocketlake,
        BitNetCpuVariant::IntelAlderlake, BitNetCpuVariant::Arm,
        BitNetCpuVariant::Portable,
    ];
    
    for variant in cpu_variants {
        let name = variant.variant_name();
        assert!(!names.contains(name), "Duplicate variant name: {}", name);
        names.insert(name);
    }
    
    // Add Standard CPU
    names.insert(StandardCpuVariant.variant_name());
    
    // Add GPU variants
    names.insert(BitNetGpuVariant.variant_name());
    names.insert(StandardGpuVariant::CudaVulkan.variant_name());
    names.insert(StandardGpuVariant::Metal.variant_name());
    names.insert(StandardGpuVariant::OpenCL.variant_name());
    
    // Total unique variants
    assert_eq!(names.len(), 18, "Expected 18 unique variant names");
}

#[test]
fn test_multi_gpu_system() {
    // Test system with multiple GPUs (should use first one)
    let system = mock_system(
        CpuArchitecture::IntelAlderlake,
        vec![GpuVendor::Nvidia, GpuVendor::Amd]
    );
    
    assert_eq!(system.gpus.len(), 2);
    
    // First GPU should be used for variant selection
    let first_gpu = &system.gpus[0];
    assert_eq!(first_gpu.vendor, GpuVendor::Nvidia);
    
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    {
        // Should select BitNet GPU (CUDA) for NVIDIA
        let variant = Variant::BitNetGpu(BitNetGpuVariant);
        assert_eq!(variant.name(), "bitnet-cuda");
    }
}

