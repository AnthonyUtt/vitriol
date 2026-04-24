extern crate gl;

use super::primitives::VertexArray;
use ultraviolet::IVec2;

pub unsafe fn init() {
    gl::Enable(gl::BLEND);
    gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

    gl::Enable(gl::DEPTH_TEST);
    gl::Enable(gl::LINE_SMOOTH);
}

pub unsafe fn set_viewport(x: i32, y: i32, width: i32, height: i32) {
    gl::Viewport(x, y, width, height);
}

pub unsafe fn get_viewport_size() -> IVec2 {
    let mut dims: [i32; 4] = [0; 4];
    gl::GetIntegerv(gl::VIEWPORT, &mut dims as *mut i32);
    IVec2::new(dims[2], dims[3])
}

pub unsafe fn set_clear_color(r: f32, g: f32, b: f32, a: f32) {
    gl::ClearColor(r, g, b, a);
}

pub unsafe fn clear() {
    gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
}

pub unsafe fn draw_indexed(vao: &VertexArray, index_count: Option<i32>) {
    vao.bind();
    let count = index_count.unwrap_or(vao.get_index_buffer().count);
    gl::DrawElements(gl::TRIANGLES, count, gl::UNSIGNED_INT, std::ptr::null());
    vao.unbind();
}

pub unsafe fn draw_lines(vao: &VertexArray, vertex_count: i32) {
    vao.bind();
    gl::DrawArrays(gl::LINES, 0, vertex_count);
    vao.unbind();
}

pub unsafe fn set_line_width(width: f32) {
    gl::LineWidth(width);
}
