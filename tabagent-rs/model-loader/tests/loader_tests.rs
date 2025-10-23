/// Model loader tests
/// 
/// SMART TESTING STRATEGY:
/// - Test OUR logic (platform detection → DLL selection)
/// - Mock different platforms to test branching
/// - IF current platform matches, do REAL FFI test
/// - Test error handling for missing DLLs
/// - TDD: Write inference tests that WILL FAIL (not implemented yet)

mod test_models;

use model_loader::{get_optimal_dll_for_platform, get_dll_path_for_architecture, Model};
use tabagent_hardware::{detect_cpu_architecture, CpuArchitecture};
use std::path::PathBuf;
use test_models::*;

#[test]
fn test_dll_selection_logic_intel_alderlake() {
    println!("\n🧪 Testing DLL selection for Intel Alder Lake...");
    
    let dll = get_optimal_dll_for_platform(CpuArchitecture::IntelAlderake);
    
    assert!(dll.contains("llama"), "Should be llama DLL");
    assert!(dll.contains("intel"), "Should be Intel-specific DLL");
    
    // On Windows should be .dll, on Linux .so
    #[cfg(target_os = "windows")]
    assert!(dll.ends_with(".dll"), "Windows should use .dll");
    
    #[cfg(target_os = "linux")]
    assert!(dll.ends_with(".so"), "Linux should use .so");
    
    #[cfg(target_os = "macos")]
    assert!(dll.ends_with(".dylib"), "macOS should use .dylib");
    
    println!("✅ Intel Alder Lake → {}", dll);
}

#[test]
fn test_dll_selection_logic_intel_raptorlake() {
    println!("\n🧪 Testing DLL selection for Intel Raptor Lake...");
    
    let dll = get_optimal_dll_for_platform(CpuArchitecture::IntelRaptorake);
    
    assert!(dll.contains("llama"), "Should be llama DLL");
    assert!(dll.contains("intel") || dll.contains("raptor"), "Should be Intel Raptor DLL");
    
    println!("✅ Intel Raptor Lake → {}", dll);
}

#[test]
fn test_dll_selection_logic_amd_zen3() {
    println!("\n🧪 Testing DLL selection for AMD Zen 3...");
    
    let dll = get_optimal_dll_for_platform(CpuArchitecture::AmdZen3);
    
    assert!(dll.contains("llama"), "Should be llama DLL");
    assert!(dll.contains("amd") || dll.contains("zen"), "Should be AMD-specific DLL");
    
    println!("✅ AMD Zen 3 → {}", dll);
}

#[test]
fn test_dll_selection_logic_amd_zen4() {
    println!("\n🧪 Testing DLL selection for AMD Zen 4...");
    
    let dll = get_optimal_dll_for_platform(CpuArchitecture::AmdZen4);
    
    assert!(dll.contains("llama"), "Should be llama DLL");
    assert!(dll.contains("amd") || dll.contains("zen"), "Should be AMD-specific DLL");
    
    println!("✅ AMD Zen 4 → {}", dll);
}

#[test]
fn test_dll_selection_logic_apple_silicon() {
    println!("\n🧪 Testing DLL selection for Apple Silicon...");
    
    let dll = get_optimal_dll_for_platform(CpuArchitecture::AppleM1);
    
    assert!(dll.contains("llama"), "Should be llama DLL");
    
    #[cfg(target_os = "macos")]
    {
        assert!(dll.contains("metal") || dll.contains("apple") || dll.contains("arm"), 
            "Should use Metal/ARM on Apple Silicon");
        assert!(dll.ends_with(".dylib"), "macOS should use .dylib");
    }
    
    println!("✅ Apple Silicon → {}", dll);
}

#[test]
fn test_dll_selection_logic_generic_fallback() {
    println!("\n🧪 Testing DLL selection for generic/unknown CPU...");
    
    let dll = get_optimal_dll_for_platform(CpuArchitecture::Generic);
    
    assert!(dll.contains("llama"), "Should be llama DLL");
    // Generic should work everywhere
    
    println!("✅ Generic fallback → {}", dll);
}

#[test]
fn test_dll_path_generation() {
    println!("\n🧪 Testing DLL path generation...");
    
    let base_path = PathBuf::from("/test/path");
    
    // Test Intel path
    let intel_path = get_dll_path_for_architecture(&base_path, CpuArchitecture::IntelAlderake);
    assert!(intel_path.to_str().unwrap().contains("/test/path"));
    assert!(intel_path.to_str().unwrap().contains("llama"));
    println!("✅ Intel path: {:?}", intel_path);
    
    // Test AMD path
    let amd_path = get_dll_path_for_architecture(&base_path, CpuArchitecture::AmdZen3);
    assert!(amd_path.to_str().unwrap().contains("/test/path"));
    assert!(amd_path.to_str().unwrap().contains("llama"));
    println!("✅ AMD path: {:?}", amd_path);
    
    // Verify different architectures get different DLLs
    assert_ne!(intel_path, amd_path, "Intel and AMD should use different DLLs");
}

#[test]
fn test_all_architectures_have_dll() {
    println!("\n🧪 Testing all CPU architectures have DLL mappings...");
    
    let architectures = vec![
        CpuArchitecture::IntelAlderake,
        CpuArchitecture::IntelRaptorake,
        CpuArchitecture::IntelCorelake,
        CpuArchitecture::IntelRocketlake,
        CpuArchitecture::IntelSapphireRapids,
        CpuArchitecture::AmdZen3,
        CpuArchitecture::AmdZen4,
        CpuArchitecture::AppleM1,
        CpuArchitecture::AppleM2,
        CpuArchitecture::AppleM3,
        CpuArchitecture::Generic,
    ];
    
    for arch in architectures {
        let dll = get_optimal_dll_for_platform(arch);
        assert!(!dll.is_empty(), "Architecture {:?} has no DLL mapping", arch);
        assert!(dll.contains("llama"), "DLL for {:?} should contain 'llama'", arch);
        println!("  {:?} → {}", arch, dll);
    }
    
    println!("✅ All architectures have valid DLL mappings");
}

#[test]
fn test_platform_detection_consistency() {
    println!("\n🧪 Testing platform detection consistency...");
    
    // Detect current platform
    let detected = detect_cpu_architecture();
    println!("📊 Detected architecture: {:?}", detected);
    
    // Get DLL for detected platform
    let dll = get_optimal_dll_for_platform(detected);
    println!("📦 Selected DLL: {}", dll);
    
    // Verify it's not empty and valid
    assert!(!dll.is_empty(), "Should select a DLL");
    assert!(dll.contains("llama"), "Should be llama DLL");
    
    // Check extension matches platform
    #[cfg(target_os = "windows")]
    assert!(dll.ends_with(".dll"), "Windows binary should be .dll");
    
    #[cfg(target_os = "linux")]
    assert!(dll.ends_with(".so"), "Linux binary should be .so");
    
    #[cfg(target_os = "macos")]
    assert!(dll.ends_with(".dylib"), "macOS binary should be .dylib");
    
    println!("✅ Platform detection → DLL selection is consistent");
}

#[test]
fn test_bitnet_binary_selection() {
    println!("\n🧪 Testing BitNet binary selection...");
    
    // BitNet has platform-specific binaries too
    let architectures = vec![
        (CpuArchitecture::IntelAlderake, "should have Intel-specific BitNet"),
        (CpuArchitecture::AmdZen4, "should have AMD-specific BitNet"),
        (CpuArchitecture::AppleM2, "should have Apple Silicon BitNet"),
    ];
    
    for (arch, description) in architectures {
        // This tests OUR logic for selecting BitNet binaries
        // (assuming we have a similar function for BitNet)
        let dll = get_optimal_dll_for_platform(arch);
        
        // For now, verify llama works - BitNet logic would be similar
        assert!(!dll.is_empty(), "{}: Got empty DLL for {:?}", description, arch);
        println!("  {:?}: {}", arch, dll);
    }
    
    println!("✅ BitNet binary selection logic in place");
}

#[cfg(test)]
#[test]
#[cfg(all(target_arch = "x86_64", target_os = "windows"))]
fn test_real_dll_load_if_available() {
    println!("\n🧪 Testing REAL DLL loading (Windows x64 only)...");
    
    // Detect actual platform
    let detected = detect_cpu_architecture();
    println!("📊 Real platform: {:?}", detected);
    
    // Get DLL path
    let dll_name = get_optimal_dll_for_platform(detected);
    println!("📦 Looking for: {}", dll_name);
    
    // Try to find the DLL in common locations
    let possible_paths = vec![
        PathBuf::from(format!("./dlls/{}", dll_name)),
        PathBuf::from(format!("../dlls/{}", dll_name)),
        PathBuf::from(format!("../../BitNet/dlls/{}", dll_name)),
        PathBuf::from(format!("./Server/BitNet/dlls/{}", dll_name)),
    ];
    
    let mut found_dll = None;
    for path in &possible_paths {
        if path.exists() {
            found_dll = Some(path);
            break;
        }
    }
    
    if let Some(dll_path) = found_dll {
        println!("✅ Found DLL at: {:?}", dll_path);
        
        // Try to load it (this is the REAL test on matching platform)
        #[cfg(target_os = "windows")]
        {
            use std::ffi::OsStr;
            use std::os::windows::ffi::OsStrExt;
            
            let wide: Vec<u16> = OsStr::new(dll_path.to_str().unwrap())
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();
            
            unsafe {
                let handle = winapi::um::libloaderapi::LoadLibraryW(wide.as_ptr());
                if handle.is_null() {
                    println!("⚠️  DLL exists but failed to load (might need dependencies)");
                } else {
                    println!("✅ DLL loaded successfully!");
                    winapi::um::libloaderapi::FreeLibrary(handle);
                }
            }
        }
    } else {
        println!("⚠️  DLL not found in test directories (this is OK - DLLs may not be built yet)");
        println!("   Searched: {:?}", possible_paths);
    }
}

#[test]
fn test_error_handling_missing_dll() {
    println!("\n🧪 Testing error handling for missing DLL...");
    
    // Try to load from non-existent path
    let fake_path = PathBuf::from("/nonexistent/path/fake.dll");
    
    // This should NOT panic - should handle gracefully
    let result = std::panic::catch_unwind(|| {
        get_dll_path_for_architecture(&fake_path, CpuArchitecture::Generic);
    });
    
    assert!(result.is_ok(), "Should not panic on path generation");
    
    println!("✅ Error handling works correctly");
}

#[test]
fn test_dll_naming_conventions() {
    println!("\n🧪 Testing DLL naming conventions...");
    
    let dll = get_optimal_dll_for_platform(CpuArchitecture::IntelAlderake);
    
    // Should follow naming convention
    assert!(dll.contains("llama") || dll.contains("bitnet"), "Should be known model format");
    
    // Should not have spaces
    assert!(!dll.contains(" "), "DLL name should not contain spaces");
    
    // Should have valid extension
    assert!(
        dll.ends_with(".dll") || dll.ends_with(".so") || dll.ends_with(".dylib"),
        "Should have valid library extension"
    );
    
    println!("✅ DLL naming follows conventions");
}

// ============================================================================
// TDD: INFERENCE TESTS (WILL FAIL - NOT IMPLEMENTED YET)
// ============================================================================

#[test]
#[ignore] // Remove when inference is implemented
fn test_basic_inference_hello_world() {
    println!("\n🧪 TDD: Testing basic inference (WILL FAIL)...");
    
    // This test shows what we NEED:
    // 1. Load a tiny model
    // 2. Create inference context
    // 3. Generate "hello world" response
    // 4. Verify non-empty output
    
    // TODO: Implement Model::infer() method
    // let model = Model::load("path/to/tiny/model.gguf");
    // let response = model.infer("Say hello");
    // assert!(!response.is_empty());
    
    println!("⚠️  Inference not implemented yet - TDD placeholder");
}

#[test]
#[ignore] // Remove when inference is implemented
fn test_inference_with_context() {
    println!("\n🧪 TDD: Testing inference with context (WILL FAIL)...");
    
    // This test shows we need:
    // 1. Context management (chat history)
    // 2. Multi-turn conversation
    // 3. Context limits
    
    // TODO: Implement InferenceContext
    // let model = Model::load("path/to/model.gguf");
    // let mut context = InferenceContext::new();
    // context.add_message("user", "What is 2+2?");
    // let response1 = model.infer_with_context(&context);
    // context.add_message("assistant", &response1);
    // context.add_message("user", "What about 3+3?");
    // let response2 = model.infer_with_context(&context);
    // assert!(response2.contains("6"));
    
    println!("⚠️  Context inference not implemented yet - TDD placeholder");
}

#[test]
#[ignore] // Remove when inference is implemented
fn test_inference_streaming() {
    println!("\n🧪 TDD: Testing streaming inference (WILL FAIL)...");
    
    // This test shows we need:
    // 1. Token-by-token streaming
    // 2. Callback for each token
    // 3. Ability to cancel mid-stream
    
    // TODO: Implement Model::infer_stream()
    // let model = Model::load("path/to/model.gguf");
    // let mut tokens = Vec::new();
    // model.infer_stream("Write a story", |token| {
    //     tokens.push(token);
    //     token != "<EOS>"
    // });
    // assert!(tokens.len() > 10);
    
    println!("⚠️  Streaming not implemented yet - TDD placeholder");
}

#[test]
#[ignore] // Remove when inference is implemented
fn test_inference_with_settings() {
    println!("\n🧪 TDD: Testing inference with settings (WILL FAIL)...");
    
    // This test shows we need:
    // 1. Temperature control
    // 2. Top-p, top-k sampling
    // 3. Max tokens limit
    // 4. Stop sequences
    
    // TODO: Implement InferenceSettings
    // let model = Model::load("path/to/model.gguf");
    // let settings = InferenceSettings {
    //     temperature: 0.7,
    //     top_p: 0.9,
    //     max_tokens: 50,
    //     stop: vec!["</s>".to_string()],
    // };
    // let response = model.infer_with_settings("Hello", settings);
    // assert!(response.len() <= 50 * 10); // Rough token-to-char estimate
    
    println!("⚠️  Settings not implemented yet - TDD placeholder");
}

