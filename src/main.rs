mod pdf;
mod app;
mod pdfium_loader;

use eframe::egui;
use std::io::Write;

/// Print to stderr, ignoring broken pipe errors.
/// This is needed for WSL/Linux environments where stderr may be disconnected.
macro_rules! safe_eprintln {
    ($($arg:tt)*) => {
        use std::io::Write;
        let _ = writeln!(std::io::stderr(), $($arg)*);
    };
}

fn main() {
    let options = eframe::NativeOptions::default();
    if let Err(err) = eframe::run_native(
        "Light PDF",
        options,
        Box::new(|_cc| Ok(Box::new(app::MyApp::default()))),
    ) {
        safe_eprintln!("App error: {err}");
    }
}

impl eframe::App for app::MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        app::render_ui(self, ctx, frame);
    }
}
