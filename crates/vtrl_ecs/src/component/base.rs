use vtrl_common::prelude::*;

use crate::prelude::*;

#[component]
pub struct TextComponent {
    pub text: String,
    pub style: TextStyle,
    pub color: Vec4,
}

#[component]
pub struct TransformComponent {
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
    pub z_index: f32,
}

impl Default for TransformComponent {
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
pub struct VelocityComponent {
    pub direction: Vec2,
    pub speed: f32,
}

#[component]
pub struct QuadComponent {
    pub size: Vec2,
    pub color: Vec4,
}

#[component]
pub struct SpriteComponent {
    pub size: Vec2,
    pub texture_handle: AssetHandle,
    pub uv: Vec4,
    pub color: Vec4,
}

#[component]
pub struct AnimationComponent {
    pub texture_handle: AssetHandle,
    pub current_frame: usize,
    pub active_animation: String,
    pub elapsed: f32,
}
