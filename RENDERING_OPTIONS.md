# PDF Rendering Options for light_pdf

## Current State

The application currently uses **lopdf**, a pure Rust PDF parser with **zero external dependencies**. This means:

✅ **Works on all platforms** (Linux, macOS, Windows) without any setup  
✅ **No .so or .dll files required** at build or runtime  
✅ **Can parse PDFs and extract metadata** (page count, etc.)  
✅ **Can navigate between pages**  
✅ **Can save/load viewing position**  

❌ **Cannot render PDF pages to images** (lopdf is a parser, not a renderer)

## Options for Adding PDF Rendering

### Option 1: Keep Current Implementation (lopdf only)
**Pros:**
- Zero dependencies
- Works everywhere out of the box
- Fast compilation
- Small binary size

**Cons:**
- No visual PDF rendering
- Can only extract metadata

**Best for:** PDF metadata viewers, PDF manipulation tools

### Option 2: Add Text Extraction (lopdf + custom parser)
**Pros:**
- Still pure Rust, no external dependencies
- Can display text content from PDFs
- Works on all platforms

**Cons:**
- Text-only view (no images, graphics, or formatting)
- Complex PDF structures may not extract properly

**Implementation:** Add text extraction from PDF content streams (partially working in current code)

### Option 3: Use pdfium-render with Dynamic Linking
**Pros:**
- Full PDF rendering to images
- Production-quality (Google's Chrome PDF engine)
- Rust-friendly API

**Cons:**
- Requires pdfium shared library (.so/.dll) at runtime
- User must install libpdfium separately
- Different setup per platform

**Setup required:**
```bash
# Linux
sudo apt-get install libpdfium-dev

# macOS
brew install pdfium

# Windows
# Download pdfium.dll and place in PATH
```

### Option 4: Use pdfium-render with Static Linking
**Pros:**
- Full PDF rendering
- Self-contained binary (no runtime dependencies)

**Cons:**
- Complex build setup
- Requires downloading pre-built pdfium static libraries
- Large binary size (~10-20 MB)
- Build script must handle multiple platforms

**Build script would need to:**
1. Detect platform (Linux/macOS/Windows)
2. Download appropriate pdfium static library
3. Configure linker flags
4. Set up build environment variables

### Option 5: Use nipdf-render
**Pros:**
- Full-featured PDF rendering
- Good Rust integration

**Cons:**
- Has external dependencies (freetype, jbig2dec, openjpeg)
- These deps require .so files unless statically linked
- Complex static linking setup for multiple libraries

### Option 6: Web-based Rendering (pdf.js via headless browser)
**Pros:**
- Excellent rendering quality
- No linking issues

**Cons:**
- Requires headless browser (Chrome/Chromium)
- Very heavy dependency
- Slow startup time

## Recommended Approach

Given your requirements (libraries OK if statically linkable via build script), I recommend:

**For immediate use:** Keep the current lopdf implementation with added text extraction. This provides useful functionality without any dependency hassles.

**For full rendering:** Implement Option 4 (pdfium-render with static linking) with a comprehensive build.sh script that:
1. Downloads pre-built pdfium static libraries from Google's releases
2. Extracts them to a known location
3. Sets `PDFIUM_DYNAMIC_LIB_PATH` environment variable
4. Configures static linking flags

## Implementation Status

Currently implemented: **Option 1** (lopdf only, no rendering)

Text extraction (Option 2) is partially implemented in the code but needs more work to handle complex PDFs reliably.

Would you like me to:
- A) Complete the text extraction feature (Option 2)?
- B) Set up pdfium with static linking (Option 4)?
- C) Keep it as-is and document the limitations?
