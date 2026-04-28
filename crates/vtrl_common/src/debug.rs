use std::sync::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    static ref DEBUG_LINES: Mutex<Vec<String>> = Mutex::new(Vec::new());
}

pub fn push_line(line: String) {
    if let Ok(mut lines) = DEBUG_LINES.lock() {
        lines.push(line);
    }
}

pub fn drain_lines() -> Vec<String> {
    if let Ok(mut lines) = DEBUG_LINES.lock() {
        std::mem::take(&mut *lines)
    } else {
        Vec::new()
    }
}

#[macro_export]
macro_rules! debug_println {
    ($($arg:tt)*) => {{
        $crate::debug::push_line(format!($($arg)*));
    }};
}
