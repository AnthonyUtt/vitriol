pub(crate) mod animations;
pub mod context;
pub mod plugin;
pub(crate) mod primitives;
pub(crate) mod renderer;
pub(crate) mod shaders;
pub(crate) mod types;

pub mod prelude {
    pub use crate::animations::*;
    pub use crate::context as render_context;
    pub use crate::primitives::*;
}
