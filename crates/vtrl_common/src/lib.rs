mod asset;
mod channel;
pub mod debug;
mod error;
mod macros;
mod message;
mod message_bus;
mod render;
mod scripting;

#[rustfmt::skip]
pub mod prelude {
    // Re-exports
    pub use log;
    pub use ultraviolet::{
        Vec2, Vec3, Vec4,
        IVec2, IVec3, IVec4,
        UVec2, UVec3, UVec4,
        Mat2, Mat3, Mat4,
        projection,
    };
    pub use serde;
    pub use inventory;
    pub use rhai;

    pub use vtrl_macros::{asset, scriptable};

    // Utilities
    pub use crate::asset::*;
    pub use crate::channel::*;
    pub use crate::error::*;
    pub use crate::message::*;
    pub mod message_bus {
        pub use crate::message_bus::*;
    }
    pub use crate::render::*;
    pub use crate::scripting::*;

    // Macros
    pub use crate::c_str;
    pub use crate::debug_println;
}
