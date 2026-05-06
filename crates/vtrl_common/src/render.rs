use serde_derive::*;

mod bitmap;
mod command;
mod context;
mod packing;
mod text;

pub use bitmap::*;
pub use command::*;
pub use context::*;
pub use packing::*;
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

pub struct Viewport {
    pub width: u32,
    pub height: u32,
}
