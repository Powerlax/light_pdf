#!/usr/bin/env python3
"""
Verification script for light_pdf DLL setup.
Checks that all necessary files are in place and configured correctly.
"""

import sys
from pathlib import Path
import json

def check_file(path, description, required=True):
    """Check if a file exists"""
    p = Path(path)
    exists = p.exists()
    status = "✓" if exists else "✗"
    req = " (REQUIRED)" if required and not exists else ""
    print(f"  {status} {description}: {path}{req}")
    return exists

def check_rust_config():
    """Check Rust configuration files"""
    print("\n📦 Rust Configuration:")

    # Check Cargo.toml
    if check_file("Cargo.toml", "Cargo.toml"):
        with open("Cargo.toml") as f:
            content = f.read()
            if "crate-type" in content and "cdylib" in content:
                print("     ✓ Configured as cdylib (DLL)")
            else:
                print("     ✗ NOT configured as cdylib!")

    # Check lib.rs
    if check_file("src/lib.rs", "src/lib.rs"):
        with open("src/lib.rs") as f:
            content = f.read()
            if "pub mod ffi" in content:
                print("     ✓ FFI module declared")
            else:
                print("     ✗ FFI module NOT declared!")

    # Check ffi.rs
    if check_file("src/ffi.rs", "src/ffi.rs (FFI layer)"):
        with open("src/ffi.rs") as f:
            content = f.read()
            if "light_pdf_run" in content:
                print("     ✓ light_pdf_run() function defined")
            else:
                print("     ✗ light_pdf_run() function missing!")

def check_python_files():
    """Check Python files"""
    print("\n🐍 Python Files:")
    check_file("light_pdf_dll.py", "light_pdf_dll.py (loader script)", required=True)

def check_documentation():
    """Check documentation"""
    print("\n📚 Documentation:")
    docs = [
        ("QUICK_START.md", "Quick start guide"),
        ("DLL_RUNNER.md", "Detailed DLL documentation"),
        ("DLL_SOLUTION.md", "Solution overview"),
    ]
    for filename, desc in docs:
        check_file(filename, desc, required=False)

def check_build_artifacts():
    """Check if DLL has been built"""
    print("\n🔨 Build Artifacts:")
    dll_windows = Path("target/release/light_pdf.dll")
    dll_linux = Path("target/release/liblight_pdf.so")
    dll_macos = Path("target/release/liblight_pdf.dylib")

    if dll_windows.exists():
        size_mb = dll_windows.stat().st_size / (1024*1024)
        print(f"  ✓ Windows DLL found: {size_mb:.1f} MB")
    elif dll_linux.exists():
        size_mb = dll_linux.stat().st_size / (1024*1024)
        print(f"  ✓ Linux SO found: {size_mb:.1f} MB")
    elif dll_macos.exists():
        size_mb = dll_macos.stat().st_size / (1024*1024)
        print(f"  ✓ macOS DYLIB found: {size_mb:.1f} MB")
    else:
        print("  ℹ No DLL built yet (run: cargo build --release --features embed-pdfium)")

def main():
    """Run all checks"""
    print("=" * 60)
    print("Light PDF DLL Setup Verification")
    print("=" * 60)

    # Check if we're in the right directory
    if not Path("Cargo.toml").exists():
        print("\n✗ Error: Cargo.toml not found!")
        print("Make sure you're in the light_pdf directory")
        sys.exit(1)

    check_rust_config()
    check_python_files()
    check_documentation()
    check_build_artifacts()

    print("\n" + "=" * 60)
    print("Next Steps:")
    print("=" * 60)
    print("\n1. Build the DLL:")
    print("   cargo build --release --features embed-pdfium")
    print("\n2. Test on this machine:")
    print("   python light_pdf_dll.py target/release/light_pdf.dll")
    print("\n3. Copy to target machine:")
    print("   - target/release/light_pdf.dll")
    print("   - light_pdf_dll.py")
    print("\n4. Run on target machine:")
    print("   python light_pdf_dll.py light_pdf.dll")
    print("\n" + "=" * 60)

if __name__ == "__main__":
    main()

