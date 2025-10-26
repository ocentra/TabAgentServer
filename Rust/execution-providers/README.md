# Execution Providers

Universal, format-agnostic execution provider system for hardware acceleration across multiple inference engines.

## Overview

This crate provides a unified interface for hardware acceleration providers that works across:
- **ONNX Runtime** (via `onnx-loader` adapter)
- **llama.cpp / BitNet** (via `gguf-loader` adapter)
- Future inference engines

## Supported Providers

### Core Providers (Implemented)
- ✅ **CPU** - Always available fallback
- ✅ **CUDA** - NVIDIA GPU acceleration (25+ config options)
- ✅ **TensorRT** - NVIDIA optimized inference (30+ options)
- ✅ **DirectML** - Windows GPU acceleration (NVIDIA/AMD/Intel)
- ✅ **OpenVINO** - Intel CPU/GPU/VPU optimization
- ✅ **ROCm** - AMD GPU acceleration (Linux)
- ✅ **CoreML** - Apple Silicon optimization (macOS)

### Future Providers (Planned)
- MIGraphX (AMD)
- QNN (Qualcomm)
- CANN (Huawei)
- WebGPU, WebNN, WASM
- ArmNN, ACL, NNAPI, XNNPACK
- TVM, Azure, and more

## Usage

```rust
use tabagent_execution_providers::*;

// Create a provider with configuration
let cuda = CUDAExecutionProvider::new()
    .with_device_id(0)
    .with_memory_limit(2_000_000_000)
    .with_use_tf32(true)
    .build();

// Check availability
if cuda.is_available().unwrap_or(false) {
    println!("CUDA is available!");
}

// Create dispatch with multiple providers (priority order)
let providers = vec![
    TensorRTExecutionProvider::new().build(),
    cuda,
    CPUExecutionProvider::new().build(), // Fallback
];

let dispatch = ExecutionProviderDispatch::new(providers);
```

## Architecture

This crate is **format-agnostic** - it doesn't know about ONNX, GGUF, or any specific inference engine. Instead:

1. **execution-providers** defines the universal interface
2. **onnx-loader** adapts universal providers → `ort::ExecutionProviderDispatch`
3. **gguf-loader** adapts universal providers → backend selection (CPU/CUDA/Vulkan)
4. **pipeline** uses universal providers for high-level orchestration

## Design Principles

Inspired by `ort`'s execution provider system:
- One file per provider
- Builder pattern with `with_*` methods
- Type-safe configuration via `ProviderConfig`
- Platform and runtime availability checks
- Zero dependencies on inference engines

