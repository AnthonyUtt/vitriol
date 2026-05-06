extern crate gl;

use vtrl_common::prelude::*;

pub fn bind_framebuffer(framebuffer_id: u32) {
    unsafe {
        gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, framebuffer_id);
    }
}

pub fn clear(color: Vec4) {
    unsafe {
        gl::ClearColor(color.x, color.y, color.z, color.w);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
}

pub fn set_blend_mode(mode: BlendMode) {
    unsafe {
        match mode {
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

pub fn draw_instanced(instance_count: u32) {
    unsafe {
        gl::DrawArraysInstanced(gl::TRIANGLES, 0, 6, instance_count as i32);
    }
}
