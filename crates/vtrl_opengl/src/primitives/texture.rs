extern crate gl;

use crate::util::resources::{Resource, ResourceManager, TextureManager};
use vtrl_common::prelude::*;

#[derive(Clone, Debug)]
pub struct TextureData {
    pub bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
}
impl Resource for TextureData {}

#[derive(Copy, Clone, Debug)]
pub struct Texture {
    pub id: u32,
}

impl Texture {
    pub fn new(path: String) -> Result<Self> {
        unsafe {
            let tex_data = TextureManager::load_direct(path)?;

            let mut id: u32 = 0;
            gl::GenTextures(1, &mut id);

            // Assign texture to a texture unit
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, id);

            // Configure scaling algorithms
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            // Configure repetition
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                tex_data.width as i32,
                tex_data.height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                tex_data.bytes.as_ptr().cast(),
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);

            // unbind texture so it can't be changed anymore
            gl::BindTexture(gl::TEXTURE_2D, 0);

            Ok(Texture { id })
        }
    }

    pub fn new_from_data(data: TextureData) -> Self {
        unsafe {
            let mut id: u32 = 0;
            gl::GenTextures(1, &mut id);

            // Assign texture to a texture unit
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, id);

            // Configure scaling algorithms
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            // Configure repetition
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                data.width as i32,
                data.height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data.bytes.as_ptr().cast(),
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);

            // unbind texture so it can't be changed anymore
            gl::BindTexture(gl::TEXTURE_2D, 0);

            Texture { id }
        }
    }

    pub fn bind(&self, slot: u32) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + slot);
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }

    pub fn destroy(&self) {
        unsafe {
            gl::DeleteTextures(1, &self.id as *const u32);
        }
    }
}
