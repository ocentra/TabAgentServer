# CPU Binaries Directory

This directory contains standalone `llama-server` binaries for inference.

## Structure

```
cpu/
├── windows/
│   ├── llama-server-bitnet.exe     (BitNet 1.58 specialized, TL2 kernels)
│   └── llama-server-standard.exe   (Regular GGUF + CUDA + Vulkan GPU)
├── macos/
│   ├── llama-server-bitnet         (BitNet 1.58 specialized, TL1 kernels)
│   └── llama-server-standard       (Regular GGUF + Metal GPU)
└── linux/
    ├── llama-server-bitnet         (BitNet 1.58 specialized, TL2 kernels)
    └── llama-server-standard       (Regular GGUF + CUDA + Vulkan GPU)
```

## What are these binaries?

### 1. `llama-server-bitnet` (BitNet Specialized)
Standalone HTTP server for **BitNet 1.58 models only** (1.58-bit ternary quantization).

**Features:**
- CPU-optimized inference with TL1/TL2 kernels
- 2-6x faster than standard GGUF inference for BitNet models
- HTTP API compatible with OpenAI format
- No Python dependencies required

### 2. `llama-server-standard` (General Purpose)
Standalone HTTP server for **all GGUF models** (Q4_K_M, Q5_K_M, Q8_0, etc.).

**Features:**
- GPU-accelerated inference (CUDA on Windows/Linux, Metal on macOS, Vulkan cross-platform)
- Supports all standard GGUF quantization formats
- HTTP API compatible with OpenAI format
- Fallback to CPU if GPU unavailable

## Usage

### BitNet Models (1.58-bit):
**Windows:**
```cmd
cd windows
llama-server-bitnet.exe --model path\to\bitnet-model.gguf --port 8081
```

**macOS/Linux:**
```bash
cd macos  # or linux
chmod +x llama-server-bitnet
./llama-server-bitnet --model path/to/bitnet-model.gguf --port 8081
```

### Standard GGUF Models (Q4/Q5/Q8 with GPU):
**Windows:**
```cmd
cd windows
llama-server-standard.exe --model path\to\model.gguf --port 8082 --n-gpu-layers 99
```

**macOS/Linux:**
```bash
cd macos  # or linux
chmod +x llama-server-standard
./llama-server-standard --model path/to/model.gguf --port 8082 --n-gpu-layers 99
```

**Note:** Use `--n-gpu-layers 99` to offload all layers to GPU for maximum performance.

## Build Instructions

**Note:** Use the automated build scripts instead:
- Windows: `build-all-windows.bat`
- Linux: `build-all-linux.sh`
- macOS: GitHub Actions (see `.github/workflows/build-macos-only.yml`)

### Manual Build - Windows (Visual Studio 2022):
```cmd
# BitNet specialized
cmake -B build-bitnet -DBITNET_X86_TL2=ON -DCMAKE_C_COMPILER=clang -DCMAKE_CXX_COMPILER=clang++ -T ClangCL -DLLAMA_BUILD_SERVER=ON
cmake --build build-bitnet --config Release -j
copy build-bitnet\bin\Release\llama-server.exe Release\cpu\windows\llama-server-bitnet.exe

# Standard with CUDA + Vulkan
cmake -B build-standard -DGGML_CUDA=ON -DGGML_VULKAN=ON -DLLAMA_BUILD_SERVER=ON
cmake --build build-standard --config Release -j
copy build-standard\bin\Release\llama-server.exe Release\cpu\windows\llama-server-standard.exe
```

### Manual Build - macOS (ARM64):
```bash
# BitNet specialized
cmake -B build-bitnet -DBITNET_ARM_TL1=ON -DCMAKE_C_COMPILER=clang -DCMAKE_CXX_COMPILER=clang++ -DLLAMA_BUILD_SERVER=ON
cmake --build build-bitnet --config Release -j
cp build-bitnet/bin/llama-server Release/cpu/macos/llama-server-bitnet

# Standard with Metal
cmake -B build-standard -DGGML_METAL=ON -DLLAMA_BUILD_SERVER=ON
cmake --build build-standard --config Release -j
cp build-standard/bin/llama-server Release/cpu/macos/llama-server-standard
```

### Manual Build - Linux (x64):
```bash
# BitNet specialized
cmake -B build-bitnet -DBITNET_X86_TL2=ON -DCMAKE_C_COMPILER=clang -DCMAKE_CXX_COMPILER=clang++ -DLLAMA_BUILD_SERVER=ON
cmake --build build-bitnet --config Release -j
cp build-bitnet/bin/llama-server Release/cpu/linux/llama-server-bitnet

# Standard with CUDA + Vulkan
cmake -B build-standard -DGGML_CUDA=ON -DGGML_VULKAN=ON -DLLAMA_BUILD_SERVER=ON
cmake --build build-standard --config Release -j
cp build-standard/bin/llama-server Release/cpu/linux/llama-server-standard
```

## Integration with TabAgent

TabAgent's `BitNetManager` spawns these binaries as subprocesses:

1. Detects platform (Windows/macOS/Linux)
2. Locates binary at: `Server/backends/bitnet/binaries/{platform}/llama-server`
3. Spawns process with model path and port
4. Communicates via HTTP API
5. Manages lifecycle (start/stop/reload)

## Performance

| Platform | Hardware | Speed | Notes |
|----------|----------|-------|-------|
| Windows x64 | Intel i9-13900K | ~15-25 tok/s | TL2 kernels |
| macOS ARM64 | Apple M2 Max | ~20-35 tok/s | TL1 kernels |
| Linux x64 | AMD Ryzen 9 7950X | ~15-25 tok/s | TL2 kernels |

*Speed depends on model size (1B/2B/7B) and CPU cores*

## File Sizes

- **Windows:** ~80-120 MB (larger due to Windows runtime)
- **macOS:** ~60-90 MB
- **Linux:** ~60-90 MB

## Notes

- These binaries are **standalone** - no external dependencies
- TL1/TL2 kernels provide significant speedup over standard GGUF
- BitNet models required (other GGUF models won't use optimized kernels)
- For GPU inference, see `../gpu/` directory

