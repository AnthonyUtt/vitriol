use std::collections::HashMap;

use vtrl_common::prelude::*;
use vtrl_opengl::prelude::*;

const MAX_LAYERS: u32 = 64;

pub struct TextureAtlas {
    tex_array: TextureArray,
    handles_to_ids: HashMap<AssetHandle, usize>,
}

impl TextureAtlas {
    pub fn new(width: u32, height: u32) -> Self {
        let tex_array = TextureArray::new(width, height, MAX_LAYERS, None, Some(4));

        Self {
            tex_array,
            handles_to_ids: HashMap::new(),
        }
    }

    pub fn register_texture(&mut self, handle: AssetHandle, data: &TextureData) -> Result<()> {
        let id = self.tex_array.add_texture(data)?;
        self.handles_to_ids.insert(handle, id);

        Ok(())
    }

    pub fn get_uv_scalar(&self, handle: AssetHandle) -> Vec2 {
        match self.handles_to_ids.get(&handle) {
            Some(id) => self.tex_array.get_uv_scalar(*id),
            None => Vec2::zero(),
        }
    }
}
