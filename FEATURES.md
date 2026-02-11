# Light PDF Reader - Features Documentation

## Overview
Light PDF Reader is a lightweight, Python-based PDF viewer designed for comfortable reading with essential features for document navigation and annotation.

## Core Features

### 1. PDF Viewing
- Opens standard PDF files using PyMuPDF (fitz) library
- High-quality rendering of PDF pages
- Support for multi-page documents
- Clean, minimalist interface

### 2. Navigation
#### Page Navigation
- **Next Page**: Right arrow key, Page Down key, or "Next ▶" button
- **Previous Page**: Left arrow key, Page Up key, or "◀ Prev" button
- Page counter showing current page and total pages

#### Scrolling
- **Mouse Wheel**: Scroll up/down within the current page
- **Scrollbars**: Vertical and horizontal scrollbars for precise navigation
- **Keyboard**: Arrow keys for page-to-page navigation

### 3. Zoom Controls
- **Zoom In**: Increase page size (up to 300% of original)
- **Zoom Out**: Decrease page size (down to 50% of original)
- **Reset Zoom**: Return to 100% zoom level
- Zoom level persists with reading position

### 4. Text Search (Ctrl+F)
#### Features:
- Search across all pages in the document
- Case-sensitive text matching
- Shows total number of search results
- Navigate through search results
- Real-time result counting

#### Usage:
1. Press Ctrl+F or click "🔍 Find" button
2. Enter search text in dialog
3. Press Enter or click "Find"
4. Application jumps to first occurrence
5. Status bar shows result count

### 5. Text Highlighting (Ctrl+H)
#### Features:
- Highlight text in yellow for emphasis
- Highlights all instances of text on current page
- Persistent highlights (saved between sessions)
- Per-page highlight management
- Clear highlights option

#### Usage:
1. Press Ctrl+H or click "📝 Highlight" button
2. Enter text to highlight
3. All instances on current page are highlighted
4. Use "Clear All" to remove highlights from current page

#### Storage:
- Highlights saved in `~/.light_pdf_highlights.json`
- Organized by PDF file path
- Includes page numbers and text coordinates

### 6. Reading Position Persistence
#### Automatically Saved:
- Current page number
- Zoom level
- Timestamp of last read

#### Automatically Restored:
- When reopening the same PDF file
- Reading position loads on application start
- No manual save required

#### Storage:
- Positions saved in `~/.light_pdf_positions.json`
- JSON format for easy inspection
- One entry per PDF file path

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Ctrl+O | Open PDF file |
| Ctrl+F | Find text |
| Ctrl+H | Highlight text |
| Ctrl+Q | Quit application |
| Left Arrow | Previous page |
| Right Arrow | Next page |
| Page Up | Previous page |
| Page Down | Next page |

## Menu Structure

### File Menu
- Open PDF (Ctrl+O)
- Exit (Ctrl+Q)

### Edit Menu
- Find (Ctrl+F)
- Highlight Text (Ctrl+H)

### View Menu
- Zoom In
- Zoom Out
- Reset Zoom

## Technical Details

### Dependencies
- **PyMuPDF (fitz)**: PDF rendering and text extraction
- **Pillow (PIL)**: Image processing for display
- **Tkinter**: GUI framework (included with Python)

### Performance
- Lightweight memory footprint
- Fast page rendering
- Efficient text search
- Minimal CPU usage when idle

### Data Format
All user data (reading positions and highlights) is stored in JSON format for transparency and portability.

Example position file:
```json
{
  "/path/to/document.pdf": {
    "page": 5,
    "zoom": 1.2
  }
}
```

Example highlights file:
```json
{
  "/path/to/document.pdf": {
    "0": [
      [100.0, 200.0, 300.0, 220.0]
    ],
    "3": [
      [50.0, 100.0, 250.0, 120.0],
      [50.0, 150.0, 280.0, 170.0]
    ]
  }
}
```

## Limitations

### Current Limitations:
- Read-only PDF viewing (no editing)
- Basic highlight feature (text matching only, no manual selection)
- No annotation support beyond highlights
- No bookmarks feature
- No printing functionality
- Search is case-sensitive only

### Future Enhancement Ideas:
- Manual text selection for highlighting
- Bookmarks/favorites
- PDF metadata viewing
- Thumbnail view
- Side-by-side page view
- Dark mode
- Print functionality
- Export highlights

## Use Cases

### Ideal For:
- Reading technical documentation
- Reviewing research papers
- Studying textbooks
- Reading eBooks
- Lightweight PDF viewing without heavy PDF editors

### Not Ideal For:
- PDF editing or annotation
- Form filling
- Digital signatures
- PDF creation or conversion
- Complex PDF manipulation

## Compatibility

### Operating Systems:
- Linux (tested)
- macOS (tkinter required)
- Windows (tkinter included)

### Python Versions:
- Python 3.7+
- Tested with Python 3.10+

### PDF Standards:
- Supports standard PDF files
- Compatible with most PDF versions
- Handles text-based PDFs well
- Image-based PDFs supported (viewing only, no text operations)
