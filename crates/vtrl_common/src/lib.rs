mod channel;
mod error;
mod macros;
mod message;
mod message_bus;
mod render;

pub mod prelude {
    // Re-exports
    pub use log;
    pub use ultraviolet::{IVec2, IVec3, IVec4, Mat2, Mat3, Mat4, Vec2, Vec3, Vec4};

    // Utilities
    pub mod channel {
        pub use crate::channel::*;
    }
    pub use crate::error::*;
    pub use crate::message::*;
    pub mod message_bus {
        pub use crate::message_bus::*;
    }
    pub use crate::render::*;

    // Macros
    pub use crate::c_str;
}
