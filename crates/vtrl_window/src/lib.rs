extern crate gl;
extern crate glfw;

use glfw::{Context, Glfw, GlfwReceiver, OpenGlProfileHint, PWindow, WindowHint};
use lazy_static::lazy_static;
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use vtrl_common::prelude::*;
use vtrl_ecs::prelude::*;

lazy_static! {
    static ref GLFW_INIT: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
}

pub struct WindowPlugin;

impl WindowPlugin {
    pub fn init(settings: WindowSettings) -> Result<(WindowContext, GlfwWrapper, Viewport)> {
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

        let glfw = GlfwWrapper(Box::new(glfw));
        let window = WindowContext {
            window: pwindow,
            events,
        };
        let viewport = Viewport {
            width: settings.width,
            height: settings.height,
        };

        Ok((window, glfw, viewport))
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

    pub fn process_events(glfw: &mut GlfwWrapper, window: &mut WindowContext, vp: &mut Viewport) {
        glfw.poll_events();

        for (_, event) in glfw::flush_messages(&window.events) {
            use glfw::WindowEvent::*;
            match event {
                Pos(x, y) => {
                    let _ = message_bus::send(WindowMessage::Reposition(x as u32, y as u32));
                }
                Size(width, height) => {
                    vp.width = width as u32;
                    vp.height = height as u32;
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
    }

    pub fn swap_buffers(window: &mut WindowContext) {
        window.window.swap_buffers();
    }
}

impl Plugin for WindowPlugin {
    fn build(&self, world: &mut World, _: &mut AssetManager) {
        world.add_system(ScheduleSlot::Init, |w, _| {
            // TODO: pull window settings from somewhere???
            let settings = WindowSettings::default();
            let (window, glfw, viewport) = Self::init(settings)
                .expect("Failed to create game window!");

            w.add_resource(window);
            w.add_resource(glfw);
            w.add_resource(viewport);
        });

        world.add_system(ScheduleSlot::First, |w, _| {
            let mut window = w.get_resource_mut::<WindowContext>()
                .expect("Unable to find window context!");
            let mut glfw = w.get_resource_mut::<GlfwWrapper>()
                .expect("Unable to find GLFW instance!");
            let mut viewport = w.get_resource_mut::<Viewport>()
                .expect("Unable to find viewport!");

            Self::process_events(&mut glfw, &mut window, &mut viewport);
        });

        world.add_system(ScheduleSlot::Last, |w, _| {
            let mut window = w.get_resource_mut::<WindowContext>()
                .expect("Unable to find window context!");

            Self::swap_buffers(&mut window);
        });
    }
}

pub struct WindowContext {
    pub window: PWindow,
    pub events: GlfwReceiver<(f64, glfw::WindowEvent)>,
}

pub struct GlfwWrapper(pub Box<Glfw>);
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
