use std::collections::HashMap;

use vtrl_common::prelude::*;

pub struct FontAtlas {
    pub id: u32,
    pub width: u32,
    pub height: u32,
    pub layers: u32,
    active_layers: Vec<bool>,
    glyphs: Vec<Option<HashMap<char, Glyph>>>,
}

impl FontAtlas {
    pub fn new(width: u32, height: u32, count: u32) -> Self {
        unsafe {
            let mut id: u32 = 0;
            gl::GenTextures(1, &mut id);

            // Bind the texture array and allocate storage space on the GPU
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, id);
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
            // glTexStorage* requires a sized internal format; gl::RED is the
            // base format used by glTexSubImage to describe source pixels.
            gl::TexStorage3D(
                gl::TEXTURE_2D_ARRAY,
                1,
                gl::R8,
                width as i32,
                height as i32,
                count as i32,
            );

            // Configure scaling/repeating
            gl::TexParameteri(
                gl::TEXTURE_2D_ARRAY,
                gl::TEXTURE_MIN_FILTER,
                gl::NEAREST as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D_ARRAY,
                gl::TEXTURE_MAG_FILTER,
                gl::NEAREST as i32,
            );
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);

            // unbind to prevent further changes
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, 0);

            let mut glyphs = Vec::with_capacity(count as usize);
            glyphs.resize_with(count as usize, || None);

            FontAtlas {
                id,
                width,
                height,
                layers: count,
                active_layers: vec![false; count as usize],
                glyphs,
            }
        }
    }

    pub fn add_font(&mut self, mut glyphs: HashMap<char, Glyph>) -> Result<usize> {
        let layer = self
            .active_layers
            .iter()
            .position(|&active| !active)
            .ok_or(VtrlError::Renderer(
                "No space left in font atlas!".to_string(),
            ))?;

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

            // iUV = (u_min, v_min, u_max, v_max), shader maps this to the
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

        self.bind(0);

        unsafe {
            gl::TexSubImage3D(
                gl::TEXTURE_2D_ARRAY,
                0_i32,
                0, // x offset
                0, // y offset
                layer as i32,
                self.width as i32,
                self.height as i32,
                1, // layer count
                gl::RED,
                gl::UNSIGNED_BYTE,
                bitmap.buffer.as_ptr().cast(),
            );
        }

        self.unbind();

        self.active_layers[layer] = true;
        self.glyphs[layer] = Some(glyphs);

        Ok(layer)
    }

    pub fn get_glyph(&self, font_id: usize, c: char) -> Option<&Glyph> {
        self.glyphs.get(font_id)?.as_ref()?.get(&c)
    }

    pub fn bind(&self, slot: u32) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + slot);
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, self.id);
        }
    }

    pub fn unbind(&self) {
        unsafe { gl::BindTexture(gl::TEXTURE_2D_ARRAY, 0) }
    }

    pub fn destroy(&self) {
        unsafe { gl::DeleteTextures(1, &self.id as *const u32) }
    }
}
