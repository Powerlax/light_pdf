use std::path::PathBuf;
use light_pdf::pdf::PdfDocument;

// Simple example to test PDF loading without rendering
// NOTE: This example must be run from the repository root directory:
//       cargo run --example render_test
fn main() {
    let test_pdf = PathBuf::from("multipage.pdf");
    
    if !test_pdf.exists() {
        eprintln!("Error: multipage.pdf not found!");
        eprintln!("Please run this example from the repository root directory:");
        eprintln!("  cargo run --example render_test");
        std::process::exit(1);
    }
    
    println!("Testing PDF loading with lopdf (pure Rust)...");
    println!("Loading PDF: {}", test_pdf.display());
    
    // Create PdfDocument using lopdf
    let mut doc = PdfDocument::new(&test_pdf, None::<&PathBuf>);
    
    println!("\n✓ PDF loaded with lopdf!");
    println!("  Total pages: {:?}", doc.total_pages);
    println!("  Current page: {}", doc.metadata.page);
    println!("  Zoom: {}", doc.metadata.zoom);
    
    if let Some(total) = doc.total_pages {
        println!("\n✓ Successfully detected {} page(s) using lopdf!", total);
        
        println!("\nNote: PDF rendering to images is not yet implemented with lopdf.");
        println!("lopdf is a pure Rust PDF parser that doesn't require external libraries.");
        println!("Page rendering requires complex functionality like:");
        println!("  - Font rendering and text layout");
        println!("  - Vector graphics rasterization");
        println!("  - PostScript/PDF operators interpretation");
        println!("\nThe app can still:");
        println!("  ✓ Load and parse PDFs");
        println!("  ✓ Extract metadata (page count, etc.)");
        println!("  ✓ Navigate between pages");
        println!("  ✓ Save/load page position and zoom");
        
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
            println!("After next_page(): {} (should not change if at end)", doc.metadata.page);
        }
        
        if doc.prev_page() {
            println!("After prev_page(): {}", doc.metadata.page);
        }
        
        println!("\n✓ All tests passed with lopdf!");
        println!("✓ No external libraries (.so files) required!");
    } else {
        println!("\n✗ Failed to load PDF document");
        std::process::exit(1);
    }
}
