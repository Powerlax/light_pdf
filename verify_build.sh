#!/bin/bash
echo "=== Verifying Build Outputs ==="
echo ""

LINUX_BIN="target/release/light_pdf"
WIN_BIN="target/x86_64-pc-windows-gnu/release/light_pdf.exe"
WIN_DLL="target/x86_64-pc-windows-gnu/release/pdfium.dll"

echo "Checking Linux binary..."
if [ -f "$LINUX_BIN" ]; then
    echo "✓ Linux binary exists: $LINUX_BIN"
    ls -lh "$LINUX_BIN"
    file "$LINUX_BIN"
else
    echo "✗ Linux binary not found: $LINUX_BIN"
fi

echo ""
echo "Checking Windows binary..."
if [ -f "$WIN_BIN" ]; then
    echo "✓ Windows binary exists: $WIN_BIN"
    ls -lh "$WIN_BIN"
    file "$WIN_BIN"
else
    echo "✗ Windows binary not found: $WIN_BIN"
fi

echo ""
echo "Checking Windows DLL..."
if [ -f "$WIN_DLL" ]; then
    echo "✓ Windows DLL exists: $WIN_DLL"
    ls -lh "$WIN_DLL"
else
    echo "✗ Windows DLL not found: $WIN_DLL"
fi

echo ""
echo "=== Summary ==="
if [ -f "$LINUX_BIN" ] && [ -f "$WIN_BIN" ] && [ -f "$WIN_DLL" ]; then
    echo "✓ All binaries built successfully!"
    echo ""
    echo "You can now:"
    echo "  - Run Linux binary: ./target/release/light_pdf"
    echo "  - Copy Windows files to Windows: target/x86_64-pc-windows-gnu/release/{light_pdf.exe,pdfium.dll}"
else
    echo "✗ Some binaries are missing. Run ./build.sh to build."
fi
