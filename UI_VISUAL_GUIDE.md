# PDF Rendering UI - Visual Guide

## Overview

This document provides a visual representation of the PDF rendering UI implementation.

## UI Layout

```
┌────────────────────────────────────────────────────────────────┐
│  Light PDF                                        [Window Bar]  │
├────────────────────────────────────────────────────────────────┤
│  File   Edit                                       [Menu Bar]   │
├────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Selected: /home/user/documents/sample.pdf                     │
│                                                                 │
├────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Opened: sample │ [Prev] [Next] Page: 2 / 10 │                │
│                 │ [Zoom -] 100% [Zoom +] [100%]                │
│                                                                 │
├────────────────────────────────────────────────────────────────┤
│ ╔══════════════════════════════════════════════════════════╗  │
│ ║                                                          ║  │
│ ║                                                          ║  │
│ ║                    PDF PAGE CONTENT                      ║  │
│ ║                                                          ║  │
│ ║               (rendered at current zoom)                 ║  │
│ ║                                                          ║  │
│ ║            [Scrollable if page is large]                 ║  │
│ ║                                                          ║  │
│ ║                                                          ║  │
│ ╚══════════════════════════════════════════════════════════╝  │
│                                                                 │
└────────────────────────────────────────────────────────────────┘
```

## Component Breakdown

### 1. Top Menu Bar
```
File   Edit
 │      │
 │      └─── Preferences...
 │
 └─── Open... (Ctrl+O)
      Quit
```

### 2. Document Info Bar
```
Selected: [full path to PDF file]
```

### 3. Control Bar
```
┌────────────────────────────────────────────────────────────┐
│ Opened: [filename] │ Navigation │ Page Info │ Zoom         │
│                    │            │           │              │
│                    │ [Prev]     │ Page: X/Y │ [Zoom -]    │
│                    │ [Next]     │           │ 100%         │
│                    │            │           │ [Zoom +]     │
│                    │            │           │ [100%]       │
└────────────────────────────────────────────────────────────┘
```

### 4. PDF Viewer Area
```
╔════════════════════════════════════════════════════════════╗
║                                                            ║
║  [Scrollable Content Area]                                ║
║                                                            ║
║  • Displays rendered PDF page                             ║
║  • Scrollbars appear for large/zoomed pages               ║
║  • Real-time rendering with pdfium                        ║
║  • Cached for performance                                 ║
║                                                            ║
╚════════════════════════════════════════════════════════════╝
```

## Interaction Flow

### Opening a PDF

```
User Action: File → Open or Ctrl+O
     │
     ├─ Windows: Native file picker dialog
     │            └─ Select PDF → Auto-open
     │
     └─ Linux/Mac: In-app file browser
                    └─ Navigate to PDF → [Select] button
```

### Navigation Flow

```
Current Page: 1
     │
     ├─ [Next] or Right Arrow
     │   └─→ Page: 2 (cache cleared, re-render)
     │
     └─ [Prev] or Left Arrow
         └─→ Page: 1 (cache cleared, re-render)
```

### Zoom Flow

```
Default Zoom: 100%
     │
     ├─ [Zoom +]
     │   └─→ 125% → 150% → 175% → ... → 400% (max)
     │       └─ Cache cleared, re-render at new size
     │
     ├─ [Zoom -]
     │   └─→ 75% → 50% → 25% (min)
     │       └─ Cache cleared, re-render at new size
     │
     └─ [100%]
         └─→ Reset to 100%
             └─ Cache cleared, re-render
```

## State Management

### Metadata Persistence

```
PdfMetadata (saved to [filename].pdf.meta.json)
├── page: usize        // Current page number (1-based)
└── zoom: f32          // Zoom level (1.0 = 100%)

Saved automatically on:
  • Page change
  • Zoom change
  • Manual "Save Metadata" (removed from UI)

Loaded automatically on:
  • Opening same PDF again
```

### Rendering Cache

```
HashMap<usize, image::DynamicImage>
     │
     ├─ Key: page_num (1-based)
     └─ Value: rendered image at current zoom
     
Cleared on:
  • Page change
  • Zoom change
  • Manual clear_cache() call
```

## Rendering Pipeline

```
User Request: Show page 3 at 150% zoom
     │
     ├─ Check cache for page 3?
     │   ├─ Found → Return cached image ✓
     │   └─ Not found → Continue ↓
     │
     ├─ Get PDF page from pdfium
     │   └─ pdfium_doc.pages().get(page_index)
     │
     ├─ Calculate render dimensions
     │   ├─ Base: 800x1000
     │   └─ Scaled: (800 * 1.5, 1000 * 1.5) = 1200x1500
     │
     ├─ Render with pdfium
     │   └─ page.render_with_config(width, height)
     │
     ├─ Convert bitmap to image
     │   └─ PdfBitmap → RGBA bytes → image::RgbaImage → DynamicImage
     │
     ├─ Store in cache
     │   └─ page_cache.insert(3, image)
     │
     └─ Display in UI
         └─ DynamicImage → egui::ColorImage → egui::TextureHandle → ui.image()
```

## Graceful Fallback (No Pdfium)

When pdfium library is not installed:

```
┌────────────────────────────────────────────────────────────┐
│  Light PDF                                                  │
├────────────────────────────────────────────────────────────┤
│  Selected: /home/user/documents/sample.pdf                 │
├────────────────────────────────────────────────────────────┤
│  Opened: sample │ [Prev] [Next] Page: 2 / 10              │
│                 │ [Zoom -] 100% [Zoom +] [100%]            │
├────────────────────────────────────────────────────────────┤
│                                                             │
│              PDF Loaded Successfully                        │
│                                                             │
│    PDF rendering requires the pdfium library.              │
│    See PDFIUM_SETUP.md for installation instructions.      │
│                                                             │
│    The application can:                                    │
│      ✓ Load and parse PDF files                           │
│      ✓ Extract metadata (page count, etc.)                │
│      ✓ Navigate between pages                             │
│      ✓ Save and load page position and zoom level         │
│                                                             │
│    📚 Install pdfium to enable PDF page rendering          │
│                                                             │
└────────────────────────────────────────────────────────────┘
```

## Keyboard Shortcuts

```
Ctrl+O        →  Open PDF file
Left Arrow    →  Previous page
Right Arrow   →  Next page
(Future: +/-  →  Zoom in/out)
```

## Example Workflow

1. **Launch Application**
   ```
   cargo run
   → Window opens with empty viewer
   ```

2. **Open PDF**
   ```
   File → Open → Select multipage.pdf
   → Loads PDF, shows page 1 at 100%
   → Displays page count: "Page: 1 / 3"
   ```

3. **Navigate**
   ```
   Click [Next] or press Right Arrow
   → Cache cleared for page 1
   → Page 2 rendered and displayed
   → "Page: 2 / 3"
   ```

4. **Zoom In**
   ```
   Click [Zoom +]
   → 100% → 125%
   → Cache cleared
   → Page 2 re-rendered at 125%
   → "125%" displayed
   ```

5. **Close and Reopen**
   ```
   Close app
   → Metadata saved: page=2, zoom=1.25
   
   Reopen same PDF
   → Automatically loads page 2 at 125%
   → Continues where you left off
   ```

## Technical Highlights

### Performance
- **Caching**: Avoids re-rendering same pages
- **On-demand**: Only renders current page
- **Efficient**: RGBA conversion optimized

### Reliability
- **Graceful degradation**: Works without pdfium
- **Error handling**: Catches library load failures
- **Safe unwinding**: Uses catch_unwind for panics

### User Experience
- **Smooth navigation**: Arrow keys + buttons
- **Intuitive zoom**: Simple +/- controls
- **Persistent state**: Remembers position
- **Clear feedback**: Shows page/zoom info

## Conclusion

The PDF rendering UI is now fully functional with:
✅ Complete rendering pipeline  
✅ Interactive zoom controls  
✅ Page caching for performance  
✅ Graceful fallback without pdfium  
✅ Comprehensive documentation  
✅ All tests passing
