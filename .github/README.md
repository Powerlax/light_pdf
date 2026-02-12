# Light PDF

A lightweight PDF viewer written in Rust with a native GUI.

## Features

✅ **PDF Viewing**
- High-quality PDF page rendering using Google's Chromium PDF engine (pdfium)
- Multi-page navigation with Previous/Next buttons or arrow keys
- Zoom in/out controls (25% to 400%)
- Page position and zoom level saved automatically

✅ **Pure Rust PDF Parsing**
- Fast PDF metadata extraction using lopdf
- Works on all platforms without external dependencies for parsing
- Page count detection

✅ **Native GUI**
- Built with eframe/egui for responsive native UI
- File browser for easy PDF selection
- Keyboard shortcuts (Ctrl+O to open, arrow keys for navigation)

✅ **Cross-Platform**
- Linux, macOS, and Windows support
- Platform-native file dialogs (Windows) or built-in browser (Linux/macOS)

## Quick Start

### Prerequisites

To enable PDF rendering, you need to install the pdfium library. See [PDFIUM_SETUP.md](../PDFIUM_SETUP.md) for detailed instructions.

**Quick install on Linux:**
```bash
wget https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-linux-x64.tgz
tar xzf pdfium-linux-x64.tgz
sudo cp lib/libpdfium.so /usr/local/lib/
sudo ldconfig
```

### Building and Running

```bash
# Clone the repository
git clone https://github.com/Powerlax/light_pdf.git
cd light_pdf

# Build
cargo build --release

# Run
cargo run --release
```

### Without pdfium

The app will still work without pdfium installed, but will show a placeholder instead of rendering PDF pages. All other features (metadata extraction, navigation, etc.) work normally.

## Usage

1. **Open a PDF**: Click File → Open, or press Ctrl+O
2. **Navigate**: Use Prev/Next buttons or Left/Right arrow keys
3. **Zoom**: Click Zoom +/- buttons or the 100% button to reset
4. **Metadata**: Your page position and zoom level are saved automatically

## Architecture

- **src/main.rs**: Entry point, eframe application setup
- **src/app.rs**: UI rendering and event handling
- **src/pdf.rs**: PDF document handling, metadata, and rendering
- **PDFIUM_SETUP.md**: Installation guide for pdfium library
- **RENDERING_IMPLEMENTATION.md**: Technical details of the rendering implementation

## Dependencies

- **eframe/egui**: Native GUI framework
- **pdfium-render**: PDF rendering (requires libpdfium at runtime)
- **lopdf**: Pure Rust PDF parser for metadata
- **image**: Image handling and conversion
- **serde/serde_json**: Metadata serialization

## Documentation

- [PDFIUM_SETUP.md](../PDFIUM_SETUP.md) - How to install pdfium for rendering
- [RENDERING_IMPLEMENTATION.md](../RENDERING_IMPLEMENTATION.md) - Technical implementation details
- [RENDERING_OPTIONS.md](../RENDERING_OPTIONS.md) - Alternative rendering approaches

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

[License information to be added]

