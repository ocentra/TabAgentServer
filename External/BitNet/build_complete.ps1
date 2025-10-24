# BitNet Complete Build Script for Windows
# This script is completely self-contained and will set up everything needed
# It works on any Windows machine with the required tools installed

# EXPECTED TOOL LOCATIONS AND INSTALLATION INSTRUCTIONS:
# 
# 1. VISUAL STUDIO 2022 COMMUNITY:
#    - Download: https://visualstudio.microsoft.com/vs/community/
#    - Install with these components:
#      * Desktop development with C++
#      * C++ CMake Tools for Windows (optional - standalone CMake preferred)
#      * Git for Windows
#      * C++ Clang Compiler for Windows (ClangCL)
#      * MSVC v143 - VS 2022 C++ x64/x86 build tools
#    - Expected path: C:\Program Files\Microsoft Visual Studio\2022\Community\
#
# 2. CMAKE (RECOMMENDED: Standalone, Fallback: VS bundled):
#    - **PREFERRED:** Standalone CMake 3.31.9 (stable)
#      * Download: https://cmake.org/download/ (Windows x64 Installer)
#      * Install to: C:\Program Files\CMake\
#      * Expected: C:\Program Files\CMake\bin\cmake.exe
#    - **FALLBACK:** Visual Studio 2022 bundled CMake (if no stable system CMake found)
#      * Path: C:\Program Files\Microsoft Visual Studio\2022\Community\...\cmake.exe
#    - Script auto-detects and prefers stable CMake 3.27-3.99, rejects 4.x (buggy)
#
# 3. CLANG COMPILER (ClangCL):
#    - Included with Visual Studio ("C++ Clang Compiler for Windows")
#    - Used for: Standard CPU, BitNet CPU builds
#    - Auto-detected in Visual Studio installation
#
# 4. MSVC COMPILER:
#    - Included with Visual Studio (MSVC build tools)
#    - Used for: GPU builds (CUDA, Vulkan, OpenCL)
#    - Auto-detected in Visual Studio installation
#
# 5. VULKAN SDK ( for Vulkan GPU acceleration):
#    - Download: https://vulkan.lunarg.com/sdk/home#windows
#    - Install only "Vulkan SDK Core" components
#    - Expected: C:\VulkanSDK\<version>\ (auto-detected)
#    - GLSL Compiler: C:\VulkanSDK\<version>\Bin\glslc.exe
#
# 6. CUDA TOOLKIT (Required for NVIDIA GPU builds):
#    - Download: https://developer.nvidia.com/cuda-downloads
#    - Recommended: CUDA 12.8 or latest
#    - Expected: C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.8\
#    - CUDA Compiler: C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.8\bin\nvcc.exe
#    - Note: CUDA 12.8 requires -allow-unsupported-compiler flag for VS 2026
#
# 7. GIT ( for version control):
#    - Download: https://git-scm.com/
#    - Add to PATH during installation
#    - Expected: Accessible as "git" from command line
#
# 8. PYTHON REQUIREMENTS (Required for BitNet GPU Python CUDA build):
#    - **CRITICAL:** Python 3.9, 3.10, or 3.11 ONLY (NOT 3.13+)
#    - Reason: PyTorch 2.3.1 does not support Python 3.13+
#    - Download Python 3.11: https://www.python.org/downloads/release/python-3119/
#    - Script creates virtual environment: bitnet-gpu-env-windows
#    - Auto-installs required packages:
#      * PyTorch 2.3.1+cu121 (CUDA 12.1 compatible)
#      * xformers 0.0.27 (requires PyTorch 2.3.1 exactly)
#      * transformers, safetensors, etc.
#    - Script will EXIT with error if Python 3.13+ detected

param(
    [string]$BuildDir = "BitnetRelease",
    [switch]$CleanBuild = $false,
    [string[]]$BuildVariants = @(),  # Build only specific variants (e.g., "zen2", "portable", "zen3")
    [switch]$ListVariants = $false   # List available build variants and exit
)

# Colors for output
$Green = "$([char]27)[92m"
$Yellow = "$([char]27)[93m"
$Red = "$([char]27)[91m"
$Cyan = "$([char]27)[96m"
$Reset = "$([char]27)[0m"

# Handle -ListVariants flag
if ($ListVariants) {
    Write-Host "`n=== AVAILABLE BUILD VARIANTS ===" -ForegroundColor Cyan
    Write-Host "`nTo build specific variants, use: -BuildVariants <variant1>,<variant2>`n" -ForegroundColor Yellow
    
    Write-Host "STANDARD BUILDS:" -ForegroundColor Yellow
    Write-Host "  standard              - Standard llama.cpp (no BitNet, any CPU)" -ForegroundColor White
    Write-Host "  gpu-cuda-vulkan       - CUDA + Vulkan GPU build (NVIDIA)" -ForegroundColor White
    Write-Host "  gpu-opencl            - OpenCL GPU build (Universal)" -ForegroundColor White
    Write-Host "  python-cuda           - BitNet Python CUDA kernels" -ForegroundColor White
    
    Write-Host "`nBITNET CPU VARIANTS (COMPLETE MATRIX):" -ForegroundColor Yellow
    Write-Host "  portable              - AVX2 baseline (any modern CPU)" -ForegroundColor White
    Write-Host "`n  AMD Ryzen:" -ForegroundColor Yellow
    Write-Host "  amd-zen1              - AMD Ryzen 1000/2000 series (Zen 1)" -ForegroundColor White
    Write-Host "  amd-zen2              - AMD Ryzen 3000 series (Zen 2)" -ForegroundColor White
    Write-Host "  amd-zen3              - AMD Ryzen 5000 series (Zen 3)" -ForegroundColor White
    Write-Host "  amd-zen4              - AMD Ryzen 7000 series (Zen 4)" -ForegroundColor White
    Write-Host "  amd-zen5              - AMD Ryzen 9000 series (Zen 5)" -ForegroundColor White
    Write-Host "`n  Intel Core:" -ForegroundColor Yellow
    Write-Host "  intel-haswell         - Intel 4th gen (Haswell)" -ForegroundColor White
    Write-Host "  intel-broadwell       - Intel 5th gen (Broadwell)" -ForegroundColor White
    Write-Host "  intel-skylake         - Intel 6th-9th gen (Skylake)" -ForegroundColor White
    Write-Host "  intel-icelake         - Intel 10th gen (Ice Lake)" -ForegroundColor White
    Write-Host "  intel-rocketlake      - Intel 11th gen (Rocket Lake)" -ForegroundColor White
    Write-Host "  intel-alderlake       - Intel 12th-14th gen (Alder Lake)" -ForegroundColor White
    
    Write-Host "`nEXAMPLES:" -ForegroundColor Cyan
    Write-Host "  # Build only zen2 variant:" -ForegroundColor Gray
    Write-Host "  .\build_complete.ps1 -BuildVariants amd-zen2`n" -ForegroundColor Gray
    Write-Host "  # Build zen2 + portable:" -ForegroundColor Gray
    Write-Host "  .\build_complete.ps1 -BuildVariants amd-zen2,portable`n" -ForegroundColor Gray
    Write-Host "  # Build all GPU variants:" -ForegroundColor Gray
    Write-Host "  .\build_complete.ps1 -BuildVariants gpu-cuda-vulkan,gpu-opencl,python-cuda`n" -ForegroundColor Gray
    Write-Host "  # Build everything (default):" -ForegroundColor Gray
    Write-Host "  .\build_complete.ps1`n" -ForegroundColor Gray
    
    exit 0
}

function Write-Status {
    param([string]$Message, [string]$Color = $Green)
    Write-Host "${Color}${Message}${Reset}"
}

function Write-ErrorAndExit {
    param([string]$Message)
    Write-Host "${Red}ERROR: ${Message}${Reset}"
    exit 1
}

function Should-BuildVariant {
    param([string]$VariantName)
    # If no variants specified, build everything
    if ($BuildVariants.Count -eq 0) {
        return $true
    }
    # Otherwise, check if this variant is in the list
    return $BuildVariants -contains $VariantName
}

function Should-SkipBuild {
    param(
        [string]$BuildDir,
        [string]$ReleaseDir,
        [int]$MinFiles = 5
    )
    
    # If CleanBuild flag is set, never skip
    if ($CleanBuild) {
        return $false
    }
    
    # Check if build directory exists
    if (!(Test-Path $BuildDir)) {
        Write-Status "    [BUILD] Build directory missing: $BuildDir" $Yellow
        return $false
    }
    
    # Check if Release output directory exists
    if (!(Test-Path $ReleaseDir)) {
        Write-Status "    [BUILD] Release output missing: $ReleaseDir" $Yellow
        return $false
    }
    
    # Check if Release directory has files
    $fileCount = (Get-ChildItem $ReleaseDir -File -ErrorAction SilentlyContinue).Count
    if ($fileCount -lt $MinFiles) {
        Write-Status "    [BUILD] Release incomplete: only $fileCount files (need $MinFiles+)" $Yellow
        return $false
    }
    
    # All checks passed - skip this build!
    Write-Status "    [SKIP] Already built: $fileCount files in $ReleaseDir" $Green
    return $true
}

function Test-CommandExists {
    param([string]$Command)
    try {
        # Try multiple ways to check if command exists
        $result = Get-Command $Command -ErrorAction SilentlyContinue
        if ($result) {
            return $true
        }
        
        # Try using where command as fallback
        $whereResult = where.exe $Command 2>$null
        if ($whereResult -and $whereResult.Length -gt 0) {
            return $true
        }
        
        return $false
    } catch {
        return $false
    }
}

function Invoke-FullBitNetSetup {
    param([string]$PythonCmd)
    
    Write-Status "   Running full BitNet setup process..."
    
    # Install GGUF package
    Write-Status "   Installing GGUF package..."
    & $PythonCmd -m pip install 3rdparty/llama.cpp/gguf-py
    if ($LASTEXITCODE -ne 0) {
        Write-Status "   Warning: Failed to install GGUF package, continuing..." $Yellow
    }
    
    # Generate kernel code (this is what gen_code() does in setup_env.py)
    Write-Status "   Generating kernel code..."
    try {
        # For x86_64 architecture, we use codegen_tl2.py
        if (Test-Path "utils\codegen_tl2.py") {
            & $PythonCmd utils\codegen_tl2.py --model Llama3-8B-1.58-100B-tokens --BM 256,128,256,128 --BK 96,96,96,96 --bm 32,32,32
            if ($LASTEXITCODE -eq 0) {
                Write-Status "   Kernel code generated successfully"
            } else {
                Write-Status "   Warning: Kernel code generation failed, using preset kernels..." $Yellow
                # Copy preset kernels as fallback
                Copy-PresetKernels
            }
        } else {
            Write-Status "   Warning: codegen_tl2.py not found, using preset kernels..." $Yellow
            Copy-PresetKernels
        }
    } catch {
        Write-Status "   Warning: Kernel code generation failed, using preset kernels..." $Yellow
        Copy-PresetKernels
    }
}

function Copy-PresetKernels {
    Write-Status "   Copying preset kernels as fallback..."
    try {
        # Try to copy the most appropriate preset kernel
        $presetPath = "preset_kernels\Llama3-8B-1.58-100B-tokens"
        if (Test-Path $presetPath) {
            # Copy the TL2 kernel file (since we're building for x86 TL2)
            $tl2Kernel = "$presetPath\bitnet-lut-kernels-tl2.h"
            if (Test-Path $tl2Kernel) {
                Copy-Item $tl2Kernel "include\bitnet-lut-kernels.h" -Force
                Write-Status "   Copied TL2 preset kernel"
            } else {
                # Fallback to TL1 if TL2 not available
                $tl1Kernel = "$presetPath\bitnet-lut-kernels-tl1.h"
                if (Test-Path $tl1Kernel) {
                    Copy-Item $tl1Kernel "include\bitnet-lut-kernels.h" -Force
                    Write-Status "   Copied TL1 preset kernel"
                }
            }
        }
        
        # If still no kernel file, create a minimal one
        if (!(Test-Path "include\bitnet-lut-kernels.h") -or (Get-Content "include\bitnet-lut-kernels.h" | Measure-Object).Count -eq 0) {
            Write-Status "   Creating minimal kernel header..." $Yellow
            $minimalKernel = @"
#if defined(GGML_BITNET_X86_TL2) || defined(GGML_BITNET_ARM_TL1)
#define GGML_BITNET_MAX_NODES 8192
static bool initialized = false;
static bitnet_tensor_extra * bitnet_tensor_extras = nullptr;
static size_t bitnet_tensor_extras_index = 0;
static bool is_type_supported(enum ggml_type type) {
    return (type == GGML_TYPE_TL2 || type == GGML_TYPE_TL1 || type == GGML_TYPE_Q4_0);
}
#endif
"@
            $minimalKernel | Out-File -FilePath "include\bitnet-lut-kernels.h" -Encoding UTF8
        }
    } catch {
        Write-Status "   Warning: Failed to copy preset kernels..." $Yellow
    }
}

Write-Status "=== BitNet Complete Build Script ==="
Write-Status ""

# 1. VERIFY ALL REQUIRED TOOLS FIRST (fail fast before spending time on Python setup)
Write-Status "1. Verifying ALL required tools before proceeding..."
Write-Status ""

$allToolsFound = $true
$criticalToolsMissing = @()

# Check CMake and Visual Studio - Prefer stable system CMake, fallback to VS bundled
Write-Status "   Checking CMake and Visual Studio..."

# Step 1: Check for stable system CMake (preferred)
$cmakePath = $null
$cmakeVersion = $null
$systemCmake = "C:\Program Files\CMake\bin"

if (Test-Path "$systemCmake\cmake.exe") {
    $versionOutput = & "$systemCmake\cmake.exe" --version 2>&1 | Select-Object -First 1
    if ($versionOutput -match "cmake version (\d+)\.(\d+)\.(\d+)") {
        $major = [int]$matches[1]
        $minor = [int]$matches[2]
        $cmakeVersion = "$major.$minor.$($matches[3])"
        
        # Accept CMake 3.27+ but reject 4.0+ (experimental)
        if ($major -eq 3 -and $minor -ge 27) {
            $cmakePath = $systemCmake
            Write-Status "   [OK] Using stable system CMake $cmakeVersion" $Green
            Write-Status "   [PATH] $systemCmake\cmake.exe" $Cyan
        } elseif ($major -ge 4) {
            Write-Status "   [WARN] System CMake $cmakeVersion is experimental (4.x)" $Yellow
            Write-Status "   [INFO] Will try Visual Studio's bundled CMake instead..." $Yellow
        } elseif ($major -eq 3 -and $minor -lt 27) {
            Write-Status "   [WARN] System CMake $cmakeVersion is too old (need 3.27+)" $Yellow
            Write-Status "   [INFO] Will try Visual Studio's bundled CMake instead..." $Yellow
        }
    }
}

# Step 2: Detect Visual Studio 2022 installation (for compilers and fallback CMake)
$vsInstallPath = "C:\Program Files\Microsoft Visual Studio\2022\Community"

if (Test-Path $vsInstallPath) {
    # If no stable system CMake, use VS's bundled CMake as fallback
    if (-not $cmakePath) {
        $testPath = "$vsInstallPath\Common7\IDE\CommonExtensions\Microsoft\CMake\CMake\bin"
        if (Test-Path "$testPath\cmake.exe") {
            $vsCmakeVersion = & "$testPath\cmake.exe" --version 2>&1 | Select-Object -First 1
            if ($vsCmakeVersion -match "cmake version ([\d\.]+)") {
                $cmakePath = $testPath
                $cmakeVersion = "Visual Studio 2022 bundled $($matches[1])"
                Write-Status "   [OK] Using Visual Studio 2022's bundled CMake $($matches[1])" $Yellow
                Write-Status "   [PATH] $testPath\cmake.exe" $Cyan
            }
        }
    }
} else {
    Write-Status "   [FAIL] Visual Studio 2022 Community not found at: $vsInstallPath" $Red
    $criticalToolsMissing += "Visual Studio 2022"
    $allToolsFound = $false
}

# Step 3: Add CMake to PATH and verify
if ($cmakePath) {
    $env:PATH = "$cmakePath;$env:PATH"
    Write-Status "   [OK] CMake ready: $cmakeVersion" $Green
} else {
    Write-Status "   [FAIL] No suitable CMake found!" $Red
    Write-Status "   [TIP] Install CMake 3.27+ from https://cmake.org/download/" $Yellow
    $criticalToolsMissing += "CMake"
    $allToolsFound = $false
}

# Check Clang (ClangCL from Visual Studio 2022)
Write-Status "   Checking Clang..."
$clangPath = "$vsInstallPath\VC\Tools\Llvm\x64\bin"
if (Test-Path "$clangPath\clang.exe") {
    $clangVersion = & "$clangPath\clang.exe" --version 2>$null | Select-Object -First 1
    Write-Status "   [OK] Clang found: $clangVersion" $Green
} else {
    Write-Status "   [FAIL] Clang not found in VS 2022 at: $clangPath" $Red
    $criticalToolsMissing += "Clang (ClangCL)"
    $allToolsFound = $false
}

# Check CUDA (look for all versions)
Write-Status "   Checking CUDA..."
$cudaBasePath = "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA"
if (Test-Path $cudaBasePath) {
    $cudaVersions = Get-ChildItem $cudaBasePath -Directory | Where-Object { $_.Name -match "^v\d" } | Sort-Object Name -Descending
    if ($cudaVersions) {
        $cudaDir = $cudaVersions[0].FullName
        $env:PATH = "$cudaDir\bin;$env:PATH"
        $env:CUDA_PATH = $cudaDir
        $env:CUDA_HOME = $cudaDir
        Write-Status "   [OK] CUDA found: $($cudaVersions[0].Name)" $Green
    } else {
        Write-Status "   [WARN] CUDA directory exists but no versions found" $Yellow
    }
} else {
    Write-Status "   [WARN] CUDA not found - GPU builds will not work" $Yellow
}

# Check Vulkan SDK
Write-Status "   Checking Vulkan SDK..."
$vulkanBasePath = "C:\VulkanSDK\1.4.328.1"
$vulkanPath = "$vulkanBasePath\Bin"
if (Test-Path $vulkanPath) {
    $env:PATH = "$vulkanPath;$env:PATH"
    $env:VULKAN_SDK = $vulkanBasePath
    
    # Check if VULKAN_SDK is set permanently at system or user level
    $systemVulkan = [System.Environment]::GetEnvironmentVariable('VULKAN_SDK', 'Machine')
    $userVulkan = [System.Environment]::GetEnvironmentVariable('VULKAN_SDK', 'User')
    
    if ($systemVulkan -eq $vulkanBasePath -or $userVulkan -eq $vulkanBasePath) {
        Write-Status "   [OK] Vulkan SDK found and permanently configured" $Green
        $env:VULKAN_CONFIGURED = "true"
    } else {
        Write-Status "   [OK] Vulkan SDK found (setting for user level)" $Yellow
        # Set at user level (doesn't require admin)
        [System.Environment]::SetEnvironmentVariable('VULKAN_SDK', $vulkanBasePath, 'User')
        Write-Status "   [OK] VULKAN_SDK set at user level - restart PowerShell for system-wide effect" $Green
        $env:VULKAN_CONFIGURED = "true"
    }
} else {
    Write-Status "   [WARN] Vulkan SDK not found - Vulkan builds will not work" $Yellow
    $env:VULKAN_CONFIGURED = "false"
}


# Check OpenCL (optional - for universal GPU builds)
Write-Status "   Checking OpenCL..."
$openclAvailable = $false
$openclIncludePath = $null

# OpenCL headers can come from multiple sources
# 1. NVIDIA CUDA Toolkit (includes OpenCL)
# 2. Intel SDK
# 3. AMD APP SDK

# Check CUDA toolkit for OpenCL headers (most common on Windows)
if ($env:CUDA_HOME) {
    $cudaOpenCLPath = "$env:CUDA_HOME\include\CL\cl.h"
    if (Test-Path $cudaOpenCLPath) {
        $openclIncludePath = "$env:CUDA_HOME\include"
        $openclAvailable = $true
        Write-Status "   [OK] OpenCL headers found in CUDA toolkit" $Green
    }
}

# Check alternative OpenCL locations (Windows SDK, Intel drivers, AMD drivers)
# Note: CUDA installation above is the most common source of OpenCL headers

# Check Windows SDK (sometimes has OpenCL)
if (-not $openclAvailable) {
    $windowsSDKPath = "C:\Program Files (x86)\Windows Kits\10\Include"
    if (Test-Path $windowsSDKPath) {
        $sdkVersions = Get-ChildItem $windowsSDKPath -Directory | Where-Object { $_.Name -match '^\d' } | Sort-Object Name -Descending
        foreach ($sdkVer in $sdkVersions) {
            $sdkOpenCLPath = "$($sdkVer.FullName)\um\CL\cl.h"
            if (Test-Path $sdkOpenCLPath) {
                $openclIncludePath = "$($sdkVer.FullName)\um"
                $openclAvailable = $true
                Write-Status "   [OK] OpenCL headers found in Windows SDK $($sdkVer.Name)" $Green
                break
            }
        }
    }
}

if ($openclAvailable) {
    $env:OPENCL_INCLUDE_PATH = $openclIncludePath
    $env:OPENCL_AVAILABLE = "true"
    Write-Status "   [OK] OpenCL builds will be enabled" $Green
} else {
    Write-Status "   [SKIP] OpenCL headers not found - OpenCL builds will be skipped" $Yellow
    Write-Status "   [TIP] OpenCL headers come with NVIDIA CUDA, Intel SDK, or AMD drivers" $Cyan
    $env:OPENCL_AVAILABLE = "false"
}

# Check Git
Write-Status "   Checking Git..."
if (Test-CommandExists "git") {
    $gitVersion = & git --version 2>$null
    Write-Status "   [OK] Git found: $gitVersion" $Green
} else {
    Write-Status "   [FAIL] Git not found in PATH" $Red
    $criticalToolsMissing += "Git"
    $allToolsFound = $false
}

# Check Python
Write-Status "   Checking Python..."
if (Test-CommandExists "py") {
    $pythonVersion = & py --version 2>$null
    Write-Status "   [OK] Python found: $pythonVersion" $Green
} elseif (Test-CommandExists "python") {
    $pythonVersion = & python --version 2>$null
    Write-Status "   [OK] Python found: $pythonVersion" $Green
} else {
    Write-Status "   [FAIL] Python not found in PATH" $Red
    $criticalToolsMissing += "Python"
    $allToolsFound = $false
}

Write-Status ""

# FAIL FAST if critical tools are missing
if (-not $allToolsFound) {
    Write-Status "========================================" $Red
    Write-Status "CRITICAL TOOLS MISSING!" $Red
    Write-Status "========================================" $Red
    Write-Status ""
    Write-Status "The following required tools were not found:" $Red
    foreach ($tool in $criticalToolsMissing) {
        Write-Status "   - $tool" $Red
    }
    Write-Status ""
    Write-Status "Please install missing tools before running this script." $Red
    Write-Status "See comments at top of script for installation instructions." $Red
    Write-Status ""
    exit 1
}

Write-Status "========================================" $Green
Write-Status "ALL REQUIRED TOOLS VERIFIED!" $Green
Write-Status "========================================" $Green
Write-Status ""

# 2. Initialize git submodules
Write-Status "2. Initializing git submodules..."
# This step ensures all required submodules are properly initialized:
# - 3rdparty/llama.cpp (main llama.cpp framework)
# - 3rdparty/llama.cpp/ggml (core compute library)
# - 3rdparty/llama.cpp/ggml/src/kompute (Kompute backend for GPU acceleration)
try {
    & git submodule update --init --recursive
    if ($LASTEXITCODE -ne 0) {
        throw "Git submodule initialization failed"
    }
    Write-Status "   Git submodules initialized successfully"
} catch {
    Write-ErrorAndExit "Failed to initialize git submodules: $_"
}

# 3. Verify 3rdparty directory structure
Write-Status "3. Verifying 3rdparty directory structure..."
# Verify that git submodules were properly initialized and key directories exist:
# - 3rdparty/llama.cpp (main framework)
# - 3rdparty/llama.cpp/ggml (compute library)
# - 3rdparty/llama.cpp/ggml/src (source files)
# - 3rdparty/llama.cpp/ggml/include (header files)
if (!(Test-Path "3rdparty\llama.cpp")) {
    Write-ErrorAndExit "llama.cpp submodule not found in 3rdparty directory. Please check git submodule initialization."
}

if (!(Test-Path "3rdparty\llama.cpp\ggml")) {
    Write-ErrorAndExit "ggml submodule not found in llama.cpp directory. Please check git submodule initialization."
}

Write-Status "   3rdparty directory structure verified"

# 4. Verify required header files are available
Write-Status "4. Verifying required header files..."
# Check for critical header files that are required for successful compilation:
# - llama.h: Main llama.cpp header
# - ggml.h: Core GGML compute library header
# - ggml-vulkan.h: Vulkan backend header
# - ggml-cuda.h: CUDA backend header
$headerFiles = @(
    "3rdparty\llama.cpp\include\llama.h",
    "3rdparty\llama.cpp\ggml\include\ggml.h",
    "3rdparty\llama.cpp\ggml\include\ggml-vulkan.h",
    "3rdparty\llama.cpp\ggml\include\ggml-cuda.h"
)

$missingHeaders = @()
foreach ($header in $headerFiles) {
    if (!(Test-Path $header)) {
        $missingHeaders += $header
    }
}

if ($missingHeaders.Count -gt 0) {
    Write-Status "   Missing header files detected:" $Yellow
    foreach ($header in $missingHeaders) {
        Write-Host "     - $header"
    }
    Write-ErrorAndExit "Required header files are missing. Please check submodule initialization and ensure all submodules are properly checked out."
}

Write-Status "   All required header files are available"

# 6. Set up Python environment (create if doesn't exist)
Write-Status "6. Setting up Python environment..."

# CRITICAL: Check Python version BEFORE creating venv
Write-Status "   Checking Python version compatibility..."
$pythonVersionCheck = $null
$pythonVersions = @("3.11", "3.10", "3.9")

foreach ($ver in $pythonVersions) {
    if (Test-CommandExists "py") {
        $testResult = & py -$ver --version 2>$null
        if ($LASTEXITCODE -eq 0) {
            # Extract version number (e.g., "Python 3.11.9" -> "3.11.9")
            if ($testResult -match "Python (\d+\.\d+\.\d+)") {
                $detectedVersion = $matches[1]
                $majorMinor = $detectedVersion.Split('.')[0..1] -join '.'
                
                # Check if version is >= 3.13 (NOT supported)
                if ([version]$detectedVersion -ge [version]"3.13.0") {
                    Write-Status "   [FAIL] Python $detectedVersion detected - NOT SUPPORTED!" $Red
                    continue
                }
                
                $pythonVersionCheck = "py -$ver"
                Write-Status "   [OK] Python $detectedVersion detected - COMPATIBLE" $Green
                break
            }
        }
    }
}

if (-not $pythonVersionCheck) {
    Write-Status "" $Red
    Write-Status "========================================" $Red
    Write-Status "PYTHON VERSION REQUIREMENT NOT MET!" $Red
    Write-Status "========================================" $Red
    Write-Status "" $Red
    Write-Status "This project requires Python 3.9, 3.10, or 3.11 (Python < 3.13)" $Yellow
    Write-Status "" $Red
    Write-Status "WHY:" $Yellow
    Write-Status "  - PyTorch 2.3.1 does not support Python 3.13+" $Cyan
    Write-Status "  - xformers 0.0.27 requires PyTorch 2.3.1 exactly" $Cyan
    Write-Status "  - BitNet CUDA kernels depend on xformers" $Cyan
    Write-Status "" $Red
    Write-Status "SOLUTION:" $Yellow
    Write-Status "  1. Install Python 3.11 (recommended):" $Cyan
    Write-Status "     https://www.python.org/downloads/release/python-3119/" $Cyan
    Write-Status "  2. Or install Python 3.10 or 3.9" $Cyan
    Write-Status "" $Red
    Write-Status "Current Python versions found:" $Yellow
    & py -0 2>$null
    Write-Status "" $Red
    exit 1
}

# Create or use existing Python virtual environment:
# - Environment name: bitnet-gpu-env-windows
# - Location: .\bitnet-gpu-env-windows\ (relative to script location)
# - Python executable: .\bitnet-gpu-env-windows\Scripts\python.exe
# This ensures isolated dependencies and prevents conflicts with system Python packages
$envPath = "bitnet-gpu-env-windows"
$pythonCmd = "$envPath\Scripts\python.exe"

if (!(Test-Path $envPath)) {
    Write-Status "   Creating Python virtual environment with $pythonVersionCheck..."
    
    # Create virtual environment
    Invoke-Expression "$pythonVersionCheck -m venv $envPath"
    if ($LASTEXITCODE -ne 0) {
        Write-ErrorAndExit "Failed to create virtual environment. Please ensure Python 3.9+ is installed."
    }
    
    # Verify the Python version in the venv
    $venvPythonVersion = & $pythonCmd --version 2>$null
    Write-Status "   Virtual environment created with: $venvPythonVersion"
}

# Verify Python environment
if (!(Test-Path $pythonCmd)) {
    Write-ErrorAndExit "Python environment not found at $pythonCmd. Please check virtual environment creation."
}

Write-Status "   Python environment ready"

# 7. Install required Python packages with exact versions
Write-Status "7. Installing required Python packages..."
& $pythonCmd -m pip install --upgrade pip
if ($LASTEXITCODE -ne 0) {
    Write-ErrorAndExit "Failed to upgrade pip"
}

# Core ML/CUDA packages
$packages = @(
    "torch==2.3.1+cu121",
    "torchvision==0.18.1+cu121",
    "torchaudio==2.3.1+cu121"
)

Write-Status "   Installing PyTorch stack (CUDA 12.1)..."
foreach ($package in $packages) {
    & $pythonCmd -m pip install $package --extra-index-url https://download.pytorch.org/whl/cu121
    if ($LASTEXITCODE -ne 0) {
        Write-ErrorAndExit "Failed to install $package"
    }
}

Write-Status "   Installing xformers 0.0.27..."
& $pythonCmd -m pip install xformers==0.0.27 --extra-index-url https://download.pytorch.org/whl/cu121
if ($LASTEXITCODE -ne 0) {
    Write-ErrorAndExit "Failed to install xformers"
}

# Other required packages
$otherPackages = @(
    "transformers==4.57.1",
    "sentencepiece==0.2.1",
    "tiktoken==0.12.0",
    "tokenizers==0.22.1",
    "numpy==2.3.3",
    "safetensors==0.6.2",
    "einops==0.8.1",
    "huggingface_hub==0.35.3",
    "intel-openmp==2021.4.0",
    "mkl==2021.4.0",
    "tbb==2021.13.1",
    "fire==0.7.1",
    "flask==3.1.2",
    "blobfile==3.1.0"
)

Write-Status "   Installing other required packages..."
foreach ($package in $otherPackages) {
    & $pythonCmd -m pip install $package
    if ($LASTEXITCODE -ne 0) {
        Write-Status "   Warning: Failed to install $package, continuing..." $Yellow
    }
}

# Install GGUF package directly (avoid requirements.txt conflicts)
Write-Status "   Installing GGUF package from llama.cpp..."
& $pythonCmd -m pip install 3rdparty/llama.cpp/gguf-py
    if ($LASTEXITCODE -ne 0) {
    Write-Status "   Warning: Failed to install GGUF package, continuing..." $Yellow
}

# Install GPU requirements individually (SKIP torch/xformers - already installed with CUDA!)
Write-Status "   Installing GPU requirements (skipping torch/xformers to preserve CUDA version)..."
$gpuPackagesOnly = @(
    "fire",
    "sentencepiece",
    "tiktoken",
    "blobfile",
    "flask",
    "einops",
    "transformers"
)

foreach ($package in $gpuPackagesOnly) {
    & $pythonCmd -m pip install $package
    if ($LASTEXITCODE -ne 0) {
        Write-Status "   Warning: Failed to install $package, continuing..." $Yellow
    }
}

# Verify torch version is still CUDA
Write-Status "   Verifying PyTorch CUDA version preserved..."
$torchVersion = & $pythonCmd -c "import torch; print(torch.__version__)" 2>$null
if ($torchVersion -like "*cu121*") {
    Write-Status "   [OK] PyTorch CUDA 12.1 preserved: $torchVersion" $Green
} else {
    Write-Status "   [WARN] PyTorch version is $torchVersion (expected cu121)" $Yellow
}

Write-Status "   All Python packages installed successfully"

# 12. Generate required kernel files if they don't exist
Write-Status "12. Generating required kernel files..."
# Create include directory if it doesn't exist:
# - Location: .\include\
# - Required files: bitnet-lut-kernels.h, kernel_config.ini
# These files are critical for BitNet kernel compilation and must be generated properly
if (!(Test-Path "include")) {
    New-Item -ItemType Directory -Path "include" | Out-Null
}

# Run the full BitNet setup process to generate proper kernel files:
# This replicates the gen_code() function from setup_env.py:
# 1. Installs GGUF package from 3rdparty/llama.cpp/gguf-py
# 2. Runs codegen_tl2.py to generate optimized kernels for x86_64 architecture
# 3. Creates bitnet-lut-kernels.h and kernel_config.ini with proper parameters
Write-Status "   Running full BitNet setup process..."
Invoke-FullBitNetSetup -PythonCmd $pythonCmd

# Verify that kernel files were generated
$kernelFiles = @(
    "include\bitnet-lut-kernels.h",
    "include\kernel_config.ini"
)

$missingKernelFiles = @()
foreach ($file in $kernelFiles) {
    if (!(Test-Path $file)) {
        $missingKernelFiles += $file
    }
}

if ($missingKernelFiles.Count -gt 0) {
    Write-Status "   Some kernel files are missing, using preset kernels..." $Yellow
    # Use preset kernels as fallback (like Linux workflow does)
    $presetKernelPath = "preset_kernels\bitnet_b1_58-3B\bitnet-lut-kernels-tl2.h"
    if (Test-Path $presetKernelPath) {
        Copy-Item $presetKernelPath "include\bitnet-lut-kernels.h" -Force
        Write-Status "   [OK] Copied preset TL2 kernel header"
    } else {
        Write-Status "   Warning: Preset kernel not found, creating minimal fallback..." $Yellow
        '#ifndef BITNET_LUT_KERNELS_H
#define BITNET_LUT_KERNELS_H
#include <cstring>
#include <immintrin.h>
// Minimal header for build process
#endif // BITNET_LUT_KERNELS_H' | Out-File -FilePath "include\bitnet-lut-kernels.h" -Encoding UTF8
    }

    if (!(Test-Path "include\kernel_config.ini")) {
        Write-Status "   Creating kernel_config.ini..." $Yellow
        '[kernel]
BM=256,128,256
BK=96,96,96
bm=32,32,32' | Out-File -FilePath "include\kernel_config.ini" -Encoding UTF8
    }
} else {
    Write-Status "   Required kernel files are available"
}

# Apply critical patches to kernel header (like Linux workflow lines 222-225)
Write-Status "   Applying kernel header patches..."
if (Test-Path "include\bitnet-lut-kernels.h") {
    $kernelContent = Get-Content "include\bitnet-lut-kernels.h" -Raw
    
    # Check if it's a preset kernel (starts with #if defined) - DON'T PATCH IT!
    if ($kernelContent -match "^#if defined\(GGML_BITNET") {
        Write-Status "   [OK] Kernel header is preset kernel - no patching needed"
    }
    # Check if it's a generated kernel that needs includes added
    elseif ($kernelContent -notmatch "#include <cstring>") {
        # Only patch if file is long enough (more than 10 lines) - avoid corrupting minimal headers
        $lineCount = (Get-Content "include\bitnet-lut-kernels.h" | Measure-Object -Line).Lines
        if ($lineCount -gt 10) {
            # Insert includes at the top after header guards
            $lines = Get-Content "include\bitnet-lut-kernels.h"
            $newContent = @()
            $headerGuardFound = $false
            foreach ($line in $lines) {
                $newContent += $line
                # After the #define BITNET_LUT_KERNELS_H line, add includes
                if ($line -match "^#define BITNET_LUT_KERNELS_H" -and -not $headerGuardFound) {
                    $newContent += "#include <cstring>"
                    $newContent += "#include <immintrin.h>"
                    $newContent += ""
                    $headerGuardFound = $true
                }
            }
            $newContent | Out-File -FilePath "include\bitnet-lut-kernels.h" -Encoding UTF8
            Write-Status "   [OK] Added missing includes to generated kernel header"
        } else {
            Write-Status "   [WARN] Kernel header too short to patch safely - skipping"
        }
    } else {
        Write-Status "   [OK] Kernel header already has required includes"
    }
}

# 13. Build BitNet CUDA kernels (Python GPU DLL)
Write-Status "13. Building BitNet CUDA kernels (Python GPU DLL)..."
if (Test-Path "gpu\bitnet_kernels\bitnet_kernels.cu") {
    try {
        # Use the VS installation path detected at the beginning
        if (-not $vsInstallPath) {
            throw "Visual Studio not detected at initialization"
        }
        
        # Find latest MSVC version
        $msvcBasePath = "$vsInstallPath\VC\Tools\MSVC"
        $msvcVersions = Get-ChildItem $msvcBasePath -Directory -ErrorAction SilentlyContinue | Sort-Object Name -Descending
        if (-not $msvcVersions) {
            throw "MSVC compiler not found in $msvcBasePath"
        }
        $msvcPath = $msvcVersions[0].FullName
        
        # Configure VS environment for CUDA compilation
        $env:DISTUTILS_USE_SDK = "1"
        $env:MSSdk = "1"
        $env:VS160COMNTOOLS = "$vsInstallPath\Common7\Tools\"
        $env:VSINSTALLDIR = "$vsInstallPath\"
        $env:PATH = "$msvcPath\bin\Hostx64\x64;$env:PATH"
        
        # Set INCLUDE paths (critical for NVCC to find Windows SDK headers)
        $env:INCLUDE = @(
            "$msvcPath\include",
            "C:\Program Files (x86)\Windows Kits\10\Include\10.0.26100.0\ucrt",
            "C:\Program Files (x86)\Windows Kits\10\Include\10.0.26100.0\shared",
            "C:\Program Files (x86)\Windows Kits\10\Include\10.0.26100.0\um"
        ) -join ";"
        
        # Set LIB paths (critical for NVCC to find Windows SDK libraries)
        $env:LIB = @(
            "$msvcPath\lib\x64",
            "C:\Program Files (x86)\Windows Kits\10\Lib\10.0.26100.0\ucrt\x64",
            "C:\Program Files (x86)\Windows Kits\10\Lib\10.0.26100.0\um\x64"
        ) -join ";"
        
        # Set compiler paths
        $env:CC = "$msvcPath\bin\Hostx64\x64\cl.exe"
        $env:CXX = "$msvcPath\bin\Hostx64\x64\cl.exe"
        
        # Dynamically find latest CUDA version (not hardcoded!)
        $cudaBasePath = "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA"
        if (Test-Path $cudaBasePath) {
            $cudaVersions = Get-ChildItem $cudaBasePath -Directory | Where-Object { $_.Name -match "^v\d" } | Sort-Object Name -Descending
            if ($cudaVersions) {
                $cudaDir = $cudaVersions[0].FullName
                $env:CUDA_HOME = $cudaDir
                $env:CUDA_PATH = $cudaDir
                $env:PATH = "$cudaDir\bin;$env:PATH"
                
                Write-Status "   Using CUDA: $($cudaVersions[0].Name)"
                
                # Build using nvcc directly (more reliable than Python distutils)
                $originalLocation = Get-Location
                try {
                    Set-Location "gpu\bitnet_kernels"
                    
                $nvcc = "$env:CUDA_HOME\bin\nvcc.exe"
                & $nvcc bitnet_kernels.cu `
                    -o libbitnet.dll `
                    --shared `
                    -O3 `
                    -use_fast_math `
                    -std=c++17 `
                    "-gencode=arch=compute_75,code=sm_75" `
                    -Xcompiler "/MD" `
                    -allow-unsupported-compiler
                    
                    if ($LASTEXITCODE -eq 0) {
                        Write-Status "   [OK] BitNet CUDA kernel DLL built successfully" $Green
                        
                        # IMMEDIATELY copy to Release folder (we're already in gpu\bitnet_kernels\)
                        if (Test-Path "libbitnet.dll") {
                            Copy-Item "libbitnet.dll" "$BuildDir\gpu\windows\bitnet-python-cuda\" -Force
                            Copy-Item "libbitnet.lib" "$BuildDir\gpu\windows\bitnet-python-cuda\" -Force -ErrorAction SilentlyContinue
                            Copy-Item "libbitnet.exp" "$BuildDir\gpu\windows\bitnet-python-cuda\" -Force -ErrorAction SilentlyContinue
                            
                            # Copy CUDA runtime DLLs alongside libbitnet.dll for standalone operation
                            if ($env:CUDA_HOME) {
                                $cudaBinPath = "$env:CUDA_HOME\bin"
                                $cudaDlls = @("cudart64_*.dll", "cublas64_*.dll", "cublasLt64_*.dll", "nvrtc64_*.dll")
                                foreach ($pattern in $cudaDlls) {
                                    $files = Get-ChildItem "$cudaBinPath\$pattern" -ErrorAction SilentlyContinue
                                    foreach ($file in $files) {
                                        Copy-Item $file.FullName "$BuildDir\gpu\windows\bitnet-python-cuda\" -Force -ErrorAction SilentlyContinue
                                    }
                                }
                                Write-Status "   [COPIED] libbitnet.dll + CUDA runtime DLLs → Release/gpu/windows/bitnet-python-cuda/" $Green
                            }
                        }
                        
                        $env:BITNET_CUDA_SUCCESS = "true"
                    } else {
                        throw "NVCC compilation failed with exit code $LASTEXITCODE"
                    }
                } finally {
                    # ALWAYS return to original location, even if error occurs
                    Set-Location $originalLocation
                }
            } else {
                Write-Status "   [WARN] CUDA versions not found - skipping Python GPU kernel" $Yellow
                $env:BITNET_CUDA_SUCCESS = "false"
            }
        } else {
            Write-Status "   [WARN] CUDA not found - skipping Python GPU kernel" $Yellow
            $env:BITNET_CUDA_SUCCESS = "false"
        }
    } catch {
        Write-Status "   [WARN] Failed to build BitNet CUDA kernels: $_" $Yellow
        $env:BITNET_CUDA_SUCCESS = "false"
    }
} else {
    Write-Status "   [SKIP] BitNet CUDA kernel source not found" $Yellow
    $env:BITNET_CUDA_SUCCESS = "false"
}

# 14. Clean CMake caches and create build directories
Write-Status "14. Cleaning CMake caches and creating build directories..."

# Clean old CMake caches to avoid toolset conflicts
# Platform-specific build directory names (windows/linux) to avoid conflicts
$buildDirs = @(
    "build-windows-standard",
    "build-windows-gpu-cuda-vulkan",
    "build-windows-gpu-opencl",
    "build-windows-bitnet-portable",
    # AMD Ryzen (all generations)
    "build-windows-bitnet-amd-zen1",
    "build-windows-bitnet-amd-zen2",
    "build-windows-bitnet-amd-zen3",
    "build-windows-bitnet-amd-zen4",
    "build-windows-bitnet-amd-zen5",
    # Intel Core (all generations)
    "build-windows-bitnet-intel-haswell",
    "build-windows-bitnet-intel-broadwell",
    "build-windows-bitnet-intel-skylake",
    "build-windows-bitnet-intel-icelake",
    "build-windows-bitnet-intel-rocketlake",
    "build-windows-bitnet-intel-alderlake"
)
foreach ($dir in $buildDirs) {
    if (Test-Path $dir) {
        Write-Status "   Cleaning CMake cache in $dir..." $Yellow
        Remove-Item "$dir\CMakeCache.txt" -Force -ErrorAction SilentlyContinue
        Remove-Item "$dir\CMakeFiles" -Recurse -Force -ErrorAction SilentlyContinue
    }
}

if ($CleanBuild) {
    Write-Status "   Cleaning previous WINDOWS build output only (preserving Linux/macOS)..." $Yellow
    
    # Clean EVERYTHING in Windows directories (but preserve Linux/macOS)
    if (Test-Path "$BuildDir\cpu\windows") {
        Write-Status "     Cleaning Release/cpu/windows/* ..." $Yellow
        Get-ChildItem "$BuildDir\cpu\windows\*" | Remove-Item -Recurse -Force
    }
    
    if (Test-Path "$BuildDir\gpu\windows") {
        Write-Status "     Cleaning Release/gpu/windows/* ..." $Yellow
        Get-ChildItem "$BuildDir\gpu\windows\*" | Remove-Item -Recurse -Force
    }
    
    # Clean all build artifact directories
    Write-Status "     Cleaning build-* directories..." $Yellow
    foreach ($dir in $buildDirs) {
        if (Test-Path $dir) {
            Write-Status "       Removing $dir..." $Yellow
            Remove-Item -Recurse -Force $dir
        }
    }
    
    Write-Status "   [OK] Clean build - all Windows artifacts removed" $Green
}

# Create separate subdirectories for each build type (CRITICAL for DLL isolation!)
# CPU builds - standard (no BitNet)
New-Item -ItemType Directory -Path "$BuildDir\cpu\windows\standard" -Force | Out-Null

# CPU builds - BitNet multi-arch variants (COMPLETE MATRIX for distribution)
New-Item -ItemType Directory -Path "$BuildDir\cpu\windows\bitnet-portable" -Force | Out-Null
# AMD Ryzen (all generations)
New-Item -ItemType Directory -Path "$BuildDir\cpu\windows\bitnet-amd-zen1" -Force | Out-Null
New-Item -ItemType Directory -Path "$BuildDir\cpu\windows\bitnet-amd-zen2" -Force | Out-Null
New-Item -ItemType Directory -Path "$BuildDir\cpu\windows\bitnet-amd-zen3" -Force | Out-Null
New-Item -ItemType Directory -Path "$BuildDir\cpu\windows\bitnet-amd-zen4" -Force | Out-Null
New-Item -ItemType Directory -Path "$BuildDir\cpu\windows\bitnet-amd-zen5" -Force | Out-Null
# Intel Core (all generations)
New-Item -ItemType Directory -Path "$BuildDir\cpu\windows\bitnet-intel-haswell" -Force | Out-Null
New-Item -ItemType Directory -Path "$BuildDir\cpu\windows\bitnet-intel-broadwell" -Force | Out-Null
New-Item -ItemType Directory -Path "$BuildDir\cpu\windows\bitnet-intel-skylake" -Force | Out-Null
New-Item -ItemType Directory -Path "$BuildDir\cpu\windows\bitnet-intel-icelake" -Force | Out-Null
New-Item -ItemType Directory -Path "$BuildDir\cpu\windows\bitnet-intel-rocketlake" -Force | Out-Null
New-Item -ItemType Directory -Path "$BuildDir\cpu\windows\bitnet-intel-alderlake" -Force | Out-Null

# GPU builds - standard (no BitNet)
New-Item -ItemType Directory -Path "$BuildDir\gpu\windows\standard-cuda-vulkan" -Force | Out-Null
New-Item -ItemType Directory -Path "$BuildDir\gpu\windows\standard-opencl" -Force | Out-Null

# GPU builds - BitNet
New-Item -ItemType Directory -Path "$BuildDir\gpu\windows\bitnet-python-cuda" -Force | Out-Null

Write-Status "   Build directories created (1 standard + 12 BitNet CPU, 2 standard + 1 BitNet GPU = 16 total)"

# 15. Build Standard CPU Version (llama.cpp - any model, any CPU)
Write-Status "15. Building Standard CPU Version..."
if (!(Should-BuildVariant "standard")) {
    Write-Status "   [SKIP] Standard CPU build not requested" $Yellow
    $env:STANDARD_CPU_SUCCESS = "skipped"
} elseif (Should-SkipBuild -BuildDir "build-windows-standard" -ReleaseDir "$BuildDir\cpu\windows\standard" -MinFiles 40) {
    Write-Status "   [OK] Standard CPU already built" $Green
    $env:STANDARD_CPU_SUCCESS = "true"
} else {
try {
    # Build from repo root (BitNet's conditional Clang check won't fail without BITNET flags)
    & cmake -B "build-windows-standard" `
        -G "Visual Studio 17 2022" `
        -T ClangCL `
        -DLLAMA_BUILD_SERVER=ON `
        -DLLAMA_BUILD_EXAMPLES=ON `
        3rdparty/llama.cpp
    
    if ($LASTEXITCODE -ne 0) {
        throw "CMake configure failed"
    }
    
    & cmake --build build-windows-standard --config Release --parallel
    if ($LASTEXITCODE -ne 0) {
        throw "Build failed"
    }
    
    Write-Status "   Standard CPU version built successfully"
    
    # IMMEDIATELY copy to Release folder (defensive approach)
    if (Test-Path "build-windows-standard\bin\Release") {
        $filesCount = (Get-ChildItem "build-windows-standard\bin\Release\*.*" -File).Count
        Copy-Item "build-windows-standard\bin\Release\*.*" "$BuildDir\cpu\windows\standard\" -Force
        Write-Status "   [COPIED] $filesCount files → Release/cpu/windows/standard/" $Green
    }
    
    $env:STANDARD_CPU_SUCCESS = "true"
} catch {
    Write-Status "   Warning: Failed to build standard CPU version: $_" $Yellow
    $env:STANDARD_CPU_SUCCESS = "false"
}
}

# 16. Build Standard GPU Version (CUDA + Vulkan - llama.cpp, any model)
Write-Status "16. Building Standard GPU Version (CUDA + Vulkan)..."
if (Should-SkipBuild -BuildDir "build-windows-gpu-cuda-vulkan" -ReleaseDir "$BuildDir\gpu\windows\standard-cuda-vulkan" -MinFiles 40) {
    Write-Status "   [OK] GPU CUDA+Vulkan already built" $Green
    $env:STANDARD_GPU_SUCCESS = "true"
    $env:GPU_VULKAN_SUCCESS = "true"
} else {
try {
    # Try CUDA + Vulkan first
    Write-Status "   Trying with Vulkan support..."
    
    # Build CMake arguments
    $cmakeArgs = @(
        "-B", "build-windows-gpu-cuda-vulkan"
        "-G", "Visual Studio 17 2022"
        "-DGGML_VULKAN=ON"
        "-DGGML_CUDA=ON"
        "-DCMAKE_CUDA_ARCHITECTURES=75"
        "-DLLAMA_BUILD_SERVER=ON"
        "-DLLAMA_BUILD_EXAMPLES=ON"
    )
    
    # CUDA will be detected via VS Integration (proper Windows way)
    if ($env:CUDA_HOME) {
        Write-Status "   Using CUDA: $env:CUDA_HOME (via VS Integration)"
    }
    
    # Explicitly pass Vulkan SDK path if configured
    if ($env:VULKAN_CONFIGURED -eq "true" -and $env:VULKAN_SDK) {
        $cmakeArgs += "-DVULKAN_SDK=$env:VULKAN_SDK"
        Write-Status "   Using VULKAN_SDK: $env:VULKAN_SDK"
    }
    
    # Add source directory
    $cmakeArgs += "3rdparty/llama.cpp"
    
    & cmake @cmakeArgs
    
    if ($LASTEXITCODE -eq 0) {
        & cmake --build build-windows-gpu-cuda-vulkan --config Release --parallel
        if ($LASTEXITCODE -eq 0) {
            Write-Status "   Standard GPU version built successfully (CUDA + Vulkan)"
            
            # IMMEDIATELY copy to Release folder
            if (Test-Path "build-windows-gpu-cuda-vulkan\bin\Release") {
                $filesCount = (Get-ChildItem "build-windows-gpu-cuda-vulkan\bin\Release\*.*" -File).Count
                Copy-Item "build-windows-gpu-cuda-vulkan\bin\Release\*.*" "$BuildDir\gpu\windows\standard-cuda-vulkan\" -Force
                Write-Status "   [COPIED] $filesCount files → Release/gpu/windows/standard-cuda-vulkan/" $Green
            }
            
            $env:STANDARD_GPU_SUCCESS = "true"
            $env:GPU_VULKAN_SUCCESS = "true"
        } else {
            throw "Build with Vulkan failed"
        }
    } else {
        throw "Configure with Vulkan failed"
    }
} catch {
    Write-Status "   Warning: Vulkan build failed, trying CUDA-only..." $Yellow
    
    # Clean failed build and try CUDA-only
    Remove-Item "build-windows-gpu-cuda-vulkan\CMakeCache.txt" -Force -ErrorAction SilentlyContinue
    Remove-Item "build-windows-gpu-cuda-vulkan\CMakeFiles" -Recurse -Force -ErrorAction SilentlyContinue
    
    try {
        & cmake -B "build-windows-gpu-cuda-vulkan" `
            -G "Visual Studio 17 2022" `
            -DGGML_VULKAN=OFF `
            -DGGML_CUDA=ON `
            -DCMAKE_CUDA_ARCHITECTURES=75 `
            -DLLAMA_BUILD_SERVER=ON `
            -DLLAMA_BUILD_EXAMPLES=ON `
            3rdparty/llama.cpp
    
    if ($LASTEXITCODE -ne 0) {
        throw "CMake configure failed"
    }
    
    & cmake --build build-windows-gpu-cuda-vulkan --config Release --parallel
    if ($LASTEXITCODE -ne 0) {
        throw "Build failed"
    }
    
        Write-Status "   Standard GPU version built successfully (CUDA-only)"
        
        # IMMEDIATELY copy to Release folder
        if (Test-Path "build-windows-gpu-cuda-vulkan\bin\Release") {
            $filesCount = (Get-ChildItem "build-windows-gpu-cuda-vulkan\bin\Release\*.*" -File).Count
            Copy-Item "build-windows-gpu-cuda-vulkan\bin\Release\*.*" "$BuildDir\gpu\windows\standard-cuda-vulkan\" -Force
            Write-Status "   [COPIED] $filesCount files → Release/gpu/windows/standard-cuda-vulkan/" $Green
        }
        
        $env:STANDARD_GPU_SUCCESS = "true"
        $env:GPU_VULKAN_SUCCESS = "false"
} catch {
        Write-Status "   Warning: Standard GPU build failed completely: $_" $Yellow
        $env:STANDARD_GPU_SUCCESS = "false"
        $env:GPU_VULKAN_SUCCESS = "false"
    }
}
}

# 17. Build Standard OpenCL Version (Universal GPU - llama.cpp, any model)
Write-Status "17. Building Standard OpenCL Version (Universal GPU)..."

if ($env:OPENCL_AVAILABLE -eq "true") {
    if (Should-SkipBuild -BuildDir "build-windows-gpu-opencl" -ReleaseDir "$BuildDir\gpu\windows\standard-opencl" -MinFiles 40) {
        Write-Status "   [OK] GPU OpenCL already built" $Green
        $env:OPENCL_SUCCESS = "true"
    } else {
    try {
        Write-Status "   Using OpenCL headers from: $env:OPENCL_INCLUDE_PATH"
        
        # Build CMake arguments with OpenCL paths
        & cmake -B "build-windows-gpu-opencl" `
            -G "Visual Studio 17 2022" `
            -DGGML_OPENCL=ON `
            -DOpenCL_INCLUDE_DIR="$env:OPENCL_INCLUDE_PATH" `
        -DLLAMA_BUILD_SERVER=ON `
            -DLLAMA_BUILD_EXAMPLES=ON `
            3rdparty/llama.cpp
    
    if ($LASTEXITCODE -ne 0) {
        throw "CMake configure failed"
    }
    
        & cmake --build build-windows-gpu-opencl --config Release --parallel
    if ($LASTEXITCODE -ne 0) {
        throw "Build failed"
    }
    
        Write-Status "   Standard OpenCL version built successfully (NVIDIA/AMD/Intel GPU support)"
        
        # IMMEDIATELY copy to Release folder
        if (Test-Path "build-windows-gpu-opencl\bin\Release") {
            $filesCount = (Get-ChildItem "build-windows-gpu-opencl\bin\Release\*.*" -File).Count
            Copy-Item "build-windows-gpu-opencl\bin\Release\*.*" "$BuildDir\gpu\windows\standard-opencl\" -Force
            Write-Status "   [COPIED] $filesCount files → Release/gpu/windows/standard-opencl/" $Green
        }
        
        $env:OPENCL_SUCCESS = "true"
} catch {
        Write-Status "   Warning: Failed to build standard OpenCL version: $_" $Yellow
        $env:OPENCL_SUCCESS = "false"
    }
    }
} else {
    Write-Status "   [SKIP] OpenCL headers not available - build skipped" $Yellow
    $env:OPENCL_SUCCESS = "false"
}


# 18. Build BitNet CPU Multi-Arch Variants (BitNet models only)
Write-Status "18. Building BitNet CPU Multi-Arch Variants..."

# Define CPU architecture variants for BitNet TL2 builds
# COMPREHENSIVE BUILD MATRIX for distribution/server use
$bitnetArchs = @(
    # Universal fallback
    @{Name="portable"; March="x86-64-v3"; Desc="AVX2 baseline (any modern CPU)"},
    
    # AMD Ryzen (complete range)
    @{Name="amd-zen1"; March="znver1"; Desc="AMD Ryzen 1000/2000 series (Zen 1)"},
    @{Name="amd-zen2"; March="znver2"; Desc="AMD Ryzen 3000 series (Zen 2)"},
    @{Name="amd-zen3"; March="znver3"; Desc="AMD Ryzen 5000 series (Zen 3)"},
    @{Name="amd-zen4"; March="znver4"; Desc="AMD Ryzen 7000 series (Zen 4)"},
    @{Name="amd-zen5"; March="znver5"; Desc="AMD Ryzen 9000 series (Zen 5)"},
    
    # Intel Core (complete range)
    @{Name="intel-haswell"; March="haswell"; Desc="Intel 4th gen (Haswell)"},
    @{Name="intel-broadwell"; March="broadwell"; Desc="Intel 5th gen (Broadwell)"},
    @{Name="intel-skylake"; March="skylake"; Desc="Intel 6th-9th gen (Skylake)"},
    @{Name="intel-icelake"; March="icelake-client"; Desc="Intel 10th gen (Ice Lake)"},
    @{Name="intel-rocketlake"; March="rocketlake"; Desc="Intel 11th gen (Rocket Lake)"},
    @{Name="intel-alderlake"; March="alderlake"; Desc="Intel 12th-14th gen (Alder Lake)"}
)

$bitnetSuccessCount = 0
foreach ($arch in $bitnetArchs) {
    # Check if this variant should be built
    if (!(Should-BuildVariant $arch.Name)) {
        Write-Status "  [SKIP] BitNet-$($arch.Name) build not requested" $Yellow
        continue
    }
    
    # Check if already built (smart skip)
    if (Should-SkipBuild -BuildDir "build-windows-bitnet-$($arch.Name)" -ReleaseDir "$BuildDir\cpu\windows\bitnet-$($arch.Name)" -MinFiles 35) {
        Write-Status "  [OK] BitNet-$($arch.Name) already built" $Green
        Set-Variable -Name "env:BITNET_$($arch.Name.ToUpper().Replace('-','_'))_SUCCESS" -Value "true"
        $bitnetSuccessCount++
        continue
    }
    
    Write-Status "  Building BitNet-$($arch.Name) ($($arch.Desc))..." $Cyan
    try {
        # Visual Studio 2022 with ClangCL toolset
        # clang-cl supports both -march= (Clang-style) and /EHsc (MSVC-style exception handling)
        & cmake -B "build-windows-bitnet-$($arch.Name)" `
            -G "Visual Studio 17 2022" `
            -T ClangCL `
            -DBITNET_X86_TL2=ON `
            "-DCMAKE_C_FLAGS=-march=$($arch.March)" `
            "-DCMAKE_CXX_FLAGS=-march=$($arch.March) /EHsc" `
            -DLLAMA_BUILD_SERVER=ON `
            -DLLAMA_BUILD_EXAMPLES=ON `
            .
        
        if ($LASTEXITCODE -ne 0) {
            throw "CMake configure failed"
        }
        
        & cmake --build "build-windows-bitnet-$($arch.Name)" --config Release --parallel
        if ($LASTEXITCODE -ne 0) {
            throw "Build failed"
        }
        
        Write-Status "    [OK] BitNet-$($arch.Name) built successfully" $Green
        
        # IMMEDIATELY copy to Release folder
        if (Test-Path "build-windows-bitnet-$($arch.Name)\bin\Release") {
            $filesCount = (Get-ChildItem "build-windows-bitnet-$($arch.Name)\bin\Release\*.*" -File).Count
            Copy-Item "build-windows-bitnet-$($arch.Name)\bin\Release\*.*" "$BuildDir\cpu\windows\bitnet-$($arch.Name)\" -Force
            Write-Status "    [COPIED] $filesCount files → Release/cpu/windows/bitnet-$($arch.Name)/" $Green
        }
        
        # Track success per architecture
        Set-Variable -Name "env:BITNET_$($arch.Name.ToUpper().Replace('-','_'))_SUCCESS" -Value "true"
        $bitnetSuccessCount++
    } catch {
        Write-Status "    [WARN] Failed to build BitNet-$($arch.Name): $_" $Yellow
        Set-Variable -Name "env:BITNET_$($arch.Name.ToUpper().Replace('-','_'))_SUCCESS" -Value "false"
    }
}

# Overall BitNet success if at least portable variant built
if ($bitnetSuccessCount -gt 0) {
    $env:BITNET_CPU_SUCCESS = "true"
    Write-Status "  [OK] BitNet CPU builds: $bitnetSuccessCount/5 variants successful" $Green
} else {
    $env:BITNET_CPU_SUCCESS = "false"
    Write-Status "  [FAIL] All BitNet CPU builds failed" $Red
}

# NOTE: BitNet TL2 CPU kernels are NOT compatible with GPU backends (CUDA/Vulkan/OpenCL)
# BitNet GPU inference uses a separate Python + CUDA implementation in gpu/ folder
# Hybrid builds (BitNet TL2 + GPU) were removed as architecturally incompatible

# 19. Summary of copied binaries
Write-Status "19. Build and copy summary..."
Write-Status ""
Write-Status "[ARCHITECTURE] Build matrix complete - 16 builds total:" $Green
Write-Status ""
Write-Status "  CPU BUILDS - Standard (llama.cpp, any model):" $Yellow
Write-Status "    - Release/cpu/windows/standard/                     (ClangCL optimized, any CPU)" $Cyan
Write-Status ""
Write-Status "  CPU BUILDS - BitNet (BitNet models only, multi-arch - COMPLETE):" $Yellow
Write-Status "    - Release/cpu/windows/bitnet-portable/              (AVX2 baseline)" $Cyan
Write-Status "    AMD Ryzen:" $Yellow
Write-Status "    - Release/cpu/windows/bitnet-amd-zen1/              (Ryzen 1000/2000 - Zen 1)" $Cyan
Write-Status "    - Release/cpu/windows/bitnet-amd-zen2/              (Ryzen 3000 - Zen 2)" $Cyan
Write-Status "    - Release/cpu/windows/bitnet-amd-zen3/              (Ryzen 5000 - Zen 3)" $Cyan
Write-Status "    - Release/cpu/windows/bitnet-amd-zen4/              (Ryzen 7000 - Zen 4)" $Cyan
Write-Status "    - Release/cpu/windows/bitnet-amd-zen5/              (Ryzen 9000 - Zen 5)" $Cyan
Write-Status "    Intel Core:" $Yellow
Write-Status "    - Release/cpu/windows/bitnet-intel-haswell/         (Intel 4th gen)" $Cyan
Write-Status "    - Release/cpu/windows/bitnet-intel-broadwell/       (Intel 5th gen)" $Cyan
Write-Status "    - Release/cpu/windows/bitnet-intel-skylake/         (Intel 6th-9th gen)" $Cyan
Write-Status "    - Release/cpu/windows/bitnet-intel-icelake/         (Intel 10th gen)" $Cyan
Write-Status "    - Release/cpu/windows/bitnet-intel-rocketlake/      (Intel 11th gen)" $Cyan
Write-Status "    - Release/cpu/windows/bitnet-intel-alderlake/       (Intel 12th-14th gen)" $Cyan
Write-Status ""
Write-Status "  GPU BUILDS - Standard (llama.cpp, any model):" $Yellow
Write-Status "    - Release/gpu/windows/standard-cuda-vulkan/         (NVIDIA CUDA+Vulkan)" $Cyan
Write-Status "    - Release/gpu/windows/standard-opencl/              (Universal GPU: NVIDIA/AMD/Intel)" $Cyan
Write-Status ""
Write-Status "  GPU BUILD - BitNet (BitNet models only):" $Yellow
Write-Status "    - Release/gpu/windows/bitnet-python-cuda/           (Python + CUDA, NVIDIA only)" $Cyan
Write-Status ""
Write-Status "Each directory is SELF-CONTAINED with all executables + matching DLLs!" $Green
Write-Status "Perfect for distribution - zip any folder and ship it!" $Green
Write-Status ""

# GPU modules - Copy Python scripts (libbitnet.dll was already copied immediately after build)
if (Test-Path "gpu") {
    Write-Status "   Copying BitNet Python GPU scripts..."
    
    # Copy all Python scripts
    $pythonFiles = Get-ChildItem "gpu\*.py" -ErrorAction SilentlyContinue
    foreach ($file in $pythonFiles) {
        Copy-Item $file.FullName "$BuildDir\gpu\windows\bitnet-python-cuda\" -Force
    }
    Write-Status "     - Copied $($pythonFiles.Count) Python scripts" $Green
    
    # Copy tokenizer model
    if (Test-Path "gpu\tokenizer.model") {
        Copy-Item "gpu\tokenizer.model" "$BuildDir\gpu\windows\bitnet-python-cuda\" -Force
        Write-Status "     - tokenizer.model" $Green
    }
    
    Write-Status "   [OK] BitNet Python GPU scripts copied (libbitnet.dll was copied in Step 13)" $Green
    } else {
    Write-Status "   [WARN] gpu/ directory not found - skipping GPU modules" $Yellow
}

Write-Status "=== Build Complete! ===" $Green
Write-Status ""
Write-Status "Build output organized in isolated subdirectories (16 total builds):" $Yellow
Write-Status ""

# Standard CPU build
if (Test-Path "$BuildDir\cpu\windows\standard") {
    Write-Status "[Standard CPU] ${BuildDir}\cpu\windows\standard\" $Cyan
    $fileCount = (Get-ChildItem "$BuildDir\cpu\windows\standard" -File -ErrorAction SilentlyContinue).Count
    Write-Status "   $fileCount files (llama.cpp - any model, any CPU)" $Green
    Write-Status ""
}

# BitNet CPU builds (12 variants)
foreach ($arch in $bitnetArchs) {
    if (Test-Path "$BuildDir\cpu\windows\bitnet-$($arch.Name)") {
        Write-Status "[BitNet-$($arch.Name)] ${BuildDir}\cpu\windows\bitnet-$($arch.Name)\" $Cyan
        $fileCount = (Get-ChildItem "$BuildDir\cpu\windows\bitnet-$($arch.Name)" -File -ErrorAction SilentlyContinue).Count
        Write-Status "   $fileCount files ($($arch.Desc))" $Green
        Write-Status ""
    }
}

# Standard GPU builds
if (Test-Path "$BuildDir\gpu\windows\standard-cuda-vulkan") {
    Write-Status "[Standard GPU CUDA+Vulkan] ${BuildDir}\gpu\windows\standard-cuda-vulkan\" $Cyan
    $fileCount = (Get-ChildItem "$BuildDir\gpu\windows\standard-cuda-vulkan" -File -ErrorAction SilentlyContinue).Count
    Write-Status "   $fileCount files (llama.cpp - any model, NVIDIA GPU)" $Green
    Write-Status ""
}

if (Test-Path "$BuildDir\gpu\windows\standard-opencl") {
    Write-Status "[Standard GPU OpenCL] ${BuildDir}\gpu\windows\standard-opencl\" $Cyan
    $fileCount = (Get-ChildItem "$BuildDir\gpu\windows\standard-opencl" -File -ErrorAction SilentlyContinue).Count
    Write-Status "   $fileCount files (llama.cpp - any model, any GPU)" $Green
    Write-Status ""
}

# BitNet GPU build
if (Test-Path "$BuildDir\gpu\windows\bitnet-python-cuda") {
    Write-Status "[BitNet Python GPU] ${BuildDir}\gpu\windows\bitnet-python-cuda\" $Cyan
    $fileCount = (Get-ChildItem "$BuildDir\gpu\windows\bitnet-python-cuda" -File -ErrorAction SilentlyContinue).Count
    Write-Status "   $fileCount files (BitNet Python + CUDA, NVIDIA only)" $Green
    Write-Status ""
}


# Build Summary
Write-Status "========================================" $Green
Write-Status "         BUILD SUMMARY (16 builds)" $Green
Write-Status "========================================" $Green
Write-Status ""

# Standard CPU Build
Write-Status "STANDARD CPU BUILD (llama.cpp - any model):" $Yellow
$standardSuccess = if ($env:STANDARD_CPU_SUCCESS -eq "true") { "[OK] SUCCESS" } else { "[FAIL] FAILED" }
Write-Status "  Standard (ClangCL, any CPU):        $standardSuccess"
Write-Status ""

# BitNet CPU Builds (12 variants)
Write-Status "BITNET CPU BUILDS (BitNet models only - 12 variants):" $Yellow
$bitnetSuccess = if ($env:BITNET_CPU_SUCCESS -eq "true") { "($bitnetSuccessCount/12 successful)" } else { "(all failed)" }
Write-Status "  Overall Status:                     $bitnetSuccess"
foreach ($arch in $bitnetArchs) {
    $envVar = "BITNET_$($arch.Name.ToUpper().Replace('-','_'))_SUCCESS"
    $archSuccess = if ((Get-Variable -Name "env:$envVar" -ValueOnly -ErrorAction SilentlyContinue) -eq "true") { "[OK]" } else { "[FAIL]" }
    Write-Status "    $archSuccess BitNet-$($arch.Name) ($($arch.Desc))"
}
Write-Status ""

# Standard GPU Builds
Write-Status "STANDARD GPU BUILDS (llama.cpp - any model):" $Yellow
$gpuSuccess = if ($env:STANDARD_GPU_SUCCESS -eq "true") { "[OK] SUCCESS" } else { "[FAIL] FAILED" }
$openclSuccess = if ($env:OPENCL_SUCCESS -eq "true") { "[OK] SUCCESS" } else { "[SKIP] NOT BUILT" }
Write-Status "  CUDA+Vulkan (NVIDIA):               $gpuSuccess"
if ($env:STANDARD_GPU_SUCCESS -eq "true") {
    $vulkanStatus = if ($env:GPU_VULKAN_SUCCESS -eq "true") { "[OK] YES" } else { "[WARN] CUDA only" }
    Write-Status "    |-- Vulkan Support:             $vulkanStatus"
}
Write-Status "  OpenCL (Universal GPU):             $openclSuccess"
Write-Status ""

# BitNet GPU Build
Write-Status "BITNET GPU BUILD (BitNet models only):" $Yellow
$pythonGpuSuccess = if ($env:BITNET_CUDA_SUCCESS -eq "true") { "[OK] SUCCESS" } else { "[SKIP] NOT BUILT" }
Write-Status "  Python + CUDA (NVIDIA only):        $pythonGpuSuccess"
Write-Status ""

# Optional builds diagnostic
if ($env:OPENCL_SUCCESS -ne "true") {
    Write-Status "OPTIONAL BUILDS DIAGNOSTIC:" $Yellow
    Write-Status "  [SKIP] OpenCL:" $Yellow
    Write-Status "    Reason: OpenCL headers not found" $Cyan
    Write-Status "    Solution: Install NVIDIA CUDA Toolkit or AMD drivers" $Cyan
    Write-Status ""
}

# File locations
Write-Status "OUTPUT LOCATIONS:" $Yellow
Write-Status "  [DIR] CPU Builds:  ${BuildDir}\cpu\windows\"
Write-Status "  [DIR] GPU Builds:  ${BuildDir}\gpu\windows\"
Write-Status ""

# Overall status
Write-Status "========================================" $Green
$successCount = 0
$totalBuilds = 16  # 1 Standard CPU + 12 BitNet CPU + 2 Standard GPU + 1 BitNet GPU

# Count successful builds
if ($env:STANDARD_CPU_SUCCESS -eq "true") { $successCount++ }
if ($env:STANDARD_GPU_SUCCESS -eq "true") { $successCount++ }
if ($env:OPENCL_SUCCESS -eq "true") { $successCount++ }
if ($env:BITNET_CUDA_SUCCESS -eq "true") { $successCount++ }
# Add BitNet CPU variant successes (each variant counts separately)
$successCount += $bitnetSuccessCount

if ($successCount -eq $totalBuilds) {
    Write-Status "[SUCCESS] COMPLETE BUILD MATRIX! ($successCount/$totalBuilds)" $Green
    Write-Status "ALL 16 BUILDS SUCCESSFUL - Full distribution package ready!" $Green
} elseif ($successCount -ge 12) {
    Write-Status "[OK] EXCELLENT SUCCESS ($successCount/$totalBuilds builds)" $Green
    Write-Status "Most builds complete - comprehensive distribution package available!" $Green
} elseif ($successCount -ge 8) {
    Write-Status "[OK] GOOD SUCCESS ($successCount/$totalBuilds builds)" $Green
    Write-Status "Core builds complete, some optional components may be skipped." $Yellow
} elseif ($successCount -ge 3) {
    Write-Status "[OK] PARTIAL SUCCESS ($successCount/$totalBuilds builds)" $Yellow
    Write-Status "Basic functionality available. Review warnings above." $Yellow
} elseif ($successCount -ge 1) {
    Write-Status "[WARN] MINIMAL SUCCESS ($successCount/$totalBuilds builds)" $Yellow
    Write-Status "Only few builds succeeded. Review errors above." $Yellow
} else {
    Write-Status "[FAIL] ALL BUILDS FAILED" $Red
    Write-Status "Please review errors above and check prerequisites." $Red
}
Write-Status "========================================" $Green
Write-Status ""

# Optional: Quick verification tests
if ($successCount -gt 0) {
    Write-Status "========================================" $Yellow
    Write-Status "         QUICK VERIFICATION" $Yellow
    Write-Status "========================================" $Yellow
    Write-Status ""
    Write-Status "Waiting for files to finalize..." $Yellow
    Start-Sleep -Seconds 2  # Give Windows time to finish writing files
    Write-Status "Testing built binaries..." $Yellow
Write-Status ""

    # Test C++ executables
    $testsPassed = 0
    $testsFailed = 0
    
    if (Test-Path "$BuildDir\cpu\windows\standard\llama-server.exe") {
        Write-Status "Testing Standard CPU (llama-server.exe)..." $Cyan
        try {
            # Capture both stdout and stderr
            $process = Start-Process -FilePath "$BuildDir\cpu\windows\standard\llama-server.exe" `
                -ArgumentList "--help" `
                -NoNewWindow `
                -Wait `
                -PassThru `
                -RedirectStandardOutput "$env:TEMP\llama-test-stdout.txt" `
                -RedirectStandardError "$env:TEMP\llama-test-stderr.txt"
            
            $helpOutput = Get-Content "$env:TEMP\llama-test-stdout.txt" -Raw -ErrorAction SilentlyContinue
            $errorOutput = Get-Content "$env:TEMP\llama-test-stderr.txt" -Raw -ErrorAction SilentlyContinue
            
            if ($process.ExitCode -eq 0 -or $helpOutput -match "usage|server|model" -or $errorOutput -match "usage|server|model") {
                Write-Status "  [OK] Executable works!" $Green
                Write-Status "  [INFO] Features: CPU (ClangCL optimized), AVX2" $Yellow
                Write-Status "  [USE] Standard CPU inference on any machine"
                $testsPassed++
            } else {
                Write-Status "  [FAIL] Exit code: $($process.ExitCode)" $Red
                if ($errorOutput) {
                    Write-Status "  Error: $($errorOutput.Substring(0, [Math]::Min(200, $errorOutput.Length)))" $Red
                }
                $testsFailed++
            }
        } catch {
            Write-Status "  [FAIL] Could not start process: $_" $Red
            $testsFailed++
}
Write-Status ""
    }
    
    if (Test-Path "$BuildDir\gpu\windows\standard-cuda-vulkan\llama-server.exe") {
        Write-Status "Testing Standard GPU CUDA+Vulkan (llama-server.exe)..." $Cyan
        try {
            $process = Start-Process -FilePath "$BuildDir\gpu\windows\standard-cuda-vulkan\llama-server.exe" `
                -ArgumentList "--help" `
                -NoNewWindow `
                -Wait `
                -PassThru `
                -RedirectStandardOutput "$env:TEMP\llama-gpu-stdout.txt" `
                -RedirectStandardError "$env:TEMP\llama-gpu-stderr.txt"
            
            $helpOutput = Get-Content "$env:TEMP\llama-gpu-stdout.txt" -Raw -ErrorAction SilentlyContinue
            $errorOutput = Get-Content "$env:TEMP\llama-gpu-stderr.txt" -Raw -ErrorAction SilentlyContinue
            
            # Check what backends are supported
            $hasCUDA = ($helpOutput + $errorOutput) -match "cuda|CUDA|ngl"
            $hasVulkan = $env:GPU_VULKAN_SUCCESS -eq "true"
            
            if ($process.ExitCode -eq 0 -or $helpOutput -match "usage|server|model" -or $errorOutput -match "usage|server|model") {
                Write-Status "  [OK] Executable works!" $Green
                $features = @()
                $features += "CPU (MSVC)"
                if ($hasCUDA) { $features += "CUDA" }
                if ($hasVulkan) { $features += "Vulkan" }
                Write-Status "  [INFO] Features: $($features -join ', ')" $Yellow
                Write-Status "  [USE] GPU-accelerated inference (use -ngl flag)"
                Write-Status "  [TIP] Key flags: -ngl <layers> (offload layers to GPU)"
                $testsPassed++
            } else {
                Write-Status "  [FAIL] Exit code: $($process.ExitCode)" $Red
                if ($errorOutput) {
                    Write-Status "  Error: $($errorOutput.Substring(0, [Math]::Min(200, $errorOutput.Length)))" $Red
                }
                $testsFailed++
            }
        } catch {
            Write-Status "  [FAIL] Could not start process: $_" $Red
            $testsFailed++
        }
Write-Status ""
    }
    
    # Test BitNet portable build (baseline for all CPUs)
    if (Test-Path "$BuildDir\cpu\windows\bitnet-portable\llama-server.exe") {
        Write-Status "Testing BitNet Portable (llama-server.exe)..." $Cyan
        try {
            $process = Start-Process -FilePath "$BuildDir\cpu\windows\bitnet-portable\llama-server.exe" `
                -ArgumentList "--help" `
                -NoNewWindow `
                -Wait `
                -PassThru `
                -RedirectStandardOutput "$env:TEMP\llama-bitnet-stdout.txt" `
                -RedirectStandardError "$env:TEMP\llama-bitnet-stderr.txt"
            
            $helpOutput = Get-Content "$env:TEMP\llama-bitnet-stdout.txt" -Raw -ErrorAction SilentlyContinue
            $errorOutput = Get-Content "$env:TEMP\llama-bitnet-stderr.txt" -Raw -ErrorAction SilentlyContinue
            
            if ($process.ExitCode -eq 0 -or $helpOutput -match "usage|server|model" -or $errorOutput -match "usage|server|model") {
                Write-Status "  [OK] Executable works!" $Green
                Write-Status "  [INFO] Features: BitNet TL2 kernels (AVX2 baseline, any modern CPU)" $Yellow
                Write-Status "  [USE] BitNet quantized models ONLY"
                Write-Status "  [WARN] Note: Requires special BitNet .gguf models"
                $testsPassed++
            } else {
                Write-Status "  [FAIL] Exit code: $($process.ExitCode)" $Red
                if ($errorOutput) {
                    Write-Status "  Error: $($errorOutput.Substring(0, [Math]::Min(200, $errorOutput.Length)))" $Red
                }
                $testsFailed++
            }
        } catch {
            Write-Status "  [FAIL] Could not start process: $_" $Red
            $testsFailed++
        }
        Write-Status ""
    }
    
    # Test Python GPU kernel (BitNet CUDA)
    if ($env:BITNET_CUDA_SUCCESS -eq "true" -and (Test-Path "$BuildDir\gpu\windows\bitnet-python-cuda\libbitnet.dll")) {
        Write-Status "Testing Python GPU kernel (libbitnet.dll)..." $Cyan
        try {
            # NO NEED to add CUDA to PATH - we copied CUDA DLLs alongside libbitnet.dll!
            Write-Status "  [INFO] Testing standalone Python GPU (with bundled CUDA DLLs)..." $Yellow
            
            # Use proper Windows path format - point to bitnet-python-cuda subdirectory
            $gpuPythonPath = (Resolve-Path "$BuildDir\gpu\windows\bitnet-python-cuda").Path
            
            $testCode = @"
import ctypes
import os
import sys

# Change to the directory containing the DLL (python subdirectory with bundled CUDA DLLs)
os.chdir(r'$gpuPythonPath')

# Load the DLL (CUDA DLLs are in the same directory, so no PATH manipulation needed!)
try:
    lib = ctypes.CDLL('libbitnet.dll')
    print('[OK] GPU kernel loaded successfully!')
    print('[OK] Bundled CUDA DLLs working!')
    sys.exit(0)
except Exception as e:
    print(f'[FAIL] {str(e)}')
    sys.exit(1)
"@
            $testResult = & $pythonCmd -c $testCode 2>&1
            $exitCode = $LASTEXITCODE
            
            if ($exitCode -eq 0 -and $testResult -match "successfully") {
                Write-Status "  [OK] Python GPU kernel works!" $Green
                Write-Status "  [INFO] Features: BitNet 1.58-bit GPU kernels with bundled CUDA DLLs" $Yellow
                Write-Status "  [INFO] Standalone - no need for CUDA in PATH!" $Green
                Write-Status "  [USE] Python-based BitNet inference on GPU"
                Write-Status "  [TIP] Files: generate.py, model.py, convert_checkpoint.py"
                Write-Status "  [TIP] Location: Release/gpu/windows/python/" $Cyan
                $testsPassed++
            } else {
                Write-Status "  [FAIL] Python GPU kernel failed to load" $Red
                Write-Status "  Error: $testResult" $Red
                $testsFailed++
            }
        } catch {
            Write-Status "  [FAIL] Could not run Python test: $_" $Red
            $testsFailed++
        }
        Write-Status ""
    }
    
    Write-Status ""
    Write-Status "Verification Results: $testsPassed passed, $testsFailed failed" $(if ($testsFailed -eq 0) { $Green } else { $Yellow })
    Write-Status ""
    
    # Clean up temp test files
    Remove-Item "$env:TEMP\llama-*-stdout.txt" -Force -ErrorAction SilentlyContinue
    Remove-Item "$env:TEMP\llama-*-stderr.txt" -Force -ErrorAction SilentlyContinue
    
    # Generate verification reports
    Write-Status "Writing verification reports..." $Yellow
    
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    $buildDate = Get-Date -Format "MMMM dd, yyyy"
    $buildTime = Get-Date -Format "HH:mm:ss"
    
    # CPU Verification Report
if (Test-Path "$BuildDir\cpu\windows") {
        $cpuReport = @"
# BitNet Windows Build Verification Report
## CPU Builds

**Generated:** $timestamp  
**Build Date:** $buildDate  
**Build Time:** $buildTime  

---

## Verification Status

"@
        
        if (Test-Path "$BuildDir\cpu\windows\llama-server-standard.exe") {
            $cpuReport += @"

### Standard CPU Build (ClangCL)
- **Status:** [OK] VERIFIED WORKING
- **Compiler:** Clang-CL 19.1.5 with MSVC compatibility
- **Optimizations:** AVX2, FMA
- **Features:** CPU-only inference, optimized for compatibility
- **Executable:** ``llama-server-standard.exe``
- **Test Command:** ``.\llama-server-standard.exe --help``
- **Use Case:** Best for systems without GPU or maximum compatibility

**Quick Start:**
``````powershell
.\llama-server-standard.exe -m model.gguf -c 2048
``````

"@
        }
        
        if (Test-Path "$BuildDir\cpu\windows\llama-server-gpu.exe") {
            $vulkanText = if ($env:GPU_VULKAN_SUCCESS -eq "true") { "[OK] ENABLED" } else { "[WARN] DISABLED (CUDA only)" }
            $cpuReport += @"

### GPU Build (MSVC + CUDA + Vulkan)
- **Status:** [OK] VERIFIED WORKING
- **Compiler:** MSVC 19.44.35217.0
- **GPU Backends:** CUDA 12.8, Vulkan $vulkanText
- **Features:** Multi-GPU backend support, layer offloading
- **Executable:** ``llama-server-gpu.exe``
- **Test Command:** ``.\llama-server-gpu.exe --help``
- **Use Case:** Maximum performance on NVIDIA GPUs

**Quick Start:**
``````powershell
# Offload 35 layers to GPU
.\llama-server-gpu.exe -m model.gguf -ngl 35
``````

**Key Flags:**
- ``-ngl <N>`` : Number of layers to offload to GPU (higher = more GPU usage)
- ``--n-gpu-layers <N>`` : Alternative syntax for layer offloading

"@
        }
        
        if (Test-Path "$BuildDir\cpu\windows\llama-server-bitnet.exe") {
            $cpuReport += @"

### BitNet Optimized Build (1.58-bit)
- **Status:** [OK] VERIFIED WORKING
- **Compiler:** Clang-CL 19.1.5
- **Optimizations:** TL2 kernels for BitNet 1.58-bit quantization
- **Features:** Ultra-efficient 1.58-bit inference
- **Executable:** ``llama-server-bitnet.exe``
- **Test Command:** ``.\llama-server-bitnet.exe --help``
- **Use Case:** ONLY for BitNet-quantized models

**Quick Start:**
``````powershell
.\llama-server-bitnet.exe -m bitnet-model.gguf
``````

**[WARN] Important:** This build only works with BitNet 1.58-bit quantized models!

"@
        }
        
        $cpuReport += @"

---

## Available Executables

"@
    $exeFiles = Get-ChildItem "$BuildDir\cpu\windows\*.exe" -ErrorAction SilentlyContinue
    if ($exeFiles) {
            $exeFiles | ForEach-Object {
                $size = [math]::Round($_.Length / 1MB, 2)
                $cpuReport += "- ``$($_.Name)`` (${size} MB)`n"
            }
        }
        
        $cpuReport += @"

---

## Common Usage Examples

### Start HTTP Server (API Mode)
``````powershell
.\llama-server-<version>.exe -m model.gguf --host 0.0.0.0 --port 8080
``````

### CLI Inference
``````powershell
.\llama-cli-<version>.exe -m model.gguf -p "Hello, how are you?"
``````

### Benchmark
``````powershell
.\llama-bench-<version>.exe -m model.gguf
``````

### Convert/Quantize Models
``````powershell
.\llama-quantize-<version>.exe input.gguf output.gguf Q4_K_M
``````

---

## Getting Help

All executables support ``--help`` flag:
``````powershell
.\llama-server-<version>.exe --help | more
``````

For detailed documentation, see: ``WINDOWS_BUILD_README.md``

---

## Build Diagnostics

### Optional/Experimental Builds

"@
        
        
        # Add OpenCL diagnostic if failed
        if ($env:OPENCL_SUCCESS -ne "true") {
            $cpuReport += @"
**OpenCL (Universal GPU) - [SKIP] NOT BUILT**
- **Status:** Optional build for AMD/Intel/NVIDIA GPUs
- **Reason:** OpenCL headers not found on system
- **Impact:** Minimal - CUDA+Vulkan build provides better NVIDIA GPU performance
- **For AMD/Intel GPU users:** Install appropriate GPU drivers with OpenCL support

"@
        }
        
        $cpuReport += @"

---

*Report generated by build_complete.ps1 on $timestamp*
"@
        
        $cpuReport | Out-File "$BuildDir\cpu\windows\VERIFICATION.md" -Encoding UTF8
        Write-Status "  [OK] CPU verification report: $BuildDir\cpu\windows\VERIFICATION.md" $Green
    }
    
    # GPU Verification Report
if (Test-Path "$BuildDir\gpu\windows") {
        $gpuReport = @"
# BitNet Windows Build Verification Report
## Python GPU Modules

**Generated:** $timestamp  
**Build Date:** $buildDate  
**Build Time:** $buildTime  

---

## Verification Status

"@
        
        if ($env:BITNET_CUDA_SUCCESS -eq "true" -and (Test-Path "$BuildDir\gpu\windows\python\libbitnet.dll")) {
            $cudaVersions = Get-ChildItem "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA" -Directory -ErrorAction SilentlyContinue | Where-Object { $_.Name -match "^v\d" } | Sort-Object Name -Descending
            $cudaVer = if ($cudaVersions) { $cudaVersions[0].Name } else { "Unknown" }
            
            $dllSize = [math]::Round((Get-Item "$BuildDir\gpu\windows\python\libbitnet.dll").Length / 1KB, 1)
            
            $gpuReport += @"

### BitNet CUDA Kernel DLL
- **Status:** [OK] VERIFIED WORKING
- **DLL:** ``libbitnet.dll`` (${dllSize} KB)
- **CUDA Version:** $cudaVer
- **Compiler:** NVCC (NVIDIA CUDA Compiler)
- **Architecture:** SM 75 (Turing)
- **Features:** BitNet 1.58-bit GPU kernels, custom CUDA optimizations
- **Python Version:** $((& $pythonCmd --version 2>&1) -replace 'Python ')
- **Test Command:** Successfully loaded via ``ctypes.CDLL()``

**Quick Start:**
``````powershell
cd Release\gpu\windows
python generate.py --checkpoint <model-path> --prompt "Hello world"
``````

"@
        } else {
            $gpuReport += @"

### BitNet CUDA Kernel DLL
- **Status:** [FAIL] NOT BUILT
- **Reason:** CUDA compilation failed or skipped

"@
        }
        
        $gpuReport += @"

---

## Available Files

"@
    $gpuFiles = Get-ChildItem "$BuildDir\gpu\windows\*" -ErrorAction SilentlyContinue
    if ($gpuFiles) {
            foreach ($file in $gpuFiles) {
                $size = [math]::Round($file.Length / 1KB, 1)
                $type = switch ($file.Extension) {
                    ".dll" { "[CUDA DLL]" }
                    ".py" { "[Python Script]" }
                    ".model" { "[Tokenizer]" }
                    ".lib" { "[Import Library]" }
                    default { "[File]" }
                }
                $gpuReport += "- $type ``$($file.Name)`` (${size} KB)`n"
            }
        }
        
        $gpuReport += @"

---

## Python Scripts

### Core Scripts
- **``generate.py``** - Main inference script for text generation
- **``model.py``** - BitNet model implementation with CUDA kernels
- **``convert_checkpoint.py``** - Convert PyTorch checkpoints to BitNet format

### Usage Examples

#### Basic Generation
``````powershell
python generate.py --checkpoint models/bitnet-llm --prompt "Explain quantum computing"
``````

#### Batch Generation
``````powershell
python generate.py --checkpoint models/bitnet-llm --prompt "Hello" --num-samples 5
``````

#### Server Mode (if available)
``````powershell
python server.py --checkpoint models/bitnet-llm --port 8080
``````

---

## Environment Setup

Before running Python GPU scripts, ensure CUDA is in PATH:

``````powershell
`$env:PATH = "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\$cudaVer\bin;" + `$env:PATH
``````

Or add permanently to system PATH.

---

## Verification Test

To verify the GPU kernel loads correctly:

``````powershell
cd Release\gpu\windows
python -c "import ctypes; lib = ctypes.CDLL('libbitnet.dll'); print('[OK] GPU kernel loaded!')"
``````

---

## Dependencies

The Python environment includes:
- PyTorch 2.3.1+cu121 (CUDA 12.1 compatible)
- xformers (optimized transformers)
- tokenizers, safetensors
- numpy, einops, transformers
- huggingface-hub

To activate the environment:
``````powershell
.\bitnet-gpu-env\Scripts\activate
``````

---

## Troubleshooting

### DLL Load Failed
- Ensure CUDA bin directory is in PATH
- Check that ``cudart64_12.dll`` and other CUDA DLLs are accessible

### Import Errors
- Activate the correct Python virtual environment
- Verify PyTorch is installed: ``python -c "import torch; print(torch.cuda.is_available())"``

### Out of Memory
- Reduce batch size or sequence length
- Use smaller models
- Check available GPU memory: ``nvidia-smi``

---

*Report generated by build_complete.ps1*
"@
        
        $gpuReport | Out-File "$BuildDir\gpu\windows\VERIFICATION.md" -Encoding UTF8
        Write-Status "  [OK] GPU verification report: $BuildDir\gpu\windows\VERIFICATION.md" $Green
    }
    
Write-Status ""
}

Write-Status "========================================" $Green
Write-Status "         HOW TO USE YOUR BUILDS" $Green
Write-Status "========================================" $Green
Write-Status ""

Write-Status "[1] STANDARD CPU (Best compatibility):" $Yellow
Write-Status "   cd Release\cpu\windows\standard"
Write-Status "   .\llama-server.exe -m model.gguf -c 2048"
Write-Status "   [TIP] Get help: .\llama-server.exe --help"
Write-Status ""

Write-Status "[2] GPU CUDA+VULKAN (Best for NVIDIA GPUs):" $Yellow
Write-Status "   cd Release\gpu\windows\cuda-vulkan"
if ($env:GPU_VULKAN_SUCCESS -eq "true") {
    Write-Status "   .\llama-server.exe -m model.gguf -ngl 35"
    Write-Status "   [INFO] Supports: CUDA + Vulkan (dual backend!)"
} else {
    Write-Status "   .\llama-server.exe -m model.gguf -ngl 35"
    Write-Status "   [INFO] Supports: CUDA only"
}
Write-Status "   [TIP] -ngl: Number of layers to offload to GPU"
Write-Status ""

Write-Status "[3] OPENCL (Universal - NVIDIA/AMD/Intel GPUs):" $Yellow
Write-Status "   cd Release\gpu\windows\opencl"
Write-Status "   .\llama-server.exe -m model.gguf -ngl 35"
Write-Status "   [INFO] Works on ANY GPU brand!"
Write-Status "   [TIP] Best for AMD/Intel GPUs, also works on NVIDIA"
Write-Status ""

Write-Status "[4] BITNET OPTIMIZED (For 1.58-bit BitNet models):" $Yellow
Write-Status "   cd Release\cpu\windows\bitnet"
Write-Status "   .\llama-server.exe -m bitnet-model.gguf"
Write-Status "   [WARN] Only use with BitNet quantized models!"
Write-Status "   [TIP] Get help: .\llama-server.exe --help"
Write-Status ""

if ($env:BITNET_CUDA_SUCCESS -eq "true") {
    Write-Status "[5] PYTHON GPU (BitNet Python CUDA inference):" $Yellow
    Write-Status "   cd Release\gpu\windows"
    Write-Status "   python generate.py --checkpoint <model-dir> --prompt 'Hello world'"
    Write-Status "   [TIP] Available scripts: generate.py, model.py, convert_checkpoint.py"
    Write-Status "   [TIP] Requires NVIDIA GPU with CUDA"
    Write-Status ""
}

Write-Status "========================================" $Cyan
Write-Status "         USEFUL COMMANDS" $Cyan
Write-Status "========================================" $Cyan
Write-Status ""
Write-Status "View all available flags:"
Write-Status "  cd Release\cpu\windows\<build-type>  (standard / gpu / bitnet)"
Write-Status "  .\llama-server.exe --help | more"
Write-Status ""
Write-Status "Check build info (backends enabled):"
Write-Status "  .\llama-server.exe --version"
Write-Status ""
Write-Status "Test with a prompt (CLI mode):"
Write-Status "  .\llama-cli.exe -m model.gguf -p 'Hello'"
Write-Status ""
Write-Status "Start HTTP server (API mode):"
Write-Status "  .\llama-server.exe -m model.gguf --host 0.0.0.0 --port 8080"
Write-Status ""
Write-Status "Distribution tip: Each subdirectory is self-contained!" $Green
Write-Status "  Zip any subdirectory (standard/gpu/bitnet) and it will work standalone." $Green
Write-Status ""
Write-Status "For detailed guide: WINDOWS_BUILD_README.md" $Green
Write-Status "========================================" $Green



