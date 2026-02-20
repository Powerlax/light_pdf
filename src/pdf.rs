use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use pdfium_render::prelude::*;
use crate::safe_eprintln;

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
    pdfium_doc: Option<pdfium_render::prelude::PdfDocument<'static>>,
    page_cache: HashMap<usize, image::DynamicImage>,
    cache_window_start: usize,
    cache_window_end: usize,
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

        let pdfium_doc = {
            std::panic::catch_unwind(|| {
                if let Some(pdfium) = crate::pdfium_loader::create_static_pdfium() {
                    pdfium.load_pdf_from_file(&file, None).ok()
                } else {
                    None
                }
            }).unwrap_or_else(|_| {
                safe_eprintln!("Pdfium library not available. PDF rendering disabled.");
                None
            })
        };

        let mut total_pages = None;
        if pdfium_doc.is_some() {
            total_pages = Some((&pdfium_doc).as_ref().
                expect("Failed to get number of pages from doc").pages().len() as usize);
        }

        Self {
            file,
            metadata_file,
            metadata,
            total_pages,
            pdfium_doc,
            page_cache: HashMap::new(),
            cache_window_start: 1,
            cache_window_end: 1,
        }
    }

    /// Save metadata to the metadata file.
    pub fn save_metadata(&mut self) -> io::Result<()> {
        match &self.metadata_file {
            Some(path) => Self::actually_write_to_metadata_file(path, &self.metadata),
            None => Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Metadata file not configured!",
            )),
        }
    }

    /// Load metadata from the given path.
    fn load_metadata_from(path: &Path) -> io::Result<PdfMetadata> {
        let data = fs::read_to_string(path)?;
        let md = serde_json::from_str(&data).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(md)
    }

    /// Actually does the heavy lifting of writing the metadata to a file.
    /// Use [PdfDocument.save_metadata] instead of calling this.
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

    /// Build a display name for the document (file stem or file name).
    pub fn display_name(&self) -> String {
        self.file
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
            .or_else(|| self.file.file_name().and_then(|s| s.to_str()).map(|s| s.to_string()))
            .unwrap_or_else(|| "Untitled".to_string())
    }

    /// Update the cache window to keep the previous 3 pages and next 3 pages around the current page.
    /// Evicts pages outside the window to free memory.
    fn update_cache_window(&mut self, current_page: usize) {
        const BUFFER_PAGES: usize = 3;
        let new_start = current_page.saturating_sub(BUFFER_PAGES).max(1);
        let new_end = if let Some(total) = self.total_pages {
            (current_page + BUFFER_PAGES).min(total)
        } else {
            current_page + BUFFER_PAGES
        };
        if new_start != self.cache_window_start || new_end != self.cache_window_end {
            self.cache_window_start = new_start;
            self.cache_window_end = new_end;
            self.page_cache.retain(|&page_num, _| {
                page_num >= self.cache_window_start && page_num <= self.cache_window_end
            });
        }
    }

    /// Render the current page to an image.
    /// Returns a cached or newly rendered image.
    pub fn render_page(&mut self, page_num: usize) -> Option<&image::DynamicImage> {
        const BASE_RENDER_WIDTH: i32 = 1920;
        const BASE_RENDER_HEIGHT: i32 = 1080;
        self.update_cache_window(page_num);
        if self.page_cache.contains_key(&page_num) {
            return self.page_cache.get(&page_num);
        }
        if let Some(ref pdfium_doc) = self.pdfium_doc {
            let page_index = page_num.saturating_sub(1);
            if let Ok(page) = pdfium_doc.pages().get(page_index as u16) {
                let width = (BASE_RENDER_WIDTH as f32 * self.metadata.zoom) as i32;
                let height = (BASE_RENDER_HEIGHT as f32 * self.metadata.zoom) as i32;
                let render_config = PdfRenderConfig::new()
                    .set_target_width(width)
                    .set_maximum_height(height);
                match page.render_with_config(&render_config) {
                    Ok(bitmap) => {
                        let width = bitmap.width() as u32;
                        let height = bitmap.height() as u32;
                        let rgba_data = bitmap.as_raw_bytes();
                        if let Some(img_buffer) = image::RgbaImage::from_raw(width, height, rgba_data.to_vec()) {
                            let dynamic_img = image::DynamicImage::ImageRgba8(img_buffer);
                            self.page_cache.insert(page_num, dynamic_img);
                            return self.page_cache.get(&page_num);
                        }
                    }
                    Err(e) => {
                        safe_eprintln!("Failed to render page {}: {:?}", page_num, e);
                    }
                }
            }
        }
        None
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
        assert_eq!(doc.metadata.page, 1);
        assert!((doc.metadata.zoom - 1.0).abs() < f32::EPSILON);
        assert_eq!(doc.next_page(), true);
        assert_eq!(doc.metadata.page, 2);
        assert_eq!(doc.prev_page(), true);
        assert_eq!(doc.metadata.page, 1);
        assert_eq!(doc.prev_page(), false);
        let data = fs::read_to_string(&meta_path).unwrap();
        assert!(data.contains("page"));
        let doc2 = PdfDocument::new(&pdf_path, Some(&meta_path));
        assert_eq!(doc2.metadata.page, 5);
        assert!((doc2.metadata.zoom - 2.5).abs() < f32::EPSILON);
    }
}
