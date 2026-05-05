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

#[component]
pub struct Camera {
    #[serde(default)]
    pub primary: bool,
    #[serde(default = "default_zoom")]
    pub zoom: f32,
    #[serde(default = "default_near")]
    pub near: f32,
    #[serde(default = "default_far")]
    pub far: f32,
}

fn default_zoom() -> f32 {
    1.0
}
fn default_near() -> f32 {
    -1.0
}
fn default_far() -> f32 {
    1.0
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            primary: false,
            zoom: 1.0,
            near: -1.0,
            far: 1.0,
        }
    }
}

impl Camera {
    /// Build the clip-from-world matrix for a 2D camera at `position` /
    /// `rotation` against a viewport of `window` pixels. World convention:
    /// origin at screen center, +y up, +x right.
    pub fn view_projection(&self, position: Vec2, rotation: f32, window: Vec2) -> Mat4 {
        let half_w = window.x * 0.5 / self.zoom;
        let half_h = window.y * 0.5 / self.zoom;
        let projection =
            projection::orthographic_gl(-half_w, half_w, -half_h, half_h, self.near, self.far);
        let view = Mat4::from_rotation_z(-rotation)
            * Mat4::from_translation(Vec3::new(-position.x, -position.y, 0.0));
        projection * view
    }
}
