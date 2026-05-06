extern crate gl;
extern crate glfw;

use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
    },
};

use vtrl_common::prelude::*;

use crate::renderer::*;
use crate::types::*;

lazy_static! {
    static ref RENDER_CONTEXT: Arc<Mutex<Box<RenderContext>>> =
        Arc::new(Mutex::new(Box::new(RenderContext::new())));
    static ref RENDER_QUEUE: RenderQueue = RenderQueue::new();
}

pub fn push_command(cmd: RenderCommand) {
    RENDER_QUEUE.push(cmd);
}

pub fn process_queue() {
    let mut ctx = RENDER_CONTEXT.lock().unwrap();
    RENDER_QUEUE.process(&mut ctx);
}

pub fn register_texture(texture: &TextureData) -> Result<usize> {
    RENDER_CONTEXT.lock().unwrap().register_texture(texture)
}

pub fn register_font(glyphs: HashMap<char, Glyph>) -> Result<usize> {
    RENDER_CONTEXT.lock().unwrap().register_font(glyphs)
}

pub fn compute_uv(texture_id: usize, uv: Vec4) -> Vec4 {
    RENDER_CONTEXT.lock().unwrap().compute_uv(texture_id, uv)
}

pub fn get_glyph(font_id: u32, c: char) -> Option<Glyph> {
    RENDER_CONTEXT.lock().ok()?.get_glyph(font_id, c)
}

pub fn window_size() -> Vec2 {
    Vec2::zero()
}

struct RenderContext {
    window_size: Vec2,
    matrix: Mat4,
    renderer: Option<Renderer>,
}

impl RenderContext {
    pub fn new() -> RenderContext {
        RenderContext {
            window_size: Vec2::zero(),
            matrix: Mat4::identity(),
            renderer: None,
        }
    }

    pub fn init(&mut self, settings: WindowSettings) -> Result<()> {
        self.matrix =
            Self::ortho_top_left_matrix(settings.width as f32, settings.height as f32, -1., 1.);
        self.renderer = Some(Renderer::new());

        Ok(())
    }

    // Matrix to be submitted to shader for converting pixels to NDC
    pub fn ortho_top_left_matrix(width: f32, height: f32, near: f32, far: f32) -> Mat4 {
        let sx = 2. / width;
        let sy = -2. / height; // negative to flip Y axis
        let sz = 2. / (far - near);
        let tx = -1.;
        let ty = 1.;
        let tz = -(far + near) / (far - near);

        Mat4 {
            cols: [
                Vec4::from([sx, 0., 0., 0.]),
                Vec4::from([0., sy, 0., 0.]),
                Vec4::from([0., 0., sz, 0.]),
                Vec4::from([tx, ty, tz, 1.]),
            ],
        }
    }

    // Render methods - unwrap is intentional here, if the renderer hasn't
    // been initialized by the time we are calling these methods then we have
    // a serious bug and need to know about it
    pub fn register_texture(&mut self, texture: &TextureData) -> Result<usize> {
        self.renderer.as_mut().unwrap().register_texture(texture)
    }

    pub fn register_font(&mut self, glyphs: HashMap<char, Glyph>) -> Result<usize> {
        self.renderer.as_mut().unwrap().register_font(glyphs)
    }

    pub fn compute_uv(&self, texture_id: usize, uv: Vec4) -> Vec4 {
        self.renderer.as_ref().unwrap().compute_uv(texture_id, uv)
    }

    pub fn get_glyph(&self, font_id: u32, c: char) -> Option<Glyph> {
        self.renderer.as_ref()?.get_glyph(font_id, c).cloned()
    }

    fn clear(&self, clear_color: Vec4) {
        let Vec4 { x, y, z, w } = clear_color;
        unsafe {
            gl::ClearColor(x, y, z, w);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

    fn set_blend_mode(&self, blend_mode: BlendMode) {
        unsafe {
            match blend_mode {
                // Alpha: standard transparency
                // src * src_alpha + dst * (1 - src_alpha)
                BlendMode::Alpha => {
                    gl::Enable(gl::BLEND);
                    gl::BlendEquation(gl::FUNC_ADD);
                    gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
                }

                // Additive: glowing effects, particles, light
                // src * src_alpha + dst * 1
                BlendMode::Additive => {
                    gl::Enable(gl::BLEND);
                    gl::BlendEquation(gl::FUNC_ADD);
                    gl::BlendFunc(gl::SRC_ALPHA, gl::ONE);
                }

                // Multiply: darkening, shadows, color mixing
                // src * dst + dst * 0
                BlendMode::Multiply => {
                    gl::Enable(gl::BLEND);
                    gl::BlendEquation(gl::FUNC_ADD);
                    gl::BlendFunc(gl::DST_COLOR, gl::ZERO);
                }

                // Pre-multiplied alpha: RGB already mutliplied by alpha
                // src * 1 + dst * (1 - src_alpha)
                BlendMode::PremultipliedAlpha => {
                    gl::Enable(gl::BLEND);
                    gl::BlendEquation(gl::FUNC_ADD);
                    gl::BlendFunc(gl::ONE, gl::ONE_MINUS_SRC_ALPHA);
                }

                // Screen: opposite of multiply, lightens. Good for glow, fog, light overlays
                // src * (1 - dst) + dst * 1
                BlendMode::Screen => {
                    gl::Enable(gl::BLEND);
                    gl::BlendEquation(gl::FUNC_ADD);
                    gl::BlendFunc(gl::ONE_MINUS_DST_COLOR, gl::ONE);
                }

                // Subtract: src removes from dst. Shadows, darkening effects
                // dst - src * src_alpha
                BlendMode::Subtract => {
                    gl::Enable(gl::BLEND);
                    gl::BlendEquation(gl::FUNC_REVERSE_SUBTRACT);
                    gl::BlendFuncSeparate(gl::SRC_ALPHA, gl::ONE, gl::ONE, gl::ONE);
                }

                // Replace: no blending, just overwrite. UI backgrounds, clear rects
                BlendMode::Replace => {
                    gl::Disable(gl::BLEND);
                }
            }
        }
    }

    pub fn begin_frame(&self, clear_color: Vec4) {
        self.clear(clear_color);
    }

    pub fn end_frame(&self) {}

    pub fn begin_pass(
        &mut self,
        name: &'static str,
        target: RenderTarget,
        clear: Option<Vec4>,
        blend_mode: Option<BlendMode>,
        view_projection: Option<Mat4>,
    ) {
        log::trace!("Starting render pass: {name}");

        match target {
            RenderTarget::Framebuffer(id) => unsafe {
                gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, id);
            },
            RenderTarget::Screen => unsafe {
                gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0);
            },
        }

        if let Some(color) = clear {
            self.clear(color);
        }

        if let Some(blend) = blend_mode {
            self.set_blend_mode(blend);
        }

        if let Some(m) = view_projection {
            self.matrix = m;
        }
    }

    pub fn end_pass(&self) {}

    pub fn draw_quad_instances(&self, instances: &[QuadInstance]) {
        self.renderer
            .as_ref()
            .unwrap()
            .draw_quad_instances(self.matrix, instances);
    }

    pub fn draw_text_instances(&self, instances: &[GlyphInstance]) {
        self.renderer
            .as_ref()
            .unwrap()
            .draw_text_instances(self.matrix, instances);
    }

    pub fn draw_line_instances(&self, instances: &[LineInstance]) {
        self.renderer
            .as_ref()
            .unwrap()
            .draw_line_instances(self.matrix, instances);
    }

    pub fn draw_circle_instances(&self, instances: &[CircleInstance]) {
        self.renderer
            .as_ref()
            .unwrap()
            .draw_circle_instances(self.matrix, instances);
    }
}

unsafe impl Send for RenderContext {}

struct RenderQueue {
    sender: Sender<RenderCommand>,
    receiver: Receiver<RenderCommand>,
}

impl RenderQueue {
    pub fn new() -> Self {
        let (sender, receiver) = unbounded();
        Self { sender, receiver }
    }

    pub fn push(&self, cmd: RenderCommand) {
        let _ = self.sender.send(cmd);
    }

    pub fn process(&self, ctx: &mut RenderContext) {
        for cmd in self.receiver.try_iter() {
            Self::execute(cmd, ctx);
        }
    }

    pub fn execute(cmd: RenderCommand, ctx: &mut RenderContext) {
        match cmd {
            RenderCommand::BeginFrame { clear_color } => ctx.begin_frame(clear_color),
            RenderCommand::EndFrame => ctx.end_frame(),
            RenderCommand::BeginPass {
                name,
                target,
                clear,
                blend_mode,
                view_projection,
            } => {
                ctx.begin_pass(name, target, clear, blend_mode, view_projection);
            }
            RenderCommand::EndPass => ctx.end_pass(),
            RenderCommand::Batch(batch) => {
                for cmd in batch.iter() {
                    Self::execute(cmd.clone(), ctx);
                }
            }
            RenderCommand::DrawQuads { instances } => ctx.draw_quad_instances(&instances),
            RenderCommand::DrawText { instances } => ctx.draw_text_instances(&instances),
            RenderCommand::DrawLines { instances } => ctx.draw_line_instances(&instances),
            RenderCommand::DrawCircles { instances } => ctx.draw_circle_instances(&instances),
        }
    }
}

impl Default for RenderQueue {
    fn default() -> Self {
        Self::new()
    }
}
