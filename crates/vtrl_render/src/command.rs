use std::sync::Arc;

use vtrl_common::prelude::*;
use vtrl_ecs::prelude::*;

#[derive(Default)]
pub struct CommandBuffer {
    commands: Vec<RenderCommand>,
}

impl CommandBuffer {
    pub fn push(&mut self, cmd: RenderCommand) {
        self.commands.push(cmd);
    }

    pub fn take(&mut self) -> Vec<RenderCommand> {
        let inner = self.commands.clone();
        self.commands = Vec::new();
        inner
    }
}

#[derive(Clone)]
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
    Complex(Arc<dyn FnOnce(&World)>),
}

impl std::fmt::Debug for RenderCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BeginFrame { clear_color } => {
                f.debug_struct("BeginFrame")
                    .field("clear_color", clear_color)
                    .finish()
            },
            Self::EndFrame => f.debug_tuple("EndFrame").finish(),
            Self::BeginPass {
                name,
                target,
                clear,
                blend_mode,
                view_projection,
            } => {
                f.debug_struct("BeginPass")
                    .field("name", name)
                    .field("target", target)
                    .field("clear", clear)
                    .field("blend_mode", blend_mode)
                    .field("matrix", view_projection)
                    .finish()
            },
            Self::EndPass => f.debug_tuple("EndPass").finish(),
            Self::Batch(cmds) => {
                f.debug_tuple("Batch")
                    .field(&format!("{} commands", cmds.len()))
                    .finish()
            },
            Self::DrawQuads { instances } => {
                f.debug_tuple("DrawQuads")
                    .field(&format!("{} instances", instances.len()))
                    .finish()
            },
            Self::DrawText { instances } => {
                f.debug_tuple("DrawText")
                    .field(&format!("{} instances", instances.len()))
                    .finish()
            },
            Self::DrawLines { instances } => {
                f.debug_tuple("DrawLines")
                    .field(&format!("{} instances", instances.len()))
                    .finish()
            },
            Self::DrawCircles { instances } => {
                f.debug_tuple("DrawCircles")
                    .field(&format!("{} instances", instances.len()))
                    .finish()
            },
            Self::Complex(_) => f.debug_tuple("Complex").field(&"<closure>").finish(),
        }
    }
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
