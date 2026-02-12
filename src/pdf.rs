use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use pdfium_render::prelude::*;

// Type alias to avoid conflict with our PdfDocument struct
type PdfiumDocument<'a> = pdfium_render::prelude::PdfDocument<'a>;

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
    document: Option<PdfiumDocument<'static>>,
    pdfium: Pdfium,
    page_cache: HashMap<usize, image::DynamicImage>,
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

        // Try to initialize Pdfium and load the document
        let pdfium = Pdfium::default();
        let (document, total_pages) = match pdfium.load_pdf_from_file(&file, None) {
            Ok(doc) => {
                let pages = doc.pages().len() as usize;
                // SAFETY: We're extending the lifetime to 'static here.
                // This is safe because the PdfiumDocument will be dropped before pdfium,
                // as document is listed before pdfium in the struct definition.
                // Rust drops struct fields in declaration order.
                let static_doc = unsafe { std::mem::transmute(doc) };
                (Some(static_doc), Some(pages))
            }
            Err(e) => {
                eprintln!("Failed to load PDF {}: {:?}", file.display(), e);
                (None, None)
            }
        };

        Self {
            file,
            metadata_file,
            metadata,
            total_pages,
            document,
            pdfium,
            page_cache: HashMap::new(),
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

    /// Render the current page to an image. Returns the cached image if available,
    /// otherwise renders the page and caches it.
    pub fn render_page(&mut self, page_num: usize) -> Option<&image::DynamicImage> {
        // Check if page is in cache
        if self.page_cache.contains_key(&page_num) {
            return self.page_cache.get(&page_num);
        }

        // Try to render the page
        if let Some(doc) = &self.document {
            // Convert usize to u16 for pdfium (page index is 0-based)
            let page_index = (page_num.saturating_sub(1)) as u16;
            
            match doc.pages().get(page_index) {
                Ok(page) => {
                    // Render with zoom level
                    let width = (page.width().value * self.metadata.zoom) as i32;
                    let render_config = PdfRenderConfig::new()
                        .set_target_width(width.max(100))
                        .set_maximum_height(4000);

                    match page.render_with_config(&render_config) {
                        Ok(bitmap) => {
                            let image_result = bitmap.as_image();
                            self.page_cache.insert(page_num, image_result);
                            self.page_cache.get(&page_num)
                        }
                        Err(e) => {
                            eprintln!("Failed to render page {}: {:?}", page_num, e);
                            None
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to get page {}: {:?}", page_num, e);
                    None
                }
            }
        } else {
            None
        }
    }

    /// Get the currently rendered page as an image.
    pub fn get_current_page_image(&mut self) -> Option<&image::DynamicImage> {
        self.render_page(self.metadata.page)
    }

    /// Clear the page cache to free memory.
    pub fn clear_cache(&mut self) {
        self.page_cache.clear();
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
}
