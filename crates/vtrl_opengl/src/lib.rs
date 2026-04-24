pub mod context;
pub mod plugin;
mod primitives;
mod renderers;
mod shaders;
mod types;

pub mod prelude {
    pub use crate::context as render_context;
    pub use crate::primitives::*;
    pub use crate::shaders::*;
    pub use crate::types::*;
}
