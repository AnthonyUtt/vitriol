use serde_derive::*;

mod bitmap;
mod camera;
mod packing;
mod queue;
mod text;

pub use bitmap::*;
pub use camera::*;
pub use packing::*;
pub use queue::*;
pub use text::*;

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

#[derive(Clone, Debug)]
pub struct TextureData {
    pub bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}
