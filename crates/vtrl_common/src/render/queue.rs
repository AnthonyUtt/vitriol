use std::sync::Arc;
use ultraviolet::{Vec2, Vec4};

use super::camera::Camera;

pub type FramebufferId = u32;

pub enum RenderTarget {
    Screen,
    Framebuffer(FramebufferId),
}

pub enum BlendMode {
    Alpha,
    Additive,
    Multiply,
}

pub enum RenderCommand {
    // Frame orchestration
    BeginFrame { clear_color: Vec4 },
    EndFrame,

    // Render pass orchestration
    BeginPass { name: &'static str, clear: Option<Vec4>, blend_mode: BlendMode, camera: Box<dyn Camera> },
    EndPass,

    // Batching
    Batch(Vec<RenderCommand>),

    // Rendering
    DrawQuads { instances: Arc<[QuadInstance]> },
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct QuadInstance {
    pub pos: Vec2,
    pub size: Vec2,
    pub rot: f32,
    pub z: f32,
    pub color: Vec4,
    pub uv: Vec4,
    pub tex: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GlyphInstance {
    pub pos: Vec2,
    pub size: Vec2,
    pub rot: f32,
    pub z: f32,
    pub color: Vec4,
    pub uv: Vec4,
    pub tex: f32,
}
