use std::sync::Arc;
use ultraviolet::{Vec2, Vec4};

pub type FramebufferId = u32;

#[derive(Debug, Clone, Copy)]
pub enum RenderTarget {
    Screen,
    Framebuffer(FramebufferId),
}

#[derive(Debug, Clone, Copy)]
pub enum BlendMode {
    Alpha,
    Additive,
    Multiply,
    PremultipliedAlpha,
    Screen,
    Subtract,
    Replace,
}

#[derive(Debug, Clone)]
pub enum RenderCommand {
    // Frame orchestration
    BeginFrame {
        clear_color: Vec4,
    },
    EndFrame,

    // Render pass orchestration
    BeginPass {
        name: &'static str,
        target: RenderTarget,
        clear: Option<Vec4>,
        blend_mode: Option<BlendMode>,
    },
    EndPass,

    // Batching
    Batch(Vec<RenderCommand>),

    // Rendering
    DrawQuads {
        instances: Arc<[QuadInstance]>,
    },
    DrawText {
        instances: Arc<[GlyphInstance]>,
    },
    DrawLines {
        instances: Arc<[LineInstance]>,
    },
    DrawCircles {
        instances: Arc<[CircleInstance]>,
    },
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

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CircleInstance {
    pub pos: Vec2,
    pub radius: f32,
    pub z: f32,
    pub color: Vec4,
    pub thickness: f32,
    pub fade: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct LineInstance {
    pub start: Vec2,
    pub end: Vec2,
    pub color: Vec4,
    pub thickness: f32,
}
