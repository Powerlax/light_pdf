#!/usr/bin/env python3
"""
Lightweight PDF Reader with text highlighting, scrolling, Ctrl+F search, and reading position save/restore.
"""
import tkinter as tk
from tkinter import filedialog, messagebox, ttk
import fitz  # PyMuPDF
from PIL import Image, ImageTk
import json
import os


class PDFReader:
    def __init__(self, root):
        self.root = root
        self.root.title("Light PDF Reader")
        self.root.geometry("900x700")
        
        # PDF document variables
        self.pdf_document = None
        self.current_page = 0
        self.total_pages = 0
        self.zoom_level = 1.0
        self.current_file = None
        
        # Highlighting and search variables
        self.highlights = {}  # {page_num: [rect1, rect2, ...]}
        self.search_results = []
        self.current_search_index = -1
        
        # Reading position file
        self.position_file = os.path.expanduser("~/.light_pdf_positions.json")
        
        # Create GUI
        self.create_menu()
        self.create_toolbar()
        self.create_canvas()
        self.create_statusbar()
        
        # Bind keyboard shortcuts
        self.root.bind('<Control-o>', lambda e: self.open_pdf())
        self.root.bind('<Control-f>', lambda e: self.show_search_dialog())
        self.root.bind('<Control-h>', lambda e: self.toggle_highlight())
        self.root.bind('<Control-q>', lambda e: self.root.quit())
        self.root.bind('<Left>', lambda e: self.prev_page())
        self.root.bind('<Right>', lambda e: self.next_page())
        self.root.bind('<Prior>', lambda e: self.prev_page())  # Page Up
        self.root.bind('<Next>', lambda e: self.next_page())    # Page Down
        
    def create_menu(self):
        """Create the menu bar"""
        menubar = tk.Menu(self.root)
        self.root.config(menu=menubar)
        
        # File menu
        file_menu = tk.Menu(menubar, tearoff=0)
        menubar.add_cascade(label="File", menu=file_menu)
        file_menu.add_command(label="Open PDF (Ctrl+O)", command=self.open_pdf)
        file_menu.add_separator()
        file_menu.add_command(label="Exit (Ctrl+Q)", command=self.root.quit)
        
        # Edit menu
        edit_menu = tk.Menu(menubar, tearoff=0)
        menubar.add_cascade(label="Edit", menu=edit_menu)
        edit_menu.add_command(label="Find (Ctrl+F)", command=self.show_search_dialog)
        edit_menu.add_command(label="Highlight Text (Ctrl+H)", command=self.toggle_highlight)
        
        # View menu
        view_menu = tk.Menu(menubar, tearoff=0)
        menubar.add_cascade(label="View", menu=view_menu)
        view_menu.add_command(label="Zoom In", command=self.zoom_in)
        view_menu.add_command(label="Zoom Out", command=self.zoom_out)
        view_menu.add_command(label="Reset Zoom", command=self.reset_zoom)
        
    def create_toolbar(self):
        """Create the toolbar with navigation buttons"""
        toolbar = tk.Frame(self.root, bd=1, relief=tk.RAISED)
        toolbar.pack(side=tk.TOP, fill=tk.X)
        
        # Open button
        open_btn = tk.Button(toolbar, text="Open PDF", command=self.open_pdf)
        open_btn.pack(side=tk.LEFT, padx=2, pady=2)
        
        # Navigation buttons
        tk.Button(toolbar, text="◀ Prev", command=self.prev_page).pack(side=tk.LEFT, padx=2, pady=2)
        tk.Button(toolbar, text="Next ▶", command=self.next_page).pack(side=tk.LEFT, padx=2, pady=2)
        
        # Page info
        self.page_label = tk.Label(toolbar, text="No PDF loaded")
        self.page_label.pack(side=tk.LEFT, padx=10)
        
        # Zoom buttons
        tk.Button(toolbar, text="Zoom +", command=self.zoom_in).pack(side=tk.LEFT, padx=2, pady=2)
        tk.Button(toolbar, text="Zoom -", command=self.zoom_out).pack(side=tk.LEFT, padx=2, pady=2)
        tk.Button(toolbar, text="Reset", command=self.reset_zoom).pack(side=tk.LEFT, padx=2, pady=2)
        
        # Search button
        tk.Button(toolbar, text="🔍 Find", command=self.show_search_dialog).pack(side=tk.LEFT, padx=2, pady=2)
        
        # Highlight button
        tk.Button(toolbar, text="📝 Highlight", command=self.toggle_highlight).pack(side=tk.LEFT, padx=2, pady=2)
        
    def create_canvas(self):
        """Create the canvas for displaying PDF pages with scrollbars"""
        # Frame to hold canvas and scrollbars
        self.canvas_frame = tk.Frame(self.root)
        self.canvas_frame.pack(fill=tk.BOTH, expand=True)
        
        # Create scrollbars
        v_scrollbar = tk.Scrollbar(self.canvas_frame, orient=tk.VERTICAL)
        v_scrollbar.pack(side=tk.RIGHT, fill=tk.Y)
        
        h_scrollbar = tk.Scrollbar(self.canvas_frame, orient=tk.HORIZONTAL)
        h_scrollbar.pack(side=tk.BOTTOM, fill=tk.X)
        
        # Create canvas
        self.canvas = tk.Canvas(
            self.canvas_frame,
            bg='gray',
            yscrollcommand=v_scrollbar.set,
            xscrollcommand=h_scrollbar.set
        )
        self.canvas.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        
        # Configure scrollbars
        v_scrollbar.config(command=self.canvas.yview)
        h_scrollbar.config(command=self.canvas.xview)
        
        # Bind mouse wheel for scrolling
        self.canvas.bind("<MouseWheel>", self.on_mouse_wheel)
        self.canvas.bind("<Button-4>", self.on_mouse_wheel)  # Linux scroll up
        self.canvas.bind("<Button-5>", self.on_mouse_wheel)  # Linux scroll down
        
    def create_statusbar(self):
        """Create the status bar"""
        self.statusbar = tk.Label(self.root, text="Ready", bd=1, relief=tk.SUNKEN, anchor=tk.W)
        self.statusbar.pack(side=tk.BOTTOM, fill=tk.X)
        
    def open_pdf(self):
        """Open a PDF file"""
        filename = filedialog.askopenfilename(
            title="Select PDF file",
            filetypes=[("PDF files", "*.pdf"), ("All files", "*.*")]
        )
        
        if filename:
            try:
                # Close previous document if any
                if self.pdf_document:
                    self.pdf_document.close()
                
                # Open new PDF
                self.pdf_document = fitz.open(filename)
                self.total_pages = len(self.pdf_document)
                self.current_file = filename
                
                # Load saved position or start at page 0
                saved_page = self.load_reading_position(filename)
                self.current_page = saved_page if saved_page is not None else 0
                
                # Load highlights for this document
                self.load_highlights(filename)
                
                # Display the page
                self.display_page()
                self.update_status(f"Opened: {os.path.basename(filename)}")
                
            except Exception as e:
                messagebox.showerror("Error", f"Failed to open PDF: {str(e)}")
                
    def display_page(self):
        """Display the current page on the canvas"""
        if not self.pdf_document:
            return
            
        try:
            # Get the page
            page = self.pdf_document[self.current_page]
            
            # Calculate zoom matrix
            mat = fitz.Matrix(self.zoom_level, self.zoom_level)
            
            # Render page to pixmap
            pix = page.get_pixmap(matrix=mat)
            
            # Convert to PIL Image
            img = Image.frombytes("RGB", [pix.width, pix.height], pix.samples)
            
            # Draw highlights on the image
            if self.current_page in self.highlights:
                from PIL import ImageDraw
                draw = ImageDraw.Draw(img, 'RGBA')
                for rect in self.highlights[self.current_page]:
                    # Scale rect by zoom level
                    scaled_rect = [coord * self.zoom_level for coord in rect]
                    draw.rectangle(scaled_rect, fill=(255, 255, 0, 100))  # Yellow highlight
            
            # Convert to PhotoImage
            self.photo = ImageTk.PhotoImage(img)
            
            # Clear canvas and display image
            self.canvas.delete("all")
            self.canvas.create_image(0, 0, anchor=tk.NW, image=self.photo)
            
            # Update canvas scroll region
            self.canvas.config(scrollregion=(0, 0, pix.width, pix.height))
            
            # Update page label
            self.page_label.config(text=f"Page {self.current_page + 1} of {self.total_pages}")
            
            # Save reading position
            if self.current_file:
                self.save_reading_position(self.current_file, self.current_page)
                
        except Exception as e:
            messagebox.showerror("Error", f"Failed to display page: {str(e)}")
            
    def next_page(self):
        """Go to next page"""
        if self.pdf_document and self.current_page < self.total_pages - 1:
            self.current_page += 1
            self.display_page()
            
    def prev_page(self):
        """Go to previous page"""
        if self.pdf_document and self.current_page > 0:
            self.current_page -= 1
            self.display_page()
            
    def zoom_in(self):
        """Increase zoom level"""
        self.zoom_level = min(self.zoom_level + 0.2, 3.0)
        self.display_page()
        
    def zoom_out(self):
        """Decrease zoom level"""
        self.zoom_level = max(self.zoom_level - 0.2, 0.5)
        self.display_page()
        
    def reset_zoom(self):
        """Reset zoom to default"""
        self.zoom_level = 1.0
        self.display_page()
        
    def on_mouse_wheel(self, event):
        """Handle mouse wheel scrolling"""
        if event.num == 4 or event.delta > 0:
            # Scroll up
            self.canvas.yview_scroll(-1, "units")
        elif event.num == 5 or event.delta < 0:
            # Scroll down
            self.canvas.yview_scroll(1, "units")
            
    def show_search_dialog(self):
        """Show the search dialog"""
        if not self.pdf_document:
            messagebox.showinfo("Info", "Please open a PDF file first.")
            return
            
        # Create search dialog
        search_window = tk.Toplevel(self.root)
        search_window.title("Find Text")
        search_window.geometry("400x100")
        search_window.transient(self.root)
        
        # Search entry
        tk.Label(search_window, text="Find:").pack(pady=5)
        search_entry = tk.Entry(search_window, width=50)
        search_entry.pack(pady=5)
        search_entry.focus()
        
        # Button frame
        btn_frame = tk.Frame(search_window)
        btn_frame.pack(pady=5)
        
        def do_search():
            search_text = search_entry.get()
            if search_text:
                self.search_text(search_text)
                search_window.destroy()
                
        def on_enter(event):
            do_search()
            
        search_entry.bind('<Return>', on_enter)
        
        tk.Button(btn_frame, text="Find", command=do_search).pack(side=tk.LEFT, padx=5)
        tk.Button(btn_frame, text="Cancel", command=search_window.destroy).pack(side=tk.LEFT, padx=5)
        
    def search_text(self, search_text):
        """Search for text in the PDF"""
        if not self.pdf_document:
            return
            
        self.search_results = []
        
        # Search through all pages
        for page_num in range(self.total_pages):
            page = self.pdf_document[page_num]
            text_instances = page.search_for(search_text)
            
            if text_instances:
                for inst in text_instances:
                    self.search_results.append((page_num, inst))
                    
        if self.search_results:
            self.current_search_index = 0
            self.show_search_result()
            self.update_status(f"Found {len(self.search_results)} instances of '{search_text}'")
        else:
            messagebox.showinfo("Search", f"Text '{search_text}' not found.")
            self.update_status("Search completed - no results found")
            
    def show_search_result(self):
        """Show the current search result"""
        if not self.search_results or self.current_search_index < 0:
            return
            
        page_num, rect = self.search_results[self.current_search_index]
        
        # Go to the page
        if page_num != self.current_page:
            self.current_page = page_num
            self.display_page()
            
        # Highlight the search result temporarily
        # (You could add visual highlighting here if desired)
        
        self.update_status(
            f"Search result {self.current_search_index + 1} of {len(self.search_results)}"
        )
        
    def toggle_highlight(self):
        """Toggle highlight for selected text area"""
        if not self.pdf_document:
            messagebox.showinfo("Info", "Please open a PDF file first.")
            return
            
        # Create highlight dialog
        highlight_window = tk.Toplevel(self.root)
        highlight_window.title("Add Highlight")
        highlight_window.geometry("400x200")
        highlight_window.transient(self.root)
        
        tk.Label(highlight_window, text="Enter text to highlight:").pack(pady=5)
        highlight_entry = tk.Entry(highlight_window, width=50)
        highlight_entry.pack(pady=5)
        highlight_entry.focus()
        
        def add_highlight():
            highlight_text = highlight_entry.get()
            if highlight_text:
                page = self.pdf_document[self.current_page]
                text_instances = page.search_for(highlight_text)
                
                if text_instances:
                    if self.current_page not in self.highlights:
                        self.highlights[self.current_page] = []
                    
                    # Add all instances as highlights
                    for inst in text_instances:
                        rect = [inst.x0, inst.y0, inst.x1, inst.y1]
                        if rect not in self.highlights[self.current_page]:
                            self.highlights[self.current_page].append(rect)
                    
                    # Save highlights
                    if self.current_file:
                        self.save_highlights(self.current_file)
                    
                    self.display_page()
                    self.update_status(f"Added highlights for '{highlight_text}'")
                    highlight_window.destroy()
                else:
                    messagebox.showinfo("Info", f"Text '{highlight_text}' not found on current page.")
                    
        def on_enter(event):
            add_highlight()
            
        highlight_entry.bind('<Return>', on_enter)
        
        btn_frame = tk.Frame(highlight_window)
        btn_frame.pack(pady=5)
        tk.Button(btn_frame, text="Highlight", command=add_highlight).pack(side=tk.LEFT, padx=5)
        tk.Button(btn_frame, text="Clear All", command=lambda: self.clear_highlights(highlight_window)).pack(side=tk.LEFT, padx=5)
        tk.Button(btn_frame, text="Cancel", command=highlight_window.destroy).pack(side=tk.LEFT, padx=5)
        
    def clear_highlights(self, window=None):
        """Clear all highlights on current page"""
        if self.current_page in self.highlights:
            del self.highlights[self.current_page]
            if self.current_file:
                self.save_highlights(self.current_file)
            self.display_page()
            self.update_status("Cleared highlights on current page")
        if window:
            window.destroy()
            
    def save_reading_position(self, filename, page_num):
        """Save the current reading position"""
        try:
            positions = {}
            if os.path.exists(self.position_file):
                with open(self.position_file, 'r') as f:
                    positions = json.load(f)
                    
            positions[filename] = {
                'page': page_num,
                'zoom': self.zoom_level
            }
            
            with open(self.position_file, 'w') as f:
                json.dump(positions, f, indent=2)
                
        except Exception as e:
            print(f"Error saving reading position: {e}")
            
    def load_reading_position(self, filename):
        """Load the saved reading position"""
        try:
            if os.path.exists(self.position_file):
                with open(self.position_file, 'r') as f:
                    positions = json.load(f)
                    
                if filename in positions:
                    page_num = positions[filename].get('page', 0)
                    self.zoom_level = positions[filename].get('zoom', 1.0)
                    return page_num
                    
        except Exception as e:
            print(f"Error loading reading position: {e}")
            
        return None
        
    def save_highlights(self, filename):
        """Save highlights for the current document"""
        try:
            # Use a separate file for highlights
            highlights_file = self.position_file.replace('.json', '_highlights.json')
            
            all_highlights = {}
            if os.path.exists(highlights_file):
                with open(highlights_file, 'r') as f:
                    all_highlights = json.load(f)
                    
            all_highlights[filename] = self.highlights
            
            with open(highlights_file, 'w') as f:
                json.dump(all_highlights, f, indent=2)
                
        except Exception as e:
            print(f"Error saving highlights: {e}")
            
    def load_highlights(self, filename):
        """Load highlights for the current document"""
        try:
            highlights_file = self.position_file.replace('.json', '_highlights.json')
            
            if os.path.exists(highlights_file):
                with open(highlights_file, 'r') as f:
                    all_highlights = json.load(f)
                    
                if filename in all_highlights:
                    # Convert page numbers from strings to integers
                    self.highlights = {int(k): v for k, v in all_highlights[filename].items()}
                else:
                    self.highlights = {}
            else:
                self.highlights = {}
                
        except Exception as e:
            print(f"Error loading highlights: {e}")
            self.highlights = {}
            
    def update_status(self, message):
        """Update the status bar"""
        self.statusbar.config(text=message)


def main():
    """Main entry point"""
    root = tk.Tk()
    app = PDFReader(root)
    root.mainloop()


if __name__ == "__main__":
    main()
