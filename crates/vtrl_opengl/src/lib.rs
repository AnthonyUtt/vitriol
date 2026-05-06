pub(crate) mod commands;
pub(crate) mod primitives;
pub(crate) mod shaders;
pub(crate) mod types;

pub mod prelude {
    pub use crate::primitives::*;
    pub use crate::shaders::*;
    pub use crate::types::*;

    pub mod commands {
        pub use crate::commands::*;
    }
}
