# Build BitNet CUDA kernel as DLL (not Python extension)
# This matches how Linux builds it as .so

# Set up VS 2022 environment
$vsPath = "C:\Program Files\Microsoft Visual Studio\2022\Community"
$msvcPath = "$vsPath\VC\Tools\MSVC\14.44.35207"

$env:PATH = "$msvcPath\bin\Hostx64\x64;$env:PATH"
$env:INCLUDE = "$msvcPath\include;C:\Program Files (x86)\Windows Kits\10\Include\10.0.26100.0\ucrt;C:\Program Files (x86)\Windows Kits\10\Include\10.0.26100.0\shared;C:\Program Files (x86)\Windows Kits\10\Include\10.0.26100.0\um"
$env:LIB = "$msvcPath\lib\x64;C:\Program Files (x86)\Windows Kits\10\Lib\10.0.26100.0\ucrt\x64;C:\Program Files (x86)\Windows Kits\10\Lib\10.0.26100.0\um\x64"

$cudaPath = "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.8"
$env:PATH = "$cudaPath\bin;$env:PATH"
$nvcc = "$cudaPath\bin\nvcc.exe"

Write-Host "Building BitNet CUDA kernel as DLL..." -ForegroundColor Green

& $nvcc bitnet_kernels.cu `
    -o libbitnet.dll `
    --shared `
    -O3 `
    -use_fast_math `
    -std=c++17 `
    "-gencode=arch=compute_75,code=sm_75" `
    -Xcompiler "/MD"

if ($LASTEXITCODE -eq 0) {
    Write-Host "[SUCCESS] libbitnet.dll built!" -ForegroundColor Green
    Copy-Item libbitnet.dll ..\..\Release\gpu\windows\ -Force
    Write-Host "Copied to Release\gpu\windows\" -ForegroundColor Green
} else {
    Write-Host "[FAILED] Build error" -ForegroundColor Red
}

