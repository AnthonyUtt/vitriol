use vtrl_common::prelude::*;

use crate::prelude::*;

#[component]
pub struct Text {
    pub text: String,
    pub style: TextStyle,
    pub color: Vec4,
}

#[component]
pub struct Transform {
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
    pub z_index: f32,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec2::zero(),
            rotation: 0.0,
            scale: Vec2::one(),
            z_index: 0.0,
        }
    }
}

#[component]
pub struct Velocity {
    pub direction: Vec2,
    pub speed: f32,
}

#[component]
pub struct Quad {
    pub size: Vec2,
    pub color: Vec4,
}

#[component]
pub struct Sprite {
    pub size: Vec2,
    pub texture_handle: AssetHandle,
    pub uv: Vec4,
    pub color: Vec4,
}

#[component]
pub struct Animation {
    pub texture_handle: AssetHandle,
    pub current_frame: usize,
    pub active_animation: String,
    pub elapsed: f32,
}

#[component]
pub struct Script {
    pub script_handle: AssetHandle,
}
