use tabagent_execution_providers::*;

#[test]
fn test_cpu_provider_always_available() {
    let cpu = CPUExecutionProvider::new().build();
    assert!(cpu.is_available().unwrap());
    assert_eq!(cpu.name(), "CPUExecutionProvider");
    assert_eq!(cpu.backend_type(), BackendType::CPU);
    assert!(cpu.supported_by_platform());
}

#[test]
fn test_cuda_provider_config() {
    let cuda = CUDAExecutionProvider::new()
        .with_device_id(1)
        .with_memory_limit(2_000_000_000)
        .with_use_tf32(true)
        .with_enable_cuda_graph(false)
        .build();
    
    assert_eq!(cuda.name(), "CUDAExecutionProvider");
    assert_eq!(cuda.backend_type(), BackendType::Cuda);
    assert_eq!(cuda.config().get_as::<i32>("device_id"), Some(1));
    assert_eq!(cuda.config().get_as::<usize>("gpu_mem_limit"), Some(2_000_000_000));
    assert_eq!(cuda.config().get_as::<bool>("use_tf32"), Some(true));
    assert_eq!(cuda.config().get_as::<bool>("enable_cuda_graph"), Some(false));
}

#[test]
fn test_tensorrt_provider_config() {
    let tensorrt = TensorRTExecutionProvider::new()
        .with_device_id(0)
        .with_max_workspace_size(1_000_000_000)
        .with_fp16_enable(true)
        .with_int8_enable(false)
        .build();
    
    assert_eq!(tensorrt.name(), "TensorrtExecutionProvider");
    assert_eq!(tensorrt.backend_type(), BackendType::TensorRT);
    assert_eq!(tensorrt.config().get_as::<i32>("device_id"), Some(0));
    assert_eq!(tensorrt.config().get_as::<usize>("trt_max_workspace_size"), Some(1_000_000_000));
    assert_eq!(tensorrt.config().get_as::<bool>("trt_fp16_enable"), Some(true));
}

#[test]
fn test_dispatch_creation() {
    let providers = vec![
        CUDAExecutionProvider::new().build(),
        CPUExecutionProvider::new().build(),
    ];
    
    let dispatch = ExecutionProviderDispatch::new(providers);
    assert_eq!(dispatch.providers().len(), 2);
}

#[test]
fn test_dispatch_filter_available() {
    let providers = vec![
        CUDAExecutionProvider::new().build(),
        CPUExecutionProvider::new().build(),
    ];
    
    let dispatch = ExecutionProviderDispatch::new(providers);
    let available = dispatch.filter_available();
    
    // CPU should always be available
    assert!(!available.is_empty());
    assert!(available.iter().any(|p| p.backend_type() == BackendType::CPU));
}

#[test]
fn test_directml_windows_only() {
    let directml = DirectMLExecutionProvider::new().build();
    
    #[cfg(target_os = "windows")]
    assert!(directml.supported_by_platform());
    
    #[cfg(not(target_os = "windows"))]
    assert!(!directml.supported_by_platform());
}

#[test]
fn test_coreml_macos_only() {
    let coreml = CoreMLExecutionProvider::new()
        .with_compute_units("ALL")
        .with_model_format("MLProgram")
        .build();
    
    #[cfg(target_os = "macos")]
    assert!(coreml.supported_by_platform());
    
    #[cfg(not(target_os = "macos"))]
    assert!(!coreml.supported_by_platform());
}

#[test]
fn test_provider_config_get_as() {
    let cuda = CUDAExecutionProvider::new()
        .with_device_id(42)
        .with_memory_limit(123456789)
        .build();
    
    // Test different type conversions
    assert_eq!(cuda.config().get_as::<i32>("device_id"), Some(42));
    assert_eq!(cuda.config().get_as::<u32>("device_id"), Some(42));
    assert_eq!(cuda.config().get("device_id"), Some("42"));
    
    // Non-existent key
    assert_eq!(cuda.config().get_as::<i32>("nonexistent"), None);
}

#[test]
fn test_backend_type_equality() {
    assert_eq!(BackendType::Cuda, BackendType::Cuda);
    assert_ne!(BackendType::Cuda, BackendType::TensorRT);
    assert_ne!(BackendType::CPU, BackendType::CoreML);
}

