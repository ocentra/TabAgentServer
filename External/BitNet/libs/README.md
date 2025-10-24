# BitNet Library Headers

This directory contains **header files** for BitNet kernels used during compilation.

## Directory Structure

```
libs/
├── include/
│   └── bitnet_kernels.h          # Universal header file
├── linux/
│   └── bitnet-kernel/
│       └── bitnet_kernels.h      # Linux-specific header
└── windows/
    └── bitnet-kernel/
        └── (no headers yet)       # Windows uses universal header
```

## ⚠️ Important: Binaries Are NOT Here!

**Binary files (`.dll`, `.so`, `.lib`) are stored in the separate `BitnetRelease/` repository:**
- https://github.com/ocentra/BitnetRelease

**Why?**
- ✅ Keeps source repo lightweight
- ✅ Avoids Git LFS issues on forks
- ✅ Separates build artifacts from source code

## Usage

These header files are used during compilation:
- CMake includes them when building BitNet-optimized variants
- They define the C interface for BitNet's custom 1.58-bit kernels
- The actual binary implementations are in `BitnetRelease/`

## For Developers

If you're building BitNet from source:
1. Headers from this directory are included automatically by CMake
2. Compiled binaries are output to `BitnetRelease/cpu/` or `BitnetRelease/gpu/`
3. Use the build scripts: `build_complete.ps1` (Windows) or `build-all-linux.sh` (Linux)

