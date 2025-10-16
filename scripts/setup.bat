@echo off
echo Setting up Tab Agent Native Host Environment...
echo.

REM Check if Python is installed
python --version >nul 2>&1
if %errorlevel% neq 0 (
    echo Error: Python is not installed or not in PATH
    echo Please install Python 3.7 or later and try again
    pause
    exit /b 1
)

REM Create virtual environment
echo Creating virtual environment...
python -m venv venv
if %errorlevel% neq 0 (
    echo Error: Failed to create virtual environment
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

REM Upgrade pip
echo Upgrading pip...
python -m pip install --upgrade pip
if %errorlevel% neq 0 (
    echo Error: Failed to upgrade pip
    pause
    exit /b 1
)

REM Install PyInstaller for building executables
echo Installing PyInstaller...
pip install pyinstaller
if %errorlevel% neq 0 (
    echo Error: Failed to install PyInstaller
    pause
    exit /b 1
)

REM Install any other requirements
if exist requirements.txt (
    echo Installing requirements...
    pip install -r requirements.txt
    if %errorlevel% neq 0 (
        echo Error: Failed to install requirements
        pause
        exit /b 1
    )
)

echo.
echo Setup completed successfully!
echo.

REM Ask if user wants to build now
set /p BUILD="Build executable now? (Y/n): "
if /i "%BUILD%"=="n" (
    echo.
    echo Skipping build. To build later, run:
    echo   cd build-tool
    echo   build.bat
    echo.
    pause
    exit /b 0
)

echo.
echo Building executable...
echo.
cd build-tool
call build.bat
cd ..

echo.
echo All done! Executable ready in TabAgentDist\NativeApp\
echo.
pause