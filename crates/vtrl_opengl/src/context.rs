extern crate gl;
extern crate glfw;

use glfw::{Context, Glfw, GlfwReceiver, OpenGlProfileHint, PWindow, WindowHint};
use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use vtrl_common::prelude::*;

use crate::renderer::*;
use crate::types::*;

lazy_static! {
    static ref GLFW_INIT: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    static ref RENDER_CONTEXT: Arc<Mutex<Box<RenderContext>>> =
        Arc::new(Mutex::new(Box::new(RenderContext::new())));
    static ref RENDER_QUEUE: RenderQueue = RenderQueue::new();
}

pub fn init(settings: WindowSettings) -> Result<()> {
    RENDER_CONTEXT.lock().unwrap().init(settings)
}

pub fn push_command(cmd: RenderCommand) {
    RENDER_QUEUE.push(cmd);
}

pub fn process_events() {
    RENDER_CONTEXT.lock().unwrap().process_events();
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
    RENDER_CONTEXT
        .lock()
        .map(|ctx| ctx.window_size())
        .unwrap_or(Vec2::zero())
}

struct GlfwWrapper(Box<Glfw>);
impl std::ops::Deref for GlfwWrapper {
    type Target = Glfw;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for GlfwWrapper {
    fn deref_mut(&mut self) -> &mut Glfw {
        &mut self.0
    }
}

struct WindowContext {
    window: PWindow,
    events: GlfwReceiver<(f64, glfw::WindowEvent)>,
}

struct RenderContext {
    glfw: Option<GlfwWrapper>,
    window: Option<WindowContext>,
    window_size: Vec2,
    matrix: Mat4,
    renderer: Option<Renderer>,
}

impl RenderContext {
    pub fn new() -> RenderContext {
        RenderContext {
            glfw: None,
            window: None,
            window_size: Vec2::zero(),
            matrix: Mat4::identity(),
            renderer: None,
        }
    }

    pub fn init(&mut self, settings: WindowSettings) -> Result<()> {
        let mut glfw = Self::init_glfw()?;

        glfw.window_hint(WindowHint::ContextVersionMajor(4));
        glfw.window_hint(WindowHint::ContextVersionMinor(5));
        glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
        #[cfg(debug_assertions)]
        glfw.window_hint(WindowHint::OpenGlDebugContext(true));

        let (mut pwindow, events) = glfw
            .create_window(
                settings.width,
                settings.height,
                settings.title,
                glfw::WindowMode::Windowed,
            )
            .ok_or(VtrlError::Window(
                "Failed to create core window!".to_string(),
            ))?;

        pwindow.make_current();
        pwindow.set_all_polling(true);

        gl::load_with(|s| match pwindow.get_proc_address(s) {
            Some(addr) => addr as *const std::ffi::c_void,
            None => std::ptr::null(),
        });
        Self::print_gl_facts()?;
        set_gl_debug_message_callback();

        // BlendFunc is set per-pass in the renderer (quads use straight
        // alpha, text uses premultiplied) — just toggle the feature on.
        unsafe { gl::Enable(gl::BLEND) };

        // On HiDPI / fractional-scaling compositors the framebuffer is larger
        // than the requested window size, so the implicit initial viewport
        // covers only a corner of it. Set the viewport from the actual
        // framebuffer size and keep it in sync on FramebufferSize events.
        let (fb_w, fb_h) = pwindow.get_framebuffer_size();
        unsafe { gl::Viewport(0, 0, fb_w, fb_h) };

        glfw.poll_events();

        self.glfw = Some(GlfwWrapper(Box::new(glfw)));
        self.window = Some(WindowContext {
            window: pwindow,
            events,
        });
        self.window_size = Vec2::new(settings.width as f32, settings.height as f32);
        self.matrix =
            Self::ortho_top_left_matrix(settings.width as f32, settings.height as f32, -1., 1.);
        self.renderer = Some(Renderer::new());

        Ok(())
    }

    fn init_glfw() -> Result<Glfw> {
        if GLFW_INIT.load(Ordering::SeqCst) {
            return Err(VtrlError::Window(
                "GLFW has already been initialized!".to_string(),
            ));
        }

        use glfw::fail_on_errors;
        let glfw = glfw::init(fail_on_errors!())
            .map_err(|_| VtrlError::Window("Failed to intialize GLFW!".to_string()))?;

        GLFW_INIT.store(true, Ordering::SeqCst);

        Ok(glfw)
    }

    fn print_gl_facts() -> Result<()> {
        unsafe {
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                return Err(VtrlError::Window(format!(
                    "OpenGL error after init: {error}"
                )));
            }

            use std::ffi::CStr;
            let vendor = CStr::from_ptr(gl::GetString(gl::VENDOR) as *const i8);
            let renderer = CStr::from_ptr(gl::GetString(gl::RENDERER) as *const i8);
            let version = CStr::from_ptr(gl::GetString(gl::VERSION) as *const i8);

            log::debug!("OpenGL Vendor: {vendor:?}");
            log::debug!("OpenGL Renderer: {renderer:?}");
            log::debug!("OpenGL Version: {version:?}");
        }

        Ok(())
    }

    fn process_events(&mut self) {
        if let Some(glfw) = &mut self.glfw {
            glfw.poll_events();
        }

        if let Some(window) = &mut self.window {
            for (_, event) in glfw::flush_messages(&window.events) {
                use glfw::WindowEvent::*;
                match event {
                    Pos(x, y) => {
                        let _ = message_bus::send(WindowMessage::Reposition(x as u32, y as u32));
                    }
                    Size(width, height) => {
                        self.window_size = Vec2::new(width as f32, height as f32);
                        self.matrix =
                            Self::ortho_top_left_matrix(width as f32, height as f32, -1., 1.);
                        let _ =
                            message_bus::send(WindowMessage::Resize(width as u32, height as u32));
                    }
                    Close => {
                        let _ = message_bus::send(SystemMessage::Shutdown);
                    }
                    Refresh => {
                        let _ = message_bus::send(WindowMessage::Refresh);
                    }
                    Focus(focus) => {
                        let _ = message_bus::send(WindowMessage::Focus(focus));
                    }
                    Iconify(iconify) => {
                        let _ = message_bus::send(WindowMessage::Minimize(iconify));
                    }
                    FramebufferSize(width, height) => {
                        unsafe { gl::Viewport(0, 0, width, height) };
                        let _ = message_bus::send(WindowMessage::FramebufferResize(
                            width as u32,
                            height as u32,
                        ));
                    }
                    MouseButton(button, action, _mods) => {
                        let state = matches!(action, glfw::Action::Press);
                        let _ = message_bus::send(WindowMessage::MouseButton(button as u32, state));
                    }
                    CursorPos(x, y) => {
                        let _ =
                            message_bus::send(WindowMessage::CursorPosition(x as f32, y as f32));
                    }
                    CursorEnter(entered) => {
                        let _ = message_bus::send(WindowMessage::CursorEnter(entered));
                    }
                    Scroll(x, y) => {
                        let _ = message_bus::send(WindowMessage::Scroll(x as f32, y as f32));
                    }
                    Key(key, _scancode, action, _mods) => {
                        let state = matches!(action, glfw::Action::Press | glfw::Action::Repeat);
                        let _ = message_bus::send(WindowMessage::Key(key as u32, state));
                    }
                    Char(char) => {
                        let _ = message_bus::send(WindowMessage::Char(char));
                    }
                    CharModifiers(char, _mods) => {
                        let _ = message_bus::send(WindowMessage::CharModifiers(char));
                    }
                    FileDrop(paths) => {
                        let _ = message_bus::send(WindowMessage::FileDropped(paths));
                    }
                    Maximize(max) => {
                        let _ = message_bus::send(WindowMessage::Maximize(max));
                    }
                    ContentScale(x, y) => {
                        let _ = message_bus::send(WindowMessage::ContentScale(x, y));
                    }
                }
            }

            window.window.swap_buffers();
        }
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

    pub fn window_size(&self) -> Vec2 {
        self.window_size
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

extern "system" fn gl_message_callback(
    _source: gl::types::GLenum,
    _gltype: gl::types::GLenum,
    _id: gl::types::GLuint,
    severity: gl::types::GLenum,
    _length: gl::types::GLsizei,
    message: *const gl::types::GLchar,
    _user_param: *mut std::ffi::c_void,
) {
    let msg = unsafe { std::ffi::CStr::from_ptr(message).to_str() };
    match severity {
        gl::DEBUG_SEVERITY_HIGH => log::error!("GL_HIGH: {:?}", msg),
        gl::DEBUG_SEVERITY_MEDIUM => log::warn!("GL_MEDIUM: {:?}", msg),
        gl::DEBUG_SEVERITY_LOW => log::info!("GL_LOW: {:?}", msg),
        gl::DEBUG_SEVERITY_NOTIFICATION => log::trace!("GL_NOTIFICATION: {:?}", msg),
        _ => log::error!("UNKNOWN SEVERITY: {:?}, MESSAGE: {:?}", severity, msg),
    };
}

type GLCallback = extern "system" fn(
    source: gl::types::GLenum,
    gltype: gl::types::GLenum,
    id: gl::types::GLuint,
    severity: gl::types::GLenum,
    length: gl::types::GLsizei,
    message: *const gl::types::GLchar,
    user_param: *mut std::ffi::c_void,
);

fn set_gl_debug_message_callback() {
    unsafe {
        let mut flags: i32 = 0;
        gl::GetIntegerv(gl::CONTEXT_FLAGS, &mut flags);
        if (flags as u32 & gl::CONTEXT_FLAG_DEBUG_BIT) != 0 {
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(Some(gl_message_callback as GLCallback), std::ptr::null());
            gl::DebugMessageControl(
                gl::DONT_CARE,
                gl::DONT_CARE,
                gl::DONT_CARE,
                0,
                std::ptr::null(),
                gl::TRUE,
            );
        }
    }
}

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
