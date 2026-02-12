//! Pdfium library loader with embedded library support.
//!
//! This module handles loading the pdfium library, with support for:
//! 1. Embedded library (extracted to temp dir at runtime) - for self-contained executables
//! 2. Library next to executable
//! 3. System library paths

use pdfium_render::prelude::*;
use std::path::PathBuf;

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

#[cfg(not(feature = "embed-pdfium"))]
const EMBEDDED_PDFIUM: &[u8] = &[];

/// Get the directory where the executable is located
fn get_exe_dir() -> Option<PathBuf> {
    std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf()))
}

/// Extract embedded pdfium library to a temporary location and return the path
#[cfg(feature = "embed-pdfium")]
fn extract_embedded_pdfium() -> Option<PathBuf> {
    use std::io::Write;

    if EMBEDDED_PDFIUM.is_empty() {
        return None;
    }

    // Use a consistent temp directory so we don't extract every time
    let temp_dir = std::env::temp_dir().join("light_pdf_libs");
    if let Err(e) = std::fs::create_dir_all(&temp_dir) {
        eprintln!("Failed to create temp dir for pdfium: {}", e);
        return None;
    }

    let lib_path = temp_dir.join(PDFIUM_LIB_NAME);
    let version_path = temp_dir.join("version.txt");

    // Use embedded data size as a simple version identifier
    let expected_version = format!("{}", EMBEDDED_PDFIUM.len());

    // Check if already extracted with correct version
    let needs_extract = if lib_path.exists() && version_path.exists() {
        match std::fs::read_to_string(&version_path) {
            Ok(v) => v.trim() != expected_version,
            Err(_) => true,
        }
    } else {
        true
    };

    if !needs_extract {
        eprintln!("Using cached pdfium (size: {} bytes)", expected_version);
        return Some(lib_path);
    }

    eprintln!("Extracting embedded pdfium ({} bytes)...", EMBEDDED_PDFIUM.len());

    // Extract the library
    match std::fs::File::create(&lib_path) {
        Ok(mut file) => {
            if let Err(e) = file.write_all(EMBEDDED_PDFIUM) {
                eprintln!("Failed to write embedded pdfium: {}", e);
                return None;
            }

            // On Unix, make the library executable
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Err(e) = std::fs::set_permissions(&lib_path, std::fs::Permissions::from_mode(0o755)) {
                    eprintln!("Failed to set permissions on pdfium: {}", e);
                }
            }

            // Write version marker
            if let Ok(mut vf) = std::fs::File::create(&version_path) {
                let _ = vf.write_all(expected_version.as_bytes());
            }

            Some(lib_path)
        }
        Err(e) => {
            eprintln!("Failed to create pdfium file: {}", e);
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
    // 1. Check if embedded library should be used
    if let Some(path) = extract_embedded_pdfium() {
        if path.exists() {
            eprintln!("Using embedded pdfium from: {}", path.display());
            return Some(path);
        }
    }

    // 2. Check next to executable
    if let Some(exe_dir) = get_exe_dir() {
        let lib_path = exe_dir.join(PDFIUM_LIB_NAME);
        if lib_path.exists() {
            eprintln!("Using pdfium from exe dir: {}", lib_path.display());
            return Some(lib_path);
        }
    }

    // 3. Check in current directory
    let cwd_path = PathBuf::from(PDFIUM_LIB_NAME);
    if cwd_path.exists() {
        eprintln!("Using pdfium from current dir: {}", cwd_path.display());
        return Some(cwd_path);
    }

    // 4. Check PDFIUM_DYNAMIC_LIB_PATH environment variable
    if let Ok(lib_dir) = std::env::var("PDFIUM_DYNAMIC_LIB_PATH") {
        let lib_path = PathBuf::from(&lib_dir).join(PDFIUM_LIB_NAME);
        if lib_path.exists() {
            eprintln!("Using pdfium from PDFIUM_DYNAMIC_LIB_PATH: {}", lib_path.display());
            return Some(lib_path);
        }
    }

    // 5. On Linux, check standard library paths
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
                eprintln!("Using pdfium from system: {}", lib_path.display());
                return Some(lib_path);
            }
        }
    }

    None
}

/// Create a Pdfium instance, trying various library locations
pub fn create_pdfium() -> Result<Pdfium, PdfiumError> {
    // Try to find and load pdfium from a specific path
    if let Some(lib_path) = find_pdfium_library() {
        let lib_path_str = lib_path.to_string_lossy().to_string();
        eprintln!("Loading pdfium from: {}", lib_path_str);

        // Use bind_to_library with the full path to the library file
        let bindings = Pdfium::bind_to_library(&lib_path_str)?;
        return Ok(Pdfium::new(bindings));
    }

    // Fall back to default behavior (system library search)
    eprintln!("No pdfium found at known locations, trying system default...");
    Ok(Pdfium::default())
}

/// Create a static/leaked Pdfium instance for use with PdfDocument<'static>
///
/// MEMORY NOTE: This leaks the Pdfium instance to satisfy lifetime requirements.
/// For a single-document app this is fine. For multi-document scenarios,
/// consider refactoring to share a single Pdfium instance.
pub fn create_static_pdfium() -> Option<&'static Pdfium> {
    match create_pdfium() {
        Ok(pdfium) => Some(Box::leak(Box::new(pdfium))),
        Err(e) => {
            eprintln!("Failed to load pdfium library: {:?}", e);
            None
        }
    }
}

