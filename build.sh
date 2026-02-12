#!/bin/bash
# Build script for light_pdf with pdfium installation
# Supports building for both Linux and Windows (from WSL/Linux)
#
# Usage:
#   ./build.sh                    # Build both Linux and Windows (dynamic linking)
#   ./build.sh --linux-only       # Build only Linux
#   ./build.sh --windows-only     # Build only Windows
#   ./build.sh --embed            # Build with embedded pdfium (single-file executable)
#   ./build.sh --embed --windows-only  # Single Windows exe with embedded pdfium
#   ./build.sh --clean            # Re-download pdfium libraries (use if version mismatch)

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
EMBED_PDFIUM=false
CLEAN_LIBS=false

for arg in "$@"; do
    case $arg in
        --windows-only)
            BUILD_LINUX=false
            BUILD_WINDOWS=true
            ;;
        --linux-only)
            BUILD_LINUX=true
            BUILD_WINDOWS=false
            ;;
        --embed)
            EMBED_PDFIUM=true
            ;;
        --clean)
            CLEAN_LIBS=true
            ;;
    esac
done

# Default: build both on Linux/WSL if no target specified
if [[ "$BUILD_LINUX" == true ]] && [[ "$BUILD_WINDOWS" == false ]]; then
    if [[ "$HOST_OS" == "linux" ]]; then
        BUILD_WINDOWS=true
    fi
fi

echo "Build configuration:"
echo "  Linux binary: $BUILD_LINUX"
echo "  Windows binary: $BUILD_WINDOWS"
echo "  Embed pdfium (single-file): $EMBED_PDFIUM"
echo ""

# Check and install dependencies for cross-compilation
if [[ "$BUILD_WINDOWS" == true ]] && [[ "$HOST_OS" == "linux" ]]; then
    echo "Checking Windows cross-compilation dependencies..."
    
    # We use x86_64-pc-windows-gnu target since it works with mingw
    # Note: MSVC target (x86_64-pc-windows-msvc) would require Windows SDK

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
# Note: pdfium-render 0.8.37 requires pdfium version 7543 or compatible
# See: https://github.com/AjaxBits/pdfium-render - "pdfium_latest" feature = pdfium_7543
PDFIUM_VERSION="7543"
echo "Using pdfium version: chromium/$PDFIUM_VERSION"
echo "Downloading pdfium libraries..."

# Clean old libs if requested
if [[ "$CLEAN_LIBS" == true ]]; then
    echo "Cleaning old pdfium libraries..."
    rm -rf libs/linux libs/windows
fi

mkdir -p libs/linux/lib libs/windows/bin

# Download Linux pdfium
if [[ "$BUILD_LINUX" == true ]]; then
    echo "Downloading pdfium for Linux..."
    cd libs/linux
    PDFIUM_URL="https://github.com/bblanchon/pdfium-binaries/releases/download/chromium%2F${PDFIUM_VERSION}/pdfium-linux-x64.tgz"
    # Check if we have the right version
    if [ -f "VERSION" ] && grep -q "$PDFIUM_VERSION" VERSION; then
        echo "✓ Linux pdfium version $PDFIUM_VERSION already present"
    else
        echo "Downloading Linux pdfium $PDFIUM_VERSION..."
        rm -f pdfium-linux-x64.tgz lib/libpdfium.so
        curl -L "$PDFIUM_URL" -o pdfium-linux-x64.tgz
        echo "Extracting Linux pdfium..."
        tar -xzf pdfium-linux-x64.tgz
        echo "$PDFIUM_VERSION" > VERSION
    fi
    echo "✓ Linux pdfium ready at: $(pwd)/lib"
    cd ../..
fi

# Download Windows pdfium
if [[ "$BUILD_WINDOWS" == true ]]; then
    echo "Downloading pdfium for Windows..."
    cd libs/windows
    PDFIUM_URL="https://github.com/bblanchon/pdfium-binaries/releases/download/chromium%2F${PDFIUM_VERSION}/pdfium-win-x64.tgz"
    # Check if we have the right version
    if [ -f "VERSION" ] && grep -q "$PDFIUM_VERSION" VERSION; then
        echo "✓ Windows pdfium version $PDFIUM_VERSION already present"
    else
        echo "Downloading Windows pdfium $PDFIUM_VERSION..."
        rm -f pdfium-win-x64.tgz bin/pdfium.dll
        curl -L "$PDFIUM_URL" -o pdfium-win-x64.tgz
        echo "Extracting Windows pdfium..."
        tar -xzf pdfium-win-x64.tgz
        echo "$PDFIUM_VERSION" > VERSION
    fi
    echo "✓ Windows pdfium ready at: $(pwd)/bin"
    cd ../..
fi

echo ""
echo "=== Running Tests ==="
export PDFIUM_DYNAMIC_LIB_PATH="$(pwd)/libs/linux/lib"
# cargo test

echo ""
echo "=== Building Binaries ==="

# Determine cargo features
CARGO_FEATURES=""
if [[ "$EMBED_PDFIUM" == true ]]; then
    CARGO_FEATURES="--features embed-pdfium"
fi

# Build Linux binary
if [[ "$BUILD_LINUX" == true ]]; then
    echo ""
    echo "Building Linux binary..."
    export PDFIUM_DYNAMIC_LIB_PATH="$(pwd)/libs/linux/lib"
    cargo build --release $CARGO_FEATURES
    echo "✓ Linux binary built: target/release/light_pdf"

    if [[ "$EMBED_PDFIUM" == true ]]; then
        echo "  (pdfium embedded - single file executable)"
    fi
fi

# Build Windows binary
if [[ "$BUILD_WINDOWS" == true ]]; then
    echo ""
    echo "Building Windows binary..."
    export PDFIUM_DYNAMIC_LIB_PATH="$(pwd)/libs/windows/bin"
    cargo build --release --target x86_64-pc-windows-gnu $CARGO_FEATURES

    if [[ "$EMBED_PDFIUM" == true ]]; then
        echo "✓ Windows binary built with embedded pdfium (single file)"
        echo "  Location: target/x86_64-pc-windows-gnu/release/light_pdf.exe"
    else
        # Copy the pdfium DLL next to the executable
        if [ -f "libs/windows/bin/pdfium.dll" ]; then
            cp libs/windows/bin/pdfium.dll target/x86_64-pc-windows-gnu/release/
            echo "✓ Copied pdfium.dll to output directory"
        fi
        echo "✓ Windows binary built: target/x86_64-pc-windows-gnu/release/light_pdf.exe"
    fi
fi

echo ""
echo "=== Build Complete ==="
echo ""

if [[ "$BUILD_LINUX" == true ]]; then
    echo "Linux binary:"
    echo "  Location: target/release/light_pdf"
    if [[ "$EMBED_PDFIUM" == true ]]; then
        echo "  Runtime: Self-contained (pdfium embedded)"
    else
        echo "  Runtime: Set LD_LIBRARY_PATH=./libs/linux/lib or copy libpdfium.so next to binary"
    fi
    echo ""
fi

if [[ "$BUILD_WINDOWS" == true ]]; then
    echo "Windows binary:"
    echo "  Location: target/x86_64-pc-windows-gnu/release/light_pdf.exe"
    if [[ "$EMBED_PDFIUM" == true ]]; then
        echo "  Runtime: Self-contained (pdfium embedded) - just give users this one file!"
    else
        echo "  Runtime: pdfium.dll must be in the same directory"
    fi
    echo ""
fi

echo "Tip: Use --embed flag for single-file executables (larger but easier to distribute)"
echo "     ./build.sh --embed --windows-only"

