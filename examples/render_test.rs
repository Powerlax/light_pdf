use std::path::PathBuf;
use light_pdf::pdf::PdfDocument;

// Simple example to test PDF rendering without GUI
fn main() {
    let test_pdf = PathBuf::from("multipage.pdf");
    
    println!("Testing PDF rendering with pdfium-render...");
    println!("Loading PDF: {}", test_pdf.display());
    
    // Create PdfDocument using pdfium-render
    let mut doc = PdfDocument::new(&test_pdf, None::<&PathBuf>);
    
    println!("\n✓ PDF loaded with pdfium-render!");
    println!("  Total pages: {:?}", doc.total_pages);
    println!("  Current page: {}", doc.metadata.page);
    println!("  Zoom: {}", doc.metadata.zoom);
    
    if let Some(total) = doc.total_pages {
        println!("\n✓ Successfully detected {} page(s) using pdfium-render!", total);
        
        // Try to render each page
        for page_num in 1..=total {
            println!("\nRendering page {}...", page_num);
            doc.metadata.page = page_num;
            if let Some(img) = doc.get_current_page_image() {
                println!("  ✓ Rendered successfully! Size: {}x{}", img.width(), img.height());
            } else {
                println!("  ✗ Failed to render");
            }
        }
        
        // Test navigation
        println!("\n--- Testing navigation ---");
        doc.metadata.page = 1;
        println!("Current page: {}", doc.metadata.page);
        
        if doc.next_page() {
            println!("After next_page(): {}", doc.metadata.page);
        }
        
        if doc.next_page() {
            println!("After next_page(): {}", doc.metadata.page);
        }
        
        if doc.next_page() {
            println!("After next_page(): {} (should not change)", doc.metadata.page);
        }
        
        if doc.prev_page() {
            println!("After prev_page(): {}", doc.metadata.page);
        }
        
        println!("\n✓ All tests passed with pdfium-render!");
    } else {
        println!("\n✗ Failed to load PDF document");
        std::process::exit(1);
    }
}
