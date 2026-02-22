/// Utility functions and macros.

use std::io::Write;

/// Prints to stderr, ignoring broken pipe errors.
/// Required because sometimes stderr may be disconnected in WSL.
#[macro_export]
macro_rules! safe_eprintln {
    ($($arg:tt)*) => {
        let _ = writeln!(std::io::stderr(), $($arg)*);
    };
}