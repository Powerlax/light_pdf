# Quick Start Guide - Light PDF Reader

## Installation

### Step 1: Install Python
Make sure you have Python 3.7 or later installed:
```bash
python3 --version
```

### Step 2: Clone the Repository
```bash
git clone https://github.com/Powerlax/light_pdf.git
cd light_pdf
```

### Step 3: Install Dependencies
```bash
pip install -r requirements.txt
```

## Running the Application

### Method 1: Direct Launch
```bash
python3 pdf_reader.py
```

### Method 2: Using Launcher Script
```bash
python3 launch.py
```

### Method 3: Open a Specific PDF
```bash
python3 launch.py /path/to/your/document.pdf
```

## First Steps

1. **Open a PDF**
   - Click "Open PDF" button
   - Or press Ctrl+O
   - Navigate to your PDF file and click "Open"

2. **Navigate Pages**
   - Use arrow keys (Left/Right) for previous/next page
   - Use mouse wheel to scroll within a page
   - Click "Prev" and "Next" buttons

3. **Search for Text**
   - Press Ctrl+F
   - Type the text you want to find
   - Press Enter

4. **Highlight Text**
   - Press Ctrl+H
   - Type the text you want to highlight
   - Press Enter
   - All instances on the current page will be highlighted

5. **Zoom In/Out**
   - Click "Zoom +" to enlarge
   - Click "Zoom -" to reduce
   - Click "Reset" to return to normal size

## Tips

- Your reading position is automatically saved - you'll return to the same page when you reopen the PDF
- Highlights persist across sessions
- Use keyboard shortcuts for faster navigation
- The status bar at the bottom shows helpful information

## Common Issues

### "No module named 'tkinter'" Error
**On Ubuntu/Debian:**
```bash
sudo apt-get install python3-tk
```

**On Fedora:**
```bash
sudo dnf install python3-tkinter
```

**On macOS:**
Tkinter is usually included with Python. If not, reinstall Python from python.org

**On Windows:**
Tkinter is included with Python by default

### "Failed to open PDF" Error
- Make sure the file is a valid PDF
- Check that you have read permissions for the file
- Try opening a different PDF to verify the application works

### Cannot Find PDF After Opening
- Check the page number - you might be on a later page
- Use arrow keys to navigate through pages
- The status bar shows "Page X of Y"

## Need Help?

See the full documentation:
- [README.md](README.md) - Complete usage guide
- [FEATURES.md](FEATURES.md) - Detailed feature documentation

## Example Session

```bash
# Install and run
cd light_pdf
pip install -r requirements.txt
python3 launch.py

# Then in the application:
# 1. Ctrl+O to open a PDF
# 2. Arrow keys to navigate
# 3. Ctrl+F to search for "example"
# 4. Ctrl+H to highlight "important"
# 5. Ctrl+Q to quit
```

Your reading position and highlights will be saved automatically!
