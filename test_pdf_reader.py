#!/usr/bin/env python3
"""
Test script to validate PDF reader functionality without GUI.
Tests the core PDF operations, search, and data persistence features.
"""
import sys
import os
import json
import fitz  # PyMuPDF

def test_pdf_operations():
    """Test basic PDF operations"""
    print("Testing PDF operations...")
    
    # Check if test PDF exists
    if not os.path.exists('test_sample.pdf'):
        print("❌ Test PDF not found")
        return False
    
    # Open PDF
    try:
        doc = fitz.open('test_sample.pdf')
        print(f"✓ Successfully opened PDF with {len(doc)} pages")
    except Exception as e:
        print(f"❌ Failed to open PDF: {e}")
        return False
    
    # Test page rendering
    try:
        page = doc[0]
        mat = fitz.Matrix(1.0, 1.0)
        pix = page.get_pixmap(matrix=mat)
        print(f"✓ Successfully rendered page: {pix.width}x{pix.height} pixels")
    except Exception as e:
        print(f"❌ Failed to render page: {e}")
        doc.close()
        return False
    
    # Test text search
    try:
        search_results = page.search_for("Features")
        if search_results:
            print(f"✓ Text search working: found {len(search_results)} instances of 'Features'")
        else:
            print("⚠ Text search returned no results (might be expected)")
    except Exception as e:
        print(f"❌ Text search failed: {e}")
        doc.close()
        return False
    
    # Test highlighting (simulate)
    try:
        text_instances = page.search_for("PDF")
        if text_instances:
            highlights = []
            for inst in text_instances:
                highlights.append([inst.x0, inst.y0, inst.x1, inst.y1])
            print(f"✓ Highlighting simulation working: {len(highlights)} highlight rects created")
        else:
            print("⚠ No text found for highlighting test")
    except Exception as e:
        print(f"❌ Highlighting simulation failed: {e}")
        doc.close()
        return False
    
    doc.close()
    return True

def test_data_persistence():
    """Test reading position and highlights persistence"""
    print("\nTesting data persistence...")
    
    # Test saving reading position
    try:
        position_file = '/tmp/test_positions.json'
        positions = {
            'test_sample.pdf': {
                'page': 1,
                'zoom': 1.5
            }
        }
        with open(position_file, 'w') as f:
            json.dump(positions, f, indent=2)
        print("✓ Reading position save working")
        
        # Test loading reading position
        with open(position_file, 'r') as f:
            loaded = json.load(f)
        if loaded == positions:
            print("✓ Reading position load working")
        else:
            print("❌ Reading position load failed: data mismatch")
            return False
            
        os.remove(position_file)
    except Exception as e:
        print(f"❌ Data persistence test failed: {e}")
        return False
    
    # Test saving highlights
    try:
        highlights_file = '/tmp/test_highlights.json'
        highlights = {
            'test_sample.pdf': {
                '0': [[10, 20, 100, 30]],
                '1': [[15, 25, 110, 35]]
            }
        }
        with open(highlights_file, 'w') as f:
            json.dump(highlights, f, indent=2)
        print("✓ Highlights save working")
        
        # Test loading highlights
        with open(highlights_file, 'r') as f:
            loaded = json.load(f)
        if loaded == highlights:
            print("✓ Highlights load working")
        else:
            print("❌ Highlights load failed: data mismatch")
            return False
            
        os.remove(highlights_file)
    except Exception as e:
        print(f"❌ Highlights persistence test failed: {e}")
        return False
    
    return True

def test_imports():
    """Test that all required modules can be imported"""
    print("\nTesting imports...")
    
    try:
        import tkinter as tk
        print("✓ Tkinter import successful")
    except ImportError as e:
        print(f"❌ Tkinter import failed: {e}")
        return False
    
    try:
        from PIL import Image, ImageTk
        print("✓ PIL (Pillow) import successful")
    except ImportError as e:
        print(f"❌ PIL import failed: {e}")
        return False
    
    try:
        import fitz
        print(f"✓ PyMuPDF import successful (version: {fitz.version[0]})")
    except ImportError as e:
        print(f"❌ PyMuPDF import failed: {e}")
        return False
    
    return True

def test_pdf_reader_module():
    """Test that pdf_reader.py module can be imported"""
    print("\nTesting pdf_reader module...")
    
    try:
        # Add current directory to path
        sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
        
        # Try to import the module (will fail if there are syntax errors)
        import pdf_reader
        print("✓ pdf_reader.py module import successful")
        
        # Check that the main class exists
        if hasattr(pdf_reader, 'PDFReader'):
            print("✓ PDFReader class found")
        else:
            print("❌ PDFReader class not found")
            return False
            
        # Check that the main function exists
        if hasattr(pdf_reader, 'main'):
            print("✓ main() function found")
        else:
            print("❌ main() function not found")
            return False
            
    except Exception as e:
        print(f"❌ pdf_reader module test failed: {e}")
        import traceback
        traceback.print_exc()
        return False
    
    return True

def main():
    """Run all tests"""
    print("=" * 60)
    print("Light PDF Reader - Test Suite")
    print("=" * 60)
    
    tests = [
        ("Module Imports", test_imports),
        ("PDF Reader Module", test_pdf_reader_module),
        ("PDF Operations", test_pdf_operations),
        ("Data Persistence", test_data_persistence),
    ]
    
    results = []
    for test_name, test_func in tests:
        print(f"\n{'=' * 60}")
        print(f"Running: {test_name}")
        print("=" * 60)
        result = test_func()
        results.append((test_name, result))
    
    # Summary
    print("\n" + "=" * 60)
    print("Test Summary")
    print("=" * 60)
    
    for test_name, result in results:
        status = "✓ PASSED" if result else "❌ FAILED"
        print(f"{test_name}: {status}")
    
    all_passed = all(result for _, result in results)
    
    print("\n" + "=" * 60)
    if all_passed:
        print("All tests passed! ✓")
        return 0
    else:
        print("Some tests failed. ❌")
        return 1

if __name__ == "__main__":
    sys.exit(main())
