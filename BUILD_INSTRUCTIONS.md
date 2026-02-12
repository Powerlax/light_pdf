# Building for Linux and Windows from WSL

The `build.sh` script now supports building binaries for both Linux and Windows platforms from a single WSL (Windows Subsystem for Linux) or Linux environment.

## Prerequisites

The script will automatically install the required dependencies:
- **mingw-w64**: Cross-compiler toolchain for Windows targets
- **Rust Windows target**: x86_64-pc-windows-gnu

## Usage

### Build for both Linux and Windows (default)
```bash
./build.sh
```

This will:
1. Download pdfium libraries for both platforms
2. Run tests
3. Build Linux binary: `target/release/light_pdf`
4. Build Windows binary: `target/x86_64-pc-windows-gnu/release/light_pdf.exe`
5. Copy `pdfium.dll` alongside the Windows executable

### Build for Linux only
```bash
./build.sh --linux-only
```

### Build for Windows only
```bash
./build.sh --windows-only
```

## Output

After a successful build, you'll find:

### Linux Binary
- **Location**: `target/release/light_pdf`
- **Size**: ~17 MB
- **Type**: ELF 64-bit executable
- **Runtime**: Requires `libpdfium.so` (included in `libs/linux/lib/`)

### Windows Binary
- **Location**: `target/x86_64-pc-windows-gnu/release/light_pdf.exe`
- **Size**: ~27 MB
- **Type**: PE32+ Windows executable
- **Runtime**: `pdfium.dll` is automatically copied to the same directory

## Directory Structure

```
light_pdf/
├── build.sh                    # Build script
├── libs/                       # Downloaded libraries (auto-created, git-ignored)
│   ├── linux/
│   │   └── lib/
│   │       └── libpdfium.so
│   └── windows/
│       └── bin/
│           └── pdfium.dll
├── target/
│   ├── release/
│   │   └── light_pdf           # Linux binary
│   └── x86_64-pc-windows-gnu/
│       └── release/
│           ├── light_pdf.exe   # Windows binary
│           └── pdfium.dll      # Windows DLL (auto-copied)
```

## Running the Binaries

### Linux
```bash
# Set library path
export LD_LIBRARY_PATH=./libs/linux/lib:$LD_LIBRARY_PATH
./target/release/light_pdf
```

Or install the library system-wide:
```bash
sudo cp libs/linux/lib/libpdfium.so /usr/local/lib/
sudo ldconfig
./target/release/light_pdf
```

### Windows
Simply run the executable - the DLL is in the same directory:
```cmd
target\x86_64-pc-windows-gnu\release\light_pdf.exe
```

Or copy both files to your desired location:
```powershell
# From WSL, copy to Windows location
cp target/x86_64-pc-windows-gnu/release/light_pdf.exe /mnt/c/Users/YourName/Desktop/
cp target/x86_64-pc-windows-gnu/release/pdfium.dll /mnt/c/Users/YourName/Desktop/
```

## First-Time Setup

The build script automatically handles setup, but if you need to do it manually:

1. Install mingw-w64:
```bash
sudo apt-get update
sudo apt-get install mingw-w64
```

2. Add Windows Rust target:
```bash
rustup target add x86_64-pc-windows-gnu
```

## Troubleshooting

### "mingw-w64 not found"
The script will try to install it automatically. If it fails:
```bash
sudo apt-get update
sudo apt-get install mingw-w64
```

### "Windows target not installed"
Run:
```bash
rustup target add x86_64-pc-windows-gnu
```

### Linking errors
Make sure mingw-w64 is properly installed:
```bash
which x86_64-w64-mingw32-gcc
```

Should output: `/usr/bin/x86_64-w64-mingw32-gcc`

## Clean Build

To start fresh:
```bash
rm -rf target/ libs/
./build.sh
```

## Notes

- The libs/ directory is gitignored - pdfium libraries are downloaded on each machine
- Build time: ~3-5 minutes for both platforms (faster on subsequent builds)
- Pdfium libraries are cached after first download (~3MB download per platform)
