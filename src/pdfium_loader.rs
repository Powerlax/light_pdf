//! Pdfium library loader with embedded library support.
//!
//! This module handles loading the pdfium library, with support for:
//! 1. Embedded library (extracted to temp dir at runtime) - for self-contained executables
//! 2. Library next to executable
//! 3. System library paths

use pdfium_render::prelude::*;
use std::path::PathBuf;
use std::io::Write;
use std::error::Error;
use crate::safe_eprintln;

#[cfg(target_os = "windows")]
const PDFIUM_LIB_NAME: &str = "pdfium.dll";

#[cfg(target_os = "linux")]
const PDFIUM_LIB_NAME: &str = "libpdfium.so";

#[cfg(target_os = "macos")]
const PDFIUM_LIB_NAME: &str = "libpdfium.dylib";

// Embed the pdfium library at compile time
#[cfg(all(target_os = "windows", feature = "embed-pdfium"))]
const EMBEDDED_PDFIUM: &[u8] = include_bytes!("../libs/windows/bin/pdfium.dll");

#[cfg(all(target_os = "linux", feature = "embed-pdfium"))]
const EMBEDDED_PDFIUM: &[u8] = include_bytes!("../libs/linux/lib/libpdfium.so");
#[cfg(all(target_os = "macos", feature = "embed-pdfium"))]
const EMBEDDED_PDFIUM: &[u8] = include_bytes!("../libs/linux/lib/libpdfium.dylib");

/// Get the directory where the executable is located
fn get_exe_dir() -> Option<PathBuf> {
    std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf()))
}

/// Extract embedded pdfium library to a temporary location and return the path
#[cfg(feature = "embed-pdfium")]
fn extract_embedded_pdfium() -> Option<PathBuf> {
    if EMBEDDED_PDFIUM.is_empty() {
        return None;
    }
    let temp_dir = std::env::temp_dir().join("light_pdf_libs");
    if let Err(e) = std::fs::create_dir_all(&temp_dir) {
        safe_eprintln!("Failed to create temp dir for pdfium: {}", e);
        return None;
    }

    let lib_path = temp_dir.join(PDFIUM_LIB_NAME);
    let version_path = temp_dir.join("version.txt");
    let expected_version = format!("{}", EMBEDDED_PDFIUM.len());
    let needs_extract = if lib_path.exists() && version_path.exists() {
        match std::fs::read_to_string(&version_path) {
            Ok(v) => v.trim() != expected_version,
            Err(_) => true,
        }
    } else {
        true
    };
    if !needs_extract {
        safe_eprintln!("Using cached pdfium (size: {} bytes)", expected_version);
        return Some(lib_path);
    }

    safe_eprintln!("Extracting embedded pdfium ({} bytes)...", EMBEDDED_PDFIUM.len());
    match std::fs::File::create(&lib_path) {
        Ok(mut file) => {
            if let Err(e) = file.write_all(EMBEDDED_PDFIUM) {
                safe_eprintln!("Failed to write embedded pdfium: {}", e);
                return None;
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Err(e) = std::fs::set_permissions(&lib_path, std::fs::Permissions::from_mode(0o755)) {
                    safe_eprintln!("Failed to set permissions on pdfium: {}", e);
                }
            }
            if let Ok(mut vf) = std::fs::File::create(&version_path) {
                let _ = vf.write_all(expected_version.as_bytes());
            }
            Some(lib_path)
        }
        Err(e) => {
            safe_eprintln!("Failed to create pdfium file: {}", e);
            None
        }
    }
}

#[cfg(not(feature = "embed-pdfium"))]
fn extract_embedded_pdfium() -> Option<PathBuf> {
    None
}

/// Find the pdfium library in various locations
fn find_pdfium_library() -> Option<PathBuf> {
    if let Some(path) = extract_embedded_pdfium() {
        if path.exists() {
            safe_eprintln!("Using embedded pdfium from: {}", path.display());
            return Some(path);
        }
    }
    if let Some(exe_dir) = get_exe_dir() {
        let lib_path = exe_dir.join(PDFIUM_LIB_NAME);
        if lib_path.exists() {
            safe_eprintln!("Using pdfium from exe dir: {}", lib_path.display());
            return Some(lib_path);
        }
    }
    let cwd_path = PathBuf::from(PDFIUM_LIB_NAME);
    if cwd_path.exists() {
        safe_eprintln!("Using pdfium from current dir: {}", cwd_path.display());
        return Some(cwd_path);
    }
    if let Ok(lib_dir) = std::env::var("PDFIUM_DYNAMIC_LIB_PATH") {
        let lib_path = PathBuf::from(&lib_dir).join(PDFIUM_LIB_NAME);
        if lib_path.exists() {
            safe_eprintln!("Using pdfium from PDFIUM_DYNAMIC_LIB_PATH: {}", lib_path.display());
            return Some(lib_path);
        }
    }

    #[cfg(target_os = "linux")]
    {
        let system_paths = [
            "/usr/lib",
            "/usr/local/lib",
            "/usr/lib/x86_64-linux-gnu",
        ];
        for path in &system_paths {
            let lib_path = PathBuf::from(path).join(PDFIUM_LIB_NAME);
            if lib_path.exists() {
                safe_eprintln!("Using pdfium from system: {}", lib_path.display());
                return Some(lib_path);
            }
        }
    }

    None
}

/// Create a Pdfium instance, trying various library locations
pub fn create_pdfium() -> Result<Pdfium,  Box<dyn Error>> {
    if let Some(lib_path) = find_pdfium_library() {
        let lib_path_str = lib_path.to_string_lossy().to_string();
        safe_eprintln!("Loading pdfium from: {}", lib_path_str);
        let bindings = Pdfium::bind_to_library(&lib_path_str)?;
        return Ok(Pdfium::new(bindings));
    }
    safe_eprintln!("No pdfium found at known locations. PDF rendering will be disabled.");
    Err("Cant find pdfium library".into())
}

/// Create a static/leaked Pdfium instance for use with PdfDocument<'static>
///
/// NOTE: This code INTENTIONALLY leaks the Pdfium instance to satisfy lifetime requirements.
pub fn create_static_pdfium() -> Option<&'static Pdfium> {
    match create_pdfium() {
        Ok(pdfium) => Some(Box::leak(Box::new(pdfium))),
        Err(..) => {
            safe_eprintln!("Failed to load pdfium library");
            None
        }
    }
}