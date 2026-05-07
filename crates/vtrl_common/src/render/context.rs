use ultraviolet::Vec4;

use crate::render::{BlendMode, RenderCommand};

pub trait RenderContext {
    fn bind_framebuffer(framebuffer_id: u32);
    fn clear(color: Vec4);
    fn set_blend_mode(mode: BlendMode);
    fn draw_instanced(instance_count: u32);
}

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
