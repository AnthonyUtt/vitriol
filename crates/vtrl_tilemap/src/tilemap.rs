use serde::{Deserialize, Serialize};

use vtrl_common::prelude::*;
use vtrl_ecs::prelude::*;

pub type TileId = u32;

#[derive(Clone)]
pub struct TileSet {
    pub buffer: Vec<u8>,
    pub tile_size: u32, // in px
    pub row_count: u32,
    pub column_count: u32,
}

#[asset]
impl Asset for TileSet {
    fn load(bytes: Vec<u8>) -> Result<TileSet> {
        if bytes.len() < 16 {
            return Err(VtrlError::Asset("Invalid TileSet data!".to_string()));
        }

        let version = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
        log::trace!("Loaded TileSet with version {version}!");
        let tile_size = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
        let columns = u32::from_le_bytes(bytes[8..12].try_into().unwrap());
        let rows = u32::from_le_bytes(bytes[12..16].try_into().unwrap());
        let img = image::load_from_memory(&bytes[16..])?.into_rgba8();

        debug_assert_eq!(img.width(), columns * tile_size);
        debug_assert_eq!(img.height(), rows * tile_size);

        Ok(TileSet {
            buffer: img.to_vec(),
            tile_size,
            row_count: rows,
            column_count: columns,
        })
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TileGrid {
    pub width: u32, // in tiles
    pub height: u32, // in tiles
    pub rows: Vec<TileId>, // row-major, indexed by (y, x)
}

#[derive(Clone, Serialize, Deserialize, Copy, Eq, PartialEq)]
pub enum TileLayer {
    Background,
    Foreground,
}

#[component]
pub struct TileMap {
    pub tileset: AssetHandle,
    pub grid: TileGrid,
    pub layer: TileLayer,
}
