@echo off
REM TabAgent Server Runner (Windows Batch)
REM Automatically kills old instances and starts the server fresh

echo.
echo ===========================================
echo    TabAgent Server Launcher
echo ===========================================
echo.

REM Kill any existing server processes
echo [1/3] Checking for running servers...
tasklist /FI "IMAGENAME eq tabagent-server.exe" 2>NUL | find /I /N "tabagent-server.exe">NUL
if "%ERRORLEVEL%"=="0" (
    echo       Found running instance, killing...
    taskkill /F /IM tabagent-server.exe >NUL 2>&1
    timeout /t 1 /nobreak >NUL
    echo       [OK] Old instance killed
) else (
    echo       [OK] No running servers found
)

echo.
echo [2/3] Building server...
cargo build --bin tabagent-server 2>&1 | findstr /C:"Finished" /C:"error:"
if errorlevel 1 (
    echo       [ERROR] Build failed!
    exit /b 1
)
echo       [OK] Build successful

echo.
echo [3/3] Starting TabAgent Server...
echo ===========================================
echo.

cargo run --bin tabagent-server

