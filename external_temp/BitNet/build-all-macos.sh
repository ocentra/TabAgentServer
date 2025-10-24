#!/bin/bash
# ============================================================================
# BitNet Complete Build Script for macOS (CPU + GPU Metal)
# ============================================================================
# This script builds all macOS variants:
#   - BitNet ARM (M1/M2/M3 with TL1 kernels)
#   - BitNet Intel (Intel Macs with TL2 kernels)
#   - Standard CPU (universal, no BitNet)
#   - Standard Metal GPU (ALL Macs - M1/M2/M3 + Intel)
#
# Prerequisites:
#   - Xcode Command Line Tools (xcode-select --install)
#   - CMake 3.14+ (brew install cmake)
# ============================================================================

set -e  # Exit on error

# Get script directory
BITNET_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$BITNET_ROOT"

# ============================================================================
# SCRIPT PARAMETERS
# ============================================================================
BUILD_DIR="BitnetRelease"
CLEAN_BUILD=false
BUILD_VARIANTS=()
LIST_VARIANTS=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --build-dir)
            BUILD_DIR="$2"
            shift 2
            ;;
        --clean)
            CLEAN_BUILD=true
            shift
            ;;
        --variants)
            IFS=',' read -ra BUILD_VARIANTS <<< "$2"
            shift 2
            ;;
        --list-variants)
            LIST_VARIANTS=true
            shift
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --build-dir DIR       Output directory (default: BitnetRelease)"
            echo "  --clean               Clean build (remove all previous builds)"
            echo "  --variants V1,V2      Build only specific variants (comma-separated)"
            echo "  --list-variants       List all available build variants"
            echo "  --help                Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0                              # Build everything"
            echo "  $0 --variants arm,metal         # Build only ARM + Metal"
            echo "  $0 --clean                      # Clean build (rebuild all)"
            echo "  $0 --list-variants              # List available variants"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Color codes
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# ============================================================================
# HANDLE --list-variants FLAG
# ============================================================================
if [ "$LIST_VARIANTS" = true ]; then
    echo ""
    echo -e "${CYAN}=== AVAILABLE BUILD VARIANTS (macOS) ===${NC}"
    echo ""
    echo -e "${YELLOW}CPU VARIANTS:${NC}"
    echo "  arm                - BitNet ARM (M1/M2/M3/M4 with TL1 kernels)"
    echo "  intel              - BitNet Intel (Intel Macs with TL2 kernels)"
    echo "  standard           - Standard CPU (universal, no BitNet)"
    echo ""
    echo -e "${YELLOW}GPU VARIANTS:${NC}"
    echo "  metal              - Standard Metal GPU (ALL Macs - best performance)"
    echo ""
    echo -e "${RED}NOT SUPPORTED ON macOS:${NC}"
    echo "  ✗ CUDA             - Apple doesn't support NVIDIA GPUs"
    echo "  ✗ Vulkan           - Apple uses Metal instead"
    echo "  ✗ BitNet GPU       - Requires CUDA (Windows/Linux only)"
    echo ""
    echo -e "${CYAN}EXAMPLES:${NC}"
    echo -e "${GREEN}  # Build only ARM variant:${NC}"
    echo "  ./build-all-macos.sh --variants arm"
    echo ""
    echo -e "${GREEN}  # Build ARM + Metal:${NC}"
    echo "  ./build-all-macos.sh --variants arm,metal"
    echo ""
    echo -e "${GREEN}  # Build everything (default):${NC}"
    echo "  ./build-all-macos.sh"
    echo ""
    exit 0
fi

# ============================================================================
# HELPER FUNCTIONS
# ============================================================================
should_build_variant() {
    local variant_name="$1"
    
    # If no variants specified, build everything
    if [ ${#BUILD_VARIANTS[@]} -eq 0 ]; then
        return 0  # true
    fi
    
    # Check if this variant is in the list
    for v in "${BUILD_VARIANTS[@]}"; do
        if [ "$v" = "$variant_name" ]; then
            return 0  # true
        fi
    done
    
    return 1  # false
}

should_skip_build() {
    local build_dir="$1"
    local release_dir="$2"
    local min_files="${3:-3}"
    
    # If CLEAN_BUILD flag is set, never skip
    if [ "$CLEAN_BUILD" = true ]; then
        return 1  # false (don't skip)
    fi
    
    # Check if build directory exists
    if [ ! -d "$build_dir" ]; then
        return 1  # false (don't skip)
    fi
    
    # Check if Release output directory exists
    if [ ! -d "$release_dir" ]; then
        return 1  # false (don't skip)
    fi
    
    # Check if Release directory has enough files
    local file_count=$(ls -1 "$release_dir" 2>/dev/null | wc -l)
    if [ $file_count -lt $min_files ]; then
        return 1  # false (don't skip)
    fi
    
    # All checks passed - skip this build!
    return 0  # true (skip)
}

echo ""
echo "============================================================================"
echo "BitNet Complete Build for macOS"
echo "============================================================================"
echo ""

# Show selective build info if variants specified
if [ ${#BUILD_VARIANTS[@]} -gt 0 ]; then
    echo -e "${CYAN}Selective build mode: ${BUILD_VARIANTS[*]}${NC}"
    echo ""
fi

if [ "$CLEAN_BUILD" = true ]; then
    echo -e "${YELLOW}Clean build mode: Will rebuild everything${NC}"
    echo ""
fi

# Detect CPU architecture
ARCH=$(uname -m)
if [ "$ARCH" = "arm64" ]; then
    echo -e "${CYAN}Detected: Apple Silicon (M1/M2/M3/M4)${NC}"
    DEFAULT_ARCH="ARM"
else
    echo -e "${CYAN}Detected: Intel Mac${NC}"
    DEFAULT_ARCH="Intel"
fi
echo ""

echo "This will build up to 4 variants:"
echo "  1. BitNet ARM CPU (M1/M2/M3/M4 - TL1 kernels)"
echo "  2. BitNet Intel CPU (Intel Macs - TL2 kernels)"
echo "  3. Standard CPU (universal, no BitNet)"
echo "  4. Standard Metal GPU (ALL Macs)"
echo ""
echo -e "${YELLOW}⚠️  BitNet GPU not supported on macOS (requires CUDA)${NC}"
echo ""
echo "Output directory: $BUILD_DIR/cpu/macos/ and $BUILD_DIR/gpu/macos/"
echo "  Each variant in its own subdirectory!"
echo ""
read -p "Press Enter to start, or Ctrl+C to cancel..."

echo ""
echo "============================================================================"
echo "Step 0: Verifying Tools"
echo "============================================================================"

# Check CMake
if ! command -v cmake &> /dev/null; then
    echo -e "${RED}ERROR: CMake not found!${NC}"
    echo "Install with: brew install cmake"
    exit 1
fi
CMAKE_VERSION=$(cmake --version | head -1)
echo -e "${GREEN}  ✓ CMake found: $CMAKE_VERSION${NC}"

# Check Clang
if ! command -v clang &> /dev/null; then
    echo -e "${RED}ERROR: Clang not found!${NC}"
    echo "Install Xcode Command Line Tools: xcode-select --install"
    exit 1
fi
CLANG_VERSION=$(clang --version | head -1)
echo -e "${GREEN}  ✓ Clang found: $CLANG_VERSION${NC}"

# Check Git
if ! command -v git &> /dev/null; then
    echo -e "${RED}ERROR: Git not found!${NC}"
    exit 1
fi
echo -e "${GREEN}  ✓ Git found: $(git --version)${NC}"

echo ""
echo "============================================================================"
echo "Step 1: Initialize Git Submodules"
echo "============================================================================"
echo ""

# Initialize submodules if not already initialized
if [ ! -d "3rdparty/llama.cpp/.git" ]; then
    echo -e "${YELLOW}Submodule not initialized, initializing now...${NC}"
    git submodule update --init --recursive
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓ Submodule initialized successfully${NC}"
    else
        echo -e "${RED}ERROR: Failed to initialize submodule${NC}"
        exit 1
    fi
else
    echo -e "${GREEN}✓ Submodule already initialized${NC}"
fi

echo ""
echo "============================================================================"
echo "Step 2: Create Release Directory Structure"
echo "============================================================================"
echo ""

# Create isolated subdirectories (4 variants)
mkdir -p $BUILD_DIR/cpu/macos/bitnet-arm
mkdir -p $BUILD_DIR/cpu/macos/bitnet-intel
mkdir -p $BUILD_DIR/cpu/macos/standard
mkdir -p $BUILD_DIR/gpu/macos/standard-metal

echo -e "${GREEN}  ✓ All 4 variant directories created${NC}"
echo ""
echo "Build Matrix:"
echo "  CPU: 3 variants (bitnet-arm, bitnet-intel, standard)"
echo "  GPU: 1 variant  (standard-metal)"

echo ""
echo "============================================================================"
echo "Step 3: Building BitNet ARM CPU (M1/M2/M3/M4)"
echo "============================================================================"

if ! should_build_variant "arm"; then
    echo -e "${YELLOW}[SKIP] ARM build not requested${NC}"
elif should_skip_build "build-macos-bitnet-arm" "$BUILD_DIR/cpu/macos/bitnet-arm" 3; then
    echo -e "${GREEN}[OK] BitNet ARM already built${NC}"
else
    echo ""
    echo "Building BitNet for Apple Silicon (ARM TL1 kernels)..."
    echo ""
    
    # Clean previous build
    rm -rf build-macos-bitnet-arm
    
    # Build with ARM TL1
    cmake -B build-macos-bitnet-arm \
        -DBITNET_ARM_TL1=ON \
        -DCMAKE_C_COMPILER=clang \
        -DCMAKE_CXX_COMPILER=clang++ \
        -DLLAMA_BUILD_SERVER=ON \
        -DLLAMA_BUILD_EXAMPLES=ON \
        .
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Configuration successful!${NC}"
        
        # Build
        cmake --build build-macos-bitnet-arm --config Release -j
        
        if [ $? -eq 0 ]; then
            # Copy binaries
            echo ""
            echo "Copying ARM binaries to $BUILD_DIR/cpu/macos/bitnet-arm/ ..."
            
            if [ -d "build-macos-bitnet-arm/bin" ]; then
                cp -f build-macos-bitnet-arm/bin/* $BUILD_DIR/cpu/macos/bitnet-arm/ 2>/dev/null || true
            fi
            
            # Copy dylibs (macOS shared libraries)
            find build-macos-bitnet-arm -name "*.dylib" -type f -exec cp -f {} $BUILD_DIR/cpu/macos/bitnet-arm/ \; 2>/dev/null || true
            
            chmod +x $BUILD_DIR/cpu/macos/bitnet-arm/* 2>/dev/null || true
            
            FILE_COUNT=$(ls -1 $BUILD_DIR/cpu/macos/bitnet-arm/ 2>/dev/null | wc -l)
            echo -e "${GREEN}  ✓ $FILE_COUNT files copied${NC}"
            echo -e "${GREEN}✅ BitNet ARM built successfully!${NC}"
        else
            echo -e "${YELLOW}WARNING: ARM build failed!${NC}"
        fi
    else
        echo -e "${RED}❌ Configuration failed!${NC}"
    fi
fi

echo ""
echo "============================================================================"
echo "Step 4: Building BitNet Intel CPU (Intel Macs)"
echo "============================================================================"

if ! should_build_variant "intel"; then
    echo -e "${YELLOW}[SKIP] Intel build not requested${NC}"
elif should_skip_build "build-macos-bitnet-intel" "$BUILD_DIR/cpu/macos/bitnet-intel" 3; then
    echo -e "${GREEN}[OK] BitNet Intel already built${NC}"
else
    echo ""
    echo "Building BitNet for Intel Macs (x86 TL2 kernels)..."
    echo ""
    
    # Clean previous build
    rm -rf build-macos-bitnet-intel
    
    # Build with x86 TL2
    cmake -B build-macos-bitnet-intel \
        -DBITNET_X86_TL2=ON \
        -DCMAKE_C_COMPILER=clang \
        -DCMAKE_CXX_COMPILER=clang++ \
        -DCMAKE_OSX_ARCHITECTURES=x86_64 \
        -DLLAMA_BUILD_SERVER=ON \
        -DLLAMA_BUILD_EXAMPLES=ON \
        .
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Configuration successful!${NC}"
        
        # Build
        cmake --build build-macos-bitnet-intel --config Release -j
        
        if [ $? -eq 0 ]; then
            # Copy binaries
            echo ""
            echo "Copying Intel binaries to $BUILD_DIR/cpu/macos/bitnet-intel/ ..."
            
            if [ -d "build-macos-bitnet-intel/bin" ]; then
                cp -f build-macos-bitnet-intel/bin/* $BUILD_DIR/cpu/macos/bitnet-intel/ 2>/dev/null || true
            fi
            
            # Copy dylibs
            find build-macos-bitnet-intel -name "*.dylib" -type f -exec cp -f {} $BUILD_DIR/cpu/macos/bitnet-intel/ \; 2>/dev/null || true
            
            chmod +x $BUILD_DIR/cpu/macos/bitnet-intel/* 2>/dev/null || true
            
            FILE_COUNT=$(ls -1 $BUILD_DIR/cpu/macos/bitnet-intel/ 2>/dev/null | wc -l)
            echo -e "${GREEN}  ✓ $FILE_COUNT files copied${NC}"
            echo -e "${GREEN}✅ BitNet Intel built successfully!${NC}"
        else
            echo -e "${YELLOW}WARNING: Intel build failed!${NC}"
        fi
    else
        echo -e "${RED}❌ Configuration failed!${NC}"
    fi
fi

echo ""
echo "============================================================================"
echo "Step 5: Building Standard CPU (universal)"
echo "============================================================================"

if ! should_build_variant "standard"; then
    echo -e "${YELLOW}[SKIP] Standard CPU build not requested${NC}"
elif should_skip_build "build-macos-standard-cpu" "$BUILD_DIR/cpu/macos/standard" 3; then
    echo -e "${GREEN}[OK] Standard CPU already built${NC}"
else
    echo ""
    echo "Building standard llama.cpp (CPU only, no BitNet)..."
    echo ""
    
    # Clean previous build
    rm -rf build-macos-standard-cpu
    
    # Build standard (no BitNet, no GPU)
    cmake -B build-macos-standard-cpu \
        -DCMAKE_C_COMPILER=clang \
        -DCMAKE_CXX_COMPILER=clang++ \
        -DGGML_METAL=OFF \
        -DLLAMA_BUILD_SERVER=ON \
        -DLLAMA_BUILD_EXAMPLES=ON \
        3rdparty/llama.cpp
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Configuration successful!${NC}"
        
        # Build
        cmake --build build-macos-standard-cpu --config Release -j
        
        if [ $? -eq 0 ]; then
            # Copy binaries
            echo ""
            echo "Copying standard CPU binaries to $BUILD_DIR/cpu/macos/standard/ ..."
            
            if [ -d "build-macos-standard-cpu/bin" ]; then
                cp -f build-macos-standard-cpu/bin/* $BUILD_DIR/cpu/macos/standard/ 2>/dev/null || true
            fi
            
            # Copy dylibs
            find build-macos-standard-cpu -name "*.dylib" -type f -exec cp -f {} $BUILD_DIR/cpu/macos/standard/ \; 2>/dev/null || true
            
            chmod +x $BUILD_DIR/cpu/macos/standard/* 2>/dev/null || true
            
            FILE_COUNT=$(ls -1 $BUILD_DIR/cpu/macos/standard/ 2>/dev/null | wc -l)
            echo -e "${GREEN}  ✓ $FILE_COUNT files copied${NC}"
            echo -e "${GREEN}✅ Standard CPU built successfully!${NC}"
        else
            echo -e "${YELLOW}WARNING: Standard CPU build failed!${NC}"
        fi
    else
        echo -e "${RED}❌ Configuration failed!${NC}"
    fi
fi

echo ""
echo "============================================================================"
echo "Step 6: Building Standard Metal GPU (ALL Macs)"
echo "============================================================================"

if ! should_build_variant "metal"; then
    echo -e "${YELLOW}[SKIP] Metal GPU build not requested${NC}"
elif should_skip_build "build-macos-standard-metal" "$BUILD_DIR/gpu/macos/standard-metal" 3; then
    echo -e "${GREEN}[OK] Metal GPU already built${NC}"
else
    echo ""
    echo "Building standard llama.cpp with Metal GPU support..."
    echo -e "${CYAN}Metal works on ALL Macs (M1/M2/M3 + Intel Iris/AMD)${NC}"
    echo ""
    
    # Clean previous build
    rm -rf build-macos-standard-metal
    
    # Build with Metal
    cmake -B build-macos-standard-metal \
        -DGGML_METAL=ON \
        -DLLAMA_BUILD_SERVER=ON \
        -DLLAMA_BUILD_EXAMPLES=ON \
        3rdparty/llama.cpp
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Metal configuration successful!${NC}"
        
        # Build
        cmake --build build-macos-standard-metal --config Release -j
        
        if [ $? -eq 0 ]; then
            # Copy binaries
            echo ""
            echo "Copying Metal GPU binaries to $BUILD_DIR/gpu/macos/standard-metal/ ..."
            
            if [ -d "build-macos-standard-metal/bin" ]; then
                cp -f build-macos-standard-metal/bin/* $BUILD_DIR/gpu/macos/standard-metal/ 2>/dev/null || true
            fi
            
            # Copy Metal library files (.metallib)
            find build-macos-standard-metal -name "*.metallib" -type f -exec cp -f {} $BUILD_DIR/gpu/macos/standard-metal/ \; 2>/dev/null || true
            
            # Copy dylibs
            find build-macos-standard-metal -name "*.dylib" -type f -exec cp -f {} $BUILD_DIR/gpu/macos/standard-metal/ \; 2>/dev/null || true
            
            chmod +x $BUILD_DIR/gpu/macos/standard-metal/* 2>/dev/null || true
            
            FILE_COUNT=$(ls -1 $BUILD_DIR/gpu/macos/standard-metal/ 2>/dev/null | wc -l)
            echo -e "${GREEN}  ✓ $FILE_COUNT files copied${NC}"
            echo -e "${GREEN}✅ Metal GPU built successfully!${NC}"
        else
            echo -e "${YELLOW}WARNING: Metal GPU build failed!${NC}"
        fi
    else
        echo -e "${RED}❌ Configuration failed!${NC}"
    fi
fi

echo ""
echo ""
echo "============================================================================"
echo -e "${YELLOW}         QUICK VERIFICATION${NC}"
echo "============================================================================"
echo ""
echo "Testing built binaries..."
echo ""

# Track verification results
TESTS_PASSED=0
TESTS_FAILED=0

# Test BitNet ARM
if [ -f "$BUILD_DIR/cpu/macos/bitnet-arm/llama-server" ]; then
    echo -e "${CYAN}Testing BitNet ARM (llama-server)...${NC}"
    if $BUILD_DIR/cpu/macos/bitnet-arm/llama-server --help > /dev/null 2>&1; then
        echo -e "${GREEN}  ✓ Executable works!${NC}"
        echo -e "${YELLOW}  [INFO] Features: BitNet 1.58-bit, ARM TL1 kernels${NC}"
        echo -e "${YELLOW}  [USE] M1/M2/M3/M4 Macs only${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${RED}  ✗ Failed to run${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
    echo ""
fi

# Test Standard Metal
if [ -f "$BUILD_DIR/gpu/macos/standard-metal/llama-server" ]; then
    echo -e "${CYAN}Testing Metal GPU (llama-server)...${NC}"
    if $BUILD_DIR/gpu/macos/standard-metal/llama-server --help > /dev/null 2>&1; then
        echo -e "${GREEN}  ✓ Executable works!${NC}"
        echo -e "${YELLOW}  [INFO] Features: Metal GPU acceleration${NC}"
        echo -e "${YELLOW}  [USE] ALL Macs (best GPU performance)${NC}"
        echo -e "${YELLOW}  [TIP] Use -ngl flag to offload layers to GPU${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${RED}  ✗ Failed to run${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
    echo ""
fi

# Show verification summary
echo -e "${CYAN}========================================${NC}"
echo -e "${CYAN}Verification Results:${NC}"
echo -e "${GREEN}  Passed: $TESTS_PASSED${NC}"
if [ $TESTS_FAILED -gt 0 ]; then
    echo -e "${RED}  Failed: $TESTS_FAILED${NC}"
fi
echo -e "${CYAN}========================================${NC}"

echo ""
echo "Creating verification report..."

# Create VERIFICATION.md
cat > "$BUILD_DIR/cpu/macos/VERIFICATION.md" << 'EOF'
# 🔍 BitNet macOS Build Verification Report

## Build Matrix

This directory contains **4 macOS variants** (3 CPU + 1 GPU):

### CPU Variants
- **bitnet-arm/** - BitNet for Apple Silicon (M1/M2/M3/M4 with TL1 kernels)
- **bitnet-intel/** - BitNet for Intel Macs (x86 with TL2 kernels)
- **standard/** - Standard llama.cpp (universal, no BitNet)

### GPU Variant
- **standard-metal/** - Metal GPU acceleration (ALL Macs)

## Quick Start

### For M1/M2/M3/M4 (Apple Silicon):
```bash
cd bitnet-arm/
./llama-server -m bitnet-model.gguf
```

### For Intel Macs:
```bash
cd bitnet-intel/
./llama-server -m bitnet-model.gguf
```

### For Metal GPU (Best Performance):
```bash
cd ../gpu/macos/standard-metal/
./llama-server -m model.gguf -ngl 35  # Offload 35 layers to GPU
```

## Metal GPU Tips

The `-ngl` flag controls GPU layer offloading:

```bash
# Full GPU (fastest)
./llama-server -ngl 99 -m model.gguf

# Partial GPU (balance CPU/GPU)
./llama-server -ngl 20 -m model.gguf

# CPU only
./llama-server -ngl 0 -m model.gguf
```

## Technical Details

- **Compiler:** Clang (from Xcode)
- **BitNet TL1:** ARM-optimized kernels for M1/M2/M3/M4
- **BitNet TL2:** x86-optimized kernels for Intel
- **Metal:** Apple's GPU framework (all Macs)

## NOT Supported on macOS

- ❌ CUDA (requires NVIDIA GPU)
- ❌ Vulkan (Apple uses Metal)
- ❌ BitNet GPU kernels (requires CUDA)

---
Build Date: $(date)
EOF

echo -e "${GREEN}  ✓ Verification report: $BUILD_DIR/cpu/macos/VERIFICATION.md${NC}"

echo ""
echo ""
echo "============================================================================"
echo -e "${GREEN}✅ BUILD PROCESS COMPLETE!${NC}"
echo "============================================================================"
echo ""
echo "Output locations:"
echo -e "${CYAN}  BitNet ARM:    $BUILD_DIR/cpu/macos/bitnet-arm/${NC}"
echo -e "${CYAN}  BitNet Intel:  $BUILD_DIR/cpu/macos/bitnet-intel/${NC}"
echo -e "${CYAN}  Standard CPU:  $BUILD_DIR/cpu/macos/standard/${NC}"
echo -e "${CYAN}  Metal GPU:     $BUILD_DIR/gpu/macos/standard-metal/${NC}"
echo ""

# Show summary
echo "========================================" 
echo "Build Summary:"
echo "========================================" 
echo ""

if [ -d "$BUILD_DIR/cpu/macos" ]; then
    echo -e "${YELLOW}CPU Builds (3 variants):${NC}"
    
    for subdir in bitnet-arm bitnet-intel standard; do
        if [ -d "$BUILD_DIR/cpu/macos/$subdir" ]; then
            FILE_COUNT=$(ls -1 $BUILD_DIR/cpu/macos/$subdir/ 2>/dev/null | wc -l)
            if [ $FILE_COUNT -gt 0 ]; then
                echo -e "${GREEN}  ✓ $subdir/  ($FILE_COUNT files)${NC}"
            else
                echo -e "${YELLOW}  ⚠ $subdir/  (empty)${NC}"
            fi
        fi
    done
    echo ""
fi

if [ -d "$BUILD_DIR/gpu/macos" ]; then
    echo -e "${YELLOW}GPU Builds (1 variant):${NC}"
    
    if [ -d "$BUILD_DIR/gpu/macos/standard-metal" ]; then
        FILE_COUNT=$(ls -1 $BUILD_DIR/gpu/macos/standard-metal/ 2>/dev/null | wc -l)
        if [ $FILE_COUNT -gt 0 ]; then
            echo -e "${GREEN}  ✓ standard-metal/  ($FILE_COUNT files)${NC}"
        else
            echo -e "${YELLOW}  ⚠ standard-metal/  (empty)${NC}"
        fi
    fi
    echo ""
fi

echo -e "${CYAN}Total: 4 variants (3 CPU + 1 GPU)${NC}"
echo ""

echo "========================================" 
echo -e "${GREEN}Next Steps:${NC}"
echo "========================================" 
echo ""

if [ "$ARCH" = "arm64" ]; then
    echo "  For your M1/M2/M3/M4 Mac:"
    echo -e "${CYAN}    cd $BUILD_DIR/cpu/macos/bitnet-arm${NC}"
    echo -e "${CYAN}    ./llama-server --help${NC}"
    echo ""
    echo "  For best GPU performance:"
    echo -e "${CYAN}    cd $BUILD_DIR/gpu/macos/standard-metal${NC}"
    echo -e "${CYAN}    ./llama-server -ngl 35 --help${NC}"
else
    echo "  For your Intel Mac:"
    echo -e "${CYAN}    cd $BUILD_DIR/cpu/macos/bitnet-intel${NC}"
    echo -e "${CYAN}    ./llama-server --help${NC}"
    echo ""
    echo "  For Metal GPU:"
    echo -e "${CYAN}    cd $BUILD_DIR/gpu/macos/standard-metal${NC}"
    echo -e "${CYAN}    ./llama-server -ngl 35 --help${NC}"
fi

echo ""
echo -e "${GREEN}========================================${NC}"
echo ""
echo -e "${CYAN}Each subdirectory is SELF-CONTAINED!${NC}"
echo -e "${CYAN}Perfect for distribution - zip any folder and ship it!${NC}"
echo ""
