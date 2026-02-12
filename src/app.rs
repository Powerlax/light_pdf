use crate::pdf::PdfDocument;
use eframe::egui;
use std::fs;
use std::path::PathBuf;

pub struct MyApp {
    pub current: Option<PdfDocument>,
    pub open_path: String,
    pub show_file_browser: bool,
    pub browser_dir: PathBuf,
    pub auto_open_on_select: bool,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            current: None,
            open_path: String::new(),
            show_file_browser: false,
            browser_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            auto_open_on_select: true,
        }
    }
}

/// Render the entire UI by delegating to smaller functions.
pub fn render_ui(app: &mut MyApp, ctx: &egui::Context, frame: &mut eframe::Frame) {
    // Keyboard shortcuts: check inside the closure-based input reader API
    let open_shortcut = ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::O));
    if open_shortcut {
        perform_open_action(app, ctx);
    }

    // Left/Right keys for page navigation (also persist on change)
    let (left_pressed, right_pressed) = ctx.input(|i| (i.key_pressed(egui::Key::ArrowLeft), i.key_pressed(egui::Key::ArrowRight)));
    if let Some(doc) = &mut app.current {
        if left_pressed {
            if doc.prev_page() {
                let _ = doc.save_metadata();
            }
        }
        if right_pressed {
            if doc.next_page() {
                let _ = doc.save_metadata();
            }
        }
    }

    render_top_menu(app, ctx, frame);
    render_central_panel(app, ctx);
    render_file_browser(app, ctx);
}

fn perform_open_action(app: &mut MyApp, ctx: &egui::Context) {
    // On Windows, open native dialog and potentially auto-open
    #[cfg(target_os = "windows")]
    {
        if let Some(path) = rfd::FileDialog::new().add_filter("PDF", &["pdf"]).pick_file() {
            app.open_path = path.to_string_lossy().to_string();
            if app.auto_open_on_select {
                let doc = PdfDocument::new(path.clone(), None::<PathBuf>);
                app.current = Some(doc);
            }
        }
    }

    // On non-Windows, show the in-app browser
    #[cfg(not(target_os = "windows"))]
    {
        app.show_file_browser = true;
        ctx.request_repaint();
    }
}

fn render_top_menu(app: &mut MyApp, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Open...").clicked() {
                    perform_open_action(app, ctx);
                    ui.close();
                }

                if ui.button("Quit").clicked() {
                    std::process::exit(0);
                }
            });

            ui.menu_button("Edit", |ui| {
                if ui.button("Preferences...").clicked() {
                    ui.close();
                }
            });
        });
    });
}

fn render_central_panel(app: &mut MyApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Selected:");
            ui.monospace(app.open_path.clone());
        });

        ui.separator();

        if let Some(doc) = &mut app.current {
            ui.horizontal(|ui| {
                ui.label(format!("Opened: {}", doc.display_name()));
                if ui.button("Prev").clicked() {
                    if doc.prev_page() {
                        let _ = doc.save_metadata();
                        doc.clear_cache(); // Clear cache when page changes
                    }
                }
                if ui.button("Next").clicked() {
                    if doc.next_page() {
                        let _ = doc.save_metadata();
                        doc.clear_cache(); // Clear cache when page changes
                    }
                }
                if ui.button("Save Metadata").clicked() {
                    let _ = doc.persist();
                }
                if let Some(total) = doc.total_pages {
                    ui.label(format!("Page: {} / {}", doc.metadata.page, total));
                } else {
                    ui.label(format!("Page: {}", doc.metadata.page));
                }
                ui.label(format!("Zoom: {:.2}", doc.metadata.zoom));
            });

            ui.separator();

            // Render the PDF page
            if let Some(img) = doc.get_current_page_image() {
                // Convert image::DynamicImage to egui texture
                let size = [img.width() as usize, img.height() as usize];
                let img_rgba = img.to_rgba8();
                let pixels: Vec<_> = img_rgba.pixels().flat_map(|p| p.0).collect();
                
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                
                let texture = ui.ctx().load_texture(
                    format!("pdf_page_{}", doc.metadata.page),
                    color_image,
                    egui::TextureOptions::default()
                );

                // Display the image in a scrollable area
                egui::ScrollArea::both().show(ui, |ui| {
                    ui.image(&texture);
                });
            } else if let Some(text) = doc.get_current_page_text() {
                // Show extracted text if rendering is not available
                ui.separator();
                ui.heading("Page Content (Text Extraction)");
                ui.label("Pure Rust text extraction - no external dependencies required!");
                ui.separator();
                
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut text.as_str())
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                            .desired_rows(30)
                    );
                });
            } else {
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.heading("PDF Loaded Successfully");
                    ui.add_space(20.0);
                    ui.label("This version uses a pure Rust PDF parser with no platform dependencies.");
                    ui.label("Text extraction from this PDF page failed or no text content found.");
                    ui.add_space(10.0);
                    ui.label("The application can:");
                    ui.label("  ✓ Load and parse PDF files");
                    ui.label("  ✓ Extract metadata (page count, etc.)");
                    ui.label("  ✓ Extract text content from pages");
                    ui.label("  ✓ Navigate between pages");
                    ui.label("  ✓ Save and load page position and zoom level");
                    ui.add_space(20.0);
                    ui.label("✓ Works on all platforms without external dependencies!");
                });
            }
        }
    });
}

fn render_file_browser(app: &mut MyApp, ctx: &egui::Context) {
    if !app.show_file_browser {
        return;
    }

    // Use local temporaries to avoid borrowing `app` mutably inside the UI closure
    let mut new_browser_dir = app.browser_dir.clone();
    let mut should_close = false;
    let mut selected: Option<PathBuf> = None;

    egui::Window::new("File Browser")
        .open(&mut app.show_file_browser)
        .default_size(egui::vec2(600.0, 400.0))
        .collapsible(false)
        .resizable(true)
        .show(ctx, |ui| {
            // header (Up + current dir)
            render_browser_header(ui, &mut new_browser_dir);

            // entries listing and selection handling
            render_browser_entries(ui, &mut new_browser_dir, &mut selected, &mut should_close);
        });

    // Apply changes from the UI closure back to app
    app.browser_dir = new_browser_dir;
    if should_close {
        if let Some(entry) = selected {
            // Populate the path
            app.open_path = entry.to_string_lossy().to_string();
            // Auto-open the document outside the closure to avoid borrowing app during UI
            if app.auto_open_on_select {
                let doc = PdfDocument::new(entry.clone(), None::<PathBuf>);
                app.current = Some(doc);
            }
        }
        app.show_file_browser = false;
    }
}

fn render_browser_header(ui: &mut egui::Ui, browser_dir: &mut PathBuf) {
    ui.horizontal(|ui| {
        if ui.button("Up").clicked() {
            if let Some(parent) = browser_dir.parent() {
                *browser_dir = parent.to_path_buf();
            }
        }
        ui.label(format!("Current: {}", browser_dir.display()));
    });
}

fn render_browser_entries(
    ui: &mut egui::Ui,
    browser_dir: &mut PathBuf,
    selected: &mut Option<PathBuf>,
    should_close: &mut bool,
) {
    let entries = read_dir_entries(browser_dir);

    egui::ScrollArea::vertical().show(ui, |ui| {
        for entry in entries {
            if entry.is_dir() {
                render_dir_entry(ui, &entry, browser_dir);
            } else {
                render_file_entry(ui, &entry, selected, should_close);
            }
        }
    });
}

fn read_dir_entries(browser_dir: &PathBuf) -> Vec<PathBuf> {
    let mut entries: Vec<_> = match fs::read_dir(browser_dir) {
        Ok(rd) => rd.filter_map(|e| e.ok()).map(|e| e.path()).collect(),
        Err(_) => Vec::new(),
    };
    // sort: directories first, then files; alphabetical
    entries.sort_by_key(|p| (p.is_file(), p.file_name().map(|s| s.to_os_string())));
    entries
}

fn render_dir_entry(ui: &mut egui::Ui, entry: &PathBuf, browser_dir: &mut PathBuf) {
    ui.horizontal(|ui| {
        ui.label(format!("[DIR] {}", entry.file_name().and_then(|s| s.to_str()).unwrap_or("")));
        if ui.button("Enter").clicked() {
            *browser_dir = entry.clone();
        }
    });
}

fn render_file_entry(
    ui: &mut egui::Ui,
    entry: &PathBuf,
    selected: &mut Option<PathBuf>,
    should_close: &mut bool,
) {
    let is_pdf = is_pdf(entry);
    ui.horizontal(|ui| {
        ui.label(entry.file_name().and_then(|s| s.to_str()).unwrap_or(""));
        if is_pdf {
            if ui.button("Select").clicked() {
                *selected = Some(entry.clone());
                *should_close = true;
            }
        }
    });
}

fn is_pdf(entry: &PathBuf) -> bool {
    entry
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.eq_ignore_ascii_case("pdf"))
        .unwrap_or(false)
}
