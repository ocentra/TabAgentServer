# Hardware Detection Crate

**Cross-platform CPU and GPU detection for optimal binary selection.**

## Purpose

The `hardware` crate detects system hardware capabilities to select the optimal AI model binaries. It answers: "What CPU architecture and GPU does this machine have?" to ensure we load the correct BitNet/llama.cpp variant from `BitNet/Release/`.

### Problem This Solves

**Before:** Python scripts tried to detect CPU with `wmic` (fails on modern Windows) or generic platform checks. No granular detection of AMD Zen2 vs Zen3, Intel Alderlake vs Raptorlake, etc.

**After:** Rust-native hardware detection with:
- CPU microarchitecture detection (AMD Zen2, Intel Alderlake, etc.)
- GPU vendor and capabilities (CUDA, Vulkan, ROCm)
- Platform-specific implementations (Windows, Linux, macOS)
- Optimal binary path generation for BitNet/llama.cpp

## Inspiration

Directly addresses your architectural concern: **"Must detect CPU/GPU and pick correct binary for all users, not just my backyard project!"**

Pattern inspired by:
- `BitNet/Release/` structure with CPU-specific folders
- llama.cpp's CPU dispatch system
- Your requirement from `DEVELOPER_GUIDE.md`

## Responsibilities

### 1. CPU Detection
- **Vendor**: AMD, Intel, ARM
- **Architecture**: Zen2, Zen3, Zen4, Alderlake, Raptorlake, etc.
- **Features**: AVX2, AVX-512, NEON
- **Core count**: Physical and logical cores

### 2. GPU Detection
- **Vendors**: NVIDIA, AMD, Intel
- **APIs**: CUDA, Vulkan, DirectX, ROCm, Metal
- **Capabilities**: Compute capability, VRAM size
- **Device selection**: Multi-GPU systems

### 3. Binary Path Resolution
- **Input**: CPU detection results
- **Output**: Path to optimal binary (e.g., `BitNet/Release/cpu/Windows/bitnet-amd-zen2/llama-server.exe`)
- **Fallback**: Generic binaries if specific variant not available

## Architecture

```
HardwareDetector
  ├── CPU Detection (cpu.rs)
  │   ├── Windows: PowerShell Get-CimInstance
  │   ├── Linux: /proc/cpuinfo parsing
  │   └── macOS: sysctl queries
  │
  ├── GPU Detection (gpu.rs)
  │   ├── CUDA: nvidia-smi / CUDA runtime
  │   ├── Vulkan: vkEnumeratePhysicalDevices
  │   └── ROCm: rocm-smi / HIP runtime
  │
  └── Platform Helpers
      ├── platform_windows.rs: WMI queries
      ├── platform_linux.rs: proc/sys parsing
      └── platform_macos.rs: sysctl/IOKit
```

### Detection Flow

```
Application Startup
    │
    ▼
detect_cpu_architecture()
    │
    ├─ Windows? → PowerShell Get-CimInstance Win32_Processor
    ├─ Linux?   → Parse /proc/cpuinfo
    └─ macOS?   → sysctl machdep.cpu
    │
    ▼
Parse CPU family/model/stepping
    │
    ▼
Match to CpuArchitecture enum
    │  ├─ AMD: Zen2, Zen3, Zen4
    │  ├─ Intel: Alderlake, Raptorlake, Skylake
    │  └─ ARM: AppleM1, AppleM2, ARMv8
    │
    ▼
get_optimal_binary_path(arch)
    │
    ▼
"BitNet/Release/cpu/{platform}/bitnet-amd-zen2/llama-server.exe"
```

## Usage

### Rust API

```rust
use tabagent_hardware::{detect_cpu_architecture, CpuArchitecture};

// Detect CPU
let cpu_arch = detect_cpu_architecture()?;
println!("Detected: {:?}", cpu_arch); // e.g., AmdZen2

// Get optimal binary path
let binary_path = match cpu_arch {
    CpuArchitecture::AmdZen2 => "BitNet/Release/cpu/Windows/bitnet-amd-zen2/llama-server.exe",
    CpuArchitecture::IntelAlderlake => "BitNet/Release/cpu/Windows/bitnet-intel-alderlake/llama-server.exe",
    _ => "BitNet/Release/cpu/Windows/generic/llama-server.exe",
};

// Load model with correct binary
model_loader::load_from_dll(binary_path, "model.gguf")?;
```

### Python API (via model-bindings)

```python
import tabagent_model

# Get CPU variant string
cpu_variant = tabagent_model.get_cpu_variant()
print(f"CPU: {cpu_variant}")  # e.g., "bitnet-amd-zen2"

# Get full binary path
binary_path = tabagent_model.get_optimal_binary("llama-server.exe")
print(f"Binary: {binary_path}")
```

## Supported Architectures

### AMD CPUs
- **Zen2** (Ryzen 3000, EPYC 7002): AVX2
- **Zen3** (Ryzen 5000, EPYC 7003): AVX2
- **Zen4** (Ryzen 7000, EPYC 9004): AVX2, AVX-512

### Intel CPUs
- **Skylake** (6th-9th gen): AVX2
- **Alderlake** (12th gen): AVX2, hybrid cores
- **Raptorlake** (13th gen): AVX2, hybrid cores
- **Sapphirerapids** (Xeon): AVX-512

### ARM CPUs
- **Apple M1/M2/M3**: ARM64, NEON
- **ARMv8**: Generic ARM64

## Performance

| Operation | Complexity | Notes |
|-----------|------------|-------|
| CPU detection | O(1) | Cached after first call |
| GPU detection | O(n) | n = number of GPUs |
| Binary path gen | O(1) | String formatting |

### Platform-Specific Performance

- **Windows**: PowerShell overhead (~50ms first call)
- **Linux**: Fast proc parsing (~1ms)
- **macOS**: sysctl overhead (~10ms)

## When Code Hits This Crate

### Flow 1: Model Loading
```
Python: model_loader.load(repo, quant)
    ↓
PyO3: get_optimal_binary()
    ↓
Rust: detect_cpu_architecture()
    ↓
Match to binary path
    ↓
model-loader: load_from_dll(path)
```

### Flow 2: Server Startup
```
native_host.py startup
    ↓
Log system capabilities
    ↓
tabagent_model.get_cpu_variant()
    ↓
Rust: detect + format
    ↓
Log: "Running on AMD Zen2"
```

### Flow 3: Build Scripts
```
build-native.ts
    ↓
Detect platform + architecture
    ↓
Copy correct binaries to TabAgentDist/
    ↓
Package for distribution
```

## Dependencies

- `log` - Logging framework
- `serde` - Serialization (for JSON output)

### Platform-Specific
- **Windows**: PowerShell (built-in)
- **Linux**: /proc filesystem
- **macOS**: sysctl (built-in)

## See Also

- **Parent**: [Main README](../README.md)
- **Model Loader**: [model-loader/README.md](../model-loader/README.md)
- **Python Bindings**: [model-bindings/README.md](../model-bindings/README.md)
- **Binary Structure**: `BitNet/Release/` directory
- **Progress**: [TODO.md](./TODO.md)

