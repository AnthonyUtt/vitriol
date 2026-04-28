use std::sync::Mutex;

use lazy_static::lazy_static;

#[cfg(debug_assertions)]
lazy_static! {
    static ref DEBUG_LINES: Mutex<Vec<String>> = Mutex::new(Vec::new());
}

#[cfg(debug_assertions)]
pub fn push_line(line: String) {
    if let Ok(mut lines) = DEBUG_LINES.lock() {
        lines.push(line);
    }
}

#[cfg(not(debug_assertions))]
pub fn push_line(_line: String) {}

#[cfg(debug_assertions)]
pub fn drain_lines() -> Vec<String> {
    if let Ok(mut lines) = DEBUG_LINES.lock() {
        std::mem::take(&mut *lines)
    } else {
        Vec::new()
    }
}

#[cfg(not(debug_assertions))]
pub fn drain_lines() -> Vec<String> {
    Vec::new()
}

/// Args are not evaluated in release builds (matches `debug_assert!`).
#[macro_export]
macro_rules! debug_println {
    ($($arg:tt)*) => {{
        #[cfg(debug_assertions)]
        $crate::debug::push_line(format!($($arg)*));
    }};
}
