#!/bin/bash
# Build script for light_pdf with static pdfium linking

set -e

echo "=== Light PDF Build Script ==="
echo ""

# Function to detect OS
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

OS=$(detect_os)
echo "Detected OS: $OS"
echo ""

# Check if static feature is enabled (it should be in Cargo.toml)
echo "Building with static pdfium linking..."
echo "The 'static' feature in pdfium-render will automatically download and"
echo "statically link the appropriate pdfium binaries for your platform."
echo ""

# Run tests
echo "Running tests..."
cargo test

echo ""
echo "Building release version..."
cargo build --release

# For Windows cross-compilation (if applicable)
if [ "$1" == "--windows" ]; then
    echo ""
    echo "Cross-compiling for Windows..."
    cargo build --target x86_64-pc-windows-gnu --release
    
    if [ -f "target/x86_64-pc-windows-gnu/release/light_pdf.exe" ]; then
        echo "Windows build successful!"
        echo "Binary location: target/x86_64-pc-windows-gnu/release/light_pdf.exe"
    fi
fi

echo ""
echo "=== Build Complete ==="
echo "Binary location: target/release/light_pdf"
echo ""
echo "Note: This build uses pdfium-render with static linking."
echo "The pdfium library is automatically downloaded and statically linked"
echo "during the build process, so no runtime .so/.dll files are needed!"
