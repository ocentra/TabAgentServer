# 🔍 BitNet macOS Build Verification Report

## Build Matrix (GitHub Actions)

This directory contains **2 CPU variants** built by GitHub Actions:

### ✅ CPU Variants Built
- **bitnet-arm/** - BitNet for Apple Silicon (M1/M2/M3/M4 with TL1 kernels)
- **standard/** - Standard llama.cpp (universal, no BitNet)

### ⏭️ Skipped on GitHub Actions
- **bitnet-intel/** - BitNet for Intel Macs (requires local Intel Mac build)
  - GitHub's macOS runners are Apple Silicon only
  - Cross-compiling ARM → x86 causes CPU detection issues
  - Intel Macs are legacy (discontinued 2020)
  - To build: run `build-all-macos.sh` locally on an Intel Mac

### GPU Variant (separate directory)
- **../gpu/macos/standard-metal/** - Metal GPU acceleration (ALL Macs)

## Quick Start

### For M1/M2/M3/M4 (Apple Silicon):
```bash
cd bitnet-arm/
./llama-server -m bitnet-model.gguf
```

### For ANY Mac (universal standard build):
```bash
cd standard/
./llama-server -m model.gguf
```

### For Metal GPU (Best Performance):
```bash
cd ../gpu/macos/standard-metal/
./llama-server -m model.gguf -ngl 35  # Offload 35 layers to GPU
```

## Technical Details

- **Compiler:** Clang (from Xcode)
- **BitNet TL1:** ARM-optimized kernels for M1/M2/M3/M4
- **BitNet TL2:** x86-optimized kernels for Intel
- **Metal:** Apple's GPU framework (all Macs)

---
Built by GitHub Actions: 6
Commit: 3bed4ddf96e8146a7c4fe471ca5787a621fb12ba
Date: 
