/// Foreign Function Interface module for calling light_pdf from Python
/// This module provides C-compatible functions that can be called via ctypes from Python

use crate::app::MyApp;

/// Run the application event loop
/// This is a blocking call that runs until the user closes the window.
/// Returns 0 on success, non-zero on error.
///
/// This function is called from Python via ctypes to start the GUI.
#[unsafe(no_mangle)]
pub extern "C" fn light_pdf_run() -> i32 {
    let options = eframe::NativeOptions::default();

    let result = eframe::run_native(
        "Light PDF",
        options,
        Box::new(|_cc| {
            Ok(Box::new(MyApp::default()))
        }),
    );

    match result {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("App error: {e}");
            1
        }
    }
}


