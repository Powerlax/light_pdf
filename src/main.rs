mod pdf;
mod app;
mod pdfium_loader;

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

