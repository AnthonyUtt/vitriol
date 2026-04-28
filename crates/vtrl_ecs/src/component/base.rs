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
