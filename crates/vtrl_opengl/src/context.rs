extern crate gl;
extern crate glfw;

use glfw::{Context, Glfw, GlfwReceiver, OpenGlProfileHint, PWindow, WindowHint};
use lazy_static::lazy_static;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

use vtrl_common::prelude::*;

use crate::renderers::*;
use crate::types::*;

lazy_static! {
    static ref GLFW_INIT: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    static ref RENDER_CONTEXT: Arc<Mutex<Box<RenderContext>>> =
        Arc::new(Mutex::new(Box::new(RenderContext::new())));
}

pub fn init(settings: WindowSettings) -> Result<()> {
    if let Ok(mut ctx) = RENDER_CONTEXT.lock() {
        ctx.init(settings)?;
    }

    Ok(())
}

pub fn process_events() {
    if let Ok(mut ctx) = RENDER_CONTEXT.lock() {
        ctx.process_events();
    }
}

pub fn clear(r: f32, g: f32, b: f32, a: f32) {
    unsafe {
        gl::ClearColor(r, g, b, a);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
}

pub fn register_texture(texture: &TextureData) -> Result<usize> {
    RENDER_CONTEXT.lock().unwrap().register_texture(texture)
}

pub fn compute_uv(texture_id: usize, uv: Vec4) -> Vec4 {
    RENDER_CONTEXT.lock().unwrap().compute_uv(texture_id, uv)
}

pub fn draw_quad_instances(instances: &[QuadInstance]) {
    if let Ok(ctx) = RENDER_CONTEXT.lock() {
        ctx.draw_quad_instances(instances);
    }
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
    matrix: Mat4,
    quad_renderer: Option<QuadRenderer>,
}

impl RenderContext {
    pub fn new() -> RenderContext {
        RenderContext {
            glfw: None,
            window: None,
            matrix: Mat4::identity(),
            quad_renderer: None,
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

        glfw.poll_events();

        self.glfw = Some(GlfwWrapper(Box::new(glfw)));
        self.window = Some(WindowContext {
            window: pwindow,
            events,
        });
        self.matrix =
            self.ortho_top_left_matrix(settings.width as f32, settings.height as f32, -1., 1.);
        self.quad_renderer = Some(QuadRenderer::new());

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
                            message_bus::send(WindowMessage::CursorPosition(x as u32, y as u32));
                    }
                    CursorEnter(entered) => {
                        let _ = message_bus::send(WindowMessage::CursorEnter(entered));
                    }
                    Scroll(x, y) => {
                        let _ = message_bus::send(WindowMessage::Scroll(x, y));
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
    pub fn ortho_top_left_matrix(&self, width: f32, height: f32, near: f32, far: f32) -> Mat4 {
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

    pub fn register_texture(&mut self, texture: &TextureData) -> Result<usize> {
        if let Some(r) = &mut self.quad_renderer {
            r.register_texture(texture)
        } else {
            Err(VtrlError::Renderer(
                "Quad renderer not initialized!".to_string(),
            ))
        }
    }

    pub fn compute_uv(&self, texture_id: usize, uv: Vec4) -> Vec4 {
        if let Some(r) = &self.quad_renderer {
            r.compute_uv(texture_id, uv)
        } else {
            Vec4::new(0., 0., 1., 1.)
        }
    }

    pub fn draw_quad_instances(&self, instances: &[QuadInstance]) {
        if let Some(r) = &self.quad_renderer {
            r.draw_quad_instances(self.matrix, instances);
        }
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
