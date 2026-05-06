use vtrl_common::prelude::*;

use crate::backend::*;

pub struct Context {

}

impl Context {

}

impl RenderContext for Context {
    fn bind_framebuffer(framebuffer_id: u32) {
        commands::bind_framebuffer(framebuffer_id);
    }

    fn clear(color: Vec4) {
        commands::clear(color);
    }

    fn set_blend_mode(mode: BlendMode) {
        commands::set_blend_mode(mode);
    }

    fn draw_instanced(instance_count: u32) {
        commands::draw_instanced(instance_count);
    }
}
