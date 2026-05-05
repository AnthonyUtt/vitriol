use std::sync::Arc;
use ultraviolet::{Mat4, Vec2, Vec4};

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
        view_projection: Option<Mat4>,
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

pub fn instances_erased<T>(instances: &[T]) -> &[RenderInstance] {
    use std::mem;
    assert_eq!(mem::size_of::<T>(), mem::size_of::<RenderInstance>());
    assert_eq!(mem::align_of::<T>(), mem::align_of::<RenderInstance>());

    unsafe { mem::transmute(instances) }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RenderInstance {
    a: Vec2,
    b: Vec2,
    c: f32,
    d: f32,
    e: Vec4,
    f: Vec4,
    g: f32,
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
    pub size: Vec2,
    pub thickness: f32,
    pub fade: f32,
    pub color: Vec4,
    pub uv: Vec4,
    pub tex: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct LineInstance {
    pub start: Vec2,
    pub end: Vec2,
    pub thickness: f32,
    pub fade: f32,
    pub color: Vec4,
    pub _uv: Vec4,
    pub _tex: f32,
}
