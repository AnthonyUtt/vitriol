extern crate gl;

use vtrl_common::prelude::*;
use crate::util::resources::{FontData, FontManager, ResourceManager};

pub struct FontAtlas {
    id: u32,
    pub bitmap: Bitmap<u8, 1>,
    pub font_data: FontData,
}
impl FontAtlas {
    pub fn new(path: String) -> Result<FontAtlas> {
        let font_data = FontManager::load_direct(path)?;
        let atlas = FontAtlas::new_from_data(font_data);
        Ok(atlas)
    }

    pub fn new_from_data(mut font_data: FontData) -> FontAtlas {
        unsafe {
            let bitmap = Self::pack_glyphs(&mut font_data);

            let mut id: u32 = 0;
            gl::GenTextures(1, &mut id);

            // Assign texture to unit
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, id);
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);

            // Configure scaling algorithms
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            // Configure repetition
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RED as i32,
                bitmap.width as i32,
                bitmap.height as i32,
                0,
                gl::RED,
                gl::UNSIGNED_BYTE,
                bitmap.buffer.as_ptr().cast(),
            );

            gl::BindTexture(gl::TEXTURE_2D, 0);

            FontAtlas {
                id,
                bitmap,
                font_data,
            }
        }
    }

    pub unsafe fn bind(&self, slot: u32) {
        gl::ActiveTexture(gl::TEXTURE0 + slot);
        gl::BindTexture(gl::TEXTURE_2D, self.id);
    }

    pub unsafe fn unbind(&self) {
        gl::BindTexture(gl::TEXTURE_2D, 0);
    }

    pub unsafe fn destory(&self) {
        gl::DeleteTextures(1, &self.id as *const u32);
    }

    // private

    fn pack_glyphs(font_data: &mut FontData) -> Bitmap<u8, 1> {
        FontAtlas::linear_pack(font_data)
    }

    fn linear_pack(font_data: &mut FontData) -> Bitmap<u8, 1> {
        // Naive approach
        // Creates texture with a single row of characters
        let mut width: u32 = 0;
        let mut height: u32 = 0;

        // Loop through all code points to get the max width/height
        // of the bitmap for the atlas
        for code_point in font_data.code_points.values() {
            width += code_point.glyph.width;
            height = std::cmp::max(height, code_point.glyph.height);
        }

        let mut bitmap = Bitmap::<u8, 1>::new(width, height);

        // Loop through code points again to add
        // all glyphs to the bitmap
        let mut x_offset: u32 = 0;
        for code_point in font_data.code_points.values_mut() {
            // Set tex_coords for glyph
            code_point.metrics.tex_coords.x = x_offset as f32 / width as f32;
            // Add glyph to map
            bitmap.put(
                &code_point.glyph.buffer,
                x_offset,
                0,
                code_point.glyph.width,
                code_point.glyph.height,
            );
            // Increment x_offset by the glyph width
            x_offset += code_point.glyph.width;
        }

        bitmap
    }
}
