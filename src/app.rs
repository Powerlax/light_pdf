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
    /// Page number from which last automatic navigation occurred (for debouncing)
    /// None indicates no automatic navigation has occurred yet
    last_auto_nav_page: Option<usize>,
    /// Whether fullscreen mode is active (hides all UI chrome)
    fullscreen: bool,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            current: None,
            open_path: String::new(),
            show_file_browser: false,
            browser_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            auto_open_on_select: true,
            last_auto_nav_page: None,
            fullscreen: false,
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

    // F11 to toggle fullscreen mode
    let f11_pressed = ctx.input(|i| i.key_pressed(egui::Key::F11));
    if f11_pressed {
        app.fullscreen = !app.fullscreen;
        // Apply fullscreen window state: enable OS fullscreen and hide window decorations
        // When fullscreen=true, we want: Fullscreen(true) and Decorations(false)
        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(app.fullscreen));
        ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(!app.fullscreen));
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
    // Don't show top menu in fullscreen mode
    if app.fullscreen {
        return;
    }
    
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
    // Zoom configuration constants
    const ZOOM_STEP: f32 = 0.25;
    const MIN_ZOOM: f32 = 0.25;
    const MAX_ZOOM: f32 = 4.0;
    
    // Configure panel with no margins/padding in fullscreen mode
    let mut panel = egui::CentralPanel::default();
    if app.fullscreen {
        panel = panel.frame(egui::Frame::none());
    }
    
    panel.show(ctx, |ui| {
        // Hide UI controls in fullscreen mode
        if !app.fullscreen {
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Selected:");
                ui.monospace(app.open_path.clone());
            });

            ui.separator();
        }

        if let Some(doc) = &mut app.current {
            // Hide controls in fullscreen mode
            if !app.fullscreen {
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
                    if let Some(total) = doc.total_pages {
                        ui.label(format!("Page: {} / {}", doc.metadata.page, total));
                    } else {
                        ui.label(format!("Page: {}", doc.metadata.page));
                    }
                    
                    ui.separator();
                    
                    // Zoom controls
                    if ui.button("Zoom -").clicked() {
                        doc.metadata.zoom = (doc.metadata.zoom - ZOOM_STEP).max(MIN_ZOOM);
                        let _ = doc.save_metadata();
                        doc.clear_cache(); // Clear cache when zoom changes
                    }
                    ui.label(format!("{:.0}%", doc.metadata.zoom * 100.0));
                    if ui.button("Zoom +").clicked() {
                        doc.metadata.zoom = (doc.metadata.zoom + ZOOM_STEP).min(MAX_ZOOM);
                        let _ = doc.save_metadata();
                        doc.clear_cache(); // Clear cache when zoom changes
                    }
                    if ui.button("100%").clicked() {
                        doc.metadata.zoom = 1.0;
                        let _ = doc.save_metadata();
                        doc.clear_cache(); // Clear cache when zoom changes
                    }
                });

                ui.separator();
            }

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

                // In fullscreen mode, scale image to fit screen while maintaining aspect ratio
                let image_widget = if app.fullscreen {
                    // Get available space
                    let available = ui.available_size();
                    
                    // Safety check: ensure we have valid dimensions to avoid division by zero
                    if size[0] > 0 && size[1] > 0 && available.x > 0.0 && available.y > 0.0 {
                        // Calculate scaling to fit while maintaining aspect ratio
                        let image_aspect = size[0] as f32 / size[1] as f32;
                        let screen_aspect = available.x / available.y;
                        
                        let fit_size = if image_aspect > screen_aspect {
                            // Image is wider than screen - fit to width
                            egui::Vec2::new(available.x, available.x / image_aspect)
                        } else {
                            // Image is taller than screen - fit to height
                            egui::Vec2::new(available.y * image_aspect, available.y)
                        };
                        
                        egui::Image::new(&texture).fit_to_exact_size(fit_size)
                    } else {
                        // Fallback to default if dimensions are invalid
                        egui::Image::new(&texture)
                    }
                } else {
                    egui::Image::new(&texture)
                };

                // Display the image in a scrollable area with page-to-page scrolling
                // Use id_salt with tuple to avoid string allocation every frame
                let scroll_output = egui::ScrollArea::both()
                    .id_salt(("pdf_scroll", doc.metadata.page))
                    .show(ui, |ui| {
                        ui.add(image_widget);
                    });

                // Detect scroll wheel input for page-to-page navigation
                // Check both raw and smooth scroll deltas to handle different input devices:
                // - raw_scroll_delta: direct scroll wheel ticks
                // - smooth_scroll_delta: trackpad/smooth scrolling
                let (raw_scroll_delta_y, smooth_scroll_delta_y) = ui.input(|i| (i.raw_scroll_delta.y, i.smooth_scroll_delta.y));
                
                // Debouncing: Track the page from which last navigation occurred
                // Reset debounce if we're on a different page (manual navigation occurred)
                let current_page = doc.metadata.page;
                if matches!(app.last_auto_nav_page, Some(p) if p != current_page) {
                    app.last_auto_nav_page = None;
                }
                
                // Only allow automatic navigation if debounce is clear
                let can_navigate = app.last_auto_nav_page.is_none();
                
                if can_navigate {
                    // Check if we should navigate to next/previous page based on scroll position
                    // Only trigger if user is scrolling significantly (threshold to avoid accidental triggers)
                    const SCROLL_THRESHOLD: f32 = 5.0;
                    const SCROLL_EDGE_TOLERANCE: f32 = 1.0;
                    
                    let state = scroll_output.state;
                    let viewport_rect = scroll_output.inner_rect;
                    let content_size = scroll_output.content_size;
                    
                    // Calculate if we're at the bottom or top of the scrollable area
                    let at_bottom = state.offset.y + viewport_rect.height() >= content_size.y - SCROLL_EDGE_TOLERANCE;
                    let at_top = state.offset.y <= SCROLL_EDGE_TOLERANCE;
                    
                    // Determine if user is scrolling down or up significantly
                    let scrolling_down = raw_scroll_delta_y < -SCROLL_THRESHOLD || smooth_scroll_delta_y < -SCROLL_THRESHOLD;
                    let scrolling_up = raw_scroll_delta_y > SCROLL_THRESHOLD || smooth_scroll_delta_y > SCROLL_THRESHOLD;
                    
                    // Check if navigation should occur
                    let should_nav_next = scrolling_down && at_bottom;
                    let should_nav_prev = scrolling_up && at_top;
                    
                    // Perform navigation if needed
                    let navigated = if should_nav_next {
                        doc.next_page()
                    } else if should_nav_prev {
                        doc.prev_page()
                    } else {
                        false
                    };
                    
                    // Handle post-navigation tasks
                    if navigated {
                        if let Err(e) = doc.save_metadata() {
                            eprintln!("Failed to save metadata: {}", e);
                        }
                        doc.clear_cache();
                        app.last_auto_nav_page = Some(current_page);
                    }
                }
            } else {
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.heading("PDF Loaded Successfully");
                    ui.add_space(20.0);
                    ui.label("PDF rendering requires the pdfium library.");
                    ui.label("See PDFIUM_SETUP.md for installation instructions.");
                    ui.add_space(10.0);
                    ui.label("The application can:");
                    ui.label("  ✓ Load and parse PDF files");
                    ui.label("  ✓ Extract metadata (page count, etc.)");
                    ui.label("  ✓ Navigate between pages");
                    ui.label("  ✓ Save and load page position and zoom level");
                    ui.add_space(20.0);
                    ui.label("📚 Install pdfium to enable PDF page rendering");
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
