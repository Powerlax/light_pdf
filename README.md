# Light PDF

A lightweight PDF viewer built with Rust and egui.

## Building

### Prerequisites

- Rust (stable toolchain)
- For cross-compiling to Windows from Linux: `mingw-w64`

### Quick Build

```bash
# Make build script executable
chmod +x build.sh

# Build for both Linux and Windows (default)
./build.sh

# Build only for Linux
./build.sh --linux-only

# Build only for Windows (from Linux/WSL)
./build.sh --windows-only

# Clean and rebuild (if pdfium version changed)
./build.sh --clean --windows-only
```

### Building a Self-Contained Executable

To create a single executable file that includes the pdfium library (no DLL needed):

```bash
# Single-file Windows executable (~40MB)
./build.sh --embed --windows-only

# Single-file Linux executable
./build.sh --embed --linux-only
```

The `--embed` flag embeds the pdfium library (version 7543) into the executable. When run, it extracts the library to a temp directory (`%TEMP%\light_pdf_libs` on Windows, `/tmp/light_pdf_libs` on Linux).

### Distribution Options

#### Option 1: Single Executable (Recommended for easy distribution)
```bash
./build.sh --embed --windows-only
```
Output: `target/x86_64-pc-windows-gnu/release/light_pdf.exe`

Just give users this one file. First launch extracts pdfium to temp dir automatically.

#### Option 2: Executable + DLL (Smaller initial size)
```bash
./build.sh --windows-only
```
Output: 
- `target/x86_64-pc-windows-gnu/release/light_pdf.exe`
- `target/x86_64-pc-windows-gnu/release/pdfium.dll`

Distribute both files together (must be in same directory).

## Runtime Requirements

### Embedded Build (`--embed`)
- No external dependencies - everything is included in the executable

### Dynamic Build (default)
- **Windows**: `pdfium.dll` must be in the same directory as the executable
- **Linux**: Either:
  - `libpdfium.so` in the same directory as the executable
  - Set `LD_LIBRARY_PATH` to include the directory containing `libpdfium.so`
  - Or copy `libpdfium.so` to `/usr/lib` or `/usr/local/lib`

## Pdfium Version Compatibility

This project uses `pdfium-render` 0.8.x which requires pdfium binaries version **7543** from [pdfium-binaries](https://github.com/bblanchon/pdfium-binaries). The build script automatically downloads the correct version.

## Features

- View PDF documents
- Navigate pages with arrow keys, PgUp/PgDn
- Zoom in/out
- Metadata persistence (remembers page position per PDF)
- Keyboard shortcuts (Ctrl+O to open)

## Development

```bash
# Run tests
cargo test

# Run locally with pdfium library
export PDFIUM_DYNAMIC_LIB_PATH="$(pwd)/libs/linux/lib"
cargo run
```

