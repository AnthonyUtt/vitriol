use serde_derive::*;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct TextStyle {
    pub font_id: u32,
    pub size: f32,
    pub slant: TextSlant,
    pub weight: TextWeight,
    pub line_height: f32,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_id: 0,
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
