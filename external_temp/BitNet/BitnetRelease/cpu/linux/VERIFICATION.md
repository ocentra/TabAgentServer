# 🔍 BitNet Linux CPU Build Verification Report

## Build Matrix

This directory contains **13 CPU variants** (1 standard + 12 BitNet optimized):

### Standard CPU (No BitNet)
- **standard/** - Standard llama.cpp, any CPU, no BitNet

### BitNet CPU Variants
- **bitnet-portable/** - AVX2 baseline (any modern x86-64 CPU)
- **bitnet-amd-zen1/** - AMD Ryzen 1000, EPYC 7001
- **bitnet-amd-zen2/** - AMD Ryzen 3000, EPYC 7002  
- **bitnet-amd-zen3/** - AMD Ryzen 5000, EPYC 7003
- **bitnet-amd-zen4/** - AMD Ryzen 7000, EPYC 7004
- **bitnet-amd-zen5/** - AMD Ryzen 9000, EPYC 7005
- **bitnet-intel-haswell/** - Intel 4th gen (2013-2015)
- **bitnet-intel-broadwell/** - Intel 5th gen (2014-2016)
- **bitnet-intel-skylake/** - Intel 6th-9th gen (2015-2019)
- **bitnet-intel-icelake/** - Intel 10th gen mobile (2019)
- **bitnet-intel-rocketlake/** - Intel 11th gen (2021)
- **bitnet-intel-alderlake/** - Intel 12th-14th gen (2021+)

## Quick Start

Each variant is self-contained with all executables and libraries!

### Test Any Variant:
```bash
cd <variant-name>/
./llama-server --help
```

### Run Standard CPU:
```bash
cd standard/
./llama-server -m model.gguf -c 2048
```

### Run BitNet (Pick Your CPU):
```bash
cd bitnet-amd-zen3/  # Or your CPU variant
./llama-server -m bitnet-model.gguf
```

## Key Flags

All executables support `--help`:
```bash
./llama-server --help | less
```

Check what features are compiled in:
```bash
./llama-server --version
```

## Technical Details

- **Compiler:** Clang (ClangCL emulation mode for BitNet)
- **BitNet TL2:** Custom C++ kernels for 1.58-bit quantization
- **Shared Libraries:** Included in each variant directory
- **SIMD:** AVX, AVX2, FMA (varies by CPU target)

## File Counts

- **bitnet-amd-zen1/**: 41 files
- **bitnet-amd-zen2/**: 41 files
- **bitnet-amd-zen3/**: 41 files
- **bitnet-intel-alderlake/**: 41 files
- **bitnet-intel-broadwell/**: 41 files
- **bitnet-intel-haswell/**: 41 files
- **bitnet-intel-icelake/**: 41 files
- **bitnet-intel-rocketlake/**: 41 files
- **bitnet-intel-skylake/**: 41 files
- **bitnet-portable/**: 41 files
- **standard/**: 58 files

---
Build Date: Tue Oct 21 21:18:32 EDT 2025
