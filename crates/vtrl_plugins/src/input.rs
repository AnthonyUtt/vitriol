use std::{
    any::TypeId,
    sync::{
        Arc, RwLock,
        atomic::{AtomicBool, Ordering},
    },
};
use once_cell::sync::Lazy;

use vtrl_common::prelude::*;
use vtrl_ecs::prelude::*;

mod types;
pub use types::*;

static GLOBAL_INPUT_STATE: Lazy<InputState> = Lazy::new(InputState::new);

#[allow(clippy::module_inception)]
pub mod input {
    use super::*;

    pub fn is_key_down(k: Key) -> bool {
        GLOBAL_INPUT_STATE.is_key_down(k)
    }

    pub fn is_key_up(k: Key) -> bool {
        GLOBAL_INPUT_STATE.is_key_up(k)
    }

    pub fn is_mouse_button_down(mb: MouseButton) -> bool {
        GLOBAL_INPUT_STATE.is_mouse_button_down(mb)
    }

    pub fn is_mouse_button_up(mb: MouseButton) -> bool {
        GLOBAL_INPUT_STATE.is_mouse_button_up(mb)
    }

    pub fn get_mouse_pos() -> Vec2 {
        GLOBAL_INPUT_STATE.get_mouse_pos()
    }

    pub fn get_mouse_scroll() -> Vec2 {
        GLOBAL_INPUT_STATE.get_mouse_scroll()
    }
}

struct InputState {
    key_state: Arc<Vec<AtomicBool>>,
    mouse_button_state: Arc<Vec<AtomicBool>>,
    mouse_position: Arc<RwLock<Vec2>>,
    mouse_scroll: Arc<RwLock<Vec2>>,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            key_state: Arc::new((0..121).map(|_| AtomicBool::new(false)).collect()),
            mouse_button_state: Arc::new((0..8).map(|_| AtomicBool::new(false)).collect()),
            mouse_position: Arc::new(RwLock::new(Vec2::zero())),
            mouse_scroll: Arc::new(RwLock::new(Vec2::zero())),
        }
    }

    pub fn is_key_down(&self, k: Key) -> bool {
        let index = k as usize;
        if index < self.key_state.len() {
            self.key_state[index].load(Ordering::Acquire)
        } else {
            false
        }
    }

    pub fn is_key_up(&self, k: Key) -> bool {
        !self.is_key_down(k)
    }

    pub fn is_mouse_button_down(&self, mb: MouseButton) -> bool {
        let index = mb as usize;
        if index < self.mouse_button_state.len() {
            self.mouse_button_state[index].load(Ordering::Acquire)
        } else {
            false
        }
    }

    pub fn is_mouse_button_up(&self, mb: MouseButton) -> bool {
        !self.is_mouse_button_down(mb)
    }

    pub fn get_mouse_pos(&self) -> Vec2 {
        match self.mouse_position.read() {
            Ok(pos) => *pos,
            Err(_) => Vec2::zero(),
        }
    }

    pub fn get_mouse_scroll(&self) -> Vec2 {
        match self.mouse_scroll.read() {
            Ok(scroll) => *scroll,
            Err(_) => Vec2::zero(),
        }
    }

    pub fn set_key(&self, k: u32, value: bool) {
        let index = k as usize;
        if index < self.key_state.len() {
            self.key_state[index].store(value, Ordering::Release);
        }
    }

    pub fn set_mouse_button(&self, mb: u32, value: bool) {
        let index = mb as usize;
        if index < self.mouse_button_state.len() {
            self.mouse_button_state[index].store(value, Ordering::Release);
        }
    }

    pub fn set_mouse_pos(&self, x: f32, y: f32) {
        let mut pos = self.mouse_position.write().unwrap();
        *pos = Vec2::new(x, y);
    }

    pub fn set_mouse_scroll(&self, x: f32, y: f32) {
        let mut scroll = self.mouse_scroll.write().unwrap();
        *scroll = Vec2::new(x, y);
    }
}

impl Default for InputState {
    fn default() -> Self { Self::new() }
}

pub struct InputPlugin;
impl Plugin for InputPlugin {
    fn build(&self, _world: &mut World) {
        message_bus::register_handler(Box::new(InputHandler), Some(TypeId::of::<WindowMessage>()))
            .expect("Unable to register input message handler!");
    }
}

struct InputHandler;
impl MessageHandler for InputHandler {
    fn call(&self, msg: &dyn Message) {
        if let Some(msg) = msg.as_any().downcast_ref::<WindowMessage>() {
            match msg {
                WindowMessage::Key(code, pressed) => {
                    GLOBAL_INPUT_STATE.set_key(*code, *pressed);
                },
                WindowMessage::MouseButton(button, pressed) => {
                    GLOBAL_INPUT_STATE.set_mouse_button(*button, *pressed);
                },
                WindowMessage::CursorPosition(x, y) => {
                    GLOBAL_INPUT_STATE.set_mouse_pos(*x, *y);
                },
                WindowMessage::Scroll(x, y) => {
                    GLOBAL_INPUT_STATE.set_mouse_scroll(*x, *y);
                },
                _ => {},
            }
        }
    }
}
