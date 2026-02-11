#!/usr/bin/env python3
"""
Light PDF Reader Launcher Script

This script launches the Light PDF Reader application.
You can optionally pass a PDF file path as an argument to open it automatically.

Usage:
    python3 launch.py
    python3 launch.py /path/to/document.pdf
"""
import sys
import os

def main():
    # Check if a file path was provided
    if len(sys.argv) > 1:
        pdf_path = sys.argv[1]
        if not os.path.exists(pdf_path):
            print(f"Error: File '{pdf_path}' not found.")
            return 1
        if not pdf_path.lower().endswith('.pdf'):
            print(f"Warning: '{pdf_path}' does not appear to be a PDF file.")
    
    # Import and run the PDF reader
    try:
        import tkinter as tk
        from pdf_reader import PDFReader
        
        root = tk.Tk()
        app = PDFReader(root)
        
        # If a PDF path was provided, open it
        if len(sys.argv) > 1:
            pdf_path = os.path.abspath(sys.argv[1])
            root.after(100, lambda: open_pdf_delayed(app, pdf_path))
        
        root.mainloop()
        return 0
        
    except ImportError as e:
        print(f"Error: Required module not found: {e}")
        print("\nPlease install the required dependencies:")
        print("  pip install -r requirements.txt")
        return 1
    except Exception as e:
        print(f"Error launching PDF reader: {e}")
        import traceback
        traceback.print_exc()
        return 1

def open_pdf_delayed(app, pdf_path):
    """Open a PDF file after the GUI has been initialized"""
    try:
        import fitz
        app.pdf_document = fitz.open(pdf_path)
        app.total_pages = len(app.pdf_document)
        app.current_file = pdf_path
        
        # Load saved position or start at page 0
        saved_page = app.load_reading_position(pdf_path)
        app.current_page = saved_page if saved_page is not None else 0
        
        # Load highlights
        app.load_highlights(pdf_path)
        
        # Display the page
        app.display_page()
        app.update_status(f"Opened: {os.path.basename(pdf_path)}")
    except Exception as e:
        print(f"Error opening PDF: {e}")

if __name__ == "__main__":
    sys.exit(main())
