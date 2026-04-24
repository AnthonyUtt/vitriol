use ultraviolet::{Vec2, Vec3, Vec4};

pub mod commands;
mod primitives;
mod shaders;

pub mod prelude {
    pub use crate::UniformType;
    pub use crate::commands;
    pub use crate::primitives::*;
    pub use crate::shaders::*;
}

#[derive(Debug, Copy, Clone)]
pub enum UniformType {
    Float,
    Vec2,
    Vec3,
    Vec4,
    Int,
    Bool,
}
impl UniformType {
    pub fn num_components(&self) -> i32 {
        match self {
            Self::Float => 1,
            Self::Vec2 => 2,
            Self::Vec3 => 3,
            Self::Vec4 => 4,
            Self::Int => 1,
            Self::Bool => 1,
        }
    }

    pub fn size(&self) -> i32 {
        match self {
            Self::Float => std::mem::size_of::<f32>() as i32,
            Self::Vec2 => std::mem::size_of::<Vec2>() as i32,
            Self::Vec3 => std::mem::size_of::<Vec3>() as i32,
            Self::Vec4 => std::mem::size_of::<Vec4>() as i32,
            Self::Int => std::mem::size_of::<u32>() as i32,
            Self::Bool => std::mem::size_of::<bool>() as i32,
        }
    }
}
