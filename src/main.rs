mod pdf;
mod app;
mod pdfium_loader;

use eframe::egui;

fn main() {
    let options = eframe::NativeOptions::default();
    if let Err(err) = eframe::run_native(
        "Light PDF",
        options,
        Box::new(|_cc| Ok(Box::new(app::MyApp::default()))),
    ) {
        eprintln!("App error: {err}");
    }
}

impl eframe::App for app::MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        app::render_ui(self, ctx, frame);
    }
}
