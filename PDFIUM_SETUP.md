# PDFium Setup Guide

This application uses [pdfium-render](https://crates.io/crates/pdfium-render) to render PDF pages. The pdfium library is required at runtime for PDF rendering to work.

## Current Status

- ✅ PDF parsing and metadata extraction (lopdf - pure Rust, always works)
- ✅ PDF page rendering code is implemented
- ⚠️  PDF rendering requires libpdfium.so at runtime

## Installation

### Linux

#### Option 1: Use pre-built binaries (Recommended)

Download the latest pdfium binary for Linux:

```bash
# For x86_64
wget https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-linux-x64.tgz
tar xzf pdfium-linux-x64.tgz
sudo cp lib/libpdfium.so /usr/local/lib/
sudo ldconfig

# For ARM64
wget https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-linux-arm64.tgz
tar xzf pdfium-linux-arm64.tgz
sudo cp lib/libpdfium.so /usr/local/lib/
sudo ldconfig
```

#### Option 2: Set LD_LIBRARY_PATH

If you don't have sudo access, you can set the library path when running:

```bash
wget https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-linux-x64.tgz
tar xzf pdfium-linux-x64.tgz
export LD_LIBRARY_PATH=$PWD/lib:$LD_LIBRARY_PATH
cargo run
```

### macOS

```bash
# Using Homebrew
brew install pdfium

# Or download manually
wget https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-mac-x64.tgz
tar xzf pdfium-mac-x64.tgz
sudo cp lib/libpdfium.dylib /usr/local/lib/
```

### Windows

Download the Windows binary:

```powershell
# Download from: https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-win-x64.tgz
# Extract and place pdfium.dll in the same directory as the executable or in PATH
```

## Building and Running

### Without pdfium (metadata only)

The app will still work without pdfium, but will show a placeholder instead of rendering PDF pages:

```bash
cargo build
cargo run
```

### With pdfium (full rendering)

After installing pdfium:

```bash
# Linux/macOS with system-installed pdfium
cargo run

# Linux/macOS with local pdfium
LD_LIBRARY_PATH=/path/to/pdfium/lib:$LD_LIBRARY_PATH cargo run

# Windows
# Place pdfium.dll in the same directory or add to PATH
cargo run
```

## Verification

To verify pdfium is working:

```bash
cargo run --example test_render
```

Expected output with pdfium:
```
✓ Page rendered successfully!
  Image dimensions: 800x1000
```

Expected output without pdfium:
```
✗ Page rendering not available (pdfium library not installed)
  This is expected in the CI environment.
```

## Version Compatibility

This application uses `pdfium-render v0.8.37`. Make sure to use a compatible pdfium library version. The latest builds from https://github.com/bblanchon/pdfium-binaries should work.

## Troubleshooting

### "libpdfium.so: cannot open shared object file"

This means pdfium is not installed or not in the library path. Follow the installation instructions above.

### "undefined symbol" errors

This usually means version incompatibility between pdfium-render and the pdfium library. Try downloading the latest pdfium binaries.

### Testing without installation

The app gracefully falls back to showing a "rendering not available" message if pdfium is not installed. This allows testing other features without pdfium.
