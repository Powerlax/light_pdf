#!/bin/bash
# Build script for light_pdf with pdfium installation
# Supports building for both Linux and Windows (from WSL/Linux)

set -e

echo "=== Light PDF Build Script ==="
echo ""

# Function to detect host OS
detect_os() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        echo "linux"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        echo "macos"
    elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]]; then
        echo "windows"
    else
        echo "unknown"
    fi
}

HOST_OS=$(detect_os)
echo "Detected Host OS: $HOST_OS"
echo ""

# Parse command line arguments
BUILD_LINUX=true
BUILD_WINDOWS=false

if [[ "$1" == "--windows-only" ]]; then
    BUILD_LINUX=false
    BUILD_WINDOWS=true
elif [[ "$1" == "--linux-only" ]]; then
    BUILD_LINUX=true
    BUILD_WINDOWS=false
else
    # Default: build both on Linux/WSL
    if [[ "$HOST_OS" == "linux" ]]; then
        BUILD_WINDOWS=true
    fi
fi

echo "Build configuration:"
echo "  Linux binary: $BUILD_LINUX"
echo "  Windows binary: $BUILD_WINDOWS"
echo ""

# Check and install dependencies for cross-compilation
if [[ "$BUILD_WINDOWS" == true ]] && [[ "$HOST_OS" == "linux" ]]; then
    echo "Checking Windows cross-compilation dependencies..."
    
    # Check if mingw-w64 is installed
    if ! command -v x86_64-w64-mingw32-gcc &> /dev/null; then
        echo "Warning: mingw-w64 not found. Installing..."
        echo "You may need to run: sudo apt-get install mingw-w64"
        if command -v apt-get &> /dev/null; then
            sudo apt-get update && sudo apt-get install -y mingw-w64 || echo "Please install mingw-w64 manually"
        fi
    else
        echo "✓ mingw-w64 found"
    fi
    
    # Check if Windows target is installed
    if ! rustup target list --installed | grep -q "x86_64-pc-windows-gnu"; then
        echo "Installing Windows target for Rust..."
        rustup target add x86_64-pc-windows-gnu
    else
        echo "✓ Windows target already installed"
    fi
    echo ""
fi

# Download pdfium libraries
echo "Downloading pdfium libraries..."
mkdir -p libs/linux libs/windows

# Download Linux pdfium
if [[ "$BUILD_LINUX" == true ]]; then
    echo "Downloading pdfium for Linux..."
    cd libs/linux
    PDFIUM_URL="https://github.com/bblanchon/pdfium-binaries/releases/download/chromium%2F6721/pdfium-linux-x64.tgz"
    if [ ! -f "pdfium-linux-x64.tgz" ]; then
        curl -L "$PDFIUM_URL" -o pdfium-linux-x64.tgz
    fi
    if [ ! -d "lib" ]; then
        echo "Extracting Linux pdfium..."
        tar -xzf pdfium-linux-x64.tgz
    fi
    echo "✓ Linux pdfium ready at: $(pwd)/lib"
    cd ../..
fi

# Download Windows pdfium
if [[ "$BUILD_WINDOWS" == true ]]; then
    echo "Downloading pdfium for Windows..."
    cd libs/windows
    PDFIUM_URL="https://github.com/bblanchon/pdfium-binaries/releases/download/chromium%2F6721/pdfium-win-x64.tgz"
    if [ ! -f "pdfium-win-x64.tgz" ]; then
        curl -L "$PDFIUM_URL" -o pdfium-win-x64.tgz
    fi
    if [ ! -d "bin" ]; then
        echo "Extracting Windows pdfium..."
        tar -xzf pdfium-win-x64.tgz
    fi
    echo "✓ Windows pdfium ready at: $(pwd)/bin"
    cd ../..
fi

echo ""
echo "=== Running Tests ==="
cargo test

echo ""
echo "=== Building Binaries ==="

# Build Linux binary
if [[ "$BUILD_LINUX" == true ]]; then
    echo ""
    echo "Building Linux binary..."
    export PDFIUM_DYNAMIC_LIB_PATH="$(pwd)/libs/linux/lib"
    cargo build --release
    echo "✓ Linux binary built: target/release/light_pdf"
fi

# Build Windows binary
if [[ "$BUILD_WINDOWS" == true ]]; then
    echo ""
    echo "Building Windows binary..."
    export PDFIUM_DYNAMIC_LIB_PATH="$(pwd)/libs/windows/bin"
    cargo build --release --target x86_64-pc-windows-gnu
    
    # Copy the pdfium DLL next to the executable
    if [ -f "libs/windows/bin/pdfium.dll" ]; then
        cp libs/windows/bin/pdfium.dll target/x86_64-pc-windows-gnu/release/
        echo "✓ Copied pdfium.dll to output directory"
    fi
    echo "✓ Windows binary built: target/x86_64-pc-windows-gnu/release/light_pdf.exe"
fi

echo ""
echo "=== Build Complete ==="
echo ""

if [[ "$BUILD_LINUX" == true ]]; then
    echo "Linux binary:"
    echo "  Location: target/release/light_pdf"
    echo "  Runtime: Set LD_LIBRARY_PATH=./libs/linux/lib or copy pdfium library to system location"
    echo ""
fi

if [[ "$BUILD_WINDOWS" == true ]]; then
    echo "Windows binary:"
    echo "  Location: target/x86_64-pc-windows-gnu/release/light_pdf.exe"
    echo "  Runtime: pdfium.dll is included in the same directory"
    echo ""
fi

echo "Note: Both builds use pdfium for PDF rendering."
echo "Pdfium libraries have been downloaded to ./libs/{linux,windows}/"
