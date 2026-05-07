extern crate gl;

#[derive(Debug, Copy, Clone)]
pub struct Texture {
    pub id: u32,
}

impl Texture {
    pub fn new(buffer: &[u8], width: u32, height: u32) -> Self {
        unsafe {
            let mut id: u32 = 0;
            gl::GenTextures(1, &mut id);

            // Assign to a texture unit
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, id);

            // Configure scaling
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            // Configure repetition
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);

            // Set texture data
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                width as i32,
                height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                buffer.as_ptr().cast(),
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);

            // unbind to prevent further changes
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
        unsafe { gl::BindTexture(gl::TEXTURE_2D, 0) }
    }

    pub fn destroy(&self) {
        unsafe { gl::DeleteTextures(1, &self.id as *const u32) }
    }
}
