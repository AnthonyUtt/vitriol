extern crate gl;

use super::TextureData;
use crate::logging::error;
use crate::util::errors::{ContextValue, EngineError, ErrorContext};
use crate::Result;
use ultraviolet::{UVec2, Vec2};

#[derive(Clone, Debug)]
pub struct TextureArray {
    pub id: u32,
    pub width: u32,
    pub height: u32,
    pub layers: u32,
    pub mip_count: u32,
    active_layers: Vec<bool>,
    uv_scalars: Vec<Vec2>,
}

impl TextureArray {
    pub fn new(width: u32, height: u32, count: u32, mip_count: Option<u32>) -> TextureArray {
        unsafe {
            let mut id: u32 = 0;
            gl::GenTextures(1, &mut id);

            // Bind the texture and allocate storage for it
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, id);
            gl::TexStorage3D(
                gl::TEXTURE_2D_ARRAY,
                mip_count.unwrap_or(1) as i32,
                gl::RGBA8,
                width as i32,
                height as i32,
                count as i32,
            );

            // Configure scaling and repeating
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

            TextureArray {
                id,
                width,
                height,
                layers: count,
                mip_count: mip_count.unwrap_or(1),
                active_layers: vec![false; count as usize],
                uv_scalars: vec![Vec2::one(); count as usize],
            }
        }
    }

    pub fn add_texture(&mut self, data: TextureData) -> Result<usize> {
        if data.width > self.width || data.height > self.height {
            error!(
                "Cannot add texture with dimensions {}x{}, max dimensions are {}x{}!",
                data.width, data.height, self.width, self.height,
            );
            return Err(EngineError::new()
                .name("InvalidDimensionsError")
                .message("Cannot add texture with given dimensions!")
                .with_context(vec![
                    ErrorContext(
                        "received",
                        ContextValue::Str(format!("{}x{}", data.width, data.height)),
                    ),
                    ErrorContext(
                        "expected",
                        ContextValue::Str(format!("{}X{}", self.width, self.height)),
                    ),
                ]));
        }

        let available_layers = {
            let mut layers: Vec<usize> = Vec::with_capacity(self.layers as usize);
            for (layer, active) in self.active_layers.iter().enumerate() {
                if !active {
                    layers.push(layer);
                }
            }
            layers
        };

        let layer = available_layers.first().ok_or(
            EngineError::new()
                .name("TextureArrayFullError")
                .message("No available layers left in texture array!"),
        )?;
        let layer = layer.to_owned();

        let uv_scalar = {
            let u_scalar = if data.width != self.width {
                data.width as f32 / self.width as f32
            } else {
                1.
            };
            let v_scalar = if data.height != self.height {
                data.height as f32 / self.height as f32
            } else {
                1.
            };
            Vec2::new(u_scalar, v_scalar)
        };

        let bytes = {
            if self.width > data.width {
                self.map_smaller_texture_to_padded_bytes(
                    data.bytes,
                    UVec2::new(data.width, data.height),
                )
            } else {
                data.bytes
            }
        };

        self.set_texture_data(layer, None, &bytes);
        self.active_layers[layer] = true;

        self.uv_scalars[layer] = uv_scalar;

        Ok(layer)
    }

    pub fn get_uv_scalar(&self, id: usize) -> Vec2 {
        if id > self.layers as usize {
            Vec2::zero()
        } else if self.active_layers[id] {
            self.uv_scalars[id]
        } else {
            Vec2::zero()
        }
    }

    pub fn remove_texture(&mut self, id: usize) -> Result<()> {
        if id >= self.layers as usize {
            return Err(EngineError::new()
                .name("IndexOutOfBoundsError")
                .message("Requested ID is outside bounds of texture array!")
                .with_context(vec![
                    ErrorContext("id", ContextValue::UIntSize(id)),
                    ErrorContext("layer_count", ContextValue::UInt32(self.layers)),
                ]));
        }

        self.active_layers[id] = false;
        Ok(())
    }

    fn set_texture_data(&self, layer: usize, mip_level: Option<u32>, bytes: &[u8]) {
        let level = mip_level.unwrap_or(0);

        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, self.id);
            gl::TexSubImage3D(
                gl::TEXTURE_2D_ARRAY,
                level as i32,
                0, // x offset
                0, // y offset
                layer as i32,
                self.width as i32,
                self.height as i32,
                1, // layer count
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                bytes.as_ptr().cast(),
            );
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, 0);
        }
    }

    fn map_smaller_texture_to_padded_bytes(
        &self,
        tex_bytes: Vec<u8>,
        tex_dimensions: UVec2,
    ) -> Vec<u8> {
        let byte_count = self.width * self.height * 4; // each rgba color is 4 bytes
        let mut buffer: Vec<u8> = vec![0; byte_count as usize];

        // for each row in the texture, add the bytes to the buffer at the correct
        // position. Since the buffer is full of zeroes, we don't need to pad, just
        // assign values for the slice
        for row in 0..tex_dimensions.y {
            let buffer_start: usize = row as usize * (self.width * 4) as usize;
            let buffer_end: usize = buffer_start + (tex_dimensions.x * 4) as usize;
            let tex_start: usize = row as usize * (tex_dimensions.x * 4) as usize;
            let tex_end: usize = tex_start + (tex_dimensions.x * 4) as usize;

            buffer[buffer_start..buffer_end].clone_from_slice(&tex_bytes[tex_start..tex_end]);
        }

        buffer
    }

    pub fn bind(&self, slot: u32) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + slot);
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, self.id);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, 0);
        }
    }

    pub fn destroy(&self) {
        unsafe {
            gl::DeleteTextures(1, &self.id as *const u32);
        }
    }
}
