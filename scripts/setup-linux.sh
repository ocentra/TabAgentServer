#!/bin/bash
echo "Setting up Tab Agent Native Host Environment (Linux)..."
echo

# Check if Python is installed
if ! command -v python3 &> /dev/null; then
    echo "Error: Python 3 is not installed or not in PATH"
    echo "Please install Python 3.7 or later and try again"
    exit 1
fi

echo "Python version:"
python3 --version
echo

# Create virtual environment
echo "Creating virtual environment..."
python3 -m venv venv
if [ $? -ne 0 ]; then
    echo "Error: Failed to create virtual environment"
    exit 1
fi

# Activate virtual environment
echo "Activating virtual environment..."
source venv/bin/activate
if [ $? -ne 0 ]; then
    echo "Error: Failed to activate virtual environment"
    exit 1
fi

# Upgrade pip
echo "Upgrading pip..."
python -m pip install --upgrade pip
if [ $? -ne 0 ]; then
    echo "Error: Failed to upgrade pip"
    exit 1
fi

# Install PyInstaller for building executables
echo "Installing PyInstaller..."
pip install pyinstaller
if [ $? -ne 0 ]; then
    echo "Error: Failed to install PyInstaller"
    exit 1
fi

# Install any other requirements
if [ -f requirements.txt ]; then
    echo "Installing requirements..."
    pip install -r requirements.txt
    if [ $? -ne 0 ]; then
        echo "Error: Failed to install requirements"
        exit 1
    fi
fi

echo
echo "Setup completed successfully!"
echo

# Ask if user wants to build now
read -p "Build executable now? (Y/n): " BUILD
if [[ "$BUILD" == "n" || "$BUILD" == "N" ]]; then
    echo
    echo "Skipping build. To build later, run:"
    echo "  cd build-tool && ./build-linux.sh"
    echo
    exit 0
fi

echo
echo "Building executable..."
echo
cd build-tool
./build-linux.sh
cd ..

echo
echo "All done! Executable ready in TabAgentDist/NativeApp/"
echo

