@echo off
echo Building Tab Agent Native Host Executable...
echo.

REM Check if virtual environment exists
if not exist venv\Scripts\activate.bat (
    echo Error: Virtual environment not found
    echo Please run setup.bat first
    pause
    exit /b 1
)

REM Activate virtual environment
echo Activating virtual environment...
call venv\Scripts\activate.bat
if %errorlevel% neq 0 (
    echo Error: Failed to activate virtual environment
    pause
    exit /b 1
)

REM Check if PyInstaller is installed
pip show pyinstaller >nul 2>&1
if %errorlevel% neq 0 (
    echo Error: PyInstaller not found
    echo Please run setup.bat first
    pause
    exit /b 1
)

REM Build the executable
echo Building executable...
pyinstaller --onefile --name tabagent-host --hidden-import=json --hidden-import=struct native_host.py
if %errorlevel% neq 0 (
    echo Error: Failed to build executable
    pause
    exit /b 1
)

echo.
echo Build completed successfully!
echo.
echo Executable location: dist\tabagent-host.exe
echo.

REM Copy the executable to TabAgentDist\NativeApp\binaries\windows
echo Copying executable to TabAgentDist\NativeApp\binaries\windows...
copy /Y "dist\tabagent-host.exe" "..\TabAgentDist\NativeApp\binaries\windows\tabagent-host.exe"
if %errorlevel% neq 0 (
    echo Error: Failed to copy executable to TabAgentDist\NativeApp\binaries\windows
    pause
    exit /b 1
)

echo Successfully copied executable to TabAgentDist\NativeApp\binaries\windows
echo.

REM Clean up build artifacts
echo Cleaning up build artifacts...
if exist "dist" rmdir /s /q "dist"
if exist "build" rmdir /s /q "build"
echo Build artifacts cleaned up.

echo.
echo Build process completed successfully!
echo.
pause