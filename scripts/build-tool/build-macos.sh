#!/bin/bash

echo "Building Tab Agent Native Host Executable for macOS..."
echo

# Check if virtual environment exists
if [ ! -f "venv/bin/activate" ]; then
    echo "Error: Virtual environment not found"
    echo "Please run setup first"
    exit 1
fi

# Activate virtual environment
echo "Activating virtual environment..."
source venv/bin/activate

# Check if PyInstaller is installed
if ! pip show pyinstaller > /dev/null 2>&1; then
    echo "Error: PyInstaller not found"
    echo "Please run setup first"
    exit 1
fi

# Build the executable
echo "Building executable..."
pyinstaller --onefile --name tabagent-host-macos --hidden-import=json --hidden-import=struct native_host.py

if [ $? -ne 0 ]; then
    echo "Error: Failed to build executable"
    exit 1
fi

echo
echo "Build completed successfully!"
echo
echo "Executable location: dist/tabagent-host-macos"
echo

# Copy the executable to TabAgentDist/NativeApp/binaries/macos
echo "Copying executable to TabAgentDist/NativeApp/binaries/macos..."
cp dist/tabagent-host-macos ../TabAgentDist/NativeApp/binaries/macos/tabagent-host

if [ $? -ne 0 ]; then
    echo "Error: Failed to copy executable to TabAgentDist/NativeApp/binaries/macos"
    exit 1
fi

echo "Successfully copied executable to TabAgentDist/NativeApp/binaries/macos"
echo

# Clean up build artifacts
echo "Cleaning up build artifacts..."
rm -rf dist build
echo "Build artifacts cleaned up."

echo
echo "Build process completed successfully!"
echo