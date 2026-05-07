extern crate freetype as ft;

use std::collections::HashMap;
use serde_derive::*;
use ultraviolet::Vec4;

use crate::{
    asset::Asset,
    error::Result,
};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct TextStyle {
    pub size: f32,
    pub slant: TextSlant,
    pub weight: TextWeight,
    pub line_height: f32,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            size: 16.0,
            slant: TextSlant::Normal,
            weight: TextWeight::Regular,
            line_height: 1.2,
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum TextSlant {
    Normal,
    Italic,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum TextWeight {
    Regular,
    Bold,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Anchor {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Glyph {
    pub advance_x: i64,
    pub advance_y: i64,
    pub width: u32,
    pub height: u32,
    pub top: u32,
    pub left: u32,
    pub uv: Vec4,
    pub buffer: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Font {
    pub glyphs: HashMap<char, Glyph>,
}

impl Asset for Font {
    fn load(bytes: Vec<u8>) -> Result<Font> {
        let library = ft::Library::init()?;
        let face = library.new_memory_face(bytes, 0)?;
        face.set_pixel_sizes(0, DEFAULT_PIXEL_HEIGHT)?;

        let glyphs = build_glyph_map(&face)?;

        Ok(Font { glyphs })
    }
}

/// Pixel height the engine rasterizes fonts at by default. Matches
/// `TextStyle::default().size` so the debug overlay's line spacing lines up
/// with the rendered glyph height.
pub const DEFAULT_PIXEL_HEIGHT: u32 = 16;

pub fn build_glyph_map(face: &ft::Face) -> Result<HashMap<char, Glyph>> {
    let charset = DEFAULT_CHARSET;
    let mut glyphs = HashMap::<char, Glyph>::with_capacity(charset.len());
    for c in charset.iter() {
        let c_char = char::from_u32(*c as u32).unwrap();
        face.load_char(c.to_owned(), ft::face::LoadFlag::RENDER)?;

        let ft_glyph = face.glyph();
        ft_glyph.render_glyph(ft::RenderMode::Normal)?;

        let glyph = Glyph {
            advance_x: ft_glyph.advance().x >> 6,
            advance_y: ft_glyph.advance().y >> 6,
            width: ft_glyph.bitmap().width() as u32,
            height: ft_glyph.bitmap().rows() as u32,
            top: ft_glyph.bitmap_top() as u32,
            left: ft_glyph.bitmap_left() as u32,
            uv: Vec4::zero(),
            buffer: ft_glyph.bitmap().buffer().to_vec(),
        };

        glyphs.insert(c_char, glyph);
    }
    Ok(glyphs)
}

// ASCII
const DEFAULT_CHARSET: [usize; 128] = [
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
    0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F,
    0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2A, 0x2B, 0x2C, 0x2D, 0x2E, 0x2F,
    0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x3E, 0x3F,
    0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4A, 0x4B, 0x4C, 0x4D, 0x4E, 0x4F,
    0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A, 0x5B, 0x5C, 0x5D, 0x5E, 0x5F,
    0x60, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x6B, 0x6C, 0x6D, 0x6E, 0x6F,
    0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x7B, 0x7C, 0x7D, 0x7E, 0x7F,
];
