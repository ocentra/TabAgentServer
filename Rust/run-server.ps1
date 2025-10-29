#!/usr/bin/env pwsh
# TabAgent Server Runner
# Automatically kills old instances and starts the server fresh
# Just double-click this file to start the server!

param(
    [string]$Mode = ""
)

# Auto-navigate to Rust directory (works from any location)
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $ScriptDir
Write-Host "Working directory: $ScriptDir" -ForegroundColor DarkGray

Write-Host "`nTabAgent Server Launcher" -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor DarkGray

# Step 1: Kill any existing server processes
Write-Host "`n[1/3] Checking for running servers..." -ForegroundColor Yellow
$processes = Get-Process -Name "tabagent-server" -ErrorAction SilentlyContinue
if ($processes) {
    Write-Host "      Found $($processes.Count) running instance(s)" -ForegroundColor Yellow
    $processes | ForEach-Object {
        Write-Host "      Killing PID $($_.Id)..." -ForegroundColor Red
        Stop-Process -Id $_.Id -Force -ErrorAction SilentlyContinue
    }
    Start-Sleep -Milliseconds 500
    Write-Host "      [OK] All old instances killed" -ForegroundColor Green
} else {
    Write-Host "      [OK] No running servers found" -ForegroundColor Green
}

# Step 2: Build the server
Write-Host "`n[2/3] Building server..." -ForegroundColor Yellow
$buildOutput = cargo build --bin tabagent-server 2>&1
$buildSuccess = $LASTEXITCODE -eq 0

if ($buildSuccess) {
    Write-Host "      [OK] Build successful" -ForegroundColor Green
} else {
    Write-Host "      [ERROR] Build failed!" -ForegroundColor Red
    Write-Host "`nBuild output:" -ForegroundColor Yellow
    $buildOutput | Select-String -Pattern "error" -Context 2,2
    exit 1
}

# Step 3: Start the server
Write-Host "`n[3/3] Starting TabAgent Server..." -ForegroundColor Yellow

# Determine effective mode
$effectiveMode = if ($Mode) { $Mode.ToLower() } else { "all" }
Write-Host "      Mode: $effectiveMode" -ForegroundColor Cyan

switch ($effectiveMode) {
    "http" {
        Write-Host "      HTTP API      -> http://localhost:3000" -ForegroundColor Green
        Write-Host "      Dashboard     -> http://localhost:3000/" -ForegroundColor Cyan
    }
    "webrtc" {
        Write-Host "      WebRTC        -> http://localhost:8002" -ForegroundColor Green
    }
    "web" {
        Write-Host "      HTTP API      -> http://localhost:3000" -ForegroundColor Green
        Write-Host "      WebRTC        -> http://localhost:8002" -ForegroundColor Green
        Write-Host "      Dashboard     -> http://localhost:3000/" -ForegroundColor Cyan
    }
    "both" {
        Write-Host "      HTTP API      -> http://localhost:3000" -ForegroundColor Green
        Write-Host "      Native Msg    -> stdin/stdout" -ForegroundColor Green
    }
    "all" {
        Write-Host "      HTTP API      -> http://localhost:3000" -ForegroundColor Green
        Write-Host "      WebRTC        -> http://localhost:8002" -ForegroundColor Green
        Write-Host "      Native Msg    -> stdin/stdout" -ForegroundColor Green
    }
    "native" {
        Write-Host "      Native Msg    -> stdin/stdout" -ForegroundColor Green
    }
}

Write-Host "`n=========================================" -ForegroundColor DarkGray
Write-Host "READY! Click to open dashboard:" -ForegroundColor Green -NoNewline
Write-Host " http://localhost:3000/" -ForegroundColor Yellow
Write-Host "=========================================`n" -ForegroundColor DarkGray

# Run the server (let it use defaults if no mode specified)
if ($Mode) {
    cargo run --bin tabagent-server -- --mode $Mode
} else {
    cargo run --bin tabagent-server
}
