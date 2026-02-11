# Light PDF Reader

A lightweight PDF reader built in Python with essential features for comfortable reading and document navigation.

## Features

- 📄 **PDF Viewing**: Open and view PDF documents with clear rendering
- 🔍 **Text Search (Ctrl+F)**: Find text across all pages of your document
- ✨ **Text Highlighting (Ctrl+H)**: Highlight important text that persists across sessions
- 📖 **Reading Position**: Automatically saves and restores your reading position
- 🖱️ **Smooth Scrolling**: Mouse wheel and keyboard navigation support
- 🔎 **Zoom Controls**: Zoom in/out to adjust text size
- ⌨️ **Keyboard Shortcuts**: Efficient navigation with keyboard controls

## Installation

1. Clone this repository:
```bash
git clone https://github.com/Powerlax/light_pdf.git
cd light_pdf
```

2. Install the required dependencies:
```bash
pip install -r requirements.txt
```

## Usage

Run the PDF reader:
```bash
python pdf_reader.py
```

### Opening a PDF

- Click the "Open PDF" button in the toolbar
- Or use the keyboard shortcut: **Ctrl+O**
- Or use the menu: File → Open PDF

### Navigation

- **Next Page**: Right Arrow, Page Down, or "Next ▶" button
- **Previous Page**: Left Arrow, Page Up, or "◀ Prev" button
- **Scroll**: Mouse wheel or scrollbars

### Searching for Text

1. Press **Ctrl+F** or click the "🔍 Find" button
2. Enter the text you want to find
3. Press Enter or click "Find"
4. The reader will show all instances found and navigate to the first one

### Highlighting Text

1. Press **Ctrl+H** or click the "📝 Highlight" button
2. Enter the text you want to highlight
3. Press Enter or click "Highlight"
4. All instances of the text on the current page will be highlighted in yellow
5. Highlights are automatically saved and will appear when you reopen the PDF

To clear highlights on the current page, use the "Clear All" button in the highlight dialog.

### Zoom Controls

- **Zoom In**: Click "Zoom +" or use View → Zoom In
- **Zoom Out**: Click "Zoom -" or use View → Zoom Out
- **Reset Zoom**: Click "Reset" or use View → Reset Zoom

### Reading Position

Your reading position (current page and zoom level) is automatically saved when you navigate through the document. When you reopen the same PDF, you'll be taken back to where you left off.

## Keyboard Shortcuts

- **Ctrl+O**: Open PDF file
- **Ctrl+F**: Find text
- **Ctrl+H**: Highlight text
- **Ctrl+Q**: Quit application
- **Left Arrow / Page Up**: Previous page
- **Right Arrow / Page Down**: Next page

## Data Storage

- Reading positions are saved in `~/.light_pdf_positions.json`
- Highlights are saved in `~/.light_pdf_highlights.json`

## Requirements

- Python 3.7+
- PyMuPDF (fitz) - For PDF rendering
- Pillow (PIL) - For image processing
- Tkinter - For GUI (usually included with Python)

## License

This project is open source and available for educational purposes.
