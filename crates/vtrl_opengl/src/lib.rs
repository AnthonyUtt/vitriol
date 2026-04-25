pub mod context;
pub mod plugin;
pub(crate) mod primitives;
pub(crate) mod renderers;
pub(crate) mod shaders;
pub(crate) mod types;

pub mod prelude {
    pub use crate::context as render_context;
    pub use crate::primitives::Texture;
}
