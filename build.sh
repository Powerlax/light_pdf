#!/bin/bash
# Build script for light_pdf with pdfium installation

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

# Check and install pdfium if needed
echo "Checking for pdfium library..."

install_pdfium() {
    case "$OS" in
        "linux")
            echo "Installing pdfium for Linux..."
            # Check if apt is available
            if command -v apt-get &> /dev/null; then
                echo "Note: libpdfium is not in standard apt repositories."
                echo "Downloading pre-built pdfium library..."
                
                # Create libs directory
                mkdir -p libs
                cd libs
                
                # Download pdfium from Google's releases
                PDFIUM_URL="https://github.com/bblanchon/pdfium-binaries/releases/download/chromium%2F6721/pdfium-linux-x64.tgz"
                if [ ! -f "pdfium-linux-x64.tgz" ]; then
                    echo "Downloading pdfium..."
                    curl -L "$PDFIUM_URL" -o pdfium-linux-x64.tgz
                fi
                
                # Extract
                if [ ! -d "pdfium-linux-x64" ]; then
                    echo "Extracting pdfium..."
                    tar -xzf pdfium-linux-x64.tgz -C .
                fi
                
                # Set environment variable for build
                export PDFIUM_DYNAMIC_LIB_PATH="$(pwd)/lib"
                echo "Pdfium installed to: $(pwd)/lib"
                cd ..
            fi
            ;;
        "macos")
            echo "Installing pdfium for macOS..."
            if command -v brew &> /dev/null; then
                # Homebrew doesn't have pdfium, so download manually
                mkdir -p libs
                cd libs
                
                PDFIUM_URL="https://github.com/bblanchon/pdfium-binaries/releases/download/chromium%2F6721/pdfium-mac-arm64.tgz"
                if [ ! -f "pdfium-mac.tgz" ]; then
                    echo "Downloading pdfium..."
                    curl -L "$PDFIUM_URL" -o pdfium-mac.tgz
                fi
                
                if [ ! -d "pdfium-mac" ]; then
                    echo "Extracting pdfium..."
                    tar -xzf pdfium-mac.tgz -C .
                fi
                
                export PDFIUM_DYNAMIC_LIB_PATH="$(pwd)/lib"
                echo "Pdfium installed to: $(pwd)/lib"
                cd ..
            fi
            ;;
        "windows")
            echo "Installing pdfium for Windows..."
            mkdir -p libs
            cd libs
            
            PDFIUM_URL="https://github.com/bblanchon/pdfium-binaries/releases/download/chromium%2F6721/pdfium-win-x64.tgz"
            if [ ! -f "pdfium-win.tgz" ]; then
                echo "Downloading pdfium..."
                curl -L "$PDFIUM_URL" -o pdfium-win.tgz
            fi
            
            if [ ! -d "pdfium-win" ]; then
                echo "Extracting pdfium..."
                tar -xzf pdfium-win.tgz -C .
            fi
            
            export PDFIUM_DYNAMIC_LIB_PATH="$(pwd)/bin"
            echo "Pdfium installed to: $(pwd)/bin"
            cd ..
            ;;
        *)
            echo "Unknown OS. Please install pdfium manually."
            exit 1
            ;;
    esac
}

# Install pdfium
install_pdfium

echo ""
echo "Building with pdfium support..."
echo ""

# Run tests
echo "Running tests..."
cargo test

echo ""
echo "Building release version..."
cargo build --release

echo ""
echo "=== Build Complete ==="
echo "Binary location: target/release/light_pdf"
echo ""
echo "Note: This build uses pdfium for PDF rendering."
echo "The pdfium library has been downloaded to ./libs/"
echo "Make sure to set LD_LIBRARY_PATH (Linux) or DYLD_LIBRARY_PATH (macOS)"
echo "when running the application, or copy the library to a system location."
