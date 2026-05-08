mod atlas;
mod plugin;
mod renderer;
mod shaders;
mod tilemap;

pub mod prelude {
    pub use crate::plugin::*;
    pub use crate::tilemap::*;
}
