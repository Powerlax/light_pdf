use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use lopdf::{Document as LopdfDocument, Object, ObjectId};
use pdfium_render::prelude::*;
use pdfium_render::pdfium::Pdfium;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfMetadata {
    pub page: usize,
    pub zoom: f32,
}

impl Default for PdfMetadata {
    fn default() -> Self {
        Self { page: 1, zoom: 1.0 }
    }
}

pub struct PdfDocument {
    pub file: PathBuf,
    pub metadata_file: Option<PathBuf>,
    pub metadata: PdfMetadata,
    pub total_pages: Option<usize>,
    document: Option<LopdfDocument>,
    pdfium_doc: Option<pdfium_render::document::PdfDocument<'static>>,
    page_cache: HashMap<usize, image::DynamicImage>,
    text_cache: HashMap<usize, String>,
}

impl PdfDocument {
    pub fn new<P: Into<PathBuf>>(file: P, metadata_file: Option<P>) -> Self {
        let file = file.into();
        let metadata_file = metadata_file.map(|p| p.into()).or_else(|| {
            file.file_name().and_then(|name| {
                let mut meta_name = name.to_os_string();
                meta_name.push(".meta.json");
                Some(file.with_file_name(meta_name))
            })
        });

        let metadata = metadata_file
            .as_ref()
            .and_then(|mf| Self::load_metadata_from(mf).ok())
            .unwrap_or_default();

        // Try to load the document using lopdf for metadata
        let (document, total_pages) = match LopdfDocument::load(&file) {
            Ok(doc) => {
                // Get page count from the document
                let pages = doc.get_pages().len();
                (Some(doc), Some(pages))
            }
            Err(e) => {
                eprintln!("Failed to load PDF with lopdf {}: {:?}", file.display(), e);
                (None, None)
            }
        };

        // Try to load with pdfium for rendering
        let pdfium_doc = match Pdfium::new(Pdfium::bind_to_statically_linked_library().unwrap()) {
            pdfium => {
                match pdfium.load_pdf_from_file(&file, None) {
                    Ok(doc) => Some(doc),
                    Err(e) => {
                        eprintln!("Failed to load PDF with pdfium {}: {:?}", file.display(), e);
                        None
                    }
                }
            }
        };

        Self {
            file,
            metadata_file,
            metadata,
            total_pages,
            document,
            pdfium_doc,
            page_cache: HashMap::new(),
            text_cache: HashMap::new(),
        }
    }

    /// Load metadata from the configured metadata file.
    pub fn load_metadata(&mut self) -> io::Result<()> {
        match &self.metadata_file {
            Some(path) => {
                let md = Self::load_metadata_from(path)?;
                self.metadata = md;
                Ok(())
            }
            None => Err(io::Error::new(
                io::ErrorKind::NotFound,
                "metadata file not configured",
            )),
        }
    }

    /// Save metadata to the configured metadata file immediately. Creates parent directories as needed.
    pub fn save_metadata(&mut self) -> io::Result<()> {
        match &self.metadata_file {
            Some(path) => Self::actually_write_to_metadata_file(path, &self.metadata),
            None => Err(io::Error::new(
                io::ErrorKind::NotFound,
                "metadata file not configured",
            )),
        }
    }

    fn load_metadata_from(path: &Path) -> io::Result<PdfMetadata> {
        let data = fs::read_to_string(path)?;
        let md = serde_json::from_str(&data).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(md)
    }

    fn actually_write_to_metadata_file(path: &Path, metadata: &PdfMetadata) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let tmp = path.with_extension("meta.tmp");
        let mut f = fs::File::create(&tmp)?;
        let data = serde_json::to_vec_pretty(metadata).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        f.write_all(&data)?;
        fs::rename(tmp, path)?;
        Ok(())
    }

    /// Set the current page (1-based). Will clamp to [1, total_pages] if total_pages is known.
    pub fn set_page(&mut self, page: usize) {
        let new_page = if let Some(total) = self.total_pages {
            page.clamp(1, total)
        } else {
            page.max(1)
        };
        self.metadata.page = new_page;
    }

    /// Advance to the next page. Returns true if the page changed.
    pub fn next_page(&mut self) -> bool {
        let current = self.metadata.page;
        let new = if let Some(total) = self.total_pages {
            if current >= total {
                current
            } else {
                current + 1
            }
        } else {
            current + 1
        };
        if new != current {
            self.metadata.page = new;
            true
        } else {
            false
        }
    }

    /// Go back to the previous page. Returns true if the page changed.
    pub fn prev_page(&mut self) -> bool {
        let current = self.metadata.page;
        if current <= 1 {
            return false;
        }
        self.metadata.page = current - 1;
        true
    }

    /// Set zoom level (e.g., 1.0 = 100%).
    pub fn set_zoom(&mut self, zoom: f32) {
        self.metadata.zoom = zoom.max(0.01);
    }

    /// Try to set total pages (useful after opening the document). If the current page is out of range,
    /// it will be clamped.
    pub fn set_total_pages(&mut self, total: usize) {
        self.total_pages = Some(total.max(1));
        if let Some(t) = self.total_pages {
            if self.metadata.page > t {
                self.metadata.page = t;
            }
        }
    }

    /// Convenience: persist current metadata and return any io error.
    pub fn persist(&mut self) -> io::Result<()> {
        match &self.metadata_file {
            Some(path) => Self::actually_write_to_metadata_file(path, &self.metadata),
            None => Err(io::Error::new(
                io::ErrorKind::NotFound,
                "metadata file not configured",
            )),
        }
    }

    /// Build a display name for the document (file stem or file name).
    pub fn display_name(&self) -> String {
        self.file
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
            .or_else(|| self.file.file_name().and_then(|s| s.to_str()).map(|s| s.to_string()))
            .unwrap_or_else(|| "Untitled".to_string())
    }

    /// Render the current page to an image using pdfium (statically linked).
    /// Returns a reference to the cached image, or None if rendering fails.
    pub fn render_page(&mut self, page_num: usize) -> Option<&image::DynamicImage> {
        // Check cache first
        if self.page_cache.contains_key(&page_num) {
            return self.page_cache.get(&page_num);
        }

        // Get the pdfium document
        let pdfium_doc = self.pdfium_doc.as_ref()?;
        
        // Get the page (pdfium uses 0-based indexing)
        let page = pdfium_doc.pages().get(page_num.saturating_sub(1) as u16).ok()?;
        
        // Render the page at a reasonable DPI (e.g., 150 DPI for screen display)
        let render_config = PdfRenderConfig::new()
            .set_target_width(1200)  // Render at ~1200px width
            .rotate_if_landscape(PdfPageRenderRotation::None, false);
        
        let bitmap = page.render_with_config(&render_config).ok()?;
        
        // Convert pdfium bitmap to image::DynamicImage
        let width = bitmap.width() as u32;
        let height = bitmap.height() as u32;
        
        // Get RGBA pixels from bitmap
        let buffer = bitmap.as_bytes();
        
        // Create an image from the buffer
        let img = match image::RgbaImage::from_raw(width, height, buffer.to_vec()) {
            Some(img) => image::DynamicImage::ImageRgba8(img),
            None => return None,
        };
        
        // Cache the rendered page
        self.page_cache.insert(page_num, img);
        self.page_cache.get(&page_num)
    }

    /// Extract text content from a specific page using lopdf.
    /// This provides a pure Rust text extraction without external dependencies.
    /// Returns the extracted text or None if extraction fails.
    pub fn extract_text(&mut self, page_num: usize) -> Option<String> {
        // Check cache first
        if let Some(cached) = self.text_cache.get(&page_num) {
            return Some(cached.clone());
        }

        let doc = self.document.as_ref()?;
        
        // Get the page IDs
        let pages = doc.get_pages();
        let page_id = pages.get(&(page_num as u32))?;
        
        // Extract text from the page
        let text = Self::extract_text_from_page(doc, *page_id).ok()?;
        
        // Cache the result
        self.text_cache.insert(page_num, text.clone());
        
        Some(text)
    }

    /// Helper function to extract text from a page object
    fn extract_text_from_page(doc: &LopdfDocument, page_id: ObjectId) -> Result<String, lopdf::Error> {
        let mut text = String::new();
        
        // Get the page object
        let page = doc.get_object(page_id)?;
        
        // Get the Contents of the page
        let contents = match page.as_dict()?.get(b"Contents") {
            Ok(obj) => obj,
            Err(_) => return Ok(text), // No content in this page
        };
        
        // Contents can be a single stream or an array of streams
        let content_streams = match contents {
            Object::Reference(ref_id) => vec![*ref_id],
            Object::Array(arr) => {
                arr.iter()
                    .filter_map(|obj| {
                        if let Object::Reference(ref_id) = obj {
                            Some(*ref_id)
                        } else {
                            None
                        }
                    })
                    .collect()
            }
            _ => vec![],
        };
        
        // Process each content stream
        for stream_id in content_streams {
            if let Ok(stream) = doc.get_object(stream_id) {
                if let Ok(stream_obj) = stream.as_stream() {
                    if let Ok(decoded) = stream_obj.decompressed_content() {
                        // Parse the content stream for text
                        text.push_str(&Self::parse_content_stream(&decoded));
                    }
                }
            }
        }
        
        Ok(text)
    }

    /// Parse a PDF content stream and extract text
    fn parse_content_stream(content: &[u8]) -> String {
        let mut text = String::new();
        let content_str = String::from_utf8_lossy(content);
        
        // Look for text between parentheses in Tj or TJ operators
        // This is a simplified parser that handles basic text extraction
        let mut in_text = false;
        let mut buffer = String::new();
        let mut escape_next = false;
        
        for line in content_str.lines() {
            let line = line.trim();
            
            // BT marks beginning of text object
            if line.contains("BT") {
                in_text = true;
                continue;
            }
            
            // ET marks end of text object  
            if line.contains("ET") {
                in_text = false;
                continue;
            }
            
            if !in_text {
                continue;
            }
            
            // Look for text show operators: Tj, TJ, ', "
            if line.contains("Tj") || line.contains("TJ") || line.ends_with('\'') || line.contains('"') {
                // Extract text between parentheses
                let chars: Vec<char> = line.chars().collect();
                let mut i = 0;
                while i < chars.len() {
                    if chars[i] == '(' && !escape_next {
                        // Start of text string
                        buffer.clear();
                        i += 1;
                        while i < chars.len() {
                            if chars[i] == '\\' && !escape_next {
                                escape_next = true;
                                i += 1;
                                continue;
                            }
                            if chars[i] == ')' && !escape_next {
                                // End of text string
                                text.push_str(&buffer);
                                text.push(' ');
                                break;
                            }
                            buffer.push(chars[i]);
                            escape_next = false;
                            i += 1;
                        }
                    }
                    escape_next = false;
                    i += 1;
                }
            }
            
            // T* operator indicates new line
            if line.contains("T*") {
                text.push('\n');
            }
        }
        
        text
    }

    /// Get the extracted text for the current page
    pub fn get_current_page_text(&mut self) -> Option<String> {
        self.extract_text(self.metadata.page)
    }

    /// Get the currently rendered page as an image.
    pub fn get_current_page_image(&mut self) -> Option<&image::DynamicImage> {
        self.render_page(self.metadata.page)
    }

    /// Clear the page cache to free memory.
    pub fn clear_cache(&mut self) {
        self.page_cache.clear();
        self.text_cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn metadata_roundtrip_and_navigation() {
        let dir = tempdir().unwrap();
        let pdf_path = dir.path().join("doc.pdf");
        fs::write(&pdf_path, b"%PDF-1.4").unwrap();
        let meta_path = dir.path().join("doc.pdf.meta.json");

        let mut doc = PdfDocument::new(&pdf_path, Some(&meta_path));

        // defaults (should not have been overwritten by any external file)
        assert_eq!(doc.metadata.page, 1);
        assert!((doc.metadata.zoom - 1.0).abs() < f32::EPSILON);

        // navigation
        doc.set_total_pages(5);
        assert_eq!(doc.next_page(), true);
        assert_eq!(doc.metadata.page, 2);
        assert_eq!(doc.prev_page(), true);
        assert_eq!(doc.metadata.page, 1);
        assert_eq!(doc.prev_page(), false);

        // set page clamp
        doc.set_page(10);
        assert_eq!(doc.metadata.page, 5);

        // zoom
        doc.set_zoom(2.5);
        assert!((doc.metadata.zoom - 2.5).abs() < f32::EPSILON);

        // persist should write immediately
        doc.persist().unwrap();
        let data = fs::read_to_string(&meta_path).unwrap();
        assert!(data.contains("page"));

        // load into new doc from the metadata file we just wrote
        let doc2 = PdfDocument::new(&pdf_path, Some(&meta_path));
        assert_eq!(doc2.metadata.page, 5);
        assert!((doc2.metadata.zoom - 2.5).abs() < f32::EPSILON);
    }

    #[test]
    fn display_name_works() {
        let doc = PdfDocument::new("/tmp/some/long-name.pdf", None::<&str>);
        assert!(doc.display_name().contains("long-name"));
    }

    #[test]
    fn text_extraction_works() {
        // Use the existing multipage.pdf which contains "Page 1", "Page 2", etc. text
        let pdf_path = PathBuf::from("multipage.pdf");
        
        if !pdf_path.exists() {
            // Skip test if file doesn't exist (e.g., when running from different directory)
            println!("Skipping text_extraction_works test - multipage.pdf not found");
            return;
        }
        
        let mut doc = PdfDocument::new(&pdf_path, None::<&PathBuf>);
        
        // Check that PDF was loaded
        assert_eq!(doc.total_pages, Some(3));
        
        // Extract text from page 1
        let text = doc.extract_text(1);
        assert!(text.is_some(), "Text extraction should succeed");
        
        let text = text.unwrap();
        assert!(text.contains("Page 1"), "Extracted text should contain 'Page 1', got: {}", text);
        
        // Test caching - second call should use cache
        let text2 = doc.extract_text(1);
        assert!(text2.is_some());
        assert_eq!(text, text2.unwrap());
        
        // Extract from page 2
        let text_page2 = doc.extract_text(2);
        assert!(text_page2.is_some());
        assert!(text_page2.unwrap().contains("Page 2"));
        
        // Test clear_cache
        doc.clear_cache();
        let text3 = doc.extract_text(1);
        assert!(text3.is_some());
    }
}
