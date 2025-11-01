# Auto-detect and set LIBCLANG_PATH for Windows builds
# Run this before building on Windows if you don't have LIBCLANG_PATH set

Write-Host "Detecting libclang.dll..." -ForegroundColor Cyan

# Check if already set
if ($env:LIBCLANG_PATH) {
    Write-Host "LIBCLANG_PATH already set: $env:LIBCLANG_PATH" -ForegroundColor Green
    exit 0
}

# Common locations
$candidates = @(
    # Visual Studio 2022
    "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\Llvm\x64\bin",
    "C:\Program Files\Microsoft Visual Studio\2022\Professional\VC\Tools\Llvm\x64\bin",
    "C:\Program Files\Microsoft Visual Studio\2022\Enterprise\VC\Tools\Llvm\x64\bin",
    "C:\Program Files\Microsoft Visual Studio\2022\BuildTools\VC\Tools\Llvm\x64\bin",
    # Visual Studio 2019
    "C:\Program Files (x86)\Microsoft Visual Studio\2019\Community\VC\Tools\Llvm\x64\bin",
    "C:\Program Files (x86)\Microsoft Visual Studio\2019\Professional\VC\Tools\Llvm\x64\bin",
    "C:\Program Files (x86)\Microsoft Visual Studio\2019\Enterprise\VC\Tools\Llvm\x64\bin",
    "C:\Program Files (x86)\Microsoft Visual Studio\2019\BuildTools\VC\Tools\Llvm\x64\bin",
    # MSYS2
    "C:\msys64\mingw64\bin",
    "C:\msys64\clang64\bin",
    # LLVM standalone
    "C:\Program Files\LLVM\bin",
    "C:\Program Files (x86)\LLVM\bin"
)

# Try each candidate
foreach ($path in $candidates) {
    $libclang = Join-Path $path "libclang.dll"
    if (Test-Path $libclang) {
        Write-Host "Found libclang.dll at: $path" -ForegroundColor Green
        $env:LIBCLANG_PATH = $path
        Write-Host "`nRun this to set permanently:" -ForegroundColor Yellow
        Write-Host "[Environment]::SetEnvironmentVariable('LIBCLANG_PATH', '$path', 'User')" -ForegroundColor White
        exit 0
    }
}

# Last resort: try where.exe
$whereOutput = & where.exe libclang.dll 2>$null
if ($whereOutput) {
    $foundPath = Split-Path -Parent $whereOutput[0]
    Write-Host "Found libclang.dll via where.exe at: $foundPath" -ForegroundColor Green
    $env:LIBCLANG_PATH = $foundPath
    Write-Host "`nRun this to set permanently:" -ForegroundColor Yellow
    Write-Host "[Environment]::SetEnvironmentVariable('LIBCLANG_PATH', '$foundPath', 'User')" -ForegroundColor White
    exit 0
}

# Not found
Write-Host "`nERROR: Could not find libclang.dll!" -ForegroundColor Red
Write-Host "`nPlease do ONE of the following:" -ForegroundColor Yellow
Write-Host "1. Install Visual Studio with C++ tools, OR" -ForegroundColor White
Write-Host "2. Install LLVM, OR" -ForegroundColor White
Write-Host "3. Set LIBCLANG_PATH manually: `$env:LIBCLANG_PATH = 'path\to\libclang.dll\directory'" -ForegroundColor White
exit 1

