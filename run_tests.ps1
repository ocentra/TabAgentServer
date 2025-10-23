# Test runner for TabAgent (PowerShell)
# Runs all Rust and Python tests
#
# Usage:
#   .\run_tests.ps1              # Run all tests
#   .\run_tests.ps1 model-cache  # Run specific crate
#   .\run_tests.ps1 python       # Run only Python tests
#
# TDD inference tests (will FAIL - not implemented yet):
#   cd tabagent-rs
#   cargo test --package model-loader -- --ignored --nocapture
#   cargo test --package tabagent-native-handler -- --ignored --nocapture

param(
    [string]$Target = "all"
)

$ErrorActionPreference = "Stop"

Write-Host "==============================" -ForegroundColor Cyan
Write-Host "🧪 TABAGENT TEST SUITE" -ForegroundColor Cyan
Write-Host "==============================" -ForegroundColor Cyan
Write-Host ""

# Navigate to Rust workspace
$scriptPath = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location "$scriptPath\tabagent-rs"

# Handle different targets
if ($Target -eq "python") {
    # Python only
    Set-Location $scriptPath
    Write-Host "🐍 Running Python tests..." -ForegroundColor Yellow
    Write-Host ""
    
    Write-Host "1️⃣  Testing secrets..." -ForegroundColor White
    python -m pytest tests/test_secrets.py -v --tb=short
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Secrets tests failed" -ForegroundColor Red
        exit 1
    }
    
    Write-Host ""
    Write-Host "2️⃣  Testing Python↔Rust bridge..." -ForegroundColor White
    python -m pytest tests/test_python_rust_bridge.py -v --tb=short
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Python↔Rust bridge tests failed" -ForegroundColor Red
        exit 1
    }
    
    Write-Host ""
    Write-Host "3️⃣  Testing API endpoints..." -ForegroundColor White
    python -m pytest tests/test_api_endpoints.py -v --tb=short
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ API endpoint tests failed" -ForegroundColor Red
        exit 1
    }
    
    Write-Host ""
    Write-Host "4️⃣  Testing backend services..." -ForegroundColor White
    python -m pytest tests/test_backend_real.py -v --tb=short
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Backend tests failed" -ForegroundColor Red
        exit 1
    }
    
    Write-Host ""
    Write-Host "✅ All Python tests passed!" -ForegroundColor Green
    exit 0
}

# Check if running single crate
$singleCrate = $null
switch ($Target) {
    "model-cache" { $singleCrate = "tabagent-model-cache" }
    "hardware" { $singleCrate = "tabagent-hardware" }
    "storage" { $singleCrate = "storage" }
    "query" { $singleCrate = "query" }
    "model-loader" { $singleCrate = "model-loader" }
    "native-handler" { $singleCrate = "tabagent-native-handler" }
}

if ($singleCrate) {
    Write-Host "📦 Testing $Target..." -ForegroundColor Yellow
    Write-Host ""
    
    cargo test --package $singleCrate --lib --tests -- --nocapture
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ $Target tests failed" -ForegroundColor Red
        exit 1
    }
    
    Write-Host ""
    Write-Host "✅ $Target tests passed!" -ForegroundColor Green
    exit 0
}

# Run all tests
Write-Host "📦 Running ALL Rust tests..." -ForegroundColor Yellow
Write-Host ""

$crates = @(
    @{Name="model-cache"; Package="tabagent-model-cache"},
    @{Name="hardware"; Package="tabagent-hardware"},
    @{Name="storage"; Package="storage"},
    @{Name="query"; Package="query"},
    @{Name="model-loader"; Package="model-loader"},
    @{Name="native-handler"; Package="tabagent-native-handler"}
)

$i = 1
foreach ($crate in $crates) {
    Write-Host "$i️⃣  Testing $($crate.Name)..." -ForegroundColor White
    cargo test --package $crate.Package --lib --tests -- --nocapture
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ $($crate.Name) tests failed" -ForegroundColor Red
        exit 1
    }
    Write-Host ""
    $i++
}

Write-Host "✅ All Rust tests passed!" -ForegroundColor Green
Write-Host ""

# Python tests
Set-Location $scriptPath
Write-Host "🐍 Running Python tests..." -ForegroundColor Yellow
Write-Host ""

python tests/test_secrets.py
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Python tests failed" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "✅ All Python tests passed!" -ForegroundColor Green
Write-Host ""

Write-Host "==============================" -ForegroundColor Cyan
Write-Host "🎉 ALL TESTS PASSED!" -ForegroundColor Green
Write-Host "==============================" -ForegroundColor Cyan

