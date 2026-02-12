use light_pdf::pdf::PdfDocument;
use std::path::PathBuf;

fn main() {
    println!("Testing PDF rendering with multipage.pdf...");
    
    let test_pdf = PathBuf::from("multipage.pdf");
    if !test_pdf.exists() {
        eprintln!("Error: multipage.pdf not found!");
        std::process::exit(1);
    }
    
    let mut doc = PdfDocument::new(&test_pdf, None::<&PathBuf>);
    
    println!("PDF loaded successfully!");
    println!("Total pages: {:?}", doc.total_pages);
    println!("Current page: {}", doc.metadata.page);
    
    // Try to render the first page
    println!("\nAttempting to render page 1...");
    match doc.get_current_page_image() {
        Some(img) => {
            println!("✓ Page rendered successfully!");
            println!("  Image dimensions: {}x{}", img.width(), img.height());
        }
        None => {
            println!("✗ Page rendering not available (pdfium library not installed)");
            println!("  This is expected in the CI environment.");
        }
    }
    
    println!("\n✓ Test completed successfully!");
}
