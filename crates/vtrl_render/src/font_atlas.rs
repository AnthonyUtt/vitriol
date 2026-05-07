use std::collections::HashMap;

use vtrl_common::prelude::*;
use vtrl_opengl::prelude::*;

const ATLAS_SIZE: u32 = 1024;
const MAX_FONTS: u32 = 32;

pub struct FontAtlas {
    tex_array: TextureArray,
    fonts: HashMap<AssetHandle, HashMap<char, Glyph>>,
    handles_to_ids: HashMap<AssetHandle, usize>,
    width: u32,
    height: u32,
    debug_font: Option<AssetHandle>,
}

impl FontAtlas {
    pub fn new() -> Self {
        let width = ATLAS_SIZE;
        let height = ATLAS_SIZE;
        let tex_array = TextureArray::new(width, height, MAX_FONTS, None, Some(1));

        Self {
            tex_array,
            fonts: HashMap::new(),
            handles_to_ids: HashMap::new(),
            width,
            height,
            debug_font: None,
        }
    }

    pub fn set_debug_font(&mut self, handle: AssetHandle, glyphs: HashMap<char, Glyph>) -> Result<()> {
        self.register_font(handle, glyphs)?;
        self.debug_font = Some(handle);
        Ok(())
    }

    pub fn get_debug_font(&self) -> Option<AssetHandle> {
        self.debug_font
    }

    pub fn register_font(&mut self, handle: AssetHandle, mut glyphs: HashMap<char, Glyph>) -> Result<()> {
        let mut bitmap = Bitmap::<u8, 1>::new(self.width, self.height);
        let mut packer = ShelfPacker {
            width: self.width,
            height: self.height,
            shelves: Vec::new(),
        };

        let inv_w = 1.0 / self.width as f32;
        let inv_h = 1.0 / self.height as f32;

        for glyph in glyphs.values_mut() {
            let buffer_position =
                packer
                    .pack(glyph.width, glyph.height)
                    .ok_or(VtrlError::Renderer(
                        "Unable to fit font into atlas!".to_string(),
                    ))?;

            bitmap.put(
                &glyph.buffer,
                buffer_position.x,
                buffer_position.y,
                glyph.width,
                glyph.height,
            );

            // UV = (u_min, v_min, u_max, v_max), shader maps this to the
            // atlas-space rectangle the glyph quad samples.
            glyph.uv = Vec4::new(
                buffer_position.x as f32 * inv_w,
                buffer_position.y as f32 * inv_h,
                (buffer_position.x + glyph.width) as f32 * inv_w,
                (buffer_position.y + glyph.height) as f32 * inv_h,
            );

            // Bitmap data is now baked into the atlas; drop the per-glyph
            // CPU copy to keep the cached glyph map small.
            glyph.buffer = Vec::new();
        }

        let data = TextureData {
            bytes: bitmap.buffer,
            width: bitmap.width,
            height: bitmap.height,
        };

        let id = self.tex_array.add_texture(&data)?;

        self.fonts.insert(handle, glyphs);
        self.handles_to_ids.insert(handle, id);

        Ok(())
    }

    pub fn get_glyph(&self, font: AssetHandle, c: char) -> Option<Glyph> {
        self.fonts.get(&font).map(|f| f.get(&c).cloned().unwrap())
    }

    pub fn get_font_tex_id(&self, font: AssetHandle) -> Option<&usize> {
        self.handles_to_ids.get(&font)
    }
}

impl Default for FontAtlas {
    fn default() -> Self { Self::new() }
}
