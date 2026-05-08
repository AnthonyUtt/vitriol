use std::collections::HashMap;

use vtrl_common::prelude::*;
use vtrl_opengl::prelude::*;

use crate::tilemap::TileSet;

const TILESET_SIZE: u32 = 1024;
const MAX_TILESETS: u32 = 16;

pub struct TileAtlas {
    tex_array: TextureArray,
    handles_to_ids: HashMap<AssetHandle, usize>,
}

impl TileAtlas {
    pub fn new() -> Self {
        let tex_array = TextureArray::new(TILESET_SIZE, TILESET_SIZE, MAX_TILESETS, None, Some(4));

        Self {
            tex_array,
            handles_to_ids: HashMap::new(),
        }
    }

    pub fn push(&mut self, handle: AssetHandle, set: TileSet) -> Result<()> {
        let data = TextureData {
            bytes: set.buffer,
            width: set.column_count * set.tile_size,
            height: set.row_count * set.tile_size,
        };

        let id = self.tex_array.add_texture(&data)?;
        self.handles_to_ids.insert(handle, id);

        Ok(())
    }

    pub fn get_uv_scalar(&self, handle: AssetHandle) -> Vec2 {
        match self.handles_to_ids.get(&handle) {
            Some(id) => self.tex_array.get_uv_scalar(*id),
            None => Vec2::zero(),
        }
    }

    pub fn get_texture_id(&self, handle: AssetHandle) -> Option<&usize> {
        self.handles_to_ids.get(&handle)
    }

    pub fn bind(&self, slot: u32) {
        self.tex_array.bind(slot)
    }

    pub fn unbind(&self) {
        self.tex_array.unbind()
    }
}

impl Default for TileAtlas {
    fn default() -> Self { Self::new() }
}
