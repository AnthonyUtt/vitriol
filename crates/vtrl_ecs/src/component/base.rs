use std::sync::Arc;

use vtrl_common::prelude::*;

use crate::prelude::*;

#[derive(Component)]
pub struct TextComponent {
    pub text: Arc<str>,
    pub style: TextStyle,
    pub color: Vec4,
}

#[derive(Component)]
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

#[derive(Component)]
pub struct VelocityComponent {
    pub direction: Vec2,
    pub speed: f32,
}

#[derive(Component)]
pub struct QuadComponent {
    pub size: Vec2,
    pub color: Vec4,
}

#[derive(Component)]
pub struct SpriteComponent {
    pub size: Vec2,
    pub texture_handle: Symbol,
    pub uv: Vec4,
    pub color: Vec4,
}

#[derive(Component)]
pub struct AnimationComponent {
    pub texture_handle: Symbol,
    pub current_frame: usize,
    pub active_animation: Arc<str>,
    pub elapsed: f32,
}
