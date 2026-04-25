use image::EncodableLayout;
use std::path::Path;

use crate::prelude::{Asset, Result};

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

#[derive(Clone, Debug)]
pub struct TextureData {
    pub bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl Asset for TextureData {
    fn load(path: &Path) -> Result<TextureData> {
        let img = image::open(path)?.into_rgba8();

        Ok(TextureData {
            bytes: img.as_bytes().to_vec(),
            width: img.width(),
            height: img.height(),
        })
    }
}
