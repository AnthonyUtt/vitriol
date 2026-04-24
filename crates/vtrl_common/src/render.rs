mod bitmap;
mod camera;

pub use bitmap::*;
pub use camera::*;

pub struct WindowSettings {
    pub width: u32,
    pub height: u32,
    pub title: &'static str,
}

impl Default for WindowSettings {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            title: "VITRIOL Engine",
        }
    }
}
